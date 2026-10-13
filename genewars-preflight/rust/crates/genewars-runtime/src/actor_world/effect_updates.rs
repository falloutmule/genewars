//! Transactional landing callbacks over the same map, players, RNG and pools.
//! Callback entry is explicit: the full inter-family scheduler belongs above
//! this layer, including prior kind1 updates and newborn scheduling.
use super::*;
use crate::physics_tables::PhysicsTables;
use crate::landing_effect_lifecycle::{self as lifecycle,RetirementUiState,RetirementUiEvent,CleanupRecord};

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct LandingPhysicsContext { pub water_height:i32, pub gravity:i16 }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LandingStepReceipt {
    pub slot:u16,
    pub phase_before:u8,
    pub phase_after:u8,
    pub contact:bool,
    pub created_effect:Option<u16>,
    pub rng_before:BlueprintRng,
    pub rng_after:BlueprintRng,
    pub draws:Vec<landing::LandingDraw>,
    pub activation:Option<ActivationBatchReceipt>,
    pub retirement:Option<EffectStepReceipt>,
    /// Source audio and message requests in their original order. The host
    /// executes these through presentation services after a successful commit.
    pub pending_events:Vec<LandingEvent>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct EffectStepReceipt {
    pub slot:u16,
    pub changed_cell:bool,
    pub retired:bool,
    pub cleaned_reference_bytes:usize,
    pub ui_after:RetirementUiState,
    pub pending_ui_events:Vec<RetirementUiEvent>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum LandingEffectCallback { Landing(LandingStepReceipt), Smoke(EffectStepReceipt), DepartureParticle(EffectStepReceipt), WaterSplash(EffectStepReceipt) }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LandingEffectPassReceipt {
    pub primary:Vec<LandingEffectCallback>,
    pub newborn:Vec<LandingEffectCallback>,
    pub queue_candidates:Vec<u16>,
    pub ui_after:RetirementUiState,
    ///4E950's984C0 palette/resource tail is presentation work, outside these
    /// numeric callbacks. Candidates still require83140 draw admission.
    pub palette_update_pending:bool,
}

impl ActorWorld {
    /// The first two4E950 scans, rechecking the live pool before every call.
    /// Fresh landing phases and code6/2F effects are admitted. Other selectors
    /// and unsupported callbacks reject the whole pass.
    pub fn step_landing_effect_callbacks(
        &mut self,context:LandingPhysicsContext,physics:&PhysicsTables,
        trig:&TrigTables,ui:&RetirementUiState,
    )->Result<LandingEffectPassReceipt,ActorWorldError> {
        self.validate_links()?;self.validate_effect_count()?;
        let mut staged=self.clone();let mut created=0u32;
        let mut receipt=LandingEffectPassReceipt { primary:Vec::new(),newborn:Vec::new(),queue_candidates:Vec::new(),
            ui_after:ui.clone(),palette_update_pending:true };
        for slot in 0..200u16 {
            let record=staged.effects.as_ref().unwrap().record(usize::from(slot))?;
            if record[6]==0||record[21]&1!=0 { continue; }
            let callback=staged.landing_effect_callback(slot,context,physics,trig,&receipt.ui_after)?;
            match &callback {
                LandingEffectCallback::Landing(e)=>{
                    created+=u32::from(e.created_effect.is_some());
                    if let Some(r)=&e.retirement { receipt.ui_after=r.ui_after.clone(); }
                },
                LandingEffectCallback::Smoke(e)|LandingEffectCallback::DepartureParticle(e)|LandingEffectCallback::WaterSplash(e)=>receipt.ui_after=e.ui_after.clone(),
            }
            receipt.primary.push(callback);
        }
        let mut newborn=lifecycle::Family4NewbornProtocol::new(created);
        while let Some((slot,after))=newborn.next(staged.effects.as_ref().unwrap().records())? {
            let pool=staged.effects.as_mut().unwrap();let before=*pool.record(usize::from(slot))?;
            pool.replace_record_checked(usize::from(slot),&before,&after)?;
            let callback=staged.landing_effect_callback(slot,context,physics,trig,&receipt.ui_after)?;
            match &callback {
                LandingEffectCallback::Landing(e)=>if let Some(r)=&e.retirement { receipt.ui_after=r.ui_after.clone(); },
                LandingEffectCallback::Smoke(e)|LandingEffectCallback::DepartureParticle(e)|LandingEffectCallback::WaterSplash(e)=>receipt.ui_after=e.ui_after.clone(),
            }
            receipt.newborn.push(callback);
        }
        receipt.queue_candidates=lifecycle::family4_queue_slots(staged.effects.as_ref().unwrap().records(),word(&staged.backing,0x3554))?;
        staged.validate_links()?;staged.validate_effect_count()?;*self=staged;Ok(receipt)
    }
    fn landing_effect_callback(
        &mut self,slot:u16,context:LandingPhysicsContext,physics:&PhysicsTables,
        trig:&TrigTables,ui:&RetirementUiState,
    )->Result<LandingEffectCallback,ActorWorldError> {
        let code=self.effects.as_ref().unwrap().record(usize::from(slot))?[69];
        match code {
            0x31=>{
                let phase=self.effects.as_ref().unwrap().record(usize::from(slot))?[70];
                match phase {
                    0=>self.step_landing_phase0(slot,context,physics,trig),
                    1=>self.step_landing_phase1(slot,context,trig),
                    2|3=>self.step_landing_departure(slot,context,physics,trig,ui),
                    _=>Err(LandingError::UnsupportedPhase(phase).into()),
                }.map(LandingEffectCallback::Landing)
            },
            6=>self.step_effect6(slot,context,physics,trig,ui).map(LandingEffectCallback::Smoke),
            0x30=>self.step_effect30(slot,ui).map(LandingEffectCallback::WaterSplash),
            0x2F=>self.step_effect2f(slot,context,physics,trig,ui).map(LandingEffectCallback::DepartureParticle),
            _=>Err(lifecycle::LifecycleError::UnknownSelector(code).into()),
        }
    }

    /// Complete stationary phase1, including the timer49 message, ordered
    /// shared RNG draws, smoke factories and phase2 launch transition.
    pub fn step_landing_phase1(&mut self,slot:u16,context:LandingPhysicsContext,trig:&TrigTables)->Result<LandingStepReceipt,ActorWorldError> {
        let before=self.checked_landing_record(slot)?;
        let (ground,shade)=self.effect_ground_sample([word(&before,7),word(&before,9)])?;
        let physics=landing::plan_landing_phase1_rest_physics(&before,ground,shade,context.water_height)?;
        let phase=landing::plan_landing_phase1_after_physics(&physics.after,to_landing_rng(self.rng))?;
        self.commit_landing_phase(slot,&before,phase,context,trig,None)
    }

    pub fn step_landing_departure(&mut self,slot:u16,context:LandingPhysicsContext,physics:&PhysicsTables,
        trig:&TrigTables,ui:&RetirementUiState)->Result<LandingStepReceipt,ActorWorldError> {
        let before=self.checked_landing_record(slot)?;let at=[word(&before,7),word(&before,9)];
        let(ground,shade)=self.effect_ground_sample(at)?;
        let flight=landing::plan_landing_rising_physics(&before,ground,shade,context.water_height,!self.move_obstacle_at(at)?,physics.angles())?;
        let phase=match before[70] {
            2=>landing::plan_landing_phase2_after_physics(&flight.after,to_landing_rng(self.rng))?,
            3=>landing::plan_landing_phase3_after_physics(&flight.after,to_landing_rng(self.rng))?,
            p=>return Err(LandingError::UnsupportedPhase(p).into()),
        };
        self.commit_landing_phase(slot,&before,phase,context,trig,Some(ui))
    }

    /// The proved grounded quiet phase1 interval, before timer49 requests
    /// additional messages/particle factories. It consumes no random draws.
    pub fn step_landing_phase1_quiet(&mut self,slot:u16,context:LandingPhysicsContext)->Result<LandingStepReceipt,ActorWorldError> {
        if !self.validate_links()?.contains(&(5,slot as i16)) { return Err(ActorWorldError::MissingEffectLink(slot)); }
        self.validate_effect_count()?;
        let pool=self.effects.as_ref().unwrap();
        if pool.binding(usize::from(slot))?!=Some(EffectAnimationBinding::EffAnimation49) { return Err(ActorWorldError::EffectBinding(slot)); }
        let before=*pool.record(usize::from(slot))?;let point=[word(&before,7),word(&before,9)];
        let (ground,shade)=self.effect_ground_sample(point)?;
        let plan=landing::plan_landing_phase1_quiet_grounded(&before,ground,shade,context.water_height)?;
        let mut staged=self.clone();staged.effects.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&plan.after)?;
        staged.validate_links()?;
        let receipt=LandingStepReceipt { slot,phase_before:1,phase_after:1,contact:false,created_effect:None,
            rng_before:self.rng,rng_after:self.rng,draws:Vec::new(),activation:None,retirement:None,pending_events:Vec::new() };
        *self=staged;Ok(receipt)
    }

    /// Shared6DF30 motion, animation, map relocation and source49D30/724C0
    /// retirement for smoke code6. Earlier/later scheduler callbacks are
    /// separate. The caller supplies its UI reference state explicitly.
    pub fn step_effect6(
        &mut self,slot:u16,context:LandingPhysicsContext,physics:&PhysicsTables,
        trig:&TrigTables,ui:&RetirementUiState,
    )->Result<EffectStepReceipt,ActorWorldError> {
        self.step_particle(slot,6,context,physics,trig,ui)
    }
    pub fn step_effect2f(&mut self,slot:u16,context:LandingPhysicsContext,physics:&PhysicsTables,
        trig:&TrigTables,ui:&RetirementUiState)->Result<EffectStepReceipt,ActorWorldError> {
        self.step_particle(slot,0x2F,context,physics,trig,ui)
    }
    pub fn step_effect30(&mut self,slot:u16,ui:&RetirementUiState)->Result<EffectStepReceipt,ActorWorldError>{
        if !self.validate_links()?.contains(&(5,slot as i16)){return Err(ActorWorldError::MissingEffectLink(slot));}self.validate_effect_count()?;
        let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        if pool.binding(slot as usize)?!=Some(EffectAnimationBinding::EffAnimation48){return Err(ActorWorldError::EffectBinding(slot));}
        let before=*pool.record(slot as usize)?;let plan=landing::plan_water_splash_update(&before)?;
        let mut staged=self.clone();staged.effects.as_mut().unwrap().replace_record_checked(slot as usize,&before,&plan.after)?;
        let receipt=if plan.retirement_required{staged.retire_effect(slot,&plan.after,ui)?}else{EffectStepReceipt{slot,changed_cell:false,retired:false,cleaned_reference_bytes:0,ui_after:ui.clone(),pending_ui_events:Vec::new()}};
        staged.validate_links()?;staged.validate_effect_count()?;*self=staged;Ok(receipt)
    }
    pub fn create_water_splash(&mut self,position:[i16;2],physical_height:i32)->Result<Option<(u16,Vec<LandingEvent>)>,ActorWorldError>{
        self.validate_links()?;self.validate_effect_count()?;
        if word(&self.backing,0x3554)==200{
            if landing::water_splash_replacement(self.effects.as_ref().unwrap().records())?.is_some(){return Err(ActorWorldError::UnsupportedEffectEvent);}return Ok(None);
        }
        let cell=floor_cell(position);let raw=reference(self.map(),cell*14+6);
        let head=if raw[2]==0{None}else{self.entity(raw)?;Some(landing::ResolvedCellHead{identity:landing::LandingReference{index:word(&raw,0),kind:raw[2]}})};
        let(g,s)=self.effect_ground_sample(position)?;let p=landing::plan_water_splash_factory(self.effects.as_ref().unwrap().records(),word(&self.backing,0x3554),position,g,s,physical_height,head)?;
        let mut staged=self.clone();staged.effects.as_mut().unwrap().commit_landing_factory(&p)?;
        let id=[p.slot as u8,(p.slot>>8)as u8,5];if raw[2]!=0{staged.replace_previous(raw,id)?;}
        staged.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&id);staged.backing[0x3554..0x3556].copy_from_slice(&p.pool_count_after.to_le_bytes());
        staged.validate_links()?;*self=staged;Ok(Some((p.slot,p.events)))
    }
    fn step_particle(&mut self,slot:u16,code:u8,context:LandingPhysicsContext,physics:&PhysicsTables,
        trig:&TrigTables,ui:&RetirementUiState)->Result<EffectStepReceipt,ActorWorldError> {
        if !self.validate_links()?.contains(&(5,slot as i16)) { return Err(ActorWorldError::MissingEffectLink(slot)); }
        self.validate_effect_count()?;
        let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        let binding=if code==6 { EffectAnimationBinding::EffAnimation6 } else { EffectAnimationBinding::EffAnimation47 };
        if pool.binding(usize::from(slot))?!=Some(binding) { return Err(ActorWorldError::EffectBinding(slot)); }
        let before=*pool.record(usize::from(slot))?;
        let at=if code==6 { landing::effect6_physics_candidate(&before)? } else { landing::effect2f_physics_candidate(&before)? };
        let (ground,shade)=self.effect_ground_sample(at)?;
        let old_height=i32::from_le_bytes(before[39..43].try_into().unwrap());
        let old_ground=i32::from_le_bytes(before[17..21].try_into().unwrap());
        let next_height=old_height.wrapping_add(i32::from(word(&before,49).wrapping_add(word(&before,55)))>>3);
        let flight=if old_height<=old_ground&&next_height<=ground {
            let candidate=if code==6 { landing::effect6_grounded_candidate(&before,ground,physics.angles(),
                trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap())? }
            else { landing::effect2f_grounded_candidate(&before,ground,physics.angles(),
                trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap())? };
            let point=[word(&candidate,7),word(&candidate,9)];let(g,s)=self.effect_ground_sample(point)?;
            if code==6 { landing::plan_effect6_grounded_after_second_sample(&before,&candidate,g,s,context.water_height,!self.move_obstacle_at(point)?)? }
            else { landing::plan_effect2f_grounded_after_second_sample(&before,&candidate,g,s,context.water_height)? }
        } else if code==6 {
            landing::plan_effect6_flight(&before,ground,shade,context.water_height,!self.move_obstacle_at(at)?,
                self.effect_height_corners(at),physics.bounce(self.terrain_class_at(at)?)?,physics.angles())?
        } else { landing::plan_effect2f_flight(&before,ground,shade,context.water_height,
            self.effect_height_corners(at),physics.bounce(self.terrain_class_at(at)?)?,physics.angles())? };
        let changed_cell=floor_cell([word(&before,7),word(&before,9)])!=floor_cell([word(&flight.after,7),word(&flight.after,9)]);
        let mut staged=self.clone();
        if changed_cell { staged.unlink_effect(slot)?; }
        staged.effects.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&flight.after)?;
        // Type5 virtual88 is1F030 (RET), so relocation has no extra callback.
        if changed_cell { staged.insert_effect_head(slot)?; }
        let moved=*staged.effects.as_ref().unwrap().record(usize::from(slot))?;
        let animation=if code==6 { landing::plan_effect6_animation_after_physics(&moved)? } else { landing::plan_effect2f_animation_after_physics(&moved)? };
        staged.effects.as_mut().unwrap().replace_record_checked(usize::from(slot),&moved,&animation.after)?;
        let mut receipt=EffectStepReceipt { slot,changed_cell,retired:animation.retirement_required,
            cleaned_reference_bytes:0,ui_after:ui.clone(),pending_ui_events:Vec::new() };
        if animation.retirement_required {
            receipt=staged.retire_effect(slot,&animation.after,ui)?;receipt.changed_cell=changed_cell;
        }
        staged.validate_links()?;staged.validate_effect_count()?;*self=staged;Ok(receipt)
    }

    fn retire_effect(&mut self,slot:u16,before:&LandingRecord,ui:&RetirementUiState)->Result<EffectStepReceipt,ActorWorldError> {
        // Temporary projection scaffolding is only used for families whose
        // native overlays have not been admitted. It is never persisted.
        let records=self.effect_cleanup_records();
        let views=records.iter().map(|(kind,slot,bytes)|CleanupRecord{kind:*kind,slot:*slot,bytes}).collect::<Vec<_>>();
        let plan=lifecycle::plan_effect6_retirement(before,word(&self.backing,0x3554),&views,ui)?;
        self.unlink_effect(slot)?;
        for patch in &plan.cleanup {
            if patch.kind==5&&patch.slot==slot { continue; }
            self.apply_cleanup_byte(*patch)?;
        }
        self.effects.as_mut().unwrap().retire_checked(usize::from(slot),before,&plan.after)?;
        self.backing[0x3554..0x3556].copy_from_slice(&plan.pool_count_after.to_le_bytes());
        Ok(EffectStepReceipt { slot,changed_cell:false,retired:true,cleaned_reference_bytes:plan.cleanup.len(),
            ui_after:plan.ui_after,pending_ui_events:plan.ui_events })
    }

    pub(super) fn effect_cleanup_records(&self)->Vec<(u8,u16,Vec<u8>)> {
        let mut records=Vec::with_capacity(568);
        for kind in [1u8,3,5,7,8] {
            let layout=self.layouts[usize::from(kind-1)];
            for slot in 0..layout.count {
                let bytes=match kind {
                    1 if self.creatures.is_some()=>self.creatures.as_ref().unwrap().records()[slot].to_vec(),
                    3=>self.actors.records()[slot].to_vec(),
                    5=>self.effects.as_ref().unwrap().records()[slot].to_vec(),
                    _=>{
                        let start=layout.file_offset+slot*layout.packed_stride;
                        let packed=&self.backing[start..start+layout.packed_stride];
                        let mut native=vec![0;layout.packed_stride+8];native[..31].copy_from_slice(&packed[..31]);native[39..].copy_from_slice(&packed[31..]);native
                    },
                };
                records.push((kind,slot as u16,bytes));
            }
        }
        records
    }
    pub(super) fn apply_cleanup_byte(&mut self,patch:lifecycle::CleanupBytePatch)->Result<(),ActorWorldError> {
        let slot=usize::from(patch.slot);let offset=usize::from(patch.offset);
        match patch.kind {
            1 if self.creatures.is_some()=>{
                let pool=self.creatures.as_mut().unwrap();let before=*pool.record(slot)?;let mut after=before;
                if before[offset]!=patch.before { return Err(ActorWorldError::CleanupBeforeImage{kind:patch.kind,slot:patch.slot,offset:patch.offset}); }
                after[offset]=patch.after;pool.replace_record_checked(slot,&before,&after)?;
            },
            3=>{
                let before=*self.actors.record(slot)?;let mut after=before;
                if before[offset]!=patch.before { return Err(ActorWorldError::CleanupBeforeImage{kind:patch.kind,slot:patch.slot,offset:patch.offset}); }
                after[offset]=patch.after;self.actors.replace_record_checked(slot,&before,&after)?;
            },
            5=>{
                let pool=self.effects.as_mut().unwrap();let before=*pool.record(slot)?;let mut after=before;
                if before[offset]!=patch.before { return Err(ActorWorldError::CleanupBeforeImage{kind:patch.kind,slot:patch.slot,offset:patch.offset}); }
                after[offset]=patch.after;pool.replace_record_checked(slot,&before,&after)?;
            },
            _=>{
                let layout=self.layouts[usize::from(patch.kind-1)];let native_offset=if offset>=39 { offset-8 } else { offset };
                let absolute=layout.file_offset+slot*layout.packed_stride+native_offset;
                if self.backing[absolute]!=patch.before { return Err(ActorWorldError::CleanupBeforeImage{kind:patch.kind,slot:patch.slot,offset:patch.offset}); }
                self.backing[absolute]=patch.after;
            },
        }
        Ok(())
    }

    /// The proved71B70 load-time numeric values: gravity remains signed1C5D;
    /// water is twice signed1C63, replacing raw serialized1C67. These values
    /// describe the load transition; they are not inferred from raw1C67.
    pub fn mission_loading_physics(&self)->LandingPhysicsContext {
        LandingPhysicsContext {
            water_height:2*i32::from(word(&self.backing,0x1C63)),
            gravity:word(&self.backing,0x1C5D),
        }
    }

    /// Run one code31 phase0 callback, including its physics, smoke factory,
    /// and contact's associated actor activation. A failure anywhere rolls
    /// back both pools, players, map links and the shared random stream.
    /// This does not advance other entities or claim a full game tick.
    pub fn step_landing_phase0(
        &mut self, slot:u16, context:LandingPhysicsContext,
        physics:&PhysicsTables, trig:&TrigTables,
    )->Result<LandingStepReceipt,ActorWorldError> {
        let before=self.checked_landing_record(slot)?;
        let at=[word(&before,7),word(&before,9)];
        let (ground,shade)=self.effect_ground_sample(at)?;
        // Type5 virtual70 is true for31/6; when collision parity enables
        //74270, clear map byte11 mask1 still proves its immediate NULL path.
        let collision_null=!self.move_obstacle_at(at)?;
        let flight=match landing::plan_landing_vertical_airborne(&before,ground,shade,context.water_height,collision_null) {
            Ok(plan)=>plan,
            Err(LandingError::ContactPhysicsRequired)=>landing::plan_landing_vertical_contact(
                &before,ground,shade,context.water_height,collision_null,self.effect_height_corners(at),
                physics.bounce(self.terrain_class_at(at)?)?,physics.angles())?,
            Err(error)=>return Err(error.into()),
        };
        let phase=landing::plan_landing_phase0_after_physics(&flight.after,ground,to_landing_rng(self.rng))?;
        self.commit_landing_phase(slot,&before,phase,context,trig,None)
    }

    fn checked_landing_record(&self,slot:u16)->Result<LandingRecord,ActorWorldError> {
        if !self.validate_links()?.contains(&(5,slot as i16)) { return Err(ActorWorldError::MissingEffectLink(slot)); }
        let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        if pool.binding(usize::from(slot))?!=Some(EffectAnimationBinding::EffAnimation49) { return Err(ActorWorldError::EffectBinding(slot)); }
        self.validate_effect_count()?;
        let before=*pool.record(usize::from(slot))?;
        if before[73]!=0 { return Err(ActorWorldError::UnsupportedEffectEvent); }
        let expected=[slot as u8,(slot>>8) as u8,5];
        if reference(self.player(before[24])?.bytes(),0x496)!=expected { return Err(ActorWorldError::AssociatedUnitUnsupported); }
        Ok(before)
    }

    fn commit_landing_phase(&mut self,slot:u16,before:&LandingRecord,phase:landing::LandingPhase0Plan,
        context:LandingPhysicsContext,trig:&TrigTables,ui:Option<&RetirementUiState>)->Result<LandingStepReceipt,ActorWorldError> {
        let old_rng=self.rng;
        let mut staged=self.clone();
        staged.effects.as_mut().unwrap().replace_record_checked(usize::from(slot),before,&phase.after)?;
        staged.rng=from_landing_rng(phase.rng_after);
        let mut receipt=LandingStepReceipt { slot,phase_before:before[70],phase_after:phase.after[70],contact:phase.contact,created_effect:None,
            rng_before:old_rng,rng_after:staged.rng,draws:phase.draws,activation:None,retirement:None,pending_events:Vec::new() };
        for event in phase.events {
            match event {
                LandingEvent::CreateEffect { object_code,position,physical_height,angles,owner,mark_newborn_on_success:true } if matches!(object_code,6|0x2F)=>{
                    if let Some((slot,draw))=staged.create_particle(object_code,owner,position,physical_height,angles,true,context,trig)? {
                        receipt.draws.push(draw);receipt.created_effect=Some(slot);
                    }
                },
                LandingEvent::ActivateOwner { owner }=>{
                    receipt.activation=Some(staged.activate_associated_unit_owned(owner,trig)?);
                },
                LandingEvent::RetireLanding { owner,reference,player_counter_after:700 } if reference.kind==5 && reference.index==slot as i16 && owner==before[24]=>{
                    staged.players[usize::from(owner)].retire_landing_reference(slot)?;
                    let dead=*staged.effects.as_ref().unwrap().record(usize::from(slot))?;
                    receipt.retirement=Some(staged.retire_effect(slot,&dead,ui.ok_or(ActorWorldError::UnsupportedEffectEvent)?)?);
                },
                event @ (LandingEvent::PlayAudio(_)|LandingEvent::StopAudio{..}|LandingEvent::AdjustAudioParameter{..}|LandingEvent::LandingMessage{..})=>receipt.pending_events.push(event),
                _=>return Err(ActorWorldError::UnsupportedEffectEvent),
            }
        }
        staged.validate_links()?;staged.validate_effect_count()?;
        receipt.rng_after=staged.rng;
        *self=staged;
        Ok(receipt)
    }

    /// Source actor footsteps use the code6 factory without the landing
    /// callback's newborn mark. Caller owns the encompassing transaction.
    pub(super) fn create_unmarked_smoke(&mut self,position:[i16;2],physical_height:i32,angles:[u16;3],
        context:LandingPhysicsContext,trig:&TrigTables)->Result<Option<(u16,landing::LandingDraw)>,ActorWorldError> {
        self.create_particle(6,0,position,physical_height,angles,false,context,trig)
    }

    fn create_particle(&mut self,object_code:u8,owner:u8,position:[i16;2],physical_height:i32,angles:[u16;3],
        mark_newborn:bool,context:LandingPhysicsContext,trig:&TrigTables)->Result<Option<(u16,landing::LandingDraw)>,ActorWorldError> {
        self.validate_effect_count()?;
        if !matches!(object_code,6|0x2f)||(object_code==6&&owner!=0) {return Err(ActorWorldError::UnsupportedEffectEvent);}
        //487AE may replace a lower-priority record. The admitted same/higher
        //priority pool proves NULL; an eligible replacement remains explicit.
        if word(&self.backing,0x3554)==200 {
            if self.effects.as_ref().unwrap().records().iter().any(|r|!matches!(r[69],6|0x31|0x2f)
                ||(object_code==6&&r[69]==0x2f&&r[21]&1==0)) {return Err(ActorWorldError::UnsupportedEffectEvent);}
            return Ok(None);
        }
        let cell=floor_cell(position);let raw=reference(self.map(),cell*14+6);
        let head=if raw[2]==0 {None}else {
            self.entity(raw)?;
            Some(landing::ResolvedCellHead{identity:landing::LandingReference{index:word(&raw,0),kind:raw[2]}})
        };
        let(g,s)=self.effect_ground_sample(position)?;let pool=self.effects.as_ref().unwrap();
        let plan=if object_code==6 {landing::plan_landing_effect6_factory(pool.records(),word(&self.backing,0x3554),
            position,g,s,physical_height,angles,head,context.gravity,to_landing_rng(self.rng),
            trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap())?}
        else {landing::plan_landing_effect2f_factory(pool.records(),word(&self.backing,0x3554),owner,
            position,g,s,physical_height,angles,head,context.gravity,to_landing_rng(self.rng),
            trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap())?};
        let slot=plan.factory.slot;self.effects.as_mut().unwrap().commit_landing_factory(&plan.factory)?;
        if mark_newborn {
            let before=*self.effects.as_ref().unwrap().record(usize::from(slot))?;let mut after=before;after[21]|=1;
            self.effects.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&after)?;
        }
        let identity=[slot as u8,(slot>>8)as u8,5];
        if raw[2]!=0 {self.replace_previous(raw,identity)?;}
        self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&identity);
        self.backing[0x3554..0x3556].copy_from_slice(&plan.factory.pool_count_after.to_le_bytes());
        self.rng=from_landing_rng(plan.rng_after);
        Ok(Some((slot,plan.draw)))
    }

    pub(super) fn validate_effect_count(&self)->Result<(),ActorWorldError> {
        let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        let declared=word(&self.backing,0x3554);
        let active=pool.records().iter().filter(|r|r[6]!=0).count();
        if declared<0||declared as usize!=active { return Err(ActorWorldError::EffectCountMismatch { declared,active }); }
        Ok(())
    }
    fn effect_ground_sample(&self,at:[i16;2])->Result<(i32,u8),ActorWorldError> {
        let g=genewars_core::cd_candidate::interpolate_height(self.map(),at[0],at[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        let s=genewars_core::cd_candidate::interpolate_entity_shade(self.map(),at[0],at[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        Ok((g,s))
    }
    fn effect_height_corners(&self,at:[i16;2])->[i16;4] {
        let x=(i32::from(at[0])>>8)&127;let y=(i32::from(at[1])>>8)&127;
        [(0,0),(1,0),(0,1),(1,1)].map(|(dx,dy)|word(self.map(),((((y+dy)&127)*128+((x+dx)&127))*14+2) as usize))
    }

    /// Intrusive list removal preserves the removed object's own reference
    /// triples. Callers replace/retire that object within the outer clone.
    fn unlink_effect(&mut self,slot:u16)->Result<(),ActorWorldError> {
        let raw=[slot as u8,(slot>>8) as u8,5];
        let record=self.entity(raw)?;
        let previous=reference(record,0);let next=reference(record,3);
        let cell=floor_cell([word(record,7),word(record,9)]);
        if previous[2]==0 {
            if word(&previous,0) as usize!=cell||reference(self.map(),cell*14+6)!=raw { return Err(ActorWorldError::StaleLink); }
            if next[2]!=0 { self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&next); }
            else { self.backing[MAP_START+cell*14+8]=0; }
        } else {
            if reference(self.entity(previous)?,3)!=raw { return Err(ActorWorldError::StaleLink); }
            self.replace_next(previous,next)?;
        }
        if next[2]!=0 {
            if reference(self.entity(next)?,0)!=raw { return Err(ActorWorldError::StaleLink); }
            self.replace_previous(next,previous)?;
        }
        Ok(())
    }
    pub(super) fn replace_next(&mut self,raw:[u8;3],next:[u8;3])->Result<(),ActorWorldError> {
        self.entity(raw)?;let index=word(&raw,0) as usize;
        match raw[2] {
            1 if self.creatures.is_some()=>{
                let pool=self.creatures.as_mut().unwrap();let before=*pool.record(index)?;let mut after=before;
                after[3..6].copy_from_slice(&next);pool.replace_record_checked(index,&before,&after)?;
            },
            3=>{
                let before=*self.actors.record(index)?;let mut after=before;after[3..6].copy_from_slice(&next);
                self.actors.replace_record_checked(index,&before,&after)?;
            },
            5 if self.effects.is_some()=>{
                let pool=self.effects.as_mut().unwrap();let before=*pool.record(index)?;let mut after=before;after[3..6].copy_from_slice(&next);
                pool.replace_record_checked(index,&before,&after)?;
            },
            _=>{
                let layout=self.layouts[usize::from(raw[2]-1)];let offset=layout.file_offset+index*layout.packed_stride+3;
                self.backing[offset..offset+3].copy_from_slice(&next);
            },
        }
        Ok(())
    }
    pub(super) fn insert_effect_head(&mut self,slot:u16)->Result<(),ActorWorldError> {
        let raw=[slot as u8,(slot>>8) as u8,5];let record=self.entity(raw)?;
        let cell=floor_cell([word(record,7),word(record,9)]);
        let head=reference(self.map(),cell*14+6);
        let mut after=*self.effects.as_ref().unwrap().record(usize::from(slot))?;let before=after;
        after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0 { self.entity(head)?;after[3..6].copy_from_slice(&head);self.replace_previous(head,raw)?; }
        else { after[5]=0; }
        self.effects.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&after)?;
        self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&raw);
        Ok(())
    }
}
fn to_landing_rng(rng:BlueprintRng)->landing::LandingRng {
    landing::LandingRng { seed:rng.seed,ring:rng.ring,draw_count:rng.draw_count }
}
fn from_landing_rng(rng:landing::LandingRng)->BlueprintRng {
    BlueprintRng { seed:rng.seed,ring:rng.ring,draw_count:rng.draw_count }
}
#[cfg(test)]
mod water_tests {
 use super::*;
 #[test]
 fn splash_is_rng_free_stationary_and_retires_from_shared_map(){
  let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337_054..337_058].copy_from_slice(&0x6beu32.to_le_bytes());
  let source=ConqMissionSource::parse(&bytes).unwrap();let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[4096,4096],b"fixture\0").unwrap());
  let mut world=ActorWorld::from_parts(&source,&[[0;200];20],core::array::from_fn(|_|ActorHandleBindings::default()),players,BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
  world.admit_effects(&[[0;81];200],[None;200]).unwrap();let rng=world.rng;
  let(slot,events)=world.create_water_splash([4096,4096],1632).unwrap().unwrap();assert_eq!(slot,0);assert_eq!(rng,world.rng);
  assert!(events.iter().any(|e|matches!(e,LandingEvent::PlayAudio(a)if a.channel==0x3b)));
  let before=*world.effects.as_ref().unwrap().record(0).unwrap();let mut calls=0;
  loop{calls+=1;let r=world.step_effect30(0,&RetirementUiState::native_empty()).unwrap();assert_eq!(world.rng,rng);if r.retired{break;}let a=world.effects.as_ref().unwrap().record(0).unwrap();assert_eq!(&a[7..21],&before[7..21]);assert_eq!(&a[39..63],&before[39..63]);assert!(calls<40);}
  assert_eq!(calls,33);assert_eq!(world.effects.as_ref().unwrap().record(0).unwrap()[6],0);assert_eq!(word(&world.backing,0x3554),0);assert!(world.validate_links().unwrap().is_empty());
 }
}

//! Atomic storage and source101D0 creation for countdown entities.
//! Countdown dispatch, movement, audio and rendering remain separate.
use super::*;
use crate::family8_creation::{self, CellHeadIdentity, CreationInput};
use crate::family8_motion::{self, Family8Animation, Family8DirectionalAnimations, Family8MotionPlan};
use crate::countdown_expiry::{self, CountdownTransition, CountdownPresentation, ExpirySpawnRequest};
use crate::physics_tables::PhysicsTables;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8CreationReceipt {
    pub slot: u16,
    pub selector: u8,
    pub input_owner: u8,
    pub position: [i16; 2],
    pub initial_height: i32,
    pub ground_height: i32,
    pub shade: u8,
    pub binding: Family8AnimationBinding,
}

/// Composed world callback plan. Retirement can omit the ordinary audio tail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8StepPlan {
    pub after:Family8Record,pub rng_after:BlueprintRng,pub old_cell:u16,pub new_cell:u16,
    pub child:Option<family8_motion::Family8ChildRequest>,
    pub pending_audio:Option<family8_motion::Family8AudioRequest>,
    pub pending_ui_message:bool,pub queue_requested:bool,
}
impl From<Family8MotionPlan> for Family8StepPlan {
    fn from(p:Family8MotionPlan)->Self{Self{after:p.after,rng_after:p.rng_after,old_cell:p.old_cell,new_cell:p.new_cell,
        child:p.child,pending_audio:Some(p.pending_audio),pending_ui_message:p.pending_ui_message,queue_requested:p.queue_requested}}
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8StepReceipt {
    pub slot: u16,
    pub plan: Family8StepPlan,
    pub child_created: Option<Family8CreationReceipt>,
    pub particle_created: Option<(u16,landing::LandingDraw)>,
    pub target_selection: Option<family8_motion::Family8TargetPlan>,
    pub retirement: Option<Family8RetirementReceipt>,
    pub parent_notified: Option<u16>,
    pub parent_response: Option<family8_motion::Family8ParentNotificationPlan>,
    pub rng_after: BlueprintRng,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8PassReceipt {
    pub countdown: CountdownTransition,
    pub expiry_attempts: Vec<(ExpirySpawnRequest, Option<Family8CreationReceipt>)>,
    pub callbacks: Vec<Family8StepReceipt>,
    pub pending_prefix: Vec<CountdownPresentation>,
    pub pending_ui_animations: [CountdownPresentation; 8],
}

impl ActorWorld {
    /// Source13780, with its numeric prefix, expiry factories, ascending live
    /// slot callbacks and pending presentation requests in their source order.
    /// Unsupported callbacks reject the entire pass, including earlier births.
    pub fn step_family8_callbacks(&mut self, selected: u8, player_count: u8,
        runtime_flags: u8, frame: u32, water: i32, physics: &PhysicsTables,
        trig: &TrigTables, animation: &Family8Animation, directions: &Family8DirectionalAnimations) -> Result<Family8PassReceipt, ActorWorldError> {
        if player_count > 4 { return Err(ActorWorldError::InvalidPlayer(player_count)); }
        self.validate_links()?;
        self.validate_family8_count()?;
        let player = self.player(selected)?;
        let count = word(&self.backing, 0x355c) as u16;
        let period = word(&self.backing, 0x1c7a) as u16;
        let active_selector0 = self.family8.as_ref().unwrap().records().iter().any(|r|r[6]!=0 && r[0x55]==0);
        let countdown = countdown_expiry::plan_countdown_transition(count, period,
            player.progress(), player.bytes()[0x607], runtime_flags, active_selector0);
        let mut staged = self.clone();
        staged.backing[0x355c..0x355e].copy_from_slice(&countdown.after.to_le_bytes());
        let mut expiry_attempts = Vec::new();
        if countdown.expired {
            let players:Vec<_> = staged.players[..usize::from(player_count)].iter().map(|p|*p.bytes()).collect();
            let requests = countdown_expiry::plan_expiry_requests(&players, |at| {
                genewars_core::cd_candidate::interpolate_height(staged.map(),at[0],at[1])
                    .map_err(|e|ActorWorldError::Terrain(e.to_string()))
            }).map_err(|e|match e {
                countdown_expiry::ExpiryRequestError::HeightSample(e)=>e,
                countdown_expiry::ExpiryRequestError::TooManyPlayers=>ActorWorldError::InvalidPlayer(player_count),
            })?;
            for request in requests {
                let result = staged.create_family8(request.selector, request.owner, request.position, request.initial_height)?;
                expiry_attempts.push((request,result));
            }
        }
        let mut callbacks = Vec::new();
        // Read activity at each visit. A newly created higher slot is visited
        // in this same pass; taking a pre-pass active-slot snapshot is wrong.
        for slot in 0..16 {
            if staged.family8.as_ref().unwrap().record(slot)?[6] != 0 {
                callbacks.push(staged.step_family8_motion(slot as u16,frame,water,physics,trig,animation,directions)?);
            }
        }
        staged.validate_links()?;
        staged.validate_family8_count()?;
        *self = staged;
        Ok(Family8PassReceipt { countdown, expiry_attempts, callbacks,
            pending_prefix: countdown_expiry::transition_requests(countdown,selected),
            pending_ui_animations: countdown_expiry::animation_requests() })
    }

    /// One source callback with live allocation and shared RNG. Any unsupported
    /// target search rejects the enclosing pass and all earlier births.
    pub fn step_family8_motion(&mut self, slot:u16, frame:u32, water:i32,
        physics:&PhysicsTables, trig:&TrigTables, animation:&Family8Animation,
        directions:&Family8DirectionalAnimations) -> Result<Family8StepReceipt,ActorWorldError> {
        #[cfg(debug_assertions)] self.capture_numeric_callback(slot as usize,frame,"family8-before");
        if !self.validate_links()?.contains(&(8,slot as i16)) {return Err(ActorWorldError::MissingFamily8Link(slot));}
        self.validate_family8_count()?;
        let pool=self.family8.as_ref().unwrap();
        let before=*pool.record(usize::from(slot))?;
        let expected=if before[0x55]==1 {Family8AnimationBinding::Base}else{Family8AnimationBinding::Offset80};
        if pool.binding(usize::from(slot))? != Some(expected) {return Err(ActorWorldError::Family8Binding(slot));}
        let mut retirement_parent=None;let mut parent_response=None;let mut parent_retirement=false;
        let (plan,particle,target_required):(Family8StepPlan,_,_)=match (before[0x55],before[0x56]) {
            (2,10|11|12)=>{
                if before[0x49]&0x10==0&&frame.wrapping_add(u32::from(slot))&31==0{let _=self.family8_current_point_accepted(slot,&before)?;}
                (family8_motion::plan_zone_marker_wait(&before,self.map(),frame,water,physics,trig,animation,self.rng)?.into(),None,false)
            },
            (0,4)=>(family8_motion::plan_selector0_state4(&before,self.map(),frame,water,physics,trig,animation,self.rng)?.into(),None,false),
            (0,5)=>{
                let accepted=if frame.wrapping_add(u32::from(slot))&31==0 {
                    self.family8_current_point_accepted(slot,&before)?
                }else{true};
                let motion=if before[0x57]==0 {
                    family8_motion::plan_selector0_state5_wait(&before,self.map(),frame,water,physics,trig,animation,self.rng,accepted)?
                }else{
                    let player=self.player(before[24])?;
                    let counter=i32::from_le_bytes(player.bytes()[0x11..0x15].try_into().unwrap());
                    let response=family8_motion::plan_selector0_state5_notified(&before,self.map(),frame,water,physics,trig,animation,self.rng,accepted,counter,word(&self.backing,0x1c7c)as u16)?;
                    let motion=response.motion.clone();parent_response=Some(response);motion
                };
                (motion.into(),None,false)
            },
            (0,8)=>{
                let accepted=if frame.wrapping_add(u32::from(slot))&31==0 {
                    self.family8_current_point_accepted(slot,&before)?
                }else{true};
                (family8_motion::plan_selector0_state8_group_departure(&before,self.map(),frame,water,physics,trig,animation,self.rng,accepted,pool.records())?.into(),None,false)
            },
            (0,9)=>{
                let accepted=if frame.wrapping_add(u32::from(slot))&31==0 {
                    self.family8_current_point_accepted(slot,&before)?
                }else{true};
                let departed=family8_motion::plan_selector0_state9_departure(&before,self.map(),frame,water,physics,trig,animation,self.rng,accepted)?;
                parent_retirement=departed.retirement_requested;
                (Family8StepPlan{after:departed.after,rng_after:departed.rng_after,old_cell:departed.old_cell,new_cell:departed.new_cell,
                    child:None,pending_audio:departed.pending_audio,pending_ui_message:false,queue_requested:departed.queue_requested},None,false)
            },
            (1,0)|(1,1)=>{
                let onward=if before[0x56]==0 {family8_motion::plan_selector1_state0(&before,self.map(),frame,water,physics,trig,directions,self.rng)?}
                    else if word(&before,0x64)>0 {
                        let accepted=if frame.wrapping_add(u32::from(slot))&31==0{self.family8_current_point_accepted(slot,&before)?}else{true};
                        let buildings=self.owner_building_records(before[24])?;
                        family8_motion::plan_selector1_state1_hover(&before,self.map(),frame,water,physics,trig,directions,self.rng,&buildings,self.backing[0x34b8],accepted)?
                    }else{family8_motion::plan_selector1_state1_travel(&before,self.map(),frame,water,physics,trig,directions,self.rng)?};
                (onward.motion.into(),onward.particle,onward.target_selection_required)
            },
            (1,3)=>{
                let accepted=if before[0x49]&0x10==0&&frame.wrapping_add(u32::from(slot))&31==0 {
                    self.family8_current_point_accepted(slot,&before)?
                }else{true};
                let returned=family8_motion::plan_selector1_state3_return(&before,self.map(),frame,water,physics,trig,directions,self.rng,pool.records(),accepted)?;
                retirement_parent=returned.retirement_parent;
                (returned.onward.motion.into(),returned.onward.particle,returned.onward.target_selection_required)
            },
            _=>return Err(family8_motion::Family8MotionError::UnsupportedRecord.into()),
        };
        let mut staged=self.clone();
        if plan.old_cell!=plan.new_cell {staged.unlink_family8(slot)?;}
        staged.family8.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&plan.after)?;
        staged.rng=plan.rng_after;
        if let Some(response)=&parent_response {
            let player=*staged.player(before[24])?.bytes();let mut after=player;
            after[0x11..0x15].copy_from_slice(&response.player_counter_after.to_le_bytes());
            staged.players[usize::from(before[24])].replace_bookkeeping_checked(&player,&after)?;
        }
        if plan.old_cell!=plan.new_cell {staged.insert_family8_head(slot)?;}
        let child_created=if let Some(child)=plan.child {
            staged.create_family8(child.selector,child.owner,child.position,child.physical_height)?
        } else {None};
        let particle_created=if let Some(particle)=particle {
            let context=staged.mission_loading_physics();
            staged.create_particle(particle.code,0,particle.position,particle.physical_height,particle.angles,false,context,trig)?
        }else{None};
        let target_selection=if target_required {
            // The particle factory precedes114B0 and can consume its own draw.
            let current=*staged.family8.as_ref().unwrap().record(slot as usize)?;
            let owner=current[24];let player=staged.player(owner)?;
            let player_target=[word(player.bytes(),9),word(player.bytes(),11)];
            let buildings=staged.owner_building_records(owner)?;
            let target=family8_motion::plan_selector1_initial_empty_building_target(&current,staged.map(),player_target,&buildings,trig,staged.rng)?;
            staged.family8.as_mut().unwrap().replace_record_checked(slot as usize,&current,&target.after)?;
            staged.rng=target.rng_after;Some(target)
        }else{None};
        let retirement=if let Some(parent)=retirement_parent {
            // Source returns from103F0 before notifying the live parent. The
            // cleanup may change reference bytes, so re-read its current record.
            let retired=staged.retire_family8_child(slot)?;
            let current=*staged.family8.as_ref().unwrap().record(usize::from(parent))?;
            let mut after=current;after[0x57]=1;
            staged.family8.as_mut().unwrap().replace_record_checked(usize::from(parent),&current,&after)?;
            Some(retired)
        }else if parent_retirement {
            // This native host has no audio consumer, hence no playing38.
            Some(staged.retire_family8_parent(slot,false)?)
        }else{None};
        let rng_after=staged.rng;
        staged.validate_links()?;staged.validate_family8_count()?;*self=staged;
        #[cfg(debug_assertions)] self.capture_numeric_callback(slot as usize,frame,"family8-after");
        Ok(Family8StepReceipt{slot,plan,child_created,particle_created,target_selection,retirement,parent_notified:retirement_parent,parent_response,rng_after})
    }

    fn family8_current_point_accepted(&self,slot:u16,record:&Family8Record)->Result<bool,ActorWorldError>{
        let point=[word(record,7),word(record,9)];
        //74D90 queries the obstacle before terrain and8B130.
        let obstacle=self.move_obstacle_at(point)?;
        if record[0x53]&(1<<self.terrain_class_at(point)?)!=0 {return Ok(false);}
        if obstacle {return Ok(false);}
        //1EE10 virtual5C: mask0800 returns zero blocker weight immediately.
        if record[0x4a]&8!=0 {return Ok(true);}
        let own=[slot as u8,0,8];let cell=floor_cell(point);
        let mut raw=reference(self.map(),cell*14+6);let mut seen=HashSet::new();let mut weight=0;
        while raw[2]!=0 {
            if !seen.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
            let r=self.entity(raw)?;
            if raw!=own {
                match raw[2] {
                    1|3|5|8=>(),
                    2=>{let layout=self.layouts[1];let offset=layout.file_offset+word(&raw,0)as usize*layout.packed_stride;
                        if self.backing[offset+0x26]&8==0{weight+=10;}},
                    kind=>return Err(ActorWorldError::UnsupportedBlockerFamily(kind)),
                }
            }
            raw=reference(r,3);
        }
        Ok(weight<10)
    }

    pub(super) fn unlink_family8(&mut self,slot:u16)->Result<(),ActorWorldError> {
        let raw=[slot as u8,(slot>>8)as u8,8];let record=self.entity(raw)?;
        let previous=reference(record,0);let next=reference(record,3);
        let cell=floor_cell([word(record,7),word(record,9)]);
        if previous[2]==0 {
            if word(&previous,0)as usize!=cell||reference(self.map(),cell*14+6)!=raw {return Err(ActorWorldError::StaleLink);}
            if next[2]!=0 {self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&next);}
            else {self.backing[MAP_START+cell*14+8]=0;}
        } else {
            if reference(self.entity(previous)?,3)!=raw {return Err(ActorWorldError::StaleLink);}
            self.replace_next(previous,next)?;
        }
        if next[2]!=0 {
            if reference(self.entity(next)?,0)!=raw {return Err(ActorWorldError::StaleLink);}
            self.replace_previous(next,previous)?;
        }
        Ok(())
    }

    fn insert_family8_head(&mut self,slot:u16)->Result<(),ActorWorldError> {
        let raw=[slot as u8,(slot>>8)as u8,8];let record=self.entity(raw)?;
        let cell=floor_cell([word(record,7),word(record,9)]);let head=reference(self.map(),cell*14+6);
        let before=*self.family8.as_ref().unwrap().record(usize::from(slot))?;let mut after=before;
        after[..2].copy_from_slice(&(cell as u16).to_le_bytes());after[2]=0;
        if head[2]!=0 {self.entity(head)?;after[3..6].copy_from_slice(&head);self.replace_previous(head,raw)?;}
        else {after[5]=0;}
        self.family8.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&after)?;
        self.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&raw);Ok(())
    }

    /// Source10090 inactive-load mapping. Native pointer bytes and logical
    /// resource bindings are explicit priors; no process pointer is invented.
    pub fn admit_family8(&mut self, prior: &[Family8Record; 16],
        bindings: [Option<Family8AnimationBinding>; 16]) -> Result<(), ActorWorldError> {
        if self.family8.is_some() { return Err(ActorWorldError::Family8AlreadyAdmitted); }
        let group = self.layouts[7];
        let pool = Family8Pool::from_packed_inactive(
            &self.backing[group.file_offset..group.file_offset + group.count * group.packed_stride], prior, bindings)?;
        let mut staged = self.clone();
        staged.family8 = Some(Box::new(pool));
        staged.validate_family8_count()?;
        staged.validate_links()?;
        *self = staged;
        Ok(())
    }

    /// Source101D0 first-inactive allocation and 8AC60 intrusive-list insertion.
    /// A full, consistent pool returns None without changing any state.
    /// This factory does not consume RNG or execute an entity callback.
    pub fn create_family8(&mut self, selector: u8, owner: u8, position: [i16; 2],
        initial_height: i32) -> Result<Option<Family8CreationReceipt>, ActorWorldError> {
        if owner >= 4 { return Err(family8_creation::CreationError::InvalidOwner.into()); }
        if selector > 2 { return Err(family8_creation::CreationError::InvalidSelector.into()); }
        self.validate_links()?;
        self.validate_family8_count()?;
        let pool = self.family8.as_ref().ok_or(ActorWorldError::Family8NotAdmitted)?;
        let Some(slot) = pool.records().iter().position(|r| r[6] == 0) else { return Ok(None); };
        let before = *pool.record(slot)?;
        let ground_height = genewars_core::cd_candidate::interpolate_height(self.map(), position[0], position[1])
            .map_err(|e| ActorWorldError::Terrain(e.to_string()))?;
        let shade = genewars_core::cd_candidate::interpolate_entity_shade(self.map(), position[0], position[1])
            .map_err(|e| ActorWorldError::Terrain(e.to_string()))?;
        let cell = floor_cell(position);
        let old_raw = reference(self.map(), cell * 14 + 6);
        let old_head = if old_raw[2] == 0 { None } else {
            self.entity(old_raw)?;
            Some(CellHeadIdentity { index: word(&old_raw, 0) as u16, kind: old_raw[2] })
        };
        let plan = family8_creation::plan_creation(&before, CreationInput {
            slot: slot as u16, owner, selector, position, ground_height, shade, initial_height, old_head })?;
        let mut staged = self.clone();
        staged.family8.as_mut().unwrap().commit_creation(slot, &before, &plan)?;
        let raw = [slot as u8, 0, 8];
        if old_head.is_some() { staged.replace_previous(old_raw, raw)?; }
        staged.backing[MAP_START + cell * 14 + 6..MAP_START + cell * 14 + 9].copy_from_slice(&raw);
        staged.backing[0x3559] += 1;
        staged.validate_links()?;
        staged.validate_family8_count()?;
        let binding = staged.family8.as_ref().unwrap().binding(slot)?.unwrap();
        *self = staged;
        Ok(Some(Family8CreationReceipt { slot: slot as u16, selector, input_owner: owner,
            position, initial_height, ground_height, shade, binding }))
    }

    pub(super) fn validate_family8_count(&self) -> Result<(), ActorWorldError> {
        let pool = self.family8.as_ref().ok_or(ActorWorldError::Family8NotAdmitted)?;
        let declared = self.backing[0x3559];
        let active = pool.records().iter().filter(|r| r[6] != 0).count();
        if usize::from(declared) != active { return Err(ActorWorldError::Family8CountMismatch { declared, active }); }
        Ok(())
    }
}

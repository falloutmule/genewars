//! Explicit native landing study over the owned world, not a complete game
//! scheduler. Caller seed/zero native priors and frozen remaining families are
//! visible development policies. Real source factories activate the actors.
use std::path::Path;
use genewars_assets::{ConqMissionSource,TextStringTable};
use genewars_core::cd_candidate::{interpolate_height,interpolate_entity_shade,camera_cells_for_target};
use genewars_runtime::{actor_world::{ActorWorld,LandingEffectCallback,GroundMoveInput,GroundMoveOutcome,ConstructionAwareActorCallback},
    actor_pool::ActorHandleBindings,blueprint::{BlueprintNames,BlueprintRng},
    player_state::{PlayerState,ActorOriginSample},actor_activation::TrigTables,
    creature_world::MuleStepContext,plant_spread::SpreadDirections,
    landing_effect_lifecycle::RetirementUiState,terrain_target::project_terrain_target};
use super::{runtime_resources::{Resources,Frame,read_bound},actor_scene};

// The desktop debug binary has a small main-thread stack on Windows. Keep
// large fixed-size native-prior fixtures off the stack while assembling the
// explicit landing study. These records are zero priors, not source data.
fn zero_record_box<const N:usize,const M:usize>() -> Box<[[u8;M];N]> {
    let boxed:Box<[[u8;M]]>=vec![[0;M];N].into_boxed_slice();
    boxed.try_into().unwrap_or_else(|_|unreachable!("fixed record count"))
}

#[derive(Clone)]
pub struct SharedMission {
    pub world:ActorWorld,
    trig:TrigTables,
    plant_spread:SpreadDirections,
    ui:RetirementUiState,
    terrain_jobs:genewars_runtime::terrain_job_pool::TerrainJobPool,
    pub frame_counter:u64,
    pub selected:Option<usize>,
    pub command_count:u64,
    pub pending_requests:u64,
    pub plant_births:u64,
    pub seed:u16,
    pub runtime_context:Option<genewars_runtime::mission_context::MissionRuntimeContext>,
    awaiting_presentation:bool,
    pub feedback:Option<String>,
    pub ui_animations:[[u8;5];8],
    dispatch_audit:super::actor_dispatch_audit::ActorDispatchAudit,
}
impl SharedMission {
    pub fn load(root:&Path,mission_bytes:&[u8],seed:u16)->Result<Self,String> {
        Self::load_internal(root,mission_bytes,seed,None,None)
    }
    /// Recovered fresh CD pipeline; opaque source stack bytes remain explicit inputs.
    pub fn load_fresh_cd(root:&Path,mission_bytes:&[u8],scratch:[u8;36],diagnostic_ring:u8)->Result<Self,String>{
        Self::load_internal(root,mission_bytes,5428,Some((scratch,diagnostic_ring)),None)
    }
    /// Successor load consumes source-projected carry; no regenerated blueprints or seed override.
    pub fn load_carried_cd(root:&Path,mission_bytes:&[u8],carry:&genewars_runtime::result_carry::CampaignCarryPlan,
        world_priors:&[u8;150],ring:u8,mode:u8,ui_animations:[[u8;5];8])->Result<Self,String>{
        if carry.players[0][0x6e7]!=1||carry.players[0][0x15]==0{return Err("Krell requires first-mission progress and retained blueprints".into());}
        let mut mission=Self::load_internal(root,mission_bytes,47117,None,Some((carry,world_priors,ring,mode)))?;
        mission.ui_animations=ui_animations;Ok(mission)
    }
    fn load_internal(root:&Path,mission_bytes:&[u8],seed:u16,fresh:Option<([u8;36],u8)>,
        carried:Option<(&genewars_runtime::result_carry::CampaignCarryPlan,&[u8;150],u8,u8)>)->Result<Self,String>{
        let reference=fresh.is_some()||carried.is_some();
        let source=ConqMissionSource::parse(mission_bytes).map_err(|e|e.to_string())?;
        let target=source.target_words(0).map_err(|e|e.to_string())?.map(|v|v as i16);
        let players=if reference{
            let retained=if let Some((carry,_,_,_))=carried{PlayerState::from_record(&carry.players[0]).map_err(|e|e.to_string())?}else{PlayerState::fresh_campaign(0,target,b"Player\0").map_err(|e|e.to_string())?};
            let mut players=core::array::from_fn(|p|PlayerState::from_record(&mission_bytes[20+p*1805..20+(p+1)*1805]).unwrap());
            players[0]=PlayerState::restored_for_mission(&mission_bytes[20..1825],0,&retained).map_err(|e|e.to_string())?;players
        }else{core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,target,b"Native study\0").unwrap())};
        let actor_priors=zero_record_box::<20,200>();
        let mut world=if let Some((carry,world_priors,ring,_))=carried{
            let globals=source.tail_globals();
            let loaded_seed=u16::from_le_bytes(globals[26..28].try_into().unwrap());
            let draws=u32::from_le_bytes(globals[18..22].try_into().unwrap());
            if source.flags_byte()!=0x38||source.active_count_byte()!=1||loaded_seed!=47117||draws!=33620{return Err("Krell runtime block differs from admitted source".into());}
            ActorWorld::from_carried_cd_mission(&source,&carry.actors,players,BlueprintRng{seed:loaded_seed,ring,draw_count:draws},world_priors)
        }else{ActorWorld::from_mission(&source,&actor_priors,[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed,ring:fresh.map_or(0,|(_,ring)|ring),draw_count:if fresh.is_some(){16855}else{0}})}.map_err(|e|e.to_string())?;
        world.initialize_mission_player_funds(source.active_count_byte()).map_err(|e|e.to_string())?;
        let building_priors=zero_record_box::<256,67>();
        world.admit_buildings(&building_priors,[None;256]).map_err(|e|e.to_string())?;
        // Preserve every source job byte, including opaque inactive data.
        let job_records=core::array::from_fn(|slot|mission_bytes[0x1ec7+176*slot..0x1ec7+176*(slot+1)].try_into().unwrap());
        let terrain_jobs=genewars_runtime::terrain_job_pool::TerrainJobPool::admit(job_records,mission_bytes[0x2ff7])
            .map_err(|e|format!("terrain-job admission: {e:?}"))?;
        let creature_priors=zero_record_box::<300,124>();
        world.admit_creatures(&creature_priors).map_err(|e|e.to_string())?;
        let effect_priors=zero_record_box::<200,81>();
        world.admit_effects(&effect_priors,[None;200]).map_err(|e|e.to_string())?;
        let family8_priors=zero_record_box::<16,102>();
        world.admit_family8(&family8_priors,[None;16]).map_err(|e|e.to_string())?;
        world.admit_plants(&read_bound(&root.join("plant-parameters.bin"))?).map_err(|e|e.to_string())?;
        let text=read_bound(&root.join("menu/text.dat"))?;
        let strings=TextStringTable::parse(&text).map_err(|e|e.to_string())?;
        let names=BlueprintNames::new(&strings.strings()[..128]).map_err(|e|e.to_string())?;
        if carried.is_none(){world.regenerate_player_blueprints(0,fresh.map_or([0;36],|(scratch,_)|scratch),&names,true).map_err(|e|e.to_string())?;}
        let origin=ActorOriginSample{position:[0,0],height:interpolate_height(world.map(),0,0).map_err(|e|e.to_string())?,
            shade:interpolate_entity_shade(world.map(),0,0).map_err(|e|e.to_string())?};
        world.prepare_fresh_player(0,if reference{mission_bytes[0x1ebd]}else{5},origin,target).map_err(|e|e.to_string())?;
        let factory=world.create_landing_core(0,0).map_err(|e|e.to_string())?;
        let mut pending_requests=0;
        for event in &factory.pending_events {
            if let genewars_runtime::landing::LandingEvent::Reveal{position,radius}=event {
                if *radius!=4096 {return Err("unsupported reveal radius".into());}
                let reveal=world.apply_reveal_4096(*position).map_err(|e|e.to_string())?;
                pending_requests+=reveal.pending_presentation_vertices.len()as u64;
            } else {pending_requests+=1;}
        }
        let runtime_context=if reference{
            let context=if let Some((_,_,_,mode))=carried{genewars_runtime::mission_context::MissionRuntimeContext::carried_cd_krell(world.rng(),mode)}else{genewars_runtime::mission_context::MissionRuntimeContext::fresh_cd_conq001(world.rng())}.map_err(str::to_owned)?;
            world.initialize_fresh_runtime_countdown(&context).map_err(str::to_owned)?;
            let visibility=world.initialize_mission_visibility(0).map_err(|e|format!("startup visibility: {e:?}"))?;
            pending_requests+=(visibility.reveal.pending_presentation_vertices.len()+visibility.zones.viewport_invalidations.len())as u64;
            //7F950 has no numeric/RNG work when all four categories remain present.
            let player=world.player(0).map_err(|e|e.to_string())?;
            if !(0..4u8).all(|category|(0..32).any(|index|{let b=player.blueprint(index).unwrap();b[0]!=0&&b[0x16]==category})){
                return Err("source7F950 missing-category backfill is not implemented".into());
            }
            Some(context)
        }else{None};
        let plant_spread=SpreadDirections::from_mapped_bytes(&read_bound(&root.join("physics/plant-spread.bin"))?).map_err(|e|e.to_string())?;
        Ok(Self{world,trig:TrigTables::generate_pinned().map_err(|e|e.to_string())?,plant_spread,ui:RetirementUiState::native_empty(),
            terrain_jobs,frame_counter:0,selected:None,command_count:0,pending_requests,plant_births:0,seed,runtime_context,awaiting_presentation:false,feedback:None,ui_animations:[genewars_runtime::ui_animation::FRESH_SOURCE_STATE;8],dispatch_audit:Default::default()})
    }
    /// Typed terminal state; an engine block remains a separate host field.
    pub fn outcome_json(&self)->String {
        use genewars_runtime::mission_outcome::MissionOutcome;
        match self.runtime_context.as_ref().map(|c|c.outcome()) {
            None=>"null".into(),
            Some(MissionOutcome::Running)=>"{\"state\":\"running\"}".into(),
            Some(MissionOutcome::Won{player})=>format!("{{\"state\":\"won\",\"player\":{player}}}"),
            Some(MissionOutcome::Lost{player,winner})=>format!("{{\"state\":\"lost\",\"player\":{player},\"winner\":{}}}",winner.map_or("null".into(),|v|v.to_string())),
        }
    }
    /// Called only after successful rendering of the complete callback frame.
    pub fn finish_outer_frame(&mut self){
        if self.awaiting_presentation{let countdown=u16::from_le_bytes(self.world.backing_bytes()[0x355c..0x355e].try_into().unwrap());let context=self.runtime_context.as_mut().unwrap();context.observe_countdown_after_callbacks(countdown);context.finish_outer_frame();self.awaiting_presentation=false;}
    }
    pub fn render(&self,resources:&Resources,target:[i16;2])->Result<Frame,String> {
        // Until original admission/order is integrated, only linked active
        // actors and active effects are drawn in ascending slot order.
        let actors=self.world.actors().records().iter().enumerate().filter_map(|(slot,r)|
            (r[6]==3&&r[0x70]&1!=0).then_some(slot)).collect::<Vec<_>>();
        let effects=self.world.effects().ok_or("effects not admitted")?.records().iter().enumerate()
            .filter_map(|(slot,r)|(r[6]!=0).then_some(slot)).collect::<Vec<_>>();
        let creatures=self.world.creatures().map(|pool|pool.records().iter().enumerate()
            .filter_map(|(slot,r)|(r[6]==1).then_some(slot)).collect::<Vec<_>>()).unwrap_or_default();
        let mut frame=actor_scene::render_actor_world_with_selected_portrait_and_zone_ui(resources,&self.world,target,0,self.runtime_context.as_ref().map_or(self.frame_counter as u8,|c|c.frame()as u8),&actors,&creatures,
            actor_scene::MuleSceneGlobals{global_shadow_mode:0,inherited_wrapper_flags:0,clip_flag_bit0:false},&effects,self.selected,self.ui_animations.map(|state|state[1]))?.frame;
        if let Some(slot)=self.selected {
            actor_scene::draw_selected_actor_text(resources,&self.world,slot,&mut frame.image)?;
            frame.rgb_sha256=genewars_runtime::identity::sha256_hex(frame.image.rgb_bytes());
        }
        // The job overlay is authoritative and is not mirrored into backing.
        // Include it in replay identity; this remains a diagnostic, not a save.
        let mut identity=b"GeneWars SharedMission jobs and countdown UI v2\0".to_vec();
        identity.extend_from_slice(frame.world_sha256.as_bytes());
        identity.push(self.terrain_jobs.count());
        for record in self.terrain_jobs.records(){identity.extend_from_slice(record);}
        identity.extend(self.ui_animations.concat());
        frame.world_sha256=genewars_runtime::identity::sha256_hex(&identity);
        Ok(frame)
    }
    /// Read-only receipt for the owned terrain-job pool, used by development
    /// acceptance checks; no task or job state is synthesized by this view.
    pub fn active_terrain_job_count(&self)->u8 { self.terrain_jobs.count() }
    pub fn terrain_job_records(&self)->&[[u8;176];25] {
        self.terrain_jobs.records()
    }
    pub fn player_funds_json(&self)->String {
        let entries=(0..4u8).map(|slot| {
            let funds=i32::from_le_bytes(self.world.player(slot).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap());
            format!("{{\"slot\":{slot},\"balance\":{funds}}}")
        }).collect::<Vec<_>>().join(",");
        format!("[{entries}]")
    }
    pub fn building_state_json(&self)->String {
        let word=|r:&[u8],p:usize|i16::from_le_bytes([r[p],r[p+1]]);
        let entries=self.world.buildings().into_iter().flat_map(|pool|pool.records().iter().enumerate())
            .filter(|(_,r)|r[6]==4).map(|(slot,r)|
                format!("{{\"slot\":{slot},\"owner\":{},\"selector\":{},\"position\":[{},{}],\"animation\":{},\"frame\":{},\"progress_word_2b\":{},\"phase_byte_2f\":{},\"flags_word_31\":{}}}",
                    r[24],r[0x2d],word(r,7),word(r,9),r[0x19],r[0x1b],word(r,0x2b),r[0x2f],u16::from_le_bytes([r[0x31],r[0x32]])))
            .collect::<Vec<_>>().join(",");
        format!("[{entries}]")
    }
    pub fn terrain_job_state_json(&self)->String {
        let entries=self.terrain_jobs.records().iter().enumerate().filter(|(_,r)|i16::from_le_bytes([r[12],r[13]])>=0)
            .map(|(slot,r)|format!("{{\"slot\":{slot},\"raw\":\"{}\"}}",r.iter().map(|b|format!("{b:02x}")).collect::<String>()))
            .collect::<Vec<_>>().join(",");
        format!("{{\"count\":{},\"records\":[{entries}]}}",self.terrain_jobs.count())
    }
    pub fn step(&mut self,resources:&Resources)->Result<String,String> {
        let mut staged=self.clone();let event=staged.step_inner(resources)?;*self=staged;Ok(event)
    }
    fn step_inner(&mut self,resources:&Resources)->Result<String,String> {
        if self.awaiting_presentation{return Err("finish the previous frame presentation before another callback".into());}
        let player_count=self.runtime_context.as_ref().map_or(4,|c|c.participating_players());
        let runtime_flags=self.runtime_context.as_ref().map_or(0,|c|c.known_runtime_flags());
        if self.runtime_context.as_ref().is_some_and(|c|c.callback_gate()!=genewars_runtime::mission_context::CallbackGate::Ready){return Err("mission callback gate is not ready".into());}
        let context=self.world.mission_loading_physics();
        let frame=self.runtime_context.as_ref().map_or_else(||u32::try_from(self.frame_counter+1).map_err(|_|"source frame counter exceeds u32"),|c|Ok(c.frame()))?;
        let ready_before: [bool;20]=std::array::from_fn(|slot|{let actor=&self.world.actors().records()[slot];actor[6]==3&&actor[0x70]&1!=0});
        let mut dispatch_observations=Vec::new();
        //1C760 runs7A280 before actor/building/creature callbacks.
        self.world.step_admitted_zone_callbacks(frame).map_err(|e|format!("zone pass: {e:?}"))?;
        // 58780 visits players and their current actor lists. 57A10 returns
        // immediately before the ready bit is set by the landing callback.
        let actors=self.world.step_actor_callbacks_with_construction_observed(player_count,frame,context,&resources.physics,&self.trig,
            resources.actors.databases(),&mut self.terrain_jobs,&resources.construction_border,
            0,self.selected.map(|slot|slot as u16),0,0,|event|dispatch_observations.push(event.clone())).map_err(|e|format!("actor pass: {e:?}"))?;
        let callbacks=actors.len();
        for callback in &actors {match callback {
            ConstructionAwareActorCallback::Ordinary(r)=>self.pending_requests+=u64::from(r.acknowledgement.is_some())+r.pending_events.len()as u64,
            ConstructionAwareActorCallback::Architect{receipt:r,..}=>{
                if r.rejected_site_message_index==Some(12){self.feedback=Some("Construction site blocked - choose another site".into());self.pending_requests+=1;}
                self.pending_requests+=r.motion.pending_events.len()as u64;
                self.pending_requests+=r.setup.as_ref().map_or(0,|p|p.viewport_invalidations.len()as u64+1);
                self.pending_requests+=r.terraform.as_ref().map_or(0,|p|p.viewport_invalidations.len()as u64+u64::from(p.pending_audio.is_some()));
                self.pending_requests+=r.terraform_wait.as_ref().map_or(0,|p|u64::from(p.pending_audio.is_some()));
                self.pending_requests+=r.cancellation.as_ref().map_or(0,|p|u64::from(p.panel_refresh_pending));
                self.pending_requests+=u64::from(r.building_effect.is_some());
                self.pending_requests+=u64::from(r.completion_message_index.is_some());
                self.pending_requests+=r.progress.as_ref().map_or(0,|p|u64::from(p.pending_audio_4d)+u64::from(p.ui_refresh_required));
            },
        }}
        // Audio is still a queued presentation request in this development host.
        // No active mixer channel is asserted; mission acceptance requires audio integration.
        let buildings=self.world.step_admitted_buildings_with_production(player_count,0,&resources.bturr_selector1,&resources.source_buildings,
            frame,&resources.physics,|_|false,context,&self.trig,&resources.construction_border).map_err(|e|e.to_string())?;
        self.pending_requests+=buildings.notices.len()as u64;
        for callback in &buildings.callbacks {
            use genewars_runtime::actor_world::AdmittedBuildingCallback;
            match callback {
                AdmittedBuildingCallback::Incomplete{plan,..}=>self.pending_requests+=u64::from(plan.stop_audio_23)+u64::from(plan.stop_audio_2f),
                AdmittedBuildingCallback::GenepodIdle{plan,..}=>self.pending_requests+=u64::from(plan.queue_requested),
                AdmittedBuildingCallback::GenepodWork{plan,..}=>self.pending_requests+=u64::from(plan.queue_requested),
                AdmittedBuildingCallback::GenepodTerminal{receipt}=>self.pending_requests+=u64::from(receipt.queue_requested)+u64::from(receipt.sound_requested.is_some())+u64::from(receipt.randomized_sound_requested.is_some()),
                AdmittedBuildingCallback::Turret{plan,..}=>self.pending_requests+=u64::from(plan.audio.is_some()),
                AdmittedBuildingCallback::Extractor{plan,..}=>self.pending_requests+=plan.audio.len()as u64,
                AdmittedBuildingCallback::PowerProducer{receipt}=>self.pending_requests+=receipt.numeric.audio.len()as u64,
                AdmittedBuildingCallback::ProducingExtractor{receipt}=>self.pending_requests+=(receipt.plan.state.audio.len()+receipt.terrain_invalidations.len())as u64,
            }
        }
        let climate=i16::from_le_bytes(self.world.backing_bytes()[0x1c61..0x1c63].try_into().unwrap());
        let creatures=self.world.step_creature_callbacks(player_count,0,u16::from(runtime_flags),
            MuleStepContext{frame_counter:frame,gravity:context.gravity,water_height:context.water_height,climate},
            &resources.physics,&self.trig,&resources.mule_animations).map_err(|e|e.to_string())?;
        self.pending_requests+=(creatures.pending_ui_animations.len()+creatures.pending_audio.len()+creatures.pending_plant_retirements.len()+creatures.pending_portrait_refreshes.len()+creatures.pending_reveal_vertices.len())as u64;
        self.pending_requests+=creatures.retirements.iter().map(|r|1+r.pending_ui_cleanup.len()as u64).sum::<u64>();
        let plants=self.world.tick_plants_with_reproduction(&self.plant_spread).map_err(|e|e.to_string())?;
        self.plant_births+=plants.events.iter().filter(|event|matches!(event,genewars_runtime::TickEvent::Created{..})).count()as u64;
        self.pending_requests+=plants.events.iter().filter(|event|matches!(event,
            genewars_runtime::TickEvent::OldFrameDirtyCheckPending{..}|genewars_runtime::TickEvent::ReferenceUiAudioCleanupPending{..})).count()as u64;
        let countdown=self.world.step_family8_callbacks(0,player_count,runtime_flags,frame,context.water_height,
            &resources.physics,&self.trig,&resources.family8_animation,&resources.family8_directions).map_err(|e|e.to_string())?;
        self.pending_requests+=(countdown.pending_prefix.len()+countdown.pending_ui_animations.len())as u64;
        self.pending_requests+=countdown.callbacks.iter().map(|c|u64::from(c.plan.pending_audio.is_some())+u64::from(c.plan.pending_ui_message)+c.retirement.as_ref().map_or(0,|r|r.stop_audio.len()as u64+u64::from(r.fade_audio.is_some()))+u64::from(c.parent_response.as_ref().is_some_and(|p|p.state7_message_variant.is_some()))).sum::<u64>();
        for (state,descriptor) in self.ui_animations.iter_mut().zip(&resources.countdown_animations){
            *state=genewars_runtime::ui_animation::step(*state,descriptor).map_err(|e|format!("countdown UI animation: {e:?}"))?.after;
        }
        self.world.step_empty_families_7_and_6().map_err(|e|e.to_string())?;
        #[cfg(debug_assertions)]
        if std::env::var("GENEWARS_EFFECT_PASS_FRAME").ok().and_then(|v|v.parse::<u32>().ok())==Some(frame){
            if let Some(path)=std::env::var_os("GENEWARS_EFFECT_PASS_FIXTURE"){
                let dir=std::path::PathBuf::from(path);std::fs::create_dir(&dir).expect("fresh immutable pre-effect fixture");
                for(name,bytes)in [("backing",self.world.backing_bytes().to_vec()),("players",(0..4u8).flat_map(|p|self.world.player(p).unwrap().bytes().iter().copied()).collect()),("actors",self.world.actors().records().concat()),("creatures",self.world.creatures().unwrap().records().concat()),("effects",self.world.effects().unwrap().records().concat()),("family8",self.world.family8().unwrap().records().concat()),("buildings",self.world.buildings().unwrap().records().concat())]{std::fs::write(dir.join(format!("{name}.bin")),bytes).unwrap();}
                let rng=self.world.rng();let mut bytes=rng.seed.to_le_bytes().to_vec();bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());std::fs::write(dir.join("rng.bin"),bytes).unwrap();
                std::fs::write(dir.join("context.txt"),format!("source frame{frame}; {context:?}; UI{:?}",self.ui)).unwrap();
            }
        }
        let result=self.world.step_landing_effect_callbacks(context,&resources.physics,&self.trig,&self.ui).map_err(|e|e.to_string())?;
        self.ui=result.ui_after;
        //1C120 runs79220 after effects. Initial light is the pinned mapped
        // candidate vector106604/608/60C; later light changes remain unproved.
        let backing=self.world.backing_bytes();
        let word=|offset|i16::from_le_bytes([backing[offset],backing[offset+1]]);
        let environment=genewars_runtime::construction_maintenance::TerrainEnvironment{
            climate_center:[word(0x1ea2),word(0x1ea4)],raw_water_height:word(0x1c63),
            climate:word(0x1c61),light:[181,0,-181]};
        let terrain=self.world.step_terrain_jobs(&mut self.terrain_jobs,environment,&resources.construction_border)
            .map_err(|e|format!("terrain-job pass: {e:?}"))?;
        self.pending_requests+=terrain.iter().map(|r|r.viewport_invalidations.len()as u64).sum::<u64>();
        let mut outcome_notice=String::new();
        //1C120 tail evaluates72ED0 after terrain maintenance, using this
        //frame's live context. A latched result is retained on later frames.
        if let Some(context)=self.runtime_context.as_mut(){
            if context.outcome()==genewars_runtime::mission_outcome::MissionOutcome::Running {
                let plan=self.world.step_mission_outcome(context,0).map_err(|e|format!("mission-outcome pass: {e:?}"))?;
                if plan.outcome!=genewars_runtime::mission_outcome::MissionOutcome::Running{outcome_notice=format!("; mission outcome {:?} at source frame {}",plan.outcome,context.frame());}
            }
        }
        let mut activations=Vec::new();
        for (slot,was_ready) in ready_before.into_iter().enumerate(){
            let actor=&self.world.actors().records()[slot];
            if !was_ready&&actor[6]==3&&actor[0x70]&1!=0{
                let snapshot=genewars_runtime::actor_world::ActorDispatchSnapshot::capture(&self.world,slot as u16).map_err(|e|format!("activation audit: {e:?}"))?;
                activations.push(super::actor_dispatch_audit::ActivationObservation{frame,actor_id:slot as u16,snapshot});
            }
        }
        self.dispatch_audit.append(dispatch_observations,activations);
        self.frame_counter+=1;
        self.awaiting_presentation=self.runtime_context.is_some();
        let(mut created,mut retired)=(0,0);
        for callback in result.primary.iter().chain(&result.newborn) {
            match callback {
                LandingEffectCallback::Landing(e)=>{
                    created+=usize::from(e.created_effect.is_some());retired+=usize::from(e.retirement.is_some());
                    self.pending_requests+=e.pending_events.len()as u64;
                    if let Some(r)=&e.retirement {self.pending_requests+=r.pending_ui_events.len()as u64;}
                },
                LandingEffectCallback::Smoke(e)|LandingEffectCallback::DepartureParticle(e)|LandingEffectCallback::WaterSplash(e)|LandingEffectCallback::EtherealParticle(e)|LandingEffectCallback::Construction(e)|LandingEffectCallback::Power(e)|LandingEffectCallback::GenepodFailure(e)=>{
                    created+=usize::from(e.created_effect.is_some());retired+=usize::from(e.retired);self.pending_requests+=e.pending_ui_events.len()as u64;
                },
            }
        }
        Ok(format!("world pass {}; actors {callbacks}; created {created}; retired {retired}; active {}; RNG draws {}; bounded world callbacks{outcome_notice}",
            self.frame_counter,result.queue_candidates.len(),self.world.rng().draw_count))
    }
    pub fn select(&mut self,slot:usize)->Result<(),String> {
        let actor=self.world.actors().record(slot).map_err(|e|e.to_string())?;
        if actor[6]!=3||actor[24]!=0||actor[0x70]&1==0 {return Err("actor is not active for this player".into());}
        self.selected=Some(slot);Ok(())
    }
    /// Development-only opcode-5 assignment for the selected, already-arrived
    /// architect. Selector 1 is the only admitted construction resource in
    /// this host. The source assignment planner owns the state/category and
    /// reinitialization guards; no site coordinates or readiness are supplied.
    pub fn assign_selected_bturr(&mut self)->Result<genewars_runtime::actor_construction::ArchitectAssignmentPlan,String> {
        let slot=self.selected.ok_or("select an architect first")?;
        let mut staged=self.world.clone();
        let plan=staged.assign_architect_construction(
            u16::try_from(slot).map_err(|_|"selected architect slot is outside the native range")?,
            0,0x0100,true,true).map_err(|e|format!("{e:?}"))?;
        self.world=staged;
        Ok(plan)
    }
    /// Source43960 menu admission followed by the shared opcode5 command.
    /// This does not change the development host's startup provenance.
    pub fn assign_selected_building(&mut self,selector:u8)->Result<genewars_runtime::actor_construction::ArchitectAssignmentPlan,String> {
        let slot=self.selected.ok_or("select an architect first")?;
        let owner=self.world.actors().record(slot).map_err(|e|e.to_string())?[24];
        let player=self.world.player(owner).map_err(|e|e.to_string())?;
        if !genewars_runtime::building_availability::available(player.bytes(),selector).map_err(|e|format!("{e:?}"))? {
            return Err(format!("building selector {selector} is not available at this campaign progress and prerequisite state"));
        }
        let mut staged=self.world.clone();
        let plan=staged.assign_architect_construction(u16::try_from(slot).map_err(|_|"actor slot outside native range")?,
            0,u16::from(selector)<<8,true,true).map_err(|e|format!("{e:?}"))?;
        self.world=staged;Ok(plan)
    }
    /// Source26890 ground construction:75080 snap, then opcode16/811A0.
    pub fn place_selected_building(&mut self,resources:&Resources,target:[i16;2],screen:[i32;2],selector:u8)->Result<String,String>{
        let slot=self.selected.ok_or("select an architect first")?;
        if screen[0]<128{return Err("point is inside the panel".into());}
        let owner=self.world.actors().record(slot).map_err(|e|e.to_string())?[24];
        let player=self.world.player(owner).map_err(|e|e.to_string())?;
        if !genewars_runtime::building_availability::available(player.bytes(),selector).map_err(|e|format!("{e:?}"))?{return Err("building is not available".into());}
        let camera=camera_cells_for_target(self.world.map(),target[0],target[1]).map_err(|e|e.to_string())?;
        let projected=project_terrain_target(self.world.map(),camera,screen).map_err(|e|e.to_string())?.point;
        let point=genewars_runtime::actor_construction_placement::construction_placement_point(projected,selector).map_err(|e|format!("{e:?}"))?;
        self.world.place_architect_construction(owner,u16::try_from(slot).map_err(|_|"actor slot outside native range")?,point,selector,&resources.physics,&self.trig,true,true).map_err(|e|format!("{e:?}"))?;
        self.command_count+=1;self.feedback=None;
        Ok(format!("actor {slot}, selector {selector}, projected {projected:?}, placement {point:?}"))
    }
    pub fn actor_dispatch_audit_json(&self)->String {self.dispatch_audit.report(std::env::var("GENEWARS_ACTOR_DISPATCH_TRACE").as_deref()==Ok("1"))}
    pub fn actor_state_json(&self)->String {
        let word=|r:&[u8],p:usize|i16::from_le_bytes([r[p],r[p+1]]);
        let entries=self.world.actors().records().iter().enumerate().filter(|(_,r)|r[6]==3).map(|(slot,r)|
            format!("{{\"slot\":{slot},\"owner\":{},\"position\":[{},{}],\"destination\":[{},{}],\"pending_destination\":[{},{}],\"state\":{},\"ready\":{},\"pending_move\":{},\"assignment_pending\":{},\"building_selector\":{},\"animation\":{},\"frame\":{},\"action_byte_59\":{},\"construction_stage_word_88\":{},\"job_slot_byte_90\":{}}}",
                r[24],word(r,7),word(r,9),word(r,11),word(r,13),word(r,0x5c),word(r,0x5e),r[0x55],r[0x70]&1!=0,r[0x57]&2!=0,r[0x82]!=0,r[0x83],r[0x19],r[0x1b],r[0x59],word(r,0x88),r[0x90]))
            .collect::<Vec<_>>().join(",");
        format!("[{entries}]")
    }
    pub fn creature_state_json(&self)->String {
        let word=|r:&[u8],p:usize|i16::from_le_bytes([r[p],r[p+1]]);
        let entries=self.world.creatures().unwrap().records().iter().enumerate().filter(|(_,r)|r[6]!=0).map(|(slot,r)|
            format!("{{\"slot\":{slot},\"owner\":{},\"species\":{},\"position\":[{},{}],\"destination\":[{},{}],\"state\":{},\"animation\":{},\"frame\":{},\"reserve\":{},\"target_kind\":{}}}",
                r[24],r[0x7a],word(r,7),word(r,9),word(r,11),word(r,13),r[0x5e],r[0x19],r[0x1b],word(r,0x68),r[0x57]))
            .collect::<Vec<_>>().join(",");
        format!("[{entries}]")
    }
    pub fn family8_state_json(&self)->String{
        let entries=self.world.family8().unwrap().records().iter().enumerate().filter(|(_,r)|r[6]!=0).map(|(slot,r)|
            format!("{{\"slot\":{slot},\"selector\":{},\"state\":{},\"owner\":{},\"flags\":{},\"animation\":{},\"frame\":{}}}",r[0x55],r[0x56],r[24],u16::from_le_bytes([r[0x15],r[0x16]]),r[0x19],r[0x1b])).collect::<Vec<_>>().join(",");
        format!("[{entries}]")
    }
    pub fn select_next(&mut self)->Result<(),String> {
        let slots=self.world.actors().records().iter().enumerate().filter_map(|(i,r)|
            (r[6]==3&&r[24]==0&&r[0x70]&1!=0).then_some(i)).collect::<Vec<_>>();
        if slots.is_empty(){return Err("landing has not activated an actor".into());}
        let index=self.selected.and_then(|s|slots.iter().position(|&i|i==s)).map_or(0,|i|(i+1)%slots.len());
        self.select(slots[index])
    }
    pub fn order(&mut self,resources:&Resources,target:[i16;2],screen:[i32;2])->Result<String,String> {
        let slot=self.selected.ok_or("select an active actor first")?;
        if screen[0]<128 {return Err("point is inside the panel".into());}
        let camera=camera_cells_for_target(self.world.map(),target[0],target[1]).map_err(|e|e.to_string())?;
        let point=project_terrain_target(self.world.map(),camera,screen).map_err(|e|e.to_string())?.point;
        let input=GroundMoveInput{buttons:8,modifiers:0,pause_flags:0,group_word_526:0,group_byte_52f:0,
            current_portrait:slot as i8,selected:[slot as u8,0,3],projected_point:point};
        let result=self.world.dispatch_ground_move_searched(0,input,&resources.physics,&self.trig).map_err(|e|e.to_string())?;
        if matches!(result,GroundMoveOutcome::Dispatched{..}) {self.command_count+=1;}
        Ok(format!("actor {slot} target {point:?}: {result:?}"))
    }
}

#[cfg(test)] mod carried_startup_tests {
    use super::*;
    #[test]fn krell_first_256_frames_render_with_original_mule_cell_entry(){
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let first=read_bound(&root.join("terrain/CONQ001.DAT")).unwrap();let initial=SharedMission::load_fresh_cd(&root,&first,[0xa5;36],17).unwrap();
        let mut carry=initial.world.plan_campaign_carry(&[0;600],0x7e,2,0,None).unwrap();carry.players[0][0x6e7]=1;
        let second=read_bound(&root.join("terrain/CONQ002.DAT")).unwrap();
        let mut loaded=SharedMission::load_carried_cd(&root,&second,&carry,&[0;150],17,0,initial.ui_animations).unwrap();
        let (resources,_,_)=Resources::load_mission(&root,genewars_runtime::identity::CdCampaignMission::Krell).unwrap();
        let before=loaded.world.clone();let prior_ui=loaded.ui_animations;
        if let Some(path)=std::env::var_os("GENEWARS_KRELL_TICK_FIXTURE"){
            let d=std::path::PathBuf::from(path);std::fs::create_dir(&d).expect("fresh immutable first-tick fixture");
            for(name,bytes)in [("backing",loaded.world.backing_bytes().to_vec()),("players",(0..4u8).flat_map(|p|loaded.world.player(p).unwrap().bytes().iter().copied()).collect()),("actors",loaded.world.actors().records().concat()),("creatures",loaded.world.creatures().unwrap().records().concat()),("effects",loaded.world.effects().unwrap().records().concat()),("family8",loaded.world.family8().unwrap().records().concat()),("buildings",loaded.world.buildings().unwrap().records().concat())]{std::fs::write(d.join(format!("{name}.bin")),bytes).unwrap();}
            let rng=loaded.world.rng();let mut bytes=rng.seed.to_le_bytes().to_vec();bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());std::fs::write(d.join("rng.bin"),bytes).unwrap();
        }
        for expected in 1..=2 {
            loaded.step(&resources).unwrap();loaded.render(&resources,[256,24320]).unwrap();loaded.finish_outer_frame();
            assert_eq!(loaded.frame_counter,expected);assert_eq!(loaded.outcome_json(),"{\"state\":\"running\"}");
        }
        assert_ne!(loaded.world,before);assert_ne!(loaded.ui_animations,prior_ui);
        // Short integrated/rendered fixture check, not native mission acceptance.
        for _ in 2..256 {
            loaded.step(&resources).unwrap();loaded.render(&resources,[256,24320]).unwrap();loaded.finish_outer_frame();
        }
        assert_eq!(loaded.frame_counter,256);assert_eq!(loaded.outcome_json(),"{\"state\":\"running\"}");
        println!("Krell integration passed256 rendered frames; not mission acceptance");
        println!("Krell actors: {}",loaded.actor_state_json());
        let audit=loaded.actor_dispatch_audit_json();
        assert!(audit.contains("\"activations\":3"),"all landed actors must be observed: {audit}");
        assert!(audit.contains("\"blueprint_equality_mismatches\":0"),"activation equality must hold: {audit}");
        println!("Krell dispatch audit: {audit}");
        if let Some(path)=std::env::var_os("GENEWARS_KRELL_AFTER_LANDING_FIXTURE"){
            let d=std::path::PathBuf::from(path);std::fs::create_dir(&d).expect("new immutable after-landing fixture");
            for(name,bytes)in [("backing",loaded.world.backing_bytes().to_vec()),("players",(0..4u8).flat_map(|p|loaded.world.player(p).unwrap().bytes().iter().copied()).collect()),("actors",loaded.world.actors().records().concat()),("creatures",loaded.world.creatures().unwrap().records().concat()),("effects",loaded.world.effects().unwrap().records().concat()),("family8",loaded.world.family8().unwrap().records().concat()),("buildings",loaded.world.buildings().unwrap().records().concat())]{std::fs::write(d.join(format!("{name}.bin")),bytes).unwrap();}
            let rng=loaded.world.rng();let mut bytes=rng.seed.to_le_bytes().to_vec();bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());std::fs::write(d.join("rng.bin"),bytes).unwrap();
        }
        if std::env::var_os("GENEWARS_KRELL_CONSTRUCTION_PROBE").is_some(){
            loaded.select(0).unwrap();
            let advance=|loaded:&mut SharedMission|->bool{
                let before=loaded.clone();
                if let Err(error)=loaded.step(&resources){
                    if let Some(root)=std::env::var_os("GENEWARS_MULE_CALLBACK_FIXTURE"){
                        let d=std::path::PathBuf::from(root).join(format!("mission-frame-{}-blocked-before",loaded.frame_counter));std::fs::create_dir(&d).expect("immutable mission failure fixture");
                        for(name,bytes)in [("backing",before.world.backing_bytes().to_vec()),("players",(0..4u8).flat_map(|p|before.world.player(p).unwrap().bytes().iter().copied()).collect()),("actors",before.world.actors().records().concat()),("creatures",before.world.creatures().unwrap().records().concat()),("effects",before.world.effects().unwrap().records().concat()),("family8",before.world.family8().unwrap().records().concat()),("buildings",before.world.buildings().unwrap().records().concat())]{std::fs::write(d.join(format!("{name}.bin")),bytes).unwrap();}
                        let rng=before.world.rng();let mut bytes=rng.seed.to_le_bytes().to_vec();bytes.push(rng.ring);bytes.extend(rng.draw_count.to_le_bytes());std::fs::write(d.join("rng.bin"),bytes).unwrap();
                        std::fs::write(d.join("boundary.json"),format!("{{\"frame\":{},\"error\":{},\"scope\":\"before entire mission callback, not per-family\"}}",loaded.frame_counter,crate::runtime_scene::json_string(&error))).unwrap();
                    }
                    println!("Krell observed boundary frame{}: {error}",loaded.frame_counter);
                    assert!(error.contains("Unsupported"),"unexpected probe error: {error}");assert_eq!(loaded.world,before.world);assert_eq!(loaded.ui_animations,before.ui_animations);assert_eq!(loaded.actor_dispatch_audit_json(),before.actor_dispatch_audit_json());
                    println!("Krell construction blocked frame{}: {error}; buildings{}; actors{}",loaded.frame_counter,loaded.building_state_json(),loaded.actor_state_json());
                    println!("{{\"status\":\"blocked\",\"classification\":\"unsupported_behavior\",\"source_frame\":{},\"transaction_unchanged\":true}}",loaded.frame_counter);return false;
                }
                loaded.render(&resources,[1536,15360]).unwrap();loaded.finish_outer_frame();true
            };
            for (selector,screen) in [(11,[320,234]),(3,[224,299]),(0,[448,226])]{
                // A completed building does not end the Architect's return animation.
                // Wait for the normal idle/new-task opcode5 admission; never edit it.
                for _ in 0..512{
                    let actor=loaded.world.actors().record(0).unwrap();
                    if actor[0x55]==1&&actor[0x82]==0{break;}
                    if !advance(&mut loaded){return;}
                }
                let event=loaded.place_selected_building(&resources,[1536,15360],screen,selector).unwrap();println!("Krell normal placement{selector}: {event}");
                let mut completed=false;
                for _ in 0..1800 {
                    if !advance(&mut loaded){return;}
                    completed=loaded.world.buildings().unwrap().records().iter().any(|r|r[6]==4&&r[45]==selector&&u16::from_le_bytes([r[49],r[50]])&1!=0);
                    if completed{println!("Krell completed{selector} frame{}; {}",loaded.frame_counter,loaded.building_state_json());break;}
                }
                assert!(completed,"ordinary Krell construction did not complete within bounded diagnostic");
            }
        }

    }

    /// Synthetic retained state exercises the real loader; this is not mission acceptance.
    #[test] fn krell_load_preserves_campaign_genetics_and_inherits_ring_without_regeneration(){
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let first=read_bound(&root.join("terrain/CONQ001.DAT")).unwrap();
        let initial=SharedMission::load_fresh_cd(&root,&first,[0xa5;36],17).unwrap();
        let mut carry=initial.world.plan_campaign_carry(&[0;600],0x7e,2,0,None).unwrap();
        carry.players[0][0x6e7]=1;
        let priors=core::array::from_fn(|i|i as u8);
        let second=read_bound(&root.join("terrain/CONQ002.DAT")).unwrap();
        let loaded=SharedMission::load_carried_cd(&root,&second,&carry,&priors,17,0,initial.ui_animations).unwrap();
        assert_eq!(loaded.world.rng(),BlueprintRng{seed:47117,ring:17,draw_count:33620});
        assert_eq!(&loaded.world.backing_bytes()[0x34ba..0x3550],&priors);
        let p=loaded.world.player(0).unwrap().bytes();assert_eq!(p[0x6e7],1);
        assert_eq!(i32::from_le_bytes(p[0x4a3..0x4a7].try_into().unwrap()),2000);
        for (before,after) in carry.players[0][0x16..0x496].chunks_exact(36).zip(p[0x16..0x496].chunks_exact(36)){
            assert_eq!(&before[..0x1c],&after[..0x1c],"retained names, skills and opaque priors");
        }
        assert_eq!(loaded.ui_animations,initial.ui_animations);
        let context=loaded.runtime_context.as_ref().unwrap();
        assert_eq!(context.origin(),genewars_runtime::mission_context::ContextOrigin::CarriedCdKrell);
        assert_eq!(context.frame(),0);assert_eq!(context.countdown(),1000);assert_eq!(context.known_runtime_flags(),0x3e);
        assert!(SharedMission::load_carried_cd(&root,&second,&carry,&priors,17,2,initial.ui_animations).is_err());
        assert!(SharedMission::load_carried_cd(&root,&first,&carry,&priors,17,0,initial.ui_animations).is_err());
    }
}

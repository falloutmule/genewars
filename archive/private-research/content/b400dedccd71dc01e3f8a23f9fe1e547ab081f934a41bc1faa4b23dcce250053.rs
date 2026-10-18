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
}
impl SharedMission {
    pub fn load(root:&Path,mission_bytes:&[u8],seed:u16)->Result<Self,String> {
        let source=ConqMissionSource::parse(mission_bytes).map_err(|e|e.to_string())?;
        let target=source.target_words(0).map_err(|e|e.to_string())?.map(|v|v as i16);
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,target,b"Native study\0").unwrap());
        let actor_priors=zero_record_box::<20,200>();
        let mut world=ActorWorld::from_mission(&source,&actor_priors,[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed,ring:0,draw_count:0}).map_err(|e|e.to_string())?;
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
        world.regenerate_player_blueprints(0,[0;36],&names,true).map_err(|e|e.to_string())?;
        let origin=ActorOriginSample{position:[0,0],height:interpolate_height(world.map(),0,0).map_err(|e|e.to_string())?,
            shade:interpolate_entity_shade(world.map(),0,0).map_err(|e|e.to_string())?};
        world.prepare_fresh_player(0,5,origin,target).map_err(|e|e.to_string())?;
        let factory=world.create_landing_core(0,0).map_err(|e|e.to_string())?;
        let mut pending_requests=0;
        for event in &factory.pending_events {
            if let genewars_runtime::landing::LandingEvent::Reveal{position,radius}=event {
                if *radius!=4096 {return Err("unsupported reveal radius".into());}
                let reveal=world.apply_reveal_4096(*position).map_err(|e|e.to_string())?;
                pending_requests+=reveal.pending_presentation_vertices.len()as u64;
            } else {pending_requests+=1;}
        }
        let plant_spread=SpreadDirections::from_mapped_bytes(&read_bound(&root.join("physics/plant-spread.bin"))?).map_err(|e|e.to_string())?;
        Ok(Self{world,trig:TrigTables::generate_pinned().map_err(|e|e.to_string())?,plant_spread,ui:RetirementUiState::native_empty(),
            terrain_jobs,frame_counter:0,selected:None,command_count:0,pending_requests,plant_births:0,seed})
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
        let mut frame=actor_scene::render_actor_world_with_selected_portrait(resources,&self.world,target,0,self.frame_counter as u8,&actors,&creatures,
            actor_scene::MuleSceneGlobals{global_shadow_mode:0,inherited_wrapper_flags:0,clip_flag_bit0:false},&effects,self.selected)?.frame;
        if let Some(slot)=self.selected {
            actor_scene::draw_selected_actor_text(resources,&self.world,slot,&mut frame.image)?;
            frame.rgb_sha256=genewars_runtime::identity::sha256_hex(frame.image.rgb_bytes());
        }
        // The job overlay is authoritative and is not mirrored into backing.
        // Include it in replay identity; this remains a diagnostic, not a save.
        let mut identity=b"GeneWars SharedMission jobs v1\0".to_vec();
        identity.extend_from_slice(frame.world_sha256.as_bytes());
        identity.push(self.terrain_jobs.count());
        for record in self.terrain_jobs.records(){identity.extend_from_slice(record);}
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
        let context=self.world.mission_loading_physics();
        let frame=u32::try_from(self.frame_counter+1).map_err(|_|"source frame counter exceeds u32")?;
        // 58780 visits players and their current actor lists. 57A10 returns
        // immediately before the ready bit is set by the landing callback.
        let actors=self.world.step_actor_callbacks_with_construction(4,frame,context,&resources.physics,&self.trig,
            resources.actors.databases(),&mut self.terrain_jobs,&resources.construction_border,
            0,self.selected.map(|slot|slot as u16),0,0).map_err(|e|format!("actor pass: {e:?}"))?;
        let callbacks=actors.len();
        for callback in &actors {match callback {
            ConstructionAwareActorCallback::Ordinary(r)=>self.pending_requests+=u64::from(r.acknowledgement.is_some())+r.pending_events.len()as u64,
            ConstructionAwareActorCallback::Architect{receipt:r,..}=>{
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
        let buildings=self.world.step_admitted_buildings_with_production(4,0,&resources.bturr_selector1,&resources.source_buildings,
            frame,&resources.physics,|_|false,context,&self.trig,&resources.construction_border).map_err(|e|e.to_string())?;
        self.pending_requests+=buildings.notices.len()as u64;
        for callback in &buildings.callbacks {
            use genewars_runtime::actor_world::AdmittedBuildingCallback;
            match callback {
                AdmittedBuildingCallback::Incomplete{plan,..}=>self.pending_requests+=u64::from(plan.stop_audio_23)+u64::from(plan.stop_audio_2f),
                AdmittedBuildingCallback::Turret{plan,..}=>self.pending_requests+=u64::from(plan.audio.is_some()),
                AdmittedBuildingCallback::Extractor{plan,..}=>self.pending_requests+=plan.audio.len()as u64,
                AdmittedBuildingCallback::PowerProducer{receipt}=>self.pending_requests+=receipt.numeric.audio.len()as u64,
                AdmittedBuildingCallback::ProducingExtractor{receipt}=>self.pending_requests+=(receipt.plan.state.audio.len()+receipt.terrain_invalidations.len())as u64,
            }
        }
        let climate=i16::from_le_bytes(self.world.backing_bytes()[0x1c61..0x1c63].try_into().unwrap());
        let creatures=self.world.step_neutral_creature_callbacks(4,
            MuleStepContext{frame_counter:frame,gravity:context.gravity,water_height:context.water_height,climate},
            &resources.physics,&self.trig,&resources.mule_animations).map_err(|e|e.to_string())?;
        self.pending_requests+=(creatures.pending_ui_animations.len()+creatures.pending_audio.len()+creatures.pending_plant_retirements.len()+creatures.pending_portrait_refreshes.len())as u64;
        self.pending_requests+=creatures.retirements.iter().map(|r|1+r.pending_ui_cleanup.len()as u64).sum::<u64>();
        let plants=self.world.tick_plants_with_reproduction(&self.plant_spread).map_err(|e|e.to_string())?;
        self.plant_births+=plants.events.iter().filter(|event|matches!(event,genewars_runtime::TickEvent::Created{..})).count()as u64;
        self.pending_requests+=plants.events.iter().filter(|event|matches!(event,
            genewars_runtime::TickEvent::OldFrameDirtyCheckPending{..}|genewars_runtime::TickEvent::ReferenceUiAudioCleanupPending{..})).count()as u64;
        let countdown=self.world.step_family8_callbacks(0,4,0,frame,context.water_height,
            &resources.physics,&self.trig,&resources.family8_animation,&resources.family8_directions).map_err(|e|e.to_string())?;
        self.pending_requests+=(countdown.pending_prefix.len()+countdown.pending_ui_animations.len())as u64;
        self.pending_requests+=countdown.callbacks.iter().map(|c|u64::from(c.plan.pending_audio.is_some())+u64::from(c.plan.pending_ui_message)+c.retirement.as_ref().map_or(0,|r|r.stop_audio.len()as u64+u64::from(r.fade_audio.is_some()))+u64::from(c.parent_response.as_ref().is_some_and(|p|p.state7_message_variant.is_some()))).sum::<u64>();
        self.world.step_empty_families_7_and_6().map_err(|e|e.to_string())?;
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
        self.frame_counter+=1;
        let(mut created,mut retired)=(0,0);
        for callback in result.primary.iter().chain(&result.newborn) {
            match callback {
                LandingEffectCallback::Landing(e)=>{
                    created+=usize::from(e.created_effect.is_some());retired+=usize::from(e.retirement.is_some());
                    self.pending_requests+=e.pending_events.len()as u64;
                    if let Some(r)=&e.retirement {self.pending_requests+=r.pending_ui_events.len()as u64;}
                },
                LandingEffectCallback::Smoke(e)|LandingEffectCallback::DepartureParticle(e)|LandingEffectCallback::WaterSplash(e)|LandingEffectCallback::EtherealParticle(e)|LandingEffectCallback::Construction(e)|LandingEffectCallback::Power(e)=>{
                    created+=usize::from(e.created_effect.is_some());retired+=usize::from(e.retired);self.pending_requests+=e.pending_ui_events.len()as u64;
                },
            }
        }
        Ok(format!("world pass {}; actors {callbacks}; created {created}; retired {retired}; active {}; RNG draws {}; bounded world callbacks",
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
            format!("{{\"slot\":{slot},\"owner\":{},\"position\":[{},{}],\"destination\":[{},{}],\"state\":{},\"animation\":{},\"frame\":{},\"reserve\":{},\"target_kind\":{}}}",
                r[24],word(r,7),word(r,9),word(r,11),word(r,13),r[0x5e],r[0x19],r[0x1b],word(r,0x68),r[0x57]))
            .collect::<Vec<_>>().join(",");
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

impl SharedMission {pub fn fixture_jobs(&self)->Vec<u8>{let mut v=vec![self.terrain_jobs.count()];for r in self.terrain_jobs.records(){v.extend(r)}v}}

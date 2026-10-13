//! Explicit native landing study over the owned world, not a complete game
//! scheduler. Caller seed/zero native priors and frozen remaining families are
//! visible development policies. Real source factories activate the actors.
use std::path::Path;
use genewars_assets::{ConqMissionSource,TextStringTable};
use genewars_core::cd_candidate::{interpolate_height,interpolate_entity_shade,camera_cells_for_target};
use genewars_runtime::{actor_world::{ActorWorld,LandingEffectCallback,GroundMoveInput,GroundMoveOutcome},
    actor_pool::ActorHandleBindings,blueprint::{BlueprintNames,BlueprintRng},
    player_state::{PlayerState,ActorOriginSample},actor_activation::TrigTables,
    creature_world::MuleStepContext,
    landing_effect_lifecycle::RetirementUiState,terrain_target::project_terrain_target};
use super::{runtime_resources::{Resources,Frame,read_bound},actor_scene};

#[derive(Clone)]
pub struct SharedMission {
    pub world:ActorWorld,
    trig:TrigTables,
    ui:RetirementUiState,
    pub frame_counter:u64,
    pub selected:Option<usize>,
    pub command_count:u64,
    pub pending_requests:u64,
    pub seed:u16,
}
impl SharedMission {
    pub fn load(root:&Path,mission_bytes:&[u8],seed:u16)->Result<Self,String> {
        let source=ConqMissionSource::parse(mission_bytes).map_err(|e|e.to_string())?;
        let target=source.target_words(0).map_err(|e|e.to_string())?.map(|v|v as i16);
        let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,target,b"Native study\0").unwrap());
        let mut world=ActorWorld::from_mission(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,
            BlueprintRng{seed,ring:0,draw_count:0}).map_err(|e|e.to_string())?;
        world.admit_creatures(&[[0;124];300]).map_err(|e|e.to_string())?;
        world.admit_effects(&[[0;81];200],[None;200]).map_err(|e|e.to_string())?;
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
        Ok(Self{world,trig:TrigTables::generate_pinned().map_err(|e|e.to_string())?,ui:RetirementUiState::native_empty(),
            frame_counter:0,selected:None,command_count:0,pending_requests,seed})
    }
    pub fn render(&self,resources:&Resources,target:[i16;2])->Result<Frame,String> {
        // Until original admission/order is integrated, only linked active
        // actors and active effects are drawn in ascending slot order.
        let actors=self.world.actors().records().iter().enumerate().filter_map(|(slot,r)|
            (r[6]==3&&r[0x70]&1!=0).then_some(slot)).collect::<Vec<_>>();
        let effects=self.world.effects().ok_or("effects not admitted")?.records().iter().enumerate()
            .filter_map(|(slot,r)|(r[6]!=0).then_some(slot)).collect::<Vec<_>>();
        actor_scene::render_actor_world_with_selected_portrait(resources,&self.world,target,0,self.frame_counter as u8,&actors,&[0],
            actor_scene::MuleSceneGlobals{global_shadow_mode:0,inherited_wrapper_flags:0,clip_flag_bit0:false},&effects,self.selected).map(|r|r.frame)
    }
    pub fn step(&mut self,resources:&Resources)->Result<String,String> {
        let mut staged=self.clone();let event=staged.step_inner(resources)?;*self=staged;Ok(event)
    }
    fn step_inner(&mut self,resources:&Resources)->Result<String,String> {
        let context=self.world.mission_loading_physics();
        let frame=u32::try_from(self.frame_counter+1).map_err(|_|"source frame counter exceeds u32")?;
        // 58780 visits players and their current actor lists. 57A10 returns
        // immediately before the ready bit is set by the landing callback.
        let actors=self.world.step_actor_callbacks_in_player_order(4,frame,context,&resources.physics,&self.trig,
            resources.actors.databases(),0,0).map_err(|e|e.to_string())?;
        let callbacks=actors.len();
        self.pending_requests+=actors.iter().filter(|r|r.acknowledgement.is_some()).count()as u64;
        self.pending_requests+=actors.iter().map(|r|r.pending_events.len()as u64).sum::<u64>();
        self.pending_requests+=self.world.step_empty_building_bookkeeping(0).map_err(|e|e.to_string())?.len()as u64;
        let climate=i16::from_le_bytes(self.world.backing_bytes()[0x1c61..0x1c63].try_into().unwrap());
        let creatures=self.world.step_neutral_creature_callbacks(4,
            MuleStepContext{frame_counter:frame,gravity:context.gravity,water_height:context.water_height,climate},
            &resources.physics,&self.trig,&resources.mule_animations).map_err(|e|e.to_string())?;
        self.pending_requests+=(creatures.pending_ui_animations.len()+creatures.pending_audio.len()+creatures.pending_plant_retirements.len())as u64;
        self.world.tick_plants().map_err(|e|e.to_string())?;
        let countdown=self.world.step_empty_family8_countdown(0,0).map_err(|e|e.to_string())?;
        self.pending_requests+=u64::from(countdown.started);
        self.world.step_empty_families_7_and_6().map_err(|e|e.to_string())?;
        let result=self.world.step_landing_effect_callbacks(context,&resources.physics,&self.trig,&self.ui).map_err(|e|e.to_string())?;
        self.ui=result.ui_after;self.frame_counter+=1;
        let(mut created,mut retired)=(0,0);
        for callback in result.primary.iter().chain(&result.newborn) {
            match callback {
                LandingEffectCallback::Landing(e)=>{
                    created+=usize::from(e.created_effect.is_some());retired+=usize::from(e.retirement.is_some());
                    self.pending_requests+=e.pending_events.len()as u64;
                    if let Some(r)=&e.retirement {self.pending_requests+=r.pending_ui_events.len()as u64;}
                },
                LandingEffectCallback::Smoke(e)|LandingEffectCallback::DepartureParticle(e)|LandingEffectCallback::WaterSplash(e)=>{
                    retired+=usize::from(e.retired);self.pending_requests+=e.pending_ui_events.len()as u64;
                },
            }
        }
        Ok(format!("world pass {}; actors {callbacks}; created {created}; retired {retired}; active {}; RNG draws {}; neutral Mule active; remaining systems frozen",
            self.frame_counter,result.queue_candidates.len(),self.world.rng().draw_count))
    }
    pub fn select(&mut self,slot:usize)->Result<(),String> {
        let actor=self.world.actors().record(slot).map_err(|e|e.to_string())?;
        if actor[6]!=3||actor[24]!=0||actor[0x70]&1==0 {return Err("actor is not active for this player".into());}
        self.selected=Some(slot);Ok(())
    }
    pub fn actor_state_json(&self)->String {
        let word=|r:&[u8],p:usize|i16::from_le_bytes([r[p],r[p+1]]);
        let entries=self.world.actors().records().iter().enumerate().filter(|(_,r)|r[6]==3).map(|(slot,r)|
            format!("{{\"slot\":{slot},\"owner\":{},\"position\":[{},{}],\"destination\":[{},{}],\"pending_destination\":[{},{}],\"state\":{},\"ready\":{},\"pending_move\":{},\"animation\":{},\"frame\":{}}}",
                r[24],word(r,7),word(r,9),word(r,11),word(r,13),word(r,0x5c),word(r,0x5e),r[0x55],r[0x70]&1!=0,r[0x57]&2!=0,r[0x19],r[0x1b]))
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

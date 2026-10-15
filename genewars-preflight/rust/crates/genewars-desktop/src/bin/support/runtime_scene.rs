//! Native development route. Original campaign movie/confirmation is pending.
use std::path::{Path,PathBuf};
use genewars_assets::{Palette256,PaletteChannelMode};
use genewars_core::cd_candidate::{MenuCallback0State,MenuPacket,MenuControl};
use genewars_render::{RgbImage,CdTitleButton};
use genewars_runtime::world::MissionWorld;
use super::{menu_art,runtime_resources::{read_bound,Resources,Frame},status_text};
use super::shared_scene::SharedMission;
#[path="building_menu.rs"] mod building_menu;
#[path="campaign_scene.rs"] mod campaign_scene;
#[path="owned_mule_control.rs"] mod owned_mule_control;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scene { Menu, Loading, Mission, Victory, Campaign, Options, Error, Quit }

#[derive(Clone)]
pub enum MissionState { Plants(MissionWorld), Landing(SharedMission) }
impl MissionState {
    fn render(&self,resources:&Resources,target:[i16;2])->Result<Frame,String> {
        match self {Self::Plants(w)=>resources.render(w,target),Self::Landing(w)=>w.render(resources,target)}
    }
    fn step(&mut self,resources:&Resources)->Result<String,String> {
        match self {Self::Plants(w)=>w.tick_plants().map(|e|format!("{e:?}")).map_err(|e|e.to_string()),Self::Landing(w)=>w.step(resources)}
    }
    pub fn source_bytes(&self)->Result<&[u8],String> {
        match self {Self::Plants(w)=>Ok(w.source_bytes()),Self::Landing(_)=>Err("owned overlays require a full-state exporter; backing alone is not a save".into())}
    }
    fn plant_count(&self)->u16 {
        match self {Self::Plants(w)=>w.plant_count(),Self::Landing(w)=>u16::from_le_bytes(w.world.backing_bytes()[0x3552..0x3554].try_into().unwrap())}
    }
    fn schedule_cursor(&self)->u16 {
        match self {Self::Plants(w)=>w.schedule_cursor(),Self::Landing(_)=>0}
    }
    fn extra_report(&self)->String {
        match self {
            Self::Plants(_)=>"\"world_mode\":\"plants\"".into(),
            Self::Landing(w)=>format!("\"world_mode\":{},\"runtime_context\":{},\"outcome\":{},\"campaign_progress\":{},\"seed\":{},\"rng_draws\":{},\"selected_actor\":{},\"issued_move_commands\":{},\"plant_births\":{},\"pending_presentation_requests\":{},\"active_effects\":{},\"actor_states\":{},\"creature_states\":{},\"family8_states\":{},\"player_funds\":{},\"buildings\":{},\"terrain_jobs\":{},\"actor_dispatch_audit\":{},\"actor_movement\":true,\"plant_updates\":true,\"creature_ai\":true,\"other_families_frozen\":true",
                json_string(match w.runtime_context.as_ref().map(|c|c.origin()){Some(genewars_runtime::mission_context::ContextOrigin::CarriedCdKrell)=>"carried-cd-explicit-opaque-priors",Some(_)=>"fresh-cd-explicit-opaque-priors",None=>"landing-study"}),json_string(&format!("{:?}",w.runtime_context)),w.outcome_json(),w.world.player(0).unwrap().bytes()[0x6e7],w.seed,w.world.rng().draw_count,w.selected.map(|s|s.to_string()).unwrap_or("null".into()),w.command_count,w.plant_births,w.pending_requests,
                w.world.effects().unwrap().records().iter().filter(|r|r[6]!=0).count(),w.actor_state_json(),w.creature_state_json(),w.family8_state_json(),w.player_funds_json(),w.building_state_json(),w.terrain_job_state_json(),w.actor_dispatch_audit_json()),
        }
    }
}

pub struct Mission {
    pub resources: Resources, pub world: MissionState, pub target: [i16;2], pub frame: Frame,
    pub ticks: u64, pub blocked: Option<String>, pub changed_frames: u64,
}

pub struct VictoryDisplay {
    pub image:RgbImage,
    pub receipt:super::result_scene::VictoryScreenReceipt,
    pub entry:genewars_runtime::result_transition::VictoryEntryPlan,
}

pub struct CampaignState {
    pub carry:genewars_runtime::result_carry::CampaignCarryPlan,
    pub world_priors:[u8;150],pub rng_ring:u8,pub mode:u8,pub frame:u32,
    pub ui_animations:[[u8;5];8],
    pub display:campaign_scene::CampaignDisplay,
}

pub struct GenepodSelection { pub slot:u16, pub state:u8, pub list:i8, pub control:super::genepod_panel::GenepodControl, pub input:super::genepod_panel::PanelInput }

pub struct Game {
    pub genepod:Option<GenepodSelection>,
    mule_focus:Option<u16>, mule_press:bool,
    mule_manager:[u8;0x59d], mule_packet:genewars_runtime::mule_human_producer::CommandPacket,
    root: PathBuf,
    pub building_menu_open:bool, building_menu_capture:Option<u8>, building_feedback:Option<String>, pub placement_selector:Option<u8>,
    pub scene: Scene, pub mission: Option<Mission>, pub victory:Option<VictoryDisplay>, pub campaign:Option<CampaignState>,
    images: Vec<RgbImage>, button: CdTitleButton,
    controls: MenuCallback0State, cursor: Option<(i32,i32)>, left: bool, right: bool, mask: u8,
    pub automatic: bool, pub start_running: bool, pub error: Option<String>,
    pub landing_seed: Option<u16>,
    pub reference_startup:bool,
    campaign_system:u8, carry_records:[u8;600],
    pub trace: Vec<String>, pub ticks: Vec<String>, pub presented: u64,
}

pub fn json_string(s:&str)->String {
    let mut out=String::from("\"");
    for c in s.chars() { match c { '"'=>out.push_str("\\\""),'\\'=>out.push_str("\\\\"),'\n'=>out.push_str("\\n"),'\r'=>out.push_str("\\r"),'\t'=>out.push_str("\\t"),c if c<' '=>out.push_str(&format!("\\u{:04x}",c as u32)),c=>out.push(c) } }
    out.push('"'); out
}

impl Game {
    pub fn open(root:&Path)->Result<Self,String> {
        let palette=Palette256::parse(&read_bound(&root.join("menu/titlescr.pal"))?,PaletteChannelMode::Rgb).map_err(|e|e.to_string())?;
        let (images,button,_,_)=menu_art::compose(&root.join("menu"),&palette)?;
        Ok(Self { genepod:None,mule_focus:None,mule_press:false,mule_manager:[0;0x59d],mule_packet:genewars_runtime::mule_human_producer::CommandPacket{words:[0;5],opcode:0},root:root.into(),building_menu_open:false,building_menu_capture:None,building_feedback:None,placement_selector:None, scene:Scene::Menu,mission:None,victory:None,campaign:None,images,button,
            controls:MenuCallback0State::default(),cursor:None,left:false,right:false,mask:0,
            automatic:false,start_running:false,error:None,landing_seed:None,reference_startup:false,campaign_system:0,carry_records:[0;600],trace:vec!["Menu".into()],ticks:Vec::new(),presented:0 })
    }

    fn transition(&mut self, scene:Scene) {
        self.clear_mule_focus();self.mule_manager=[0;0x59d];self.mule_packet=genewars_runtime::mule_human_producer::CommandPacket{words:[0;5],opcode:0};
        self.building_menu_open=false;self.building_menu_capture=None;self.building_feedback=None;self.placement_selector=None;
        self.genepod=None;self.scene=scene; self.trace.push(format!("{scene:?}"));
        self.controls=MenuCallback0State::default();self.mask=0;self.left=false;self.right=false;
    }

    fn hit(&self,c:MenuControl)->bool {
        let Some((x,y))=self.cursor else {return false;};
        let top=match c {MenuControl::Id0=>115,MenuControl::Id3=>186,MenuControl::Id2=>257,_=>return false};
        match (x.checked_sub(140),y.checked_sub(top)) { (Some(x),Some(y))=>self.button.hit_test(x,y), _=>false }
    }

    pub fn pointer(&mut self,cursor:Option<(i32,i32)>) {if cursor.is_none(){self.mule_press=false;}self.cursor=cursor;self.packet(false,false);}
    pub fn left(&mut self,down:bool) {
        let changed=self.left!=down;self.left=down;if !changed{return;}
        if self.building_menu_open {
            let choices=self.building_choices();let hit=building_menu::hit(&choices,self.cursor);
            if down{self.building_menu_capture=hit;}else{
                let captured=self.building_menu_capture.take();
                if captured.is_some()&&captured==hit{
                    let selector=captured.unwrap();
                    match self.choose_building_type(selector){
                        Ok(())=>{self.building_menu_open=false;},
                        Err(error)=>{self.error=Some(error.clone());self.building_feedback=Some(error);}
                    }
                }
            }
            return;
        }
        self.packet(down,!down);
    }
    fn building_choices(&self)->Vec<u8>{
        let Some(m)=&self.mission else{return Vec::new();};
        let MissionState::Landing(w)=&m.world else{return Vec::new();};
        w.world.player(0).map(|p|building_menu::choices(p.bytes())).unwrap_or_default()
    }
    pub fn toggle_building_menu(&mut self){
        if self.scene==Scene::Mission&&self.mission.as_ref().is_some_and(|m|matches!(m.world,MissionState::Landing(_))){
            self.clear_mule_focus();self.building_menu_open=!self.building_menu_open;self.building_menu_capture=None;self.building_feedback=None;
            self.automatic=false;self.trace.push(format!("ConstructionMenu:{}",self.building_menu_open));
        }
    }
    pub fn right(&mut self,down:bool) {if self.mule_focus.is_some(){if down{self.clear_mule_focus();}self.right=down;return;}let pressed=down&&!self.right;self.right=down;self.packet(false,false);if pressed&&!self.building_menu_open&&self.scene==Scene::Mission {if let Some((x,y))=self.cursor {if let Some(selector)=self.placement_selector{if let Err(e)=self.place_selected_building_type(selector,[x,y]){self.error=Some(e);}}else{self.order([x,y]);}}}}
    pub fn cancel_capture(&mut self) {self.clear_mule_focus();self.building_menu_capture=None;self.cursor=None;self.left=false;self.right=false;self.packet(false,true);}

    fn packet(&mut self,press:bool,release:bool) {
        if self.scene==Scene::Mission { if self.mule_focus.is_some(){if let Err(e)=self.mule_port_packet(press,release){self.error=Some(e);}return;} if let Err(e)=self.genepod_packet(press,release){self.error=Some(e);} return; }
        if self.scene!=Scene::Menu {return;}
        let first=MenuControl::hit_order().into_iter().find(|&c|self.hit(c));
        let captured=self.controls.captured().is_some_and(|c|self.hit(c));
        let result=self.controls.update(MenuPacket {byte8:u8::from(press)|(u8::from(release)<<1)|(u8::from(self.left)<<2),byte9:u8::from(self.right)},first,captured);
        if let Some(mask)=result.input_mask_write {self.mask=mask;}
        if result.event_id>=0 && self.mask&8!=0 {
            match result.event_id {0=>self.transition(Scene::Loading),3=>self.transition(Scene::Options),2=>self.transition(Scene::Quit),_=>{}}
        }
    }


    fn genepod_packet(&mut self,press:bool,release:bool)->Result<(),String>{
        let pointer=self.cursor.map(|(x,y)|[x,y]);
        if press && pointer.is_some_and(|p|p[0]>=128) && !self.building_menu_open {
            self.open_clicked_genepod(pointer.unwrap())?;
        }
        let Some(mut panel)=self.genepod.take() else{return Ok(());};
        let result=(||{
            let m=self.mission.as_mut().ok_or("missing mission")?;
            let MissionState::Landing(w)=&mut m.world else{return Ok(());};
            let building=*w.world.buildings().ok_or("building pool unavailable")?.record(usize::from(panel.slot)).map_err(|e|format!("{e:?}"))?;
            if building[24]!=0||u16::from_le_bytes([building[49],building[50]])&0x41!=1 {return Ok(());}
            panel.state=super::genepod_panel::PanelInput::ready(panel.state,&building).panel_state_after;
            let feedback=panel.input.poll(&panel.control,[0,0],pointer,if press{1}else if release{2}else if self.left{4}else{0})?;
            if feedback.action==8 {
                let mut proposed=w.clone();
                let receipt=proposed.world.dispatch_genepod_mule_input(panel.slot,genewars_runtime::genepod_human_producer::ProducerInput{
                    selected_player_index:0,panel_state:panel.state,species_list_index:panel.list,mouse_flags:feedback.event,captured_opaque_hit:true,
                },true).map_err(|e|format!("{e:?}"))?;
                let frame=proposed.render(&m.resources,m.target)?;
                panel.state=receipt.producer.panel_state_after;
                self.trace.push(format!("GenepodInput:slot={} action={} event={} outcome={:?}",panel.slot,feedback.action,feedback.event,receipt.producer.outcome));
                self.building_feedback=Some(format!("GENEPOD: {:?}",receipt.producer.outcome));
                *w=proposed;m.frame=frame;
            }
            Ok(())
        })();self.genepod=Some(panel);result
    }
    fn open_clicked_genepod(&mut self,mouse:[i32;2])->Result<(),String>{
        let Some(m)=&self.mission else{return Ok(());};let MissionState::Landing(w)=&m.world else{return Ok(());};
        let camera=genewars_core::cd_candidate::camera_cells_for_target(w.world.map(),m.target[0],m.target[1]).map_err(|e|e.to_string())?;
        let mut hits=Vec::new();
        for(slot,r)in w.world.buildings().ok_or("building pool unavailable")?.records().iter().enumerate(){
            if r[6]!=4||r[23]==0{continue;}
            let position=[i16::from_le_bytes([r[7],r[8]]),i16::from_le_bytes([r[9],r[10]])];
            let origin=genewars_core::cd_candidate::entity_view_position(w.world.map(),position,camera,0).map_err(|e|e.to_string())?;
            let frames=match m.resources.building_resource(r[45])?{
                super::runtime_resources::BuildingResourceRef::Source(db)=>db.animation(usize::from(r[25])).map_err(|e|e.to_string())?,
                super::runtime_resources::BuildingResourceRef::Bturr(_)=>continue,
            };
            if super::building_pick::hit_animation(frames,r[27],origin,mouse).map_err(|e|format!("{e:?}"))?.is_some(){hits.push((slot,*r));}
        }
        if hits.len()>1{return Err("Overlapping building selection requires original mixed queue order".into());}
        let Some((slot,r))=hits.pop()else{return Ok(());};
        if r[45]!=0||r[24]!=0{return Ok(());}
        let control=super::genepod_panel::GenepodControl::parse(&read_bound(&self.root.join("ui/GENEPANL.ANB"))?)?;
        self.genepod=Some(GenepodSelection{slot:slot as u16,state:0,list:r[60]as i8,control,input:Default::default()});
        self.building_feedback=Some("GENEPOD SELECTED - CLICK MULE CONTROL".into());self.trace.push(format!("SelectedGenepod:{slot}"));Ok(())
    }

    /// Invoked after a loading redraw, and by the identical headless driver.
    pub fn finish_loading(&mut self) {
        if self.scene!=Scene::Loading {return;}
        let loaded=Resources::load(&self.root).and_then(|(resources,world,target)| {
            let world=if self.reference_startup {MissionState::Landing(SharedMission::load_fresh_cd(&self.root,world.source_bytes(),[0;36],0)?)}else if let Some(seed)=self.landing_seed {MissionState::Landing(SharedMission::load(&self.root,world.source_bytes(),seed)?)}else{MissionState::Plants(world)};
            let frame=world.render(&resources,target)?;
            Ok(Mission {resources,world,target,frame,ticks:0,blocked:None,changed_frames:0})
        });
        match loaded {Ok(mission)=>{self.mission=Some(mission);self.automatic=self.start_running;self.error=None;self.ticks.clear();self.transition(Scene::Mission);},
            Err(e)=>{self.error=Some(e);self.mission=None;self.transition(Scene::Error);}}
    }

    pub fn tick(&mut self) {
        if self.scene!=Scene::Mission {return;}
        // No pool generation token exists: expire port focus before any callbacks
        // can retire and reuse a creature slot. Commands themselves remain active.
        self.clear_mule_focus();
        let Some(m)=&mut self.mission else {return;};
        if m.blocked.is_some() {return;}
        if self.ticks.len()>=32768 {m.blocked=Some("Development trace limit (32768 steps) reached".into());return;}
        // Cross-subsystem transaction: a drawable frame must exist before the
        // tentative world can replace the currently displayed one.
        let mut proposed=m.world.clone();
        let result=proposed.step(&m.resources).and_then(|event| {
            let frame=proposed.render(&m.resources,m.target)?;
            if let MissionState::Landing(w)=&mut proposed{w.finish_outer_frame();}
            Ok((event,frame))
        });
        match result {
            Ok((event,frame))=>{
                if frame.rgb_sha256!=m.frame.rgb_sha256 {m.changed_frames+=1;}
                m.ticks+=1;
                self.ticks.push(format!("{{\"step\":{},\"world_sha256\":{},\"rgb_sha256\":{},\"event\":{}}}",m.ticks,json_string(&frame.world_sha256),json_string(&frame.rgb_sha256),json_string(&event)));
                m.world=proposed;m.frame=frame;
            },
            Err(e)=>{m.blocked=Some(e);self.automatic=false;}
        }
    }

    pub fn pan(&mut self,dx:i16,dy:i16) {
        if self.scene!=Scene::Mission {return;}
        let Some(m)=&mut self.mission else {return;};
        let target=[m.target[0].wrapping_add(dx),m.target[1].wrapping_add(dy)];
        match m.world.render(&m.resources,target) {Ok(frame)=>{m.frame=frame;m.target=target;},Err(e)=>self.error=Some(e)}
    }

    /// Development actor selection; original portrait/hit selection remains
    /// separate from the source ground-command dispatcher used below.
    pub fn select_actor(&mut self,slot:Option<usize>) {
        self.clear_mule_focus();
        if self.scene!=Scene::Mission {return;}
        let Some(m)=&mut self.mission else{return;};
        let MissionState::Landing(current)=&m.world else{return;};
        let mut proposed=current.clone();
        let result=if let Some(i)=slot {proposed.select(i)}else{proposed.select_next()};
        match result.and_then(|()|proposed.render(&m.resources,m.target)) {
            Ok(frame)=>{self.trace.push(format!("SelectedActor:{}",proposed.selected.unwrap()));m.world=MissionState::Landing(proposed);m.frame=frame;self.error=None;},
            Err(e)=>self.error=Some(e),
        }
    }
    pub fn order(&mut self,screen:[i32;2]) {
        if self.scene!=Scene::Mission {return;}
        let Some(m)=&mut self.mission else{return;};
        let MissionState::Landing(current)=&m.world else{return;};
        let mut proposed=current.clone();
        match proposed.order(&m.resources,m.target,screen).and_then(|receipt|proposed.render(&m.resources,m.target).map(|frame|(receipt,frame))) {
            Ok((receipt,frame))=>{m.world=MissionState::Landing(proposed);m.frame=frame;self.error=None;self.trace.push(receipt);},
            Err(e)=>self.error=Some(e),
        }
    }

    /// Explicit development command for a picked ground move. It uses the
    /// same screen-to-terrain projection and owned opcode-3 resolver as a
    /// right-click; no building site is synthesized from these coordinates.
    pub fn build_order(&mut self,screen:[i32;2]) {
        if self.scene!=Scene::Mission {return;}
        let before=self.trace.len();
        self.order(screen);
        if self.trace.len()>before {
            if let Some(entry)=self.trace.last_mut(){*entry=format!("DevelopmentBuildMove: {entry}");}
        }
    }

    /// Assign the selected architect to the only admitted native build
    /// selector (1 / BTURR). The opcode-5 planner enforces active category-0
    /// architect, idle state 1, no reinitialization, and selector bounds.
    /// This does not set readiness, choose a site, or advance callbacks.
    pub fn start_selected_building(&mut self)->Result<(),String> {
        self.start_building_command(None)
    }
    pub fn start_selected_building_type(&mut self,selector:u8)->Result<(),String> {
        self.start_building_command(Some(selector))
    }
    /// Port chooser geometry, original select-then-target construction semantics.
    pub fn choose_building_type(&mut self,selector:u8)->Result<(),String>{
        self.clear_mule_focus();
        if self.scene!=Scene::Mission||!self.building_choices().contains(&selector){return Err("building is not available".into());}
        self.placement_selector=Some(selector);self.error=None;
        self.building_feedback=Some(format!("{} SELECTED - RIGHT CLICK GROUND TO BUILD",building_menu::label(selector)));
        self.trace.push(format!("ConstructionSelected: {selector}"));Ok(())
    }
    pub fn place_selected_building_type(&mut self,selector:u8,screen:[i32;2])->Result<(),String>{
        if self.scene!=Scene::Mission{return Err("construction placement requires mission scene".into());}
        let m=self.mission.as_mut().ok_or("mission scene lacks world")?;
        let MissionState::Landing(current)=&m.world else{return Err("construction placement requires owned actor world".into());};
        let mut proposed=current.clone();
        let result=proposed.place_selected_building(&m.resources,m.target,screen,selector)?;
        let frame=proposed.render(&m.resources,m.target)?;m.world=MissionState::Landing(proposed);m.frame=frame;
        self.placement_selector=None;self.error=None;self.building_feedback=Some(format!("BUILD ORDER: {}",building_menu::label(selector)));
        self.trace.push(format!("ConstructionPlacement: {result}"));Ok(())
    }
    fn start_building_command(&mut self,selector:Option<u8>)->Result<(),String> {
        self.clear_mule_focus();
        if self.scene!=Scene::Mission {return Err("build-start requires the mission scene".into());}
        let m=self.mission.as_mut().ok_or("mission scene lacks world")?;
        let MissionState::Landing(current)=&m.world else{return Err("build-start requires the architect development world".into());};
        if current.runtime_context.is_some() {
            let player=current.world.player(0).map_err(|e|e.to_string())?;
            if !genewars_runtime::building_availability::available(player.bytes(),selector.unwrap_or(1)).map_err(|e|format!("{e:?}"))?{return Err("building is not available".into());}
        }
        let mut proposed=current.clone();
        let slot=proposed.selected.ok_or("select an architect first")?;
        let plan=match selector{Some(selector)=>proposed.assign_selected_building(selector)?,None=>proposed.assign_selected_bturr()?};
        let frame=proposed.render(&m.resources,m.target)?;
        m.world=MissionState::Landing(proposed);
        m.frame=frame;
        self.error=None;
        self.trace.push(format!("DevelopmentBuildStart: architect {slot}, selector {}, setup remains callback-driven",plan.building_selector));
        Ok(())
    }

    pub fn escape(&mut self) {
        if self.mule_focus.is_some(){self.clear_mule_focus();return;}
        if self.genepod.take().is_some(){self.building_feedback=None;return;}
        if self.building_menu_open{self.building_menu_open=false;self.building_menu_capture=None;return;}
        if self.placement_selector.take().is_some(){self.building_feedback=None;return;}
        match self.scene {Scene::Menu=>self.transition(Scene::Quit),Scene::Quit=>{},_=>{self.automatic=false;self.mission=None;self.victory=None;self.campaign=None;self.error=None;self.transition(Scene::Menu);}}
    }
    /// Packet21 continuation enters the source victory view on a fresh key edge.
    fn enter_victory_view(&mut self)->Result<(),String>{
        let m=self.mission.as_mut().ok_or("victory requires a mission")?;
        if m.blocked.is_some(){return Err("resolve the engine boundary before result entry".into());}
        let MissionState::Landing(current)=&m.world else{return Err("victory requires owned mission state".into());};
        let mut proposed=current.clone();
        let context=proposed.runtime_context.as_mut().ok_or("victory requires source runtime context")?;
        let request=genewars_runtime::result_transition::continue_after_win(context.known_runtime_flags(),context.runtime_mode().ok_or("unknown runtime mode")?).map_err(|e|format!("{e:?}"))?;
        if request!=Some(genewars_runtime::result_transition::ResultViewRequest::Victory1863DC){return Err("no original victory request".into());}
        let ticket=proposed.world.prepare_victory_entry(context,0,0).map_err(|e|format!("{e:?}"))?;
        let resources=super::result_scene::VictoryResources::load(&self.root)?;
        let(image,receipt)=resources.render(&ticket.plan().players_after[0])?;
        //7DBC0 ignores the movie service's failed-open return. This host has
        //no admitted decoder or mixer, so record unavailable, never played.
        //The original assets, palette mappings, text bounds and static view
        //are prepared above; transition() closes controls and consumes input.
        let services=genewars_runtime::actor_world::VictoryPresentationStatus::Completed{
            ordered_service_calls:ticket.plan().service_calls.clone(),
            biowin_receipt:"MovieUnavailable: Rust decoder not implemented; source1BF5D/7DBC0 ignores playback failure".into(),
            audio_receipt:"AudioUnavailable: native mixer not implemented; no playback asserted".into(),
        };
        let entry=proposed.world.commit_victory_entry(context,&ticket,&services).map_err(|e|format!("{e:?}"))?;
        let frame=proposed.render(&m.resources,m.target)?;
        m.world=MissionState::Landing(proposed);m.frame=frame;
        self.victory=Some(VictoryDisplay{image,receipt,entry});self.automatic=false;self.error=None;
        self.transition(Scene::Victory);Ok(())
    }
    /// Source7DDD0 then72E50: project carried state and discard the old arena.
    fn enter_campaign_view(&mut self)->Result<(),String>{
        let m=self.mission.as_ref().ok_or("campaign continuation requires mission state")?;
        let MissionState::Landing(w)=&m.world else{return Err("campaign continuation requires owned world".into());};
        let context=w.runtime_context.as_ref().ok_or("campaign continuation requires runtime context")?;
        let progress=w.world.player(0).map_err(|e|e.to_string())?.bytes()[0x6e7];
        let request=genewars_runtime::result_transition::advance_victory(1,context.known_runtime_flags(),progress).map_err(|e|format!("{e:?}"))?;
        if request!=Some(genewars_runtime::result_transition::ResultViewRequest::Campaign186F19){return Err("no original campaign request".into());}
        let carry=w.world.plan_campaign_carry(&self.carry_records,context.known_runtime_flags(),context.runtime_mode().ok_or("unknown result mode")?,0,w.selected.map(|s|s as u8)).map_err(|e|format!("campaign carry: {e:?}"))?;
        let resources=genewars_core::cd_candidate::CampaignResourcePlan::new(carry.players[0][0x6e7]as i8,self.campaign_system).map_err(|e|format!("{e:?}"))?;
        let display=campaign_scene::CampaignDisplay::load(&self.root,resources)?;
        let saved_frame=self.victory.as_ref().ok_or("campaign continuation lacks victory entry")?.entry.saved_frame;
        let resume=genewars_runtime::result_transition::resume_after_view_entry(context.runtime_mode().unwrap()&!1,0,carry.frame_after,saved_frame);
        let state=CampaignState{carry,world_priors:w.world.backing_bytes()[0x34ba..0x3550].try_into().unwrap(),rng_ring:w.world.rng().ring,mode:resume.mode_after,frame:resume.frame_after,ui_animations:w.ui_animations,display};
        self.campaign_system=state.display.resources.effective_system;self.carry_records=state.carry.records_184d7c;
        self.campaign=Some(state);self.victory=None;self.mission=None;self.automatic=false;self.error=None;
        self.transition(Scene::Campaign);Ok(())
    }
    pub fn toggle_run(&mut self) {
        if self.scene==Scene::Campaign {
            let loaded=(||{
                let c=self.campaign.as_ref().ok_or("missing campaign continuation")?;
                let (resources,source,target)=Resources::load_mission(&self.root,genewars_runtime::identity::CdCampaignMission::Krell)?;
                let world=MissionState::Landing(SharedMission::load_carried_cd(&self.root,source.source_bytes(),&c.carry,&c.world_priors,c.rng_ring,c.mode,c.ui_animations)?);
                let frame=world.render(&resources,target)?;
                Ok::<_,String>(Mission{resources,world,target,frame,ticks:0,blocked:None,changed_frames:0})
            })();
            match loaded {Ok(m)=>{self.mission=Some(m);self.campaign=None;self.error=None;self.ticks.clear();self.transition(Scene::Mission);},Err(e)=>{self.error=Some(e);}}
            return;
        }

        if self.scene==Scene::Victory {
            if let Err(error)=self.enter_campaign_view(){self.error=Some(error);}return;
        }
        if self.scene==Scene::Mission&&self.mission.as_ref().is_some_and(|m|matches!(&m.world,MissionState::Landing(w) if w.runtime_context.as_ref().is_some_and(|c|matches!(c.outcome(),genewars_runtime::mission_outcome::MissionOutcome::Won{..})))) {
            if let Err(error)=self.enter_victory_view(){self.error=Some(error);}return;
        }
        if self.scene==Scene::Options {self.start_running=!self.start_running;}
        else if self.scene==Scene::Mission && self.mission.as_ref().is_some_and(|m|m.blocked.is_none()) {self.automatic=!self.automatic;}
    }

    pub fn image(&self)->Result<RgbImage,String> {
        let selected=match self.controls.selected(){Some(MenuControl::Id0)=>1,Some(MenuControl::Id3)=>2,Some(MenuControl::Id2)=>3,_=>0};
        let flat = RgbImage::from_rgb24(640,480,vec![18;640*480*3]).map_err(|e|e.to_string())?;
        let (base,lines)=match self.scene {
            Scene::Menu=>(&self.images[selected],vec!["RUST DEVELOPMENT BUILD - SINGLE PLAYER: LOAD WORLD".into(),if self.reference_startup{"FIRST MISSION - RECONSTRUCTION IN PROGRESS".into()}else if self.landing_seed.is_some(){"LANDING STUDY - OTHER SYSTEMS PAUSED".into()}else{"PLANTS ONLY - PLAYER COMMANDS NOT IMPLEMENTED".into()}]),
            Scene::Loading=>(&self.images[0],vec!["LOADING CONQ001 - VERIFYING RESOURCE HASHES AND WORLD LINKS".into()]),
            Scene::Options=>(&flat,vec!["DEVELOPMENT OPTIONS".into(),format!("AUTOMATIC WORLD STEPS: {} - SPACE / ENTER: TOGGLE",if self.start_running{"ON"}else{"OFF"}),"10 STEPS/SECOND - ORIGINAL TIMING NOT ESTABLISHED".into(),"ESC: RETURN TO MENU".into()]),
            Scene::Mission=>{
                let m=self.mission.as_ref().ok_or("mission scene lacks world")?;
                let status=m.blocked.as_deref().map(|e|format!("STOPPED: {e}")).unwrap_or_else(||format!("{} - SPACE: STEP / ENTER: RUN / ESC: MENU",if self.automatic{"RUNNING"}else{"PAUSED"}));
                let status=if let MissionState::Landing(w)=&m.world {
                    match w.runtime_context.as_ref().filter(|_|m.blocked.is_none()).map(|c|c.outcome()) {
                        Some(genewars_runtime::mission_outcome::MissionOutcome::Won{..})=>"MISSION WON - ENTER: RESULTS".into(),
                        Some(genewars_runtime::mission_outcome::MissionOutcome::Lost{..})=>"MISSION LOST - RESULT FLOW INTEGRATION PENDING".into(),
                        _=>status,
                    }
                }else{status};
                let lines=if let MissionState::Landing(w)=&m.world {
                    vec![format!("{} - STEP {} - FOCUS {}",match w.runtime_context.as_ref().map(|c|c.origin()){Some(genewars_runtime::mission_context::ContextOrigin::CarriedCdKrell)=>"KRELL - RECONSTRUCTION",Some(_)=>"EXCELAND - RECONSTRUCTION",None=>"LANDING STUDY"},m.ticks,self.mule_focus.map(|s|format!("MULE {s}")).or_else(||w.selected.map(|s|format!("ACTOR {s}"))).unwrap_or("NONE".into())),
                        self.error.clone().or_else(||self.mule_focus.and(self.building_feedback.clone())).or_else(||w.feedback.clone()).or_else(||self.building_feedback.clone()).unwrap_or(status),"TAB: ACTOR / M: PORT MULE FOCUS / C: BUILD".into(),if w.runtime_context.is_some(){let p=w.world.player(0).map_err(|e|e.to_string())?.bytes();if p[0x6e7]==1 {format!("ZONE CREDIT: {}/1000 - RESOURCES: {}",i32::from_le_bytes(p[0xd..0x11].try_into().unwrap()),i32::from_le_bytes(p[0x4a3..0x4a7].try_into().unwrap()))}else{format!("PHOTON PROCESSORS: {}/3 - RESOURCES: {}",p[0x606],i32::from_le_bytes(p[0x4a3..0x4a7].try_into().unwrap()))}}else{"PORT CONTROLS; SOURCE OPCODE16 PLACEMENT / B: DEV BTURR".into()}]
                }else{vec![format!("PLANTS {} - STEP {} - CURSOR {}",m.world.plant_count(),m.ticks,m.world.schedule_cursor()),status,"ARROWS: VIEW - OTHER ENTITY SYSTEMS SUSPENDED".into()]};
                (&m.frame.image,lines)
            },
            Scene::Campaign=>{
                let campaign=self.campaign.as_ref().ok_or("campaign scene lacks carried state")?;
                (&campaign.display.image,vec![format!("NEXT MISSION: {}",campaign.display.name),self.error.clone().unwrap_or("ENTER: START MISSION".into())])
            },
            Scene::Victory=>{
                let result=self.victory.as_ref().ok_or("victory scene lacks prepared display")?;
                (&result.image,vec![self.error.clone().unwrap_or("ENTER: CONTINUE".into())])
            },
            Scene::Error=>(&flat,vec!["MISSION LOAD FAILED - NO WORLD COMMITTED".into(),self.error.clone().unwrap_or_default(),"ESC: RETURN TO MENU".into()]),
            Scene::Quit=>(&self.images[0],vec!["CLOSING".into()]),
        };
        if self.scene==Scene::Mission&&self.building_menu_open {
            let m=self.mission.as_ref().unwrap();let MissionState::Landing(w)=&m.world else{return Err("construction chooser requires actor world".into());};
            let player=w.world.player(0).map_err(|e|format!("{e:?}"))?;
            let funds=i32::from_le_bytes(player.bytes()[0x4a3..0x4a7].try_into().unwrap());
            return status_text::overlay(base,&building_menu::lines(&self.building_choices(),funds,w.selected,self.building_feedback.as_deref()));
        }
        let mut composed=if matches!(self.scene,Scene::Mission|Scene::Victory) {status_text::mission_overlay(base,&lines)}else{status_text::overlay(base,&lines)}?;
        if self.scene==Scene::Mission {if let (Some(panel),Some(m))=(&self.genepod,&self.mission){
            let mut indices=vec![0u8;640*480];panel.control.draw_face([0,0],[640,480],640,&mut indices)?;
            let mut rgb=composed.rgb_bytes().to_vec();let palette=m.resources.palette_entries();
            for y in 344..373{for x in 26..55{if panel.control.hit([0,0],Some([x,y])){let i=y as usize*640+x as usize;rgb[i*3..i*3+3].copy_from_slice(&palette[usize::from(indices[i])].map(|v|(u16::from(v)*255/63)as u8));}}}
            composed=RgbImage::from_rgb24(640,480,rgb).map_err(|e|e.to_string())?;
        }}Ok(composed)
    }

    pub fn report(&self)->String {
        let trace=self.trace.iter().map(|s|json_string(s)).collect::<Vec<_>>().join(",");
        let mission=self.mission.as_ref().map(|m|format!("{{\"steps\":{},\"plants\":{},\"cursor\":{},\"target\":[{},{}],\"changed_world_frames\":{},\"world_sha256\":{},\"rgb_sha256\":{},\"queue\":{:?},\"blocked\":{},{}}}",m.ticks,m.world.plant_count(),m.world.schedule_cursor(),m.target[0],m.target[1],m.changed_frames,json_string(&m.frame.world_sha256),json_string(&m.frame.rgb_sha256),m.frame.queue,m.blocked.as_ref().map(|s|json_string(s)).unwrap_or("null".into()),m.world.extra_report())).unwrap_or("null".into());
        let campaign=self.campaign.as_ref().map(|c|format!("{{\"progress\":{},\"next_mission\":{},\"carry_projected\":true,\"next_mission_initialized\":false,\"retired_actors\":{:?},\"players_sha256\":{},\"preserved_world_sha256\":{},\"rng_ring\":{},\"mode_after_exit\":{},\"frame_after_view_resume\":{}}}",c.carry.players[0][0x6e7],json_string(&c.display.name),c.carry.retired_actors,json_string(&genewars_runtime::identity::sha256_hex(&c.carry.players.concat())),json_string(&genewars_runtime::identity::sha256_hex(&c.world_priors)),c.rng_ring,c.mode,c.frame)).unwrap_or("null".into());
        let result_view=self.victory.as_ref().map(|v|format!("{{\"stage\":{},\"saved_frame\":{},\"feedback_text_index\":{},\"statistics\":{:?},\"movie\":\"unavailable_decoder\",\"audio\":\"unavailable_mixer\",\"campaign_transition_accepted\":false}}",v.entry.stage_after,v.entry.saved_frame,v.receipt.feedback_text_index,v.receipt.statistic_values)).unwrap_or("null".into());
        format!("{{\"scene\":{},\"scene_history\":[{}],\"result_view\":{},\"campaign\":{},\"mission\":{},\"ticks\":[{}],\"presented_frames\":{},\"error\":{},\"construction_menu_open\":{},\"construction_choices\":{:?},\"scene_dispatch\":true,\"gameplay\":false,\"full_world_tick\":false,\"original_startup_sequence\":false,\"timer_policy\":\"port-owned 10 Hz when enabled\",\"render_policy\":\"full redraw; source-player0 view; radius4096 view-only factor; GENEPANL development backing\"}}\n",json_string(&format!("{:?}",self.scene)),trace,result_view,campaign,mission,self.ticks.join(","),self.presented,self.error.as_ref().map(|s|json_string(s)).unwrap_or("null".into()),self.building_menu_open,self.building_choices())
    }
}


#[cfg(test)] mod genepod_native_fixture_tests {
    use super::*;
    #[test]
    fn original_alpha_and_game_dispatch_drive_capture_without_forced_activation() {
        // Synthetic completed building fixture, explicitly NOT campaign acceptance.
        let root=Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let mut game=Game::open(&root).unwrap();game.reference_startup=true;game.scene=Scene::Loading;game.finish_loading();
        assert_eq!(game.scene,Scene::Mission,"{:?}",game.error);
        let (slot,click)={
            let m=game.mission.as_mut().unwrap();let MissionState::Landing(w)=&mut m.world else{panic!()};
            let slot=w.world.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;
            w.world.progress_admitted_grade0_building(slot,280,false).unwrap();
            m.target=[19839,12927];m.frame=w.render(&m.resources,m.target).unwrap();
            let r=w.world.buildings().unwrap().record(slot as usize).unwrap();
            let camera=genewars_core::cd_candidate::camera_cells_for_target(w.world.map(),m.target[0],m.target[1]).unwrap();
            let origin=genewars_core::cd_candidate::entity_view_position(w.world.map(),[19839,12927],camera,0).unwrap();
            let super::super::runtime_resources::BuildingResourceRef::Source(db)=m.resources.building_resource(0).unwrap()else{panic!()};
            let frames=db.animation(r[25]as usize).unwrap();
            let click=(0..480).flat_map(|y|(128..640).map(move|x|[x,y])).find(|&q|super::super::building_pick::hit_animation(frames,r[27],origin,q).unwrap().is_some()).unwrap();
            (slot,click)
        };
        game.pointer(Some((click[0],click[1])));game.left(true);game.left(false);
        assert_eq!(game.genepod.as_ref().map(|p|p.slot),Some(slot),"{:?}",game.error);
        game.image().unwrap();
        let before=game.mission.as_ref().unwrap().world.clone();
        game.pointer(Some((26,344)));game.left(true);game.left(false);
        assert!(!game.genepod.as_ref().unwrap().input.capture.captured);
        assert_eq!(game.mission.as_ref().unwrap().world.extra_report(),before.extra_report());
        game.pointer(Some((40,358)));game.left(true);
        assert!(game.genepod.as_ref().unwrap().input.capture.captured);
        assert_eq!(game.genepod.as_ref().unwrap().state,2);
        game.pointer(Some((60,358)));game.left(false);
        assert!(game.trace.last().unwrap().contains("event=32776"));
        assert!(!game.genepod.as_ref().unwrap().input.capture.captured);
        game.pointer(Some((40,358)));game.left(true);game.left(false);
        assert!(game.trace.last().unwrap().contains("event=8"),"{:?}",game.trace.last());
        assert!(game.building_feedback.as_ref().unwrap().starts_with("GENEPOD:"));
        assert!(game.error.is_none(),"{:?}",game.error);
        game.image().unwrap();
    }
}

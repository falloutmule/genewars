//! Native development route. Original campaign movie/confirmation is pending.
use std::path::{Path,PathBuf};
use genewars_assets::{Palette256,PaletteChannelMode};
use genewars_core::cd_candidate::{MenuCallback0State,MenuPacket,MenuControl};
use genewars_render::{RgbImage,CdTitleButton};
use genewars_runtime::world::MissionWorld;
use super::{menu_art,runtime_resources::{read_bound,Resources,Frame},status_text};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Scene { Menu, Loading, Mission, Options, Error, Quit }

pub struct Mission {
    pub resources: Resources, pub world: MissionWorld, pub target: [i16;2], pub frame: Frame,
    pub ticks: u64, pub blocked: Option<String>, pub changed_frames: u64,
}

pub struct Game {
    root: PathBuf,
    pub scene: Scene, pub mission: Option<Mission>,
    images: Vec<RgbImage>, button: CdTitleButton,
    controls: MenuCallback0State, cursor: Option<(i32,i32)>, left: bool, right: bool, mask: u8,
    pub automatic: bool, pub start_running: bool, pub error: Option<String>,
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
        Ok(Self { root:root.into(), scene:Scene::Menu,mission:None,images,button,
            controls:MenuCallback0State::default(),cursor:None,left:false,right:false,mask:0,
            automatic:false,start_running:false,error:None,trace:vec!["Menu".into()],ticks:Vec::new(),presented:0 })
    }

    fn transition(&mut self, scene:Scene) {
        self.scene=scene; self.trace.push(format!("{scene:?}"));
        self.controls=MenuCallback0State::default();self.mask=0;self.left=false;self.right=false;
    }

    fn hit(&self,c:MenuControl)->bool {
        let Some((x,y))=self.cursor else {return false;};
        let top=match c {MenuControl::Id0=>115,MenuControl::Id3=>186,MenuControl::Id2=>257,_=>return false};
        match (x.checked_sub(140),y.checked_sub(top)) { (Some(x),Some(y))=>self.button.hit_test(x,y), _=>false }
    }

    pub fn pointer(&mut self,cursor:Option<(i32,i32)>) {self.cursor=cursor;self.packet(false,false);}
    pub fn left(&mut self,down:bool) {let changed=self.left!=down;self.left=down;if changed {self.packet(down,!down);}}
    pub fn right(&mut self,down:bool) {self.right=down;self.packet(false,false);}
    pub fn cancel_capture(&mut self) {self.cursor=None;self.left=false;self.right=false;self.packet(false,true);}

    fn packet(&mut self,press:bool,release:bool) {
        if self.scene!=Scene::Menu {return;}
        let first=MenuControl::hit_order().into_iter().find(|&c|self.hit(c));
        let captured=self.controls.captured().is_some_and(|c|self.hit(c));
        let result=self.controls.update(MenuPacket {byte8:u8::from(press)|(u8::from(release)<<1)|(u8::from(self.left)<<2),byte9:u8::from(self.right)},first,captured);
        if let Some(mask)=result.input_mask_write {self.mask=mask;}
        if result.event_id>=0 && self.mask&8!=0 {
            match result.event_id {0=>self.transition(Scene::Loading),3=>self.transition(Scene::Options),2=>self.transition(Scene::Quit),_=>{}}
        }
    }

    /// Invoked after a loading redraw, and by the identical headless driver.
    pub fn finish_loading(&mut self) {
        if self.scene!=Scene::Loading {return;}
        let loaded=Resources::load(&self.root).and_then(|(resources,world,target)| {
            let frame=resources.render(&world,target)?;
            Ok(Mission {resources,world,target,frame,ticks:0,blocked:None,changed_frames:0})
        });
        match loaded {Ok(mission)=>{self.mission=Some(mission);self.automatic=self.start_running;self.error=None;self.ticks.clear();self.transition(Scene::Mission);},
            Err(e)=>{self.error=Some(e);self.mission=None;self.transition(Scene::Error);}}
    }

    pub fn tick(&mut self) {
        if self.scene!=Scene::Mission {return;}
        let Some(m)=&mut self.mission else {return;};
        if m.blocked.is_some() {return;}
        if self.ticks.len()>=4096 {m.blocked=Some("Development trace limit (4096 steps) reached".into());return;}
        // Cross-subsystem transaction: a drawable frame must exist before the
        // tentative world can replace the currently displayed one.
        let mut proposed=m.world.clone();
        let result=proposed.tick_plants().map_err(|e|e.to_string()).and_then(|event| {
            let frame=m.resources.render(&proposed,m.target)?; Ok((event,frame))
        });
        match result {
            Ok((event,frame))=>{
                if frame.rgb_sha256!=m.frame.rgb_sha256 {m.changed_frames+=1;}
                m.ticks+=1;
                self.ticks.push(format!("{{\"step\":{},\"world_sha256\":{},\"rgb_sha256\":{},\"event\":{}}}",m.ticks,json_string(&frame.world_sha256),json_string(&frame.rgb_sha256),json_string(&format!("{event:?}"))));
                m.world=proposed;m.frame=frame;
            },
            Err(e)=>{m.blocked=Some(e);self.automatic=false;}
        }
    }

    pub fn pan(&mut self,dx:i16,dy:i16) {
        if self.scene!=Scene::Mission {return;}
        let Some(m)=&mut self.mission else {return;};
        let target=[m.target[0].wrapping_add(dx),m.target[1].wrapping_add(dy)];
        match m.resources.render(&m.world,target) {Ok(frame)=>{m.frame=frame;m.target=target;},Err(e)=>self.error=Some(e)}
    }

    pub fn escape(&mut self) {
        match self.scene {Scene::Menu=>self.transition(Scene::Quit),Scene::Quit=>{},_=>{self.automatic=false;self.mission=None;self.error=None;self.transition(Scene::Menu);}}
    }
    pub fn toggle_run(&mut self) {
        if self.scene==Scene::Options {self.start_running=!self.start_running;}
        else if self.scene==Scene::Mission && self.mission.as_ref().is_some_and(|m|m.blocked.is_none()) {self.automatic=!self.automatic;}
    }

    pub fn image(&self)->Result<RgbImage,String> {
        let selected=match self.controls.selected(){Some(MenuControl::Id0)=>1,Some(MenuControl::Id3)=>2,Some(MenuControl::Id2)=>3,_=>0};
        let flat = RgbImage::from_rgb24(640,480,vec![18;640*480*3]).map_err(|e|e.to_string())?;
        let (base,lines)=match self.scene {
            Scene::Menu=>(&self.images[selected],vec!["RUST DEVELOPMENT BUILD - SINGLE PLAYER: LOAD WORLD".into(),"PLANTS ONLY - PLAYER COMMANDS NOT IMPLEMENTED".into()]),
            Scene::Loading=>(&self.images[0],vec!["LOADING CONQ001 - VERIFYING RESOURCE HASHES AND WORLD LINKS".into()]),
            Scene::Options=>(&flat,vec!["DEVELOPMENT OPTIONS".into(),format!("AUTOMATIC PLANT STEPS: {} - SPACE / ENTER: TOGGLE",if self.start_running{"ON"}else{"OFF"}),"10 STEPS/SECOND - ORIGINAL TIMING NOT ESTABLISHED".into(),"ESC: RETURN TO MENU".into()]),
            Scene::Mission=>{
                let m=self.mission.as_ref().ok_or("mission scene lacks world")?;
                let status=m.blocked.as_deref().map(|e|format!("STOPPED: {e}")).unwrap_or_else(||format!("{} - SPACE: STEP / ENTER: RUN / ESC: MENU",if self.automatic{"RUNNING"}else{"PAUSED"}));
                (&m.frame.image,vec![format!("PLANTS {} - STEP {} - CURSOR {}",m.world.plant_count(),m.ticks,m.world.schedule_cursor()),status,"ARROWS: VIEW - OTHER ENTITY SYSTEMS SUSPENDED".into()])
            },
            Scene::Error=>(&flat,vec!["MISSION LOAD FAILED - NO WORLD COMMITTED".into(),self.error.clone().unwrap_or_default(),"ESC: RETURN TO MENU".into()]),
            Scene::Quit=>(&self.images[0],vec!["CLOSING".into()]),
        };
        status_text::overlay(base,&lines)
    }

    pub fn report(&self)->String {
        let trace=self.trace.iter().map(|s|json_string(s)).collect::<Vec<_>>().join(",");
        let mission=self.mission.as_ref().map(|m|format!("{{\"steps\":{},\"plants\":{},\"cursor\":{},\"target\":[{},{}],\"changed_world_frames\":{},\"world_sha256\":{},\"rgb_sha256\":{},\"queue\":{:?},\"blocked\":{}}}",m.ticks,m.world.plant_count(),m.world.schedule_cursor(),m.target[0],m.target[1],m.changed_frames,json_string(&m.frame.world_sha256),json_string(&m.frame.rgb_sha256),m.frame.queue,m.blocked.as_ref().map(|s|json_string(s)).unwrap_or("null".into()))).unwrap_or("null".into());
        format!("{{\"scene\":{},\"scene_history\":[{}],\"mission\":{},\"ticks\":[{}],\"presented_frames\":{},\"error\":{},\"scene_dispatch\":true,\"gameplay\":false,\"full_world_tick\":false,\"original_startup_sequence\":false,\"timer_policy\":\"port-owned 10 Hz when enabled\",\"render_policy\":\"full redraw; source-player0 view; radius4096 view-only factor; GENEPANL development backing\"}}\n",json_string(&format!("{:?}",self.scene)),trace,mission,self.ticks.join(","),self.presented,self.error.as_ref().map(|s|json_string(s)).unwrap_or("null".into()))
    }
}

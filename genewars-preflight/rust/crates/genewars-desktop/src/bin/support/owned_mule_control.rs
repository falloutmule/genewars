//! Qualified port keyboard focus. Original mixed-family picking is pending.
//! Zero manager/packet priors are host state; no original selection flags are written.
use super::{Game,Scene,MissionState};
use genewars_runtime::{creature_world::CreatureAnimationBinding,mule_human_producer as producer};

impl Game {
    pub(super) fn clear_mule_focus(&mut self) {
        if self.mule_focus.take().is_some(){self.building_feedback=None;}
        self.mule_press=false;
    }
    pub fn focus_next_mule(&mut self) {
        self.mule_press=false;
        if self.scene!=Scene::Mission||self.building_menu_open||self.placement_selector.is_some(){return;}
        let Some(m)=&self.mission else{return;};
        if m.blocked.is_some(){return;}
        let MissionState::Landing(w)=&m.world else{return;};
        if w.runtime_context.as_ref().is_some_and(|c|c.outcome()!=genewars_runtime::mission_outcome::MissionOutcome::Running){return;}
        let Some(pool)=w.world.creatures()else{return;};
        let slots=pool.records().iter().enumerate().filter_map(|(slot,r)|
            (r[6]==1&&r[24]==0&&r[122]==4&&r[94]<24&&i16::from_le_bytes([r[15],r[16]])==slot as i16&&pool.binding(slot).ok()==Some(Some(CreatureAnimationBinding::MuleSelector4))).then_some(slot as u16)).collect::<Vec<_>>();
        let next=slots.iter().copied().find(|s|self.mule_focus.is_none_or(|old|*s>old)).or_else(||slots.first().copied());
        self.clear_mule_focus();
        if let Some(slot)=next {
            self.mule_focus=Some(slot);self.genepod=None;self.automatic=false;self.error=None;
            self.building_feedback=Some(format!("MULE {slot} - LEFT CLICK GROUND (PORT CONTROL)"));
            self.trace.push(format!("PortMuleFocus:{slot}"));
        }
    }
    pub(super) fn mule_port_packet(&mut self,press:bool,release:bool)->Result<(),String>{
        let terrain=self.cursor.is_some_and(|(x,y)|(128..640).contains(&x)&&(0..480).contains(&y));
        if press{self.mule_press=terrain&&!self.right;}
        if !release{return Ok(());}
        let armed=std::mem::take(&mut self.mule_press);
        if !armed||!terrain||self.right{return Ok(());}
        let slot=self.mule_focus.ok_or("no port Mule focus")?;
        let m=self.mission.as_mut().ok_or("mission unavailable")?;
        if m.blocked.is_some(){return Err("resolve the engine boundary before issuing orders".into());}
        let MissionState::Landing(current)=&m.world else{return Err("owned world unavailable".into());};
        if current.runtime_context.as_ref().is_some_and(|c|c.outcome()!=genewars_runtime::mission_outcome::MissionOutcome::Running){return Err("mission has ended".into());}
        let camera=genewars_core::cd_candidate::camera_cells_for_target(current.world.map(),m.target[0],m.target[1]).map_err(|e|e.to_string())?;
        let (x,y)=self.cursor.unwrap();
        let point=genewars_runtime::terrain_target::project_terrain_target(current.world.map(),camera,[x,y]).map_err(|e|e.to_string())?.point;
        let mut proposed=current.clone();
        let owner=proposed.world.player(0).map_err(|e|e.to_string())?.bytes()[1];
        let receipt=proposed.world.dispatch_owned_mule_action5(Some(slot),self.mule_packet,&self.mule_manager,producer::ProducerInput{selected_player_index:0,selected_player_owner:owner,suppression_flags:0,pause_flags:0,hover_consumed_cl:0,projected_point:point},false).map_err(|e|format!("{e:?}"))?;
        if receipt.command.is_none(){return Err(format!("Mule order not issued: {:?}",receipt.producer.outcome));}
        let frame=proposed.render(&m.resources,m.target)?;
        proposed.command_count+=1;
        m.world=MissionState::Landing(proposed);m.frame=frame;
        self.mule_manager=receipt.producer.manager_after;self.mule_packet=receipt.producer.packet_after;self.error=None;
        self.trace.push(format!("PortMuleOrder:{slot}:{point:?}"));Ok(())
    }
}

#[cfg(test)] mod tests {
    use super::*;
    fn game()->Game {
        let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
        let mut game=Game::open(&root).unwrap();game.landing_seed=Some(5428);game.scene=Scene::Loading;game.finish_loading();game
    }
    fn world(game:&Game)->genewars_runtime::actor_world::ActorWorld {
        let MissionState::Landing(w)=&game.mission.as_ref().unwrap().world else{panic!()};w.world.clone()
    }
    #[test] fn port_mule_capture_cannot_issue_on_unarmed_panel_or_cancelled_release(){
        let mut game=game();let before=world(&game);
        // Synthetic frontend focus only: all existing creatures remain neutral.
        game.mule_focus=Some(0);game.pointer(Some((320,240)));game.left(false);
        assert!(game.error.is_none());assert_eq!(world(&game),before);
        game.pointer(Some((40,358)));game.left(true);game.pointer(Some((320,240)));game.left(false);
        assert!(game.error.is_none());assert_eq!(world(&game),before);
        game.left(true);game.cancel_capture();game.pointer(Some((320,240)));game.left(false);
        assert!(game.error.is_none());assert_eq!(game.mule_focus,None);assert_eq!(world(&game),before);
        game.mule_focus=Some(0);game.left(true);game.pointer(None);game.pointer(Some((320,240)));game.left(false);
        assert!(game.error.is_none());assert_eq!(world(&game),before);
    }
    #[test] fn port_mule_focus_expires_before_callbacks_and_construction(){
        let mut game=game();let before=world(&game);
        game.focus_next_mule();assert_eq!(game.mule_focus,None,"neutral creatures are not selectable");
        game.mule_focus=Some(0);game.mule_press=true;game.right(true);
        assert_eq!(game.mule_focus,None);assert!(!game.mule_press);assert_eq!(world(&game),before);
        game.mule_focus=Some(0);game.mule_press=true;let _=game.start_selected_building();
        assert_eq!(game.mule_focus,None);assert!(!game.mule_press);assert_eq!(world(&game),before);
        game.mule_focus=Some(0);game.mule_press=true;game.tick();
        assert_eq!(game.mule_focus,None);assert!(!game.mule_press);
    }
}

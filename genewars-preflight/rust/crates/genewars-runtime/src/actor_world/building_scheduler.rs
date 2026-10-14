//!47F80 admitted selector1 callbacks and player maintenance.
use super::*;
use crate::bookkeeping::{self,BookkeepingNotice};
use crate::building_callback::IncompleteBuildingPlan;
use crate::building_resources::BuildingResourceDatabase;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct IncompleteBuildingPass {
    pub callbacks:Vec<(u16,IncompleteBuildingPlan)>,
    pub notices:Vec<(u8,BookkeepingNotice)>,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Selector1BuildingCallback {
    Incomplete{slot:u16,plan:IncompleteBuildingPlan},
    QuietCompleted{slot:u16,plan:crate::completed_building::QuietBuildingPlan},
    TrackingCompleted{slot:u16,plan:crate::completed_building::TurretTrackingPlan},
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Selector1BuildingPass {
    pub callbacks:Vec<Selector1BuildingCallback>,
    pub notices:Vec<(u8,BookkeepingNotice)>,
}

impl ActorWorld {
    ///47F80 slot order with incomplete and quiet completed selector1 callbacks.
    /// Target acquisition, newborn/destruction and other selectors remain
    /// explicit errors. A later callback error rolls back the entire pass.
    pub fn step_selector1_building_callbacks(&mut self,player_count:u8,selected:u8,
        resource:&BuildingResourceDatabase)->Result<Selector1BuildingPass,ActorWorldError> {
        self.step_selector1_building_pass(player_count,selected,resource,None)
    }

    ///47F80 slot order with the source frame counter and physics table used
    /// by completed turrets. Projectile creation, newborn/destruction and
    /// other selectors remain explicit errors with whole-pass rollback.
    pub fn step_tracking_selector1_building_callbacks(&mut self,player_count:u8,selected:u8,
        resource:&BuildingResourceDatabase,frame_counter:u32,
        physics:&crate::physics_tables::PhysicsTables)->Result<Selector1BuildingPass,ActorWorldError> {
        self.step_selector1_building_pass(player_count,selected,resource,Some((frame_counter,physics)))
    }

    fn step_selector1_building_pass(&mut self,player_count:u8,selected:u8,
        resource:&BuildingResourceDatabase,tracking:Option<(u32,&crate::physics_tables::PhysicsTables)>)
        ->Result<Selector1BuildingPass,ActorWorldError> {
        if player_count>4||selected>=4{return Err(ActorWorldError::InvalidPlayer(player_count.max(selected)))}
        self.validate_building_counts()?;
        let period=u16::from_le_bytes(self.backing[0x1c7a..0x1c7c].try_into().unwrap());
        let mut staged=self.clone();let mut callbacks=Vec::new();let mut notices=Vec::new();
        for owner in 0..player_count {
            let start=usize::from(owner)*64;
            bookkeeping::plan_selector1_player(staged.players[usize::from(owner)].bytes(),
                &staged.buildings.as_ref().unwrap().records()[start..start+64],selected,period)?;
            for slot in start..start+64 {
                let record=staged.buildings.as_ref().unwrap().record(slot)?;
                if record[6]==0{continue}
                callbacks.push(if record[0x31]&1!=0 {
                    if let Some((frame,physics))=tracking {
                        Selector1BuildingCallback::TrackingCompleted{slot:slot as u16,
                            plan:staged.step_tracking_completed_selector1_building(slot as u16,resource,frame,physics)?}
                    }else{
                        Selector1BuildingCallback::QuietCompleted{slot:slot as u16,
                            plan:staged.step_quiet_completed_selector1_building(slot as u16,resource)?}
                    }
                }else{
                    Selector1BuildingCallback::Incomplete{slot:slot as u16,
                        plan:staged.step_incomplete_selector1_building(slot as u16,resource)?}
                });
            }
            let before=*staged.players[usize::from(owner)].bytes();
            let plan=bookkeeping::plan_selector1_player(&before,
                &staged.buildings.as_ref().unwrap().records()[start..start+64],selected,period)?;
            staged.players[usize::from(owner)].replace_bookkeeping_checked(&before,&plan.after)?;
            notices.extend(plan.notices.into_iter().map(|notice|(owner,notice)));
        }
        *self=staged;Ok(Selector1BuildingPass{callbacks,notices})
    }

    /// Live per-player64-slot order, then47F80 player tail. Newborn marking,
    /// completed buildings and other selectors reject the entire pass.
    pub fn step_incomplete_building_callbacks(&mut self,player_count:u8,selected:u8,
        resource:&BuildingResourceDatabase)->Result<IncompleteBuildingPass,ActorWorldError> {
        if player_count>4||selected>=4{return Err(ActorWorldError::InvalidPlayer(player_count.max(selected)))}
        self.validate_building_counts()?;
        let period=u16::from_le_bytes(self.backing[0x1c7a..0x1c7c].try_into().unwrap());
        let mut staged=self.clone();let mut callbacks=Vec::new();let mut notices=Vec::new();
        for owner in 0..player_count {
            let start=usize::from(owner)*64;
            // Validate the full partition before applying its callbacks.
            bookkeeping::plan_incomplete_selector1_player(staged.players[usize::from(owner)].bytes(),
                &staged.buildings.as_ref().unwrap().records()[start..start+64],selected,period)?;
            for slot in start..start+64 {
                if staged.buildings.as_ref().unwrap().record(slot)?[6]==0{continue}
                callbacks.push((slot as u16,staged.step_incomplete_selector1_building(slot as u16,resource)?));
            }
            let before=*staged.players[usize::from(owner)].bytes();
            let plan=bookkeeping::plan_incomplete_selector1_player(&before,
                &staged.buildings.as_ref().unwrap().records()[start..start+64],selected,period)?;
            staged.players[usize::from(owner)].replace_bookkeeping_checked(&before,&plan.after)?;
            notices.extend(plan.notices.into_iter().map(|notice|(owner,notice)));
        }
        *self=staged;Ok(IncompleteBuildingPass{callbacks,notices})
    }
}

//!47F80 admitted pass: incomplete selector1 callbacks and player maintenance.
use super::*;
use crate::bookkeeping::{self,BookkeepingNotice};
use crate::building_callback::IncompleteBuildingPlan;
use crate::building_resources::BuildingResourceDatabase;

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct IncompleteBuildingPass {
    pub callbacks:Vec<(u16,IncompleteBuildingPlan)>,
    pub notices:Vec<(u8,BookkeepingNotice)>,
}

impl ActorWorld {
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

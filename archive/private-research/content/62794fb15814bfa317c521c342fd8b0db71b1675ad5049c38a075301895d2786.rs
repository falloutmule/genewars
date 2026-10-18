use super::*;
use crate::bookkeeping::{self,BookkeepingNotice,CountdownPlan};

impl ActorWorld {
    /// Source47F80 numeric player updates, requiring every building slot to
    /// be inactive. Returned notices require the separate audio/UI subsystem.
    pub fn step_empty_building_bookkeeping(&mut self,selected:u8)->Result<Vec<(u8,BookkeepingNotice)>,ActorWorldError> {
        if selected>=4{return Err(ActorWorldError::InvalidPlayer(selected));}
        if self.buildings.as_ref().is_some_and(|pool|pool.records().iter().any(|r|r[6]!=0)){return Err(ActorWorldError::UnsupportedBookkeepingFamily(4));}
        // Scan raw activity bytes, including slots outside the declared count.
        let group=self.layouts[3];
        for i in 0..group.count {if self.backing[group.file_offset+i*group.packed_stride+6]!=0{return Err(ActorWorldError::UnsupportedBookkeepingFamily(4));}}
        let period=u16::from_le_bytes(self.backing[0x1c7a..0x1c7c].try_into().unwrap());
        let mut staged=self.clone();let mut notices=Vec::new();
        for player in 0..4usize {
            let before=*staged.players[player].bytes();
            let plan=bookkeeping::plan_empty_building_player(&before,selected,period)?;
            staged.players[player].replace_bookkeeping_checked(&before,&plan.after)?;
            notices.extend(plan.notices.into_iter().map(|n|(player as u8,n)));
        }
        *self=staged;Ok(notices)
    }

    /// Source13780 empty-family8 count transition. `runtime_flags` is the
    /// explicit118770 byte. Its UI animation/output requests remain external.
    pub fn step_empty_family8_countdown(&mut self,selected:u8,runtime_flags:u8)->Result<CountdownPlan,ActorWorldError> {
        let player=self.player(selected)?;
        if self.family8.as_ref().is_some_and(|pool|pool.records().iter().any(|r|r[6]!=0)) {
            return Err(ActorWorldError::UnsupportedBookkeepingFamily(8));
        }
        let group=self.layouts[7];
        for i in 0..group.count {if self.backing[group.file_offset+i*group.packed_stride+6]!=0{return Err(ActorWorldError::UnsupportedBookkeepingFamily(8));}}
        let count=u16::from_le_bytes(self.backing[0x355c..0x355e].try_into().unwrap());
        let period=u16::from_le_bytes(self.backing[0x1c7a..0x1c7c].try_into().unwrap());
        let plan=bookkeeping::plan_empty_family8_countdown(count,period,player.progress(),player.bytes()[0x607],runtime_flags)?;
        self.backing[0x355c..0x355e].copy_from_slice(&plan.after.to_le_bytes());Ok(plan)
    }
}

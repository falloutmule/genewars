//! Final neutral BONES callback:8ACC0 unlink,724C0 references and kind1 count.
use super::*;
use crate::creature_world::CommonPhysicsPlan;
use crate::landing_effect_lifecycle::{self as lifecycle, CleanupBytePatch};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MuleRetirementReceipt {
    pub before:CreatureRecord,
    pub after:CreatureRecord,
    pub cleanup:Vec<CleanupBytePatch>,
    pub count_after:i16,
    /// Source6A1AD writes this display-phase byte; the native host keeps the
    /// request explicit rather than claiming the original presentation loop.
    pub pending_display_phase_1185a8:u8,
}

impl ActorWorld {
    pub(super) fn retire_mule_bones_after_callback(&mut self,before:&CreatureRecord,
        plan:&CommonPhysicsPlan<124>)->Result<MuleRetirementReceipt,ActorWorldError> {
        let links=self.validate_links()?;
        if !links.contains(&(1,0)){return Err(ActorWorldError::InvalidReference([0,0,1]));}
        let pool=self.creatures.as_ref().ok_or(ActorWorldError::UnsupportedCreatureSchedule)?;
        if pool.record(0)?!=before{return Err(CreatureError::StaleRecord(0).into());}
        let declared=word(&self.backing,0x3550);
        let active=pool.records().iter().filter(|record|record[6]!=0).count();
        if declared<=0||declared as usize!=active{return Err(ActorWorldError::CreatureCountMismatch{declared,active});}
        if plan.old_cell!=plan.physics_cell||plan.physics_cell!=plan.new_cell
            ||plan.old_cell!=floor_cell([word(before,7),word(before,9)])
            ||plan.after[..6]!=before[..6]||plan.after[7..11]!=before[7..11] {
            return Err(CreatureError::UnsupportedMotionState.into());
        }
        let mut staged=self.clone();
        staged.creatures.as_mut().unwrap().replace_record_checked(0,before,&plan.after)?;
        staged.unlink_mule(plan.old_cell)?;
        let dead=landing::LandingReference{index:0,kind:1};
        let mut cleanup=Vec::new();
        for(kind,slot,record)in staged.effect_cleanup_records(){
            for patch in lifecycle::plan_reference_cleanup(kind,slot,&record,dead)?{
                staged.apply_cleanup_byte(patch)?;cleanup.push(patch);
            }
        }
        let current=*staged.creatures.as_ref().unwrap().record(0)?;
        staged.creatures.as_mut().unwrap().retire_bones_checked(0,&current)?;
        let count_after=declared-1;staged.backing[0x3550..0x3552].copy_from_slice(&count_after.to_le_bytes());
        let after=*staged.creatures.as_ref().unwrap().record(0)?;
        staged.validate_links()?;
        let active=staged.creatures.as_ref().unwrap().records().iter().filter(|record|record[6]!=0).count();
        if count_after as usize!=active{return Err(ActorWorldError::CreatureCountMismatch{declared:count_after,active});}
        *self=staged;
        Ok(MuleRetirementReceipt{before:*before,after,cleanup,count_after,pending_display_phase_1185a8:11})
    }
}

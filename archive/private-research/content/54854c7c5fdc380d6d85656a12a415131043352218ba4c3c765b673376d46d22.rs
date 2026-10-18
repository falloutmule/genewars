//! Source103F0 selector1 retirement, with native-empty GUI policy.
use super::*;
use crate::landing_effect_lifecycle::{self as lifecycle, CleanupBytePatch};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8RetirementReceipt {
    pub slot:u16,
    pub before:Family8Record,
    pub after:Family8Record,
    pub cleanup:Vec<CleanupBytePatch>,
    pub count_after:u8,
    /// Source virtual48 request; audio execution is a separate subsystem.
    pub stop_audio:u16,
}

impl ActorWorld {
    /// Stop53, unlink8ACC0, clean724C0 references, clear6, then decrement count.
    /// Retired link/index/pointer bytes survive. The caller may refresh the
    /// parent only after this transaction succeeds.
    pub fn retire_family8_child(&mut self,slot:u16)->Result<Family8RetirementReceipt,ActorWorldError>{
        self.validate_links()?;self.validate_family8_count()?;
        let before=*self.family8.as_ref().unwrap().record(usize::from(slot))?;
        if before[6]!=8||before[0x55]!=1 {
            return Err(crate::family8_motion::Family8MotionError::UnsupportedRecord.into());
        }
        if !self.validate_links()?.contains(&(8,slot as i16)) {
            return Err(ActorWorldError::MissingFamily8Link(slot));
        }
        let dead=landing::LandingReference{index:slot as i16,kind:8};
        let mut staged=self.clone();staged.unlink_family8(slot)?;
        let mut cleanup=Vec::new();
        for(kind,index,record)in staged.effect_cleanup_records(){
            for patch in lifecycle::plan_reference_cleanup(kind,index,&record,dead)?{
                staged.apply_cleanup_byte(patch)?;cleanup.push(patch);
            }
        }
        let current=*staged.family8.as_ref().unwrap().record(usize::from(slot))?;
        staged.family8.as_mut().unwrap().retire_child_checked(usize::from(slot),&current)?;
        staged.backing[0x3559]-=1;
        let after=*staged.family8.as_ref().unwrap().record(usize::from(slot))?;
        let count_after=staged.backing[0x3559];
        staged.validate_links()?;staged.validate_family8_count()?;*self=staged;
        Ok(Family8RetirementReceipt{slot,before,after,cleanup,count_after,stop_audio:0x53})
    }
}

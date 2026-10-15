//! Bounded owned-Mule decision at 63ECC..64CD7, before the common fallback.
//! The caller establishes the full callback's neighborhood/eligibility gates.
//! This is not a full AI callback; it deliberately returns to common choice.
use crate::blueprint::BlueprintRng;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum OwnedIdleError { Identity, UnsupportedState, InvalidRng, BuildingActionRequired, CurrentCellActionRequired, QueryUnavailable }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct OwnedIdleContext {
    /// True only after the original preceding neighborhood eligibility branch.
    pub neighborhood_eligible:bool,
    /// Exact 5B100/virtual+8C result for the current postphysics cell.
    pub current_cell_requires_escape:bool,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct OwnedIdlePlan {
    pub before:[u8;124],pub after:[u8;124],
    pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,
    pub delay:i16,
    /// Original 14E remains false: continue at64CD7, not callback completion.
    pub continue_common_choice:bool,
}
/// Queries the owner's building list twice, selector masks4 then16, flags0x30.
/// A present result requires a separate source-backed action branch.
pub fn plan_idle_without_building_action<F>(before:&[u8;124],rng:BlueprintRng,context:OwnedIdleContext,mut nearest:F)->Result<OwnedIdlePlan,OwnedIdleError>
where F:FnMut(u8,u8,[i16;2])->Result<Option<u16>,OwnedIdleError> {
    if before[6]!=1||before[0x7a]!=4||before[0x18]>=4{return Err(OwnedIdleError::Identity);}
    if !context.neighborhood_eligible||before[0x5e]!=8||before[0x78]!=0||before[0x60]&0x42!=0||before[0x61]!=0{return Err(OwnedIdleError::UnsupportedState);}
    if rng.ring>=32{return Err(OwnedIdleError::InvalidRng);}
    let position=[i16::from_le_bytes([before[7],before[8]]),i16::from_le_bytes([before[9],before[10]])];
    let first=nearest(4,0x30,position)?;let second=nearest(16,0x30,position)?;
    if first.is_some()||second.is_some(){return Err(OwnedIdleError::BuildingActionRequired);}
    if context.current_cell_requires_escape{return Err(OwnedIdleError::CurrentCellActionRequired);}
    let mut after=*before;let mut next=rng;
    next.seed=next.seed.wrapping_mul(0xe62d);next.ring=(next.ring+1)&31;next.draw_count=next.draw_count.wrapping_add(1);
    let delay=i16::from((next.seed>>8)as u8%20)+10;
    after[0x5e]=8;after[0x6c..0x6e].copy_from_slice(&delay.to_le_bytes());
    Ok(OwnedIdlePlan{before:*before,after,rng_before:rng,rng_after:next,delay,continue_common_choice:true})
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct OwnedTargetPlan { pub before:[u8;124],pub after:[u8;124],pub target:[i16;2] }
///64CEF..64DEF: caller establishes DL==0 at the common-choice branch.
/// The predicate is the exact74CC0 query using record53 and this actor.
/// False requires original target search, which this bounded plan rejects.
pub fn plan_explicit_target<F>(before:&[u8;124],common_choice_eligible:bool,mut accept:F)->Result<OwnedTargetPlan,OwnedIdleError>
where F:FnMut([i16;2],u8)->Result<bool,OwnedIdleError> {
    if before[6]!=1||before[0x18]>=4||before[0x7a]!=4{return Err(OwnedIdleError::Identity);}
    if !common_choice_eligible||before[0x5e]!=8||before[0x78]!=7||before[0x61]&0x80==0||before[0x60]&2!=0||before[0x4f]==4{return Err(OwnedIdleError::UnsupportedState);}
    let target=[i16::from_le_bytes([before[0x6e],before[0x6f]]),i16::from_le_bytes([before[0x70],before[0x71]])];
    if !accept(target,before[0x53])?{return Err(OwnedIdleError::CurrentCellActionRequired);}
    let mut after=*before;after[11..13].copy_from_slice(&target[0].to_le_bytes());after[13..15].copy_from_slice(&target[1].to_le_bytes());
    after[0x54]=20;after[0x16]&=0x3f;
    let flags=(u16::from_le_bytes([after[0x49],after[0x4a]])|0x14)&0xeffc;after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());after[0x4f]=0;after[0x52]=0;
    let percent=75+25*i32::from(after[0x76].min(100))/100;
    let speed=((704*percent/100)*(i32::from(after[0x7b])+1)>>8)as i16;
    after[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());after[0x60]|=0x80;
    Ok(OwnedTargetPlan{before:*before,after,target})
}

//! Settled source57A10 state9 action13/14/15 animation tail after534D0.
use crate::actor_motion::{advance_actor_animation,ActorMotionError};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionAnimationError{UnsupportedActor,Animation(ActorMotionError)}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionAnimationPlan{pub after:[u8;200],pub animation:u8,pub queue_requested:bool}
pub fn plan_settled_construction_animation(before:&[u8;200],holds:&[u8],flags:u8)
    ->Result<ConstructionAnimationPlan,ConstructionAnimationError>{
    let control=u16::from_le_bytes(before[0x70..0x72].try_into().unwrap());
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=0||!(13..=15).contains(&before[0x59])
        ||control&0x50!=0||control&0x180==0{return Err(ConstructionAnimationError::UnsupportedActor);}
    if holds.is_empty()||holds.len()>255||flags&!1!=0{return Err(ConstructionAnimationError::Animation(ActorMotionError::InvalidAnimation));}
    let mut after=*before;
    if after[0x19]!=after[0x59]{after[0x19]=after[0x59];after[0x1c]=1;after[0x1d]=0;}
    after=advance_actor_animation(&after,holds,flags).map_err(ConstructionAnimationError::Animation)?;
    Ok(ConstructionAnimationPlan{after,animation:before[0x59],queue_requested:true})
}

//! 583D1..5876F post-stage animation/queue/footstep tail for an architect.
//! Resource metadata belongs to the selected animation returned by the selector.
use crate::actor_motion::{self,ActorMotionError};
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ArchitectTailPlan {
    pub after:[u8;200],
    pub selected_state:u8,
    pub queue_requested:bool,
    pub footstep_query:Option<[i16;2]>,
}
/// Select before looking up frame holds. This admits ordinary construction only.
pub fn architect_tail_animation(before:&[u8;200])->Result<u8,ActorMotionError>{
    if before[6]!=3{return Err(ActorMotionError::Inactive);}
    let cancelled=before[0x7e]==255&&before[0x82]==0&&before[0x58]==0&&before[0x57]&3==0;
    if before[0x55]!=9||(!cancelled&&before[0x7e]!=0)||before[0x70]&1==0{return Err(ActorMotionError::UnsupportedState);}
    match actor_motion::actor_selected_animation_state(before)? {
        1=>Ok(before[0x19]),
        8=>{
            let yaw=i16::from_le_bytes(before[0x39..0x3b].try_into().unwrap());
            if !(0..=2048).contains(&yaw){return Err(ActorMotionError::InvalidAngleTable);}
            Ok(((((i32::from(yaw)>>7)+1)>>1)+1)as u8&7)
        },
        9 if (13..=15).contains(&before[0x59])=>Ok(before[0x59]),
        _=>Err(ActorMotionError::UnsupportedState),
    }
}
pub fn plan_architect_callback_tail(before:&[u8;200],frame:u32,holds:&[u8],flags:u8)->Result<ArchitectTailPlan,ActorMotionError>{
    let animation=architect_tail_animation(before)?;
    let selected_state=actor_motion::actor_selected_animation_state(before)?;
    let mut after=*before;
    if selected_state==8 {
        if after[0x19]!=animation{after[0x19]=animation;after[0x1c]=1;}
        let speed=i16::from_le_bytes(after[0x41..0x43].try_into().unwrap());
        after[0x1d]=((1024-i32::from(speed))/256)as u8;
    }else if selected_state==9&&after[0x19]!=animation{
        after[0x19]=animation;after[0x1c]=1;after[0x1d]=0;
    }
    after=actor_motion::advance_actor_animation(&after,holds,flags)?;
    let footstep_query=actor_motion::actor_footstep_query_position(&after,frame)?;
    Ok(ArchitectTailPlan{after,selected_state,queue_requested:true,footstep_query})
}

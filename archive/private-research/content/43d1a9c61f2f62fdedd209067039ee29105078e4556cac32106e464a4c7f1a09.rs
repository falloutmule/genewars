//! Source57AAA cancelled architect transition, after the real6FB60 callback.
use crate::actor_motion::{self,ActorMotionError,ActorCallbackRng,ActorCallbackDraw,ActorIdleContext,ActorIdlePlan};
pub fn plan_cancelled_architect_idle(before:&[u8;200],blueprint:&[u8;36],frame:u32,mut rng:ActorCallbackRng,context:ActorIdleContext,empty_scorer:bool)
    ->Result<ActorIdlePlan,ActorMotionError>{
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=255||before[0x82]!=0||before[0x58]!=0
        ||before[0x57]&3!=0||before[0x70]&1==0||before[0x70]&0x70!=0||before[0x71]&0x10!=0
        ||blueprint[0x16]!=0||blueprint[0x15]>=3{return Err(ActorMotionError::UnsupportedState);}
    if rng.ring>=32{return Err(ActorMotionError::InvalidRng);}
    let mut after=*before;after[0x55]=1;after[0x68..0x6c].fill(0);after[0x19]=8;after[0x1c]=1;after[0x1d]=16;after[0x1b]=0;
    let prior=rng;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
    if (rng.seed>>8)%4<2{after[0x70]|=4}else{after[0x70]&=!4}
    let draw=ActorCallbackDraw{seed_before:prior.seed,seed_after:rng.seed,ring_slot:prior.ring,journal_code:0x21b};
    // The shared helper applies the same blueprint periodic update; its fields
    // do not affect the preceding21B draw or the category0 transition.
    let mut plan=actor_motion::plan_actor_idle_with_empty_scorer(&after,blueprint,frame,rng,context,empty_scorer,false)?;
    plan.draws.insert(0,draw);Ok(plan)
}

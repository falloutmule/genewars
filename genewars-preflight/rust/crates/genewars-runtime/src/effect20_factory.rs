//! Null-associated effect20 numeric factory, source48790. Full-pool priority
//! replacement, map insertion, resource binding and lifecycle remain caller work.
use crate::landing::{LandingRecord,LandingFactoryPlan,LandingError,ResolvedCellHead};
pub fn plan_factory(pool:&[LandingRecord;200],count:i16,owner:u8,
 at:[i16;2],ground:i32,shade:u8,height:i32,angles:[u16;3],head:Option<ResolvedCellHead>,
 sine:&[i32;2048],cosine:&[i32;2048])->Result<LandingFactoryPlan,LandingError>{
 // The common1E600 +48790 velocity/link initialization is identical to15.
 // Exact104EB4 selector20 descriptor is[0,1,42,255,0,0]: physicalflags0.
 let mut plan=crate::water_effect::plan_factory(0x15,pool,count,owner,at,ground,shade,height,angles,head,sine,cosine)?;
 plan.after_nonpointer[21..23].fill(0);plan.after_nonpointer[69]=0x20;
 // Source48790 selector20 null-associated branch clears kind only;
 // the prior reference index47/48 is deliberately retained.
 plan.after_nonpointer[73]=0;plan.animation_index=32;Ok(plan)
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Effect20AnimationPlan{pub after:LandingRecord,pub retirement_required:bool}
/// Shared4B630 lifetime/7AC00 portion AFTER actual6DF30, not a physics substitute.
/// Exact EFF32 has10 frames, zero holds, delay0; class0 retires on frame9.
pub fn plan_animation_after_physics(before:&LandingRecord)->Result<Effect20AnimationPlan,LandingError>{
 if before[6]!=5||before[69]!=0x20||before[73]!=0{return Err(LandingError::InvalidRecord);}
 if before[28]!=1||before[29]!=0{return Err(LandingError::UnsupportedAnimationState);}
 let mut after=*before;let timer=i16::from_le_bytes([before[74],before[75]]);
 if timer>=0{after[74..76].copy_from_slice(&timer.wrapping_sub(1).to_le_bytes());}
 after[26]=0;after[30]=0;after[27]=(after[27].min(9)+1)%10;
 let retirement_required=i16::from_le_bytes([after[74],after[75]])<0&&after[27]==9;
 Ok(Effect20AnimationPlan{after,retirement_required})
}

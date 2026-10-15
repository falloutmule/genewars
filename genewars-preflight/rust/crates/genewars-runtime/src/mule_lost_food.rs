//!66080..65C5B null-food reset. Caller proves odd(frame+signedID),
//! saved-current-cell66112 predicate FALSE, and handles5C920 selection UI.
//! Domain excludes60bit2, so5C920 returns without selection mutation.
//! Result is BEFORE ordinary tail decrements timer; no arrival prerequisite.
use crate::blueprint::BlueprintRng;
#[derive(Debug,Clone,PartialEq,Eq)]pub enum LostFoodError{Record,Rng}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct LostFoodPlan{pub record_after:[u8;124],pub rng_after:BlueprintRng}
pub fn plan_lost_food(before:&[u8;124],mut rng:BlueprintRng)->Result<LostFoodPlan,LostFoodError>{
 if before[6]!=1||before[0x5e]!=1||before[0x57]!=0||before[0x60]&0x42!=0x40{return Err(LostFoodError::Record);}
 if rng.ring>=32{return Err(LostFoodError::Rng);}
 let mut a=*before;a[0x15]&=0xdf;a[0x33..0x37].fill(0);
 let flags=u16::from_le_bytes([a[0x60],a[0x61]])&0xd71a;a[0x60..0x62].copy_from_slice(&flags.to_le_bytes());a[0x5d]=0;a[0x57]=0;
 rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
 a[0x5e]=8;let delay=(rng.seed>>8)%20+10;a[0x6c..0x6e].copy_from_slice(&delay.to_le_bytes());
 Ok(LostFoodPlan{record_after:a,rng_after:rng})
}

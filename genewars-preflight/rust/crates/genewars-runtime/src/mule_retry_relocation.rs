//!5E271..5E3B1 ground-Mule retry relocation after the reset/delay draw.
//! Caller proves original5B100 true and supplies the reset-wait afterimage/RNG.
//! Two73C20 draws only; no acceptance query or ordinary AI continuation here.
use crate::{actor_activation::TrigTables,blueprint::BlueprintRng};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum RetryRelocationError{Record,Rng}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct RetryRelocationPlan{pub record_after:[u8;124],pub rng_after:BlueprintRng}
pub fn plan_relocation(before:&[u8;124],mut rng:BlueprintRng,trig:&TrigTables)->Result<RetryRelocationPlan,RetryRelocationError>{
 if before[6]!=1||before[24]!=4||before[0x7a]!=4||before[0x5e]!=8||before[0x57]!=0||before[0x5d]!=0||before[0x60]&2!=0{return Err(RetryRelocationError::Record);}
 if rng.ring>=32{return Err(RetryRelocationError::Rng);}
 let word=|at|i16::from_le_bytes([before[at],before[at+1]]);
 let sample=crate::construction_work::sample_construction_work_point([word(7),word(9)],rng.seed,trig.cosine().try_into().unwrap(),trig.sine().try_into().unwrap());
 rng.seed=sample.seed_after;rng.ring=(rng.ring+2)&31;rng.draw_count=rng.draw_count.wrapping_add(2);
 let mut a=*before;a[0x16]&=0x3f;a[0x54]=20;let flags=(u16::from_le_bytes([a[0x49],a[0x4a]])|0x14)&0xeffc;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
 for axis in 0..2{a[11+axis*2..13+axis*2].copy_from_slice(&sample.point[axis].to_le_bytes());}
 a[0x4f]=0;a[0x52]=0;let speed=crate::creature_world::mule_requested_speed(&a);a[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());a[0x6c..0x6e].copy_from_slice(&0x7fffu16.to_le_bytes());a[0x60]|=0x80;
 Ok(RetryRelocationPlan{record_after:a,rng_after:rng})
}

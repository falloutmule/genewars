//! Exact bounded owned attack range-miss6152C..615C3/60291.
//! Source player-creation/instructions-0005ED20.txt; actual5892 squared
//! physicaldistance16481 >16384. Caller proves resolved live neutral target
//! outside603B6 range AND source virtual8C/5B100 current-cell returnsfalse.
//! True escape continuation615C9 is unsupported. No5CC60 reset is executed:
//! refs/flags/task/reserve/requested/current speed remain byte-identical.
//! Private literal source proof mule-owned-attack-miss-proof/
//!442d242912ba44d091c14f2a9c25c386, actualentry plus explicitlysynthetic seeds.
use crate::{blueprint::BlueprintRng,creature_world::CreatureRecord};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Error{Identity,State,References,InvalidRng,UnsupportedEscape}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct Draw{pub journal:u16,pub seed_before:u16,pub seed_after:u16,pub value:u8}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Plan{pub after:CreatureRecord,pub rng_after:BlueprintRng,pub delay:i16,pub draw:Draw}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
/// Current is postphysics/post-animation-completion, state21/timer0.
/// Range/current-cell predicates belong to world; this planner does not infer
/// a miss from yaw or replace5B100. Delay precedes ordinary shared-tail decrement.
pub fn plan(current:&CreatureRecord,rng_before:BlueprintRng,current_cell_requires_escape:bool)->Result<Plan,Error>{
 let id=word(current,15);
 if current[6]!=1||current[24]!=0||current[122]!=4||!(0..300).contains(&id){return Err(Error::Identity);}
 if current[94]!=21||!matches!(current[96],8|0x88)||current[97]!=0||current[120]!=7||current[118]>90||current[119]!=255||word(current,108)!=0||word(current,106)<=0||word(current,104)<0||word(current,73)as u16&0x400!=0{return Err(Error::State);}
 let target=word(current,85);
 if !(0..300).contains(&target)||target==id||current[0x4f]!=0||current[0x5d]!=0||current[82]!=1||current[87]!=1||current[90]!=1||current[80..83]!=current[85..88]||current[88..91]!=current[85..88]{return Err(Error::References);}
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 if current_cell_requires_escape{return Err(Error::UnsupportedEscape);}
 let mut rng=rng_before;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);let value=(rng.seed>>8)as u8;let delay=10+i16::from(value%20);
 let mut after=*current;after[94]=8;after[108..110].copy_from_slice(&delay.to_le_bytes());
 Ok(Plan{after,rng_after:rng,delay,draw:Draw{journal:0xbbd,seed_before:rng_before.seed,seed_after:rng.seed,value}})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]
 #[ignore="requires GENEWARS_OWNED_ATTACK_MISS_CASES literal source fixtures"]
 fn literal_range_miss_preserves_all_except_state_and_delay(){
  let data=std::fs::read(std::env::var_os("GENEWARS_OWNED_ATTACK_MISS_CASES").expect("set cases.bin path")).unwrap();assert_eq!(data.len(),5*262);
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for(index,c)in data.chunks_exact(262).enumerate(){let current:CreatureRecord=c[..124].try_into().unwrap();let r=rng(&c[124..131]);let p=plan(&current,r,false).unwrap();assert_eq!(p.after.as_slice(),&c[131..255],"case{index} complete record");assert_eq!(p.rng_after,rng(&c[255..262]));assert_eq!(p.draw.journal,0xbbd);
   for i in 0..124{if !matches!(i,94|108|109){assert_eq!(p.after[i],current[i],"opaque{i:x}");}}
   assert_eq!(plan(&current,r,true),Err(Error::UnsupportedEscape));let mut broken=current;broken[87]=0;let saved=broken;assert_eq!(plan(&broken,r,false),Err(Error::References));assert_eq!(broken,saved);
  }
 }
}

//! Bounded authentic5882 neutral state20 completed-hit reacquisition.
//! Source zero20 dispatch5F4C4/query5F5A9..5FBE7/5FF29->5B580.
//! World must prove current-cell8B7D0false at5F97F, sole live owned enemy,
//! no friends or candidate-specific draws (owner4 ignores alliances), known typed Mule bindings.
//! One enemy means health256-((weight/512)<<5)+trait4; no enemy-health term.
//! Fourdraw successfulCD6 branch only; all flee/operator alternatives reject.
//! Stops after5B580 state1 writes, before caller continuation/animation/tail.
//! Pinned species4 params1046B0: radius512/trait4=10/trait24=90;
//! object3 SHA256aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87.
use crate::{blueprint::BlueprintRng,creature_world::{CreatureRecord,mule_requested_speed}};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Error { Identity, Current, Enemy, QueryDomain, InvalidRng, UnsupportedDisposition }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum DrawPurpose { NeighborhoodX, NeighborhoodY, DispositionGate, ConfidenceGate }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Draw { pub purpose:DrawPurpose,pub journal:u16,pub seed_before:u16,pub seed_after:u16,pub value:u8 }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Plan {
 pub after:CreatureRecord,pub enemy_slot:u16,pub query_weight:i32,pub confidence:i32,
 pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,pub draws:[Draw;4],
}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn put(r:&mut CreatureRecord,at:usize,v:i16){r[at..at+2].copy_from_slice(&v.to_le_bytes());}
fn cell_delta(a:i16,b:i16)->i32{
 let a=i32::from((a as u16&0x7fff)>>8);let b=i32::from((b as u16&0x7fff)>>8);
 (b-a+64).rem_euclid(128)-64
}
/// Animation-completed current is state20/task0/timer0, before the two query draws.
/// Sole-hostile/no-friend world admission is external, not inferred from these
/// two records. Query visits the enemy's cell once; orientation does not alter
/// its weight. The CD6 second roll must stay below confidence; all flee/operator paths reject.
///5B5CC preserves reference58 because task0 is zero; goalXY is not copied.
/// Returned timer32767 precedes the shared callback tail. No target mutation,
/// animation, actor actions, physics, world linking or gameplay credit occurs.
pub fn plan_reacquire(current:&CreatureRecord,enemy:&CreatureRecord,rng_before:BlueprintRng)
 ->Result<Plan,Error>{
 let id=word(current,15);let target=word(enemy,15);
 if !(0..300).contains(&id)||!(0..300).contains(&target)||id==target{return Err(Error::Identity);}
 if current[6]!=1||current[24]!=4||current[122]!=4||current[94]!=20
  ||current[0x60]!=8||current[0x61]!=0x40||current[0x78]!=0
  ||current[0x76]>90||current[0x77]!=255||current[0x7b]!=255
  ||current[0x15]!=0x80||!matches!(current[0x16],2|0x82)
  ||current[0x4f]!=0||current[0x52]!=1||current[0x57]!=1||current[0x5a]!=0||current[0x5d]!=0
  ||[0x50,0x55].iter().any(|&at|word(current,at)!=target)
  ||word(current,0x6c)!=0||word(current,0x6a)<=0||word(current,0x64)<=0
  ||word(current,0x6a)>word(current,0x64)||word(current,0x66)<=2
  ||word(current,0x68)<0||word(current,0x62)<=0
  ||word(current,0x68)>(word(current,0x62)>>1)
  ||word(current,0x49)as u16&0x100!=0{return Err(Error::Current);}
 if enemy[6]!=1||enemy[24]!=0||enemy[122]!=4||enemy[94]!=21
  ||enemy[0x60]!=0x88||enemy[0x61]!=0||enemy[0x78]!=7
  ||enemy[0x4f]!=0||enemy[0x5d]!=0||enemy[0x52]!=1||enemy[0x57]!=1
  ||word(enemy,0x50)!=id||word(enemy,0x55)!=id||enemy[0x5a]!=1||word(enemy,0x58)!=id
  ||word(enemy,0x49)as u16&0x400!=0||word(enemy,0x6a)<=0
 {return Err(Error::Enemy);}
 let dx=cell_delta(word(current,7),word(enemy,7));let dy=cell_delta(word(current,9),word(enemy,9));
 let distance=(dx*dx+dy*dy)*256*256;
 if distance>=1024*1024{return Err(Error::QueryDomain);}
 let weight=1024-genewars_core::cd_candidate::integer_sqrt(distance as u32)as i32;
 //5FA24..5FA99: health256 - ((bestEnemyWeight/count/radius)<<5)+trait4.
 //5FA9C skips friend contribution for externally-proved count0. Enemy health
 //does not enter kind1 query6B838/6B9CF..6BA47 weight; only virtual68 alive does.
 // Signed IDIV ordering is deliberately retained before each left shift.
 let confidence=i32::from(word(current,0x6a))*256/i32::from(word(current,0x64))
  -((weight/512)<<5)+10;
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut rng=rng_before;
 let draws=[DrawPurpose::NeighborhoodX,DrawPurpose::NeighborhoodY,DrawPurpose::DispositionGate,DrawPurpose::ConfidenceGate].map(|purpose|{
  let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
  Draw{purpose,journal:match purpose{DrawPurpose::NeighborhoodX=>0x6bd,DrawPurpose::NeighborhoodY=>0x6be,_=>0xcd6},seed_before:before,seed_after:rng.seed,value:(rng.seed>>8)as u8}
 });
 //5FB5C:CD6%100<=trait24;5FBE7:CD6byte<confidence chooses5FF1F.
 if draws[2].value%100>90||i32::from(draws[3].value)>=confidence{return Err(Error::UnsupportedDisposition);}
 let mut after=*current;
 //5B580 source local reset / kind1 continuation.
 after[0x15]&=0xdf;after[0x33..0x37].fill(0);put(&mut after,0x60,(word(current,0x60)as u16&0xd71a)as i16);
 after[0x5d]=0;after[0x57]=0;put(&mut after,0x6c,0);
 put(&mut after,0x55,target);after[0x57]=1;
 //5B5CC task0 skips retained human-target58; all three opaque bytes survive.
 //5B580 source local reset / kind1 continuation.
 after[0x16]&=0x3f;put(&mut after,0x49,((word(current,0x49)as u16|0x14)&0xeffc)as i16);
 after[0x54]=20;after[0x4f]=0;put(&mut after,0x50,target);after[0x52]=1;
 let speed=mule_requested_speed(&after);put(&mut after,0x3f,speed);put(&mut after,0x6c,32767);
 after[0x60]|=0xc0;after[94]=1;
 Ok(Plan{after,enemy_slot:target as u16,query_weight:weight,confidence,rng_before,rng_after:rng,draws})
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]
 #[ignore="requires GENEWARS_NEUTRAL_HIT_REACQUIRE_CASES literal source cases"]
 fn literal_actual_reacquisition_record_rng_and_zero_reserve(){
  let data=std::fs::read(std::env::var_os("GENEWARS_NEUTRAL_HIT_REACQUIRE_CASES").expect("set cases.bin path")).unwrap();assert_eq!(data.len(),4*386);
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for(index,c)in data.chunks_exact(386).enumerate(){let current:CreatureRecord=c[..124].try_into().unwrap();let enemy:CreatureRecord=c[124..248].try_into().unwrap();let r=rng(&c[248..255]);let saved=(current,enemy);let p=plan_reacquire(&current,&enemy,r).unwrap();assert_eq!(word(&current,104),0);assert_eq!(p.after.as_slice(),&c[255..379],"case{index} fullrecord");assert_eq!(p.rng_after,rng(&c[379..386]));assert_eq!(p.confidence,194);assert_eq!(p.query_weight,1024);assert_eq!(p.draws.map(|d|d.journal),[0x6bd,0x6be,0xcd6,0xcd6]);assert_eq!((current,enemy),saved);assert_eq!(&p.after[88..91],&current[88..91]);
   for(at,value)in[(94,21),(120,7),(0x57,0),(0x61,0)]{let mut bad=current;bad[at]=value;let before=bad;assert!(plan_reacquire(&bad,&enemy,r).is_err());assert_eq!(bad,before);}
   let mut dead=enemy;dead[94]=24;assert_eq!(plan_reacquire(&current,&dead,r),Err(Error::Enemy));
   assert_eq!(plan_reacquire(&current,&enemy,BlueprintRng{seed:18153,..r}),Err(Error::UnsupportedDisposition));
  }
 }
}

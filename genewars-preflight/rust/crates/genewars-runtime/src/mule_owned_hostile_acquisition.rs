//! Bounded owned-task7 hostile acquisition at62949..62E1F/5B580.
//! The world must prove one live neutral Mule query winner, no friendly count,
//! no candidate-specific draws, exact MuleSelector4 bindings and no alliance.
//! Source8D930 samples circle radius1024 in256 steps;6B7BE subtracts sample
//! distance.62935..62987 computes health256 - ((enemyweight/512)<<5)+10.
//! Pinned species4 parameters1046B0: radius512, confidencebase10, gate90 at+24h.
//! Mapped object3 SHA256 aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87.
//! Only the four-draw branch gate<=90 then confidenceRoll<confidence is covered.
//! This is not a flee planner or a combat/actor-response implementation.
use crate::{blueprint::BlueprintRng,creature_world::{CreatureRecord,mule_requested_speed}};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum OwnedHostileError { Identity, Current, Enemy, QueryDomain, InvalidRng, UnsupportedDisposition }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum DrawPurpose { NeighborhoodX, NeighborhoodY, DispositionGate, ConfidenceGate }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct OwnedHostileDraw { pub purpose:DrawPurpose,pub seed_before:u16,pub seed_after:u16,pub value:u8 }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct OwnedHostilePlan {
 pub after:CreatureRecord,pub enemy_slot:u16,pub query_weight:i32,pub confidence:i32,
 pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,pub draws:[OwnedHostileDraw;4],
}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn put(r:&mut CreatureRecord,at:usize,v:i16){r[at..at+2].copy_from_slice(&v.to_le_bytes());}
fn cell_delta(a:i16,b:i16)->i32{
 let a=i32::from((a as u16&0x7fff)>>8);let b=i32::from((b as u16&0x7fff)>>8);
 (b-a+64).rem_euclid(128)-64
}
/// postphysics current is state8/task7/timer0, before the two query draws.
/// Sole-hostile/no-friend world admission is external, not inferred from these
/// two records. Query visits the enemy's cell once; orientation does not alter
/// its weight. The source's3-draw alternative and all flee paths reject.
///5B5CC writes reference58 because task7 is nonzero; goalXY is not copied.
/// Returned timer32767 precedes the shared callback tail. No target mutation,
/// animation, actor actions, physics, world linking or gameplay credit occurs.
pub fn plan_acquisition(current:&CreatureRecord,enemy:&CreatureRecord,rng_before:BlueprintRng)
 ->Result<OwnedHostilePlan,OwnedHostileError>{
 use OwnedHostileError as Error;
 let id=word(current,15);let target=word(enemy,15);
 if !(0..300).contains(&id)||!(0..300).contains(&target)||id==target{return Err(Error::Identity);}
 if current[6]!=1||current[24]!=0||current[122]!=4||current[94]!=8
  ||current[0x60]!=8||current[0x61]!=0||current[0x78]!=7
  ||current[0x76]>90||current[0x77]!=255||current[0x7b]!=255
  ||current[0x15]!=0x80||!matches!(current[0x16],2|0x82)
  ||current[0x4f]!=0||current[0x52]!=0||current[0x57]!=0||current[0x5a]!=0||current[0x5d]!=0
  ||word(current,0x6c)!=0||word(current,0x6a)<=0||word(current,0x64)<=0
  ||word(current,0x6a)>word(current,0x64)||word(current,0x66)<=2
  ||word(current,0x68)<=0||word(current,0x62)<=0
  ||word(current,0x68)>(word(current,0x62)>>1)
  ||word(current,0x49)as u16&0x100!=0{return Err(Error::Current);}
 if enemy[6]!=1||enemy[24]!=4||enemy[122]!=4||enemy[94]!=1
  ||enemy[0x60]!=0xc8||enemy[0x61]!=0x40||enemy[0x78]!=0
  ||enemy[0x4f]!=0||enemy[0x5d]!=0||enemy[0x52]!=1||enemy[0x57]!=1
  ||word(enemy,0x50)!=id||word(enemy,0x55)!=id
  ||word(enemy,0x49)as u16&0x400!=0||word(enemy,0x6a)<=0
 {return Err(Error::Enemy);}
 let dx=cell_delta(word(current,7),word(enemy,7));let dy=cell_delta(word(current,9),word(enemy,9));
 let distance=(dx*dx+dy*dy)*256*256;
 if distance>1024*1024{return Err(Error::QueryDomain);}
 let weight=1024-genewars_core::cd_candidate::integer_sqrt(distance as u32)as i32;
 // One enemy in the maximum-weight direction, count1, friendlycount0.
 // Signed IDIV ordering is deliberately retained before each left shift.
 let confidence=i32::from(word(current,0x6a))*256/i32::from(word(current,0x64))
  -((weight/512)<<5)+10;
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut rng=rng_before;
 let draws=[DrawPurpose::NeighborhoodX,DrawPurpose::NeighborhoodY,DrawPurpose::DispositionGate,DrawPurpose::ConfidenceGate].map(|purpose|{
  let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
  OwnedHostileDraw{purpose,seed_before:before,seed_after:rng.seed,value:(rng.seed>>8)as u8}
 });
 if draws[2].value%100>90||i32::from(draws[3].value)>=confidence{return Err(Error::UnsupportedDisposition);}
 let mut after=*current;
 //5B58A→5CC60 local reset, preserving opaque null indices and intrusive links.
 after[0x15]&=0xdf;after[0x33..0x37].fill(0);put(&mut after,0x60,(word(current,0x60)as u16&0xd71a)as i16);
 after[0x5d]=0;after[0x57]=0;put(&mut after,0x6c,0);
 put(&mut after,0x55,target);after[0x57]=1;
 //5B5CC..5B5E8 retains the human-task target separately.
 put(&mut after,0x58,target);after[0x5a]=1;
 //Kind1 jump table5B56C→5B6C3; virtual10(enemy) is false here.
 after[0x16]&=0x3f;put(&mut after,0x49,((word(current,0x49)as u16|0x14)&0xeffc)as i16);
 after[0x54]=20;after[0x4f]=0;put(&mut after,0x50,target);after[0x52]=1;
 let speed=mule_requested_speed(&after);put(&mut after,0x3f,speed);put(&mut after,0x6c,32767);
 after[0x60]|=0xc0;after[94]=1;
 Ok(OwnedHostilePlan{after,enemy_slot:target as u16,query_weight:weight,confidence,rng_before,rng_after:rng,draws})
}
#[cfg(test)]mod tests{
 use super::*;
 fn fixture()->(CreatureRecord,CreatureRecord,BlueprintRng){
  let mut a=[0xa5;124];let mut b=[0x5a;124];
  for(r,id,owner,state)in[(&mut a,1,0,8),(&mut b,5,4,1)]{r[6]=1;r[24]=owner;r[122]=4;r[94]=state;put(r,15,id);put(r,0x49,0);put(r,0x6a,59);r[0x4f]=0;r[0x5d]=0;}
  a[0x60]=8;a[0x61]=0;a[0x78]=7;a[0x76]=37;a[0x77]=255;a[0x7b]=255;a[0x15]=0x80;a[0x16]=2;
  for at in[0x52,0x57,0x5a]{a[at]=0;}put(&mut a,0x6c,0);put(&mut a,0x66,9000);put(&mut a,0x62,130);put(&mut a,0x64,59);put(&mut a,0x68,15);
  put(&mut a,7,658);put(&mut a,9,24211);put(&mut b,7,805);put(&mut b,9,23607);
  b[0x60]=0xc8;b[0x61]=0x40;b[0x78]=0;put(&mut b,0x50,1);b[0x52]=1;put(&mut b,0x55,1);b[0x57]=1;
  (a,b,BlueprintRng{seed:28533,ring:21,draw_count:49310})
 }
 #[test]fn actual_disposition_selects_live_enemy_and_retains_task_reference(){
  let(a,b,rng)=fixture();let p=plan_acquisition(&a,&b,rng).unwrap();assert_eq!(p.query_weight,452);assert_eq!(p.confidence,266);assert_eq!(p.draws.map(|d|d.value),[181,48,211,248]);
  assert_eq!(&p.after[0x50..0x53],&[5,0,1]);assert_eq!(&p.after[0x55..0x58],&[5,0,1]);assert_eq!(&p.after[0x58..0x5b],&[5,0,1]);assert_eq!(p.after[0x78],7);
  assert_eq!(&p.after[11..15],&a[11..15]);assert_eq!(&p.after[..6],&a[..6]);assert_eq!(&p.after[31..39],&a[31..39]);assert_eq!(word(&p.after,0x3f),591);assert_eq!(p.after[94],1);assert_eq!(p.after[0x60],0xc8);assert_eq!(p.after[0x61],0);
 }
 #[test]fn confidence_and_other_source_domains_reject_and_inputs_stay_opaque(){
  let(a,b,mut rng)=fixture();let saved=(a,b);
  let mut low=a;put(&mut low,0x6a,1);assert_eq!(plan_acquisition(&low,&b,rng),Err(OwnedHostileError::UnsupportedDisposition));
  let mut close=b;put(&mut close,7,658);put(&mut close,9,24211);assert_eq!(plan_acquisition(&a,&close,rng),Err(OwnedHostileError::UnsupportedDisposition));
  let mut dead=b;dead[94]=24;assert_eq!(plan_acquisition(&a,&dead,rng),Err(OwnedHostileError::Enemy));
  let mut idle=a;idle[0x78]=0;assert_eq!(plan_acquisition(&idle,&b,rng),Err(OwnedHostileError::Current));
  rng.ring=31;rng.draw_count=u32::MAX-2;let p=plan_acquisition(&a,&b,rng).unwrap();assert_eq!(p.rng_after.ring,3);assert_eq!(p.rng_after.draw_count,1);assert_eq!((a,b),saved);
 }
 #[test]fn signed15_seam_preserves_single_sample_confidence(){
  let(mut a,mut b,rng)=fixture();put(&mut a,7,32700);put(&mut b,7,100);let p=plan_acquisition(&a,&b,rng).unwrap();assert_eq!(p.query_weight,452);assert_eq!(p.confidence,266);
 }
}

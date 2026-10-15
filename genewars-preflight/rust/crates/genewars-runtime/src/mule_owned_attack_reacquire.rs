//! Exact bounded post-attack owned reacquisition6096A..612A2/5B580.
//! Caller proves successful60332 range/damage and BA4<=trait24 branch,
//! safe current cell8B7D0=false, exactly one enemy/no friends, alliance-clear,
//! no candidate-specific draws and query winner equal supplied neutral Mule20.
//! No state21→8 normalization: original current21 persists through query.
//! Source player-creation/instructions-0005ed20.txt60D91..60F53 and terrain-map-
//! refs/instructions-0005b580.txt; literal actual5876 subbranch + synthetic seeds:
//! private/review/mule-owned-attack-reacquire-proof/d6f46c3e3893429caa2683657f941f26.
//! Mule1046B0 radius512/trait4=10/trait24=90; pinned object3 SHA256
//! aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87.
//! Stops before virtual38/5C840; no damage/attack animation/audio occurs here.
use crate::{blueprint::BlueprintRng,creature_world::{CreatureRecord,mule_requested_speed}};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ReacquireError { Identity, Current, Enemy, QueryDomain, InvalidRng, UnsupportedDisposition }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum DrawPurpose { NeighborhoodX, NeighborhoodY, DispositionGate, ConfidenceGate }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ReacquireDraw { pub purpose:DrawPurpose,pub journal:u16,pub seed_before:u16,pub seed_after:u16,pub value:u8 }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ReacquirePlan {
 pub after:CreatureRecord,pub enemy_slot:u16,pub query_weight:i32,pub confidence:i32,
 pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,pub draws:[ReacquireDraw;4],
}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn put(r:&mut CreatureRecord,at:usize,v:i16){r[at..at+2].copy_from_slice(&v.to_le_bytes());}
fn cell_delta(a:i16,b:i16)->i32{
 let a=i32::from((a as u16&0x7fff)>>8);let b=i32::from((b as u16&0x7fff)>>8);
 (b-a+64).rem_euclid(128)-64
}
/// PostBA4 current is state21/task7/timer0, before the two query draws.
/// Sole-hostile/no-friend world admission is external, not inferred from these
/// two records. Query visits the enemy's cell once; orientation does not alter
/// its weight. The CD6 second roll must stay below confidence; all flee/operator paths reject.
///5B5CC writes reference58 because task7 is nonzero; goalXY is not copied.
/// Returned timer32767 precedes the shared callback tail. No target mutation,
/// animation, actor actions, physics, world linking or gameplay credit occurs.
pub fn plan_reacquire(current:&CreatureRecord,enemy:&CreatureRecord,rng_before:BlueprintRng)
 ->Result<ReacquirePlan,ReacquireError>{
 use ReacquireError as Error;
 let id=word(current,15);let target=word(enemy,15);
 if !(0..300).contains(&id)||!(0..300).contains(&target)||id==target{return Err(Error::Identity);}
 if current[6]!=1||current[24]!=0||current[122]!=4||current[94]!=21
  ||current[0x60]!=0x88||current[0x61]!=0||current[0x78]!=7
  ||current[0x76]>90||current[0x77]!=255||current[0x7b]!=255
  ||current[0x15]!=0x80||!matches!(current[0x16],2|0x82)
  ||current[0x4f]!=0||current[0x52]!=1||current[0x57]!=1||current[0x5a]!=1||current[0x5d]!=0
  ||[0x50,0x55,0x58].iter().any(|&at|word(current,at)!=target)
  ||word(current,0x6c)!=0||word(current,0x6a)<=0||word(current,0x64)<=0
  ||word(current,0x6a)>word(current,0x64)||word(current,0x66)<=2
  ||word(current,0x68)<=0||word(current,0x62)<=0
  ||word(current,0x68)>(word(current,0x62)>>1)
  ||word(current,0x49)as u16&0x100!=0{return Err(Error::Current);}
 if enemy[6]!=1||enemy[24]!=4||enemy[122]!=4||enemy[94]!=20
  ||enemy[0x60]!=8||enemy[0x61]!=0x40||enemy[0x78]!=0
  ||enemy[0x4f]!=0||enemy[0x5d]!=0||enemy[0x52]!=1||enemy[0x57]!=1
  ||word(enemy,0x50)!=id||word(enemy,0x55)!=id
  ||word(enemy,0x49)as u16&0x400!=0||word(enemy,0x6a)<=0
 {return Err(Error::Enemy);}
 let dx=cell_delta(word(current,7),word(enemy,7));let dy=cell_delta(word(current,9),word(enemy,9));
 let distance=(dx*dx+dy*dy)*256*256;
 if distance>=1024*1024{return Err(Error::QueryDomain);}
 let weight=1024-genewars_core::cd_candidate::integer_sqrt(distance as u32)as i32;
 //60D91..60E04: health256 - ((bestEnemyWeight/count/radius)<<5)+trait4.
 //60E07 skips friend contribution for externally-proved count0. Enemy health
 //does not enter kind1 query6B838/6B9CF..6BA47 weight; only virtual68 alive does.
 // Signed IDIV ordering is deliberately retained before each left shift.
 let confidence=i32::from(word(current,0x6a))*256/i32::from(word(current,0x64))
  -((weight/512)<<5)+10;
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut rng=rng_before;
 let draws=[DrawPurpose::NeighborhoodX,DrawPurpose::NeighborhoodY,DrawPurpose::DispositionGate,DrawPurpose::ConfidenceGate].map(|purpose|{
  let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
  ReacquireDraw{purpose,journal:match purpose{DrawPurpose::NeighborhoodX=>0x6bd,DrawPurpose::NeighborhoodY=>0x6be,_=>0xcd6},seed_before:before,seed_after:rng.seed,value:(rng.seed>>8)as u8}
 });
 //60ECA:CD6%100<=trait24;60F53:CD6byte<confidence chooses61294.
 if draws[2].value%100>90||i32::from(draws[3].value)>=confidence{return Err(Error::UnsupportedDisposition);}
 let mut after=*current;
 //5B58Aâ†’5CC60 local reset, preserving opaque null indices and intrusive links.
 after[0x15]&=0xdf;after[0x33..0x37].fill(0);put(&mut after,0x60,(word(current,0x60)as u16&0xd71a)as i16);
 after[0x5d]=0;after[0x57]=0;put(&mut after,0x6c,0);
 put(&mut after,0x55,target);after[0x57]=1;
 //5B5CC..5B5E8 retains the human-task target separately.
 put(&mut after,0x58,target);after[0x5a]=1;
 //Kind1 jump table5B56Câ†’5B6C3; virtual10(enemy) is false here.
 after[0x16]&=0x3f;put(&mut after,0x49,((word(current,0x49)as u16|0x14)&0xeffc)as i16);
 after[0x54]=20;after[0x4f]=0;put(&mut after,0x50,target);after[0x52]=1;
 let speed=mule_requested_speed(&after);put(&mut after,0x3f,speed);put(&mut after,0x6c,32767);
 after[0x60]|=0xc0;after[94]=1;
 Ok(ReacquirePlan{after,enemy_slot:target as u16,query_weight:weight,confidence,rng_before,rng_after:rng,draws})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]
 #[ignore="requires GENEWARS_OWNED_REACQUIRE_CASES literal source fixture"]
 fn literal_reacquisition_matches_full_record_and_rng(){
  let data=std::fs::read(std::env::var_os("GENEWARS_OWNED_REACQUIRE_CASES").expect("set cases.bin path")).unwrap();assert_eq!(data.len(),4*386);
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for(index,c)in data.chunks_exact(386).enumerate(){let current:CreatureRecord=c[..124].try_into().unwrap();let enemy:CreatureRecord=c[124..248].try_into().unwrap();let before=(current,enemy);
   let p=plan_reacquire(&current,&enemy,rng(&c[248..255])).unwrap();assert_eq!(p.after.as_slice(),&c[255..379],"record case{index}");assert_eq!(p.rng_after,rng(&c[379..386]));assert_eq!(p.query_weight,1024);assert_eq!(p.confidence,202);assert_eq!(p.draws.map(|d|d.journal),[0x6bd,0x6be,0xcd6,0xcd6]);assert_eq!((current,enemy),before);
   let mut initial=current;initial[94]=8;assert_eq!(plan_reacquire(&initial,&enemy,p.rng_before),Err(ReacquireError::Current));
   let mut dead=enemy;dead[94]=24;assert_eq!(plan_reacquire(&current,&dead,p.rng_before),Err(ReacquireError::Enemy));
   let mut broken=current;broken[0x5a]=0;assert_eq!(plan_reacquire(&broken,&enemy,p.rng_before),Err(ReacquireError::Current));
   assert_eq!(plan_reacquire(&current,&enemy,BlueprintRng{seed:28533,..p.rng_before}),Err(ReacquireError::UnsupportedDisposition));
  }
 }
}

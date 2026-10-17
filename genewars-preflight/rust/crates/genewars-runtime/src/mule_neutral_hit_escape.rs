//! Bounded neutral state20 animation-completed flee caller5F4C4..5FF13.
//! Caller proves health-positive task0, one live enemy/no friends, alliances,
//! safe current-cell8B7D0=false at5F97F, and no candidate-specific draws.
//! Source player-creation/instructions-0005ED20.txt; Mule1046B0 parameters
//! radius512/trait4=10/trait24=90.6BDD0 in initial-creature/0006B700 is
//! class==3; sourceMuleclass104A6C=4 makes it false, no callback fabricated.
//! Private literal proof775c3fd69ae244ee937e0e61a897c39d under
//! private/review/mule-neutral-hit-escape-proof; frozen5882, NOTcampaign.
use crate::{blueprint::BlueprintRng,creature_world::{CreatureRecord,mule_requested_speed},physics_tables::PhysicsTables};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Error{Identity,Current,Enemy,QueryDomain,InvalidRng,UnsupportedDisposition}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum Purpose{NeighborhoodX,NeighborhoodY,Disposition,Confidence,Distance,Heading}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct Draw{pub purpose:Purpose,pub journal:u16,pub seed_before:u16,pub seed_after:u16,pub value:u8}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Plan{pub after:CreatureRecord,pub rng_after:BlueprintRng,pub heading:i16,pub distance:i16,pub query_weight:i32,pub confidence:i32,pub enemy_sector:u8,pub draws:[Draw;6]}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn put(r:&mut CreatureRecord,at:usize,v:i16){r[at..at+2].copy_from_slice(&v.to_le_bytes());}
fn cell_delta(a:i16,b:i16)->i32{let a=i32::from((a as u16&32767)>>8);let b=i32::from((b as u16&32767)>>8);(b-a+64).rem_euclid(128)-64}
/// Current is postphysics/post5EB30, pre-query, state20/timer0. Stops before
///707A0; does not accept terrain or construct a waypoint itself. World must
/// separately prove no current-cell escape branch and a sole enemy winner.
pub fn plan(current:&CreatureRecord,enemy:&CreatureRecord,rng_before:BlueprintRng,physics:&PhysicsTables)->Result<Plan,Error>{
 let id=word(current,15);let target=word(enemy,15);
 if current[6]!=1||current[24]!=4||current[122]!=4||enemy[6]!=1||enemy[24]!=0||enemy[122]!=4||!(0..300).contains(&id)||!(0..300).contains(&target)||id==target{return Err(Error::Identity);}
 let reference=[target as u8,(target>>8)as u8,1];let reciprocal=[id as u8,(id>>8)as u8,1];
 if current[94]!=20||current[0x60]!=8||current[0x61]!=0x40||current[120]!=0||word(current,108)!=0||current[83]!=1||current[118]>90||current[119]!=255||current[123]!=255
 ||current[0x4f]!=0||current[0x5d]!=0||current[80..83]!=reference||current[85..88]!=reference
 ||word(current,106)<=0||word(current,100)<=0||word(current,106)>word(current,100)||word(current,102)<=2||word(current,104)<0||word(current,98)<=0||word(current,104)>(word(current,98)>>1)||word(current,73)as u16&0x500!=0{return Err(Error::Current);}
 if enemy[94]!=21||enemy[0x60]!=0x88||enemy[0x61]!=0||enemy[120]!=7||enemy[118]>90||enemy[0x4f]!=0||enemy[0x5d]!=0||word(enemy,106)<=0||word(enemy,73)as u16&0x400!=0
 ||[80,85,88].iter().any(|&at|enemy[at..at+3]!=reciprocal){return Err(Error::Enemy);}
 let dx=cell_delta(word(current,7),word(enemy,7));let dy=cell_delta(word(current,9),word(enemy,9));let distance_squared=(dx*dx+dy*dy)*65536;
 if distance_squared>=1024*1024{return Err(Error::QueryDomain);}
 let query_weight=1024-genewars_core::cd_candidate::integer_sqrt(distance_squared as u32)as i32;
 let confidence=i32::from(word(current,106))*256/i32::from(word(current,100))-((query_weight/512)<<5)+10;
 //6B794/73970 sector: angle of exact record positions, not cell centers.
 let doubled=[word(enemy,7).wrapping_mul(2).wrapping_sub(word(current,7).wrapping_mul(2)),word(enemy,9).wrapping_mul(2).wrapping_sub(word(current,9).wrapping_mul(2))];
 let enemy_sector=((crate::actor_motion::angle(i32::from(doubled[0]),i32::from(doubled[1]),physics.angles())&2047)>>7)as u8;
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut rng=rng_before;let draws=[Purpose::NeighborhoodX,Purpose::NeighborhoodY,Purpose::Disposition,Purpose::Confidence,Purpose::Distance,Purpose::Heading].map(|purpose|{
  let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);let journal=match purpose{Purpose::NeighborhoodX=>0x6bd,Purpose::NeighborhoodY=>0x6be,Purpose::Disposition|Purpose::Confidence=>0xcd6,Purpose::Distance=>0xce3,Purpose::Heading=>0x21};Draw{purpose,journal,seed_before:before,seed_after:rng.seed,value:(rng.seed>>8)as u8}
 });
 //5FB5C gate<=90;5FBE7 confidence roll>=confidence enters flee.
 if draws[2].value%100>90||i32::from(draws[3].value)<confidence{return Err(Error::UnsupportedDisposition);}
 let mut after=*current;after[21]&=0xdf;after[0x33..0x37].fill(0);put(&mut after,96,(word(current,96)as u16&0xd71a)as i16);after[0x5d]=0;after[0x57]=0;
 after[94]=0;let speed=mule_requested_speed(&after);put(&mut after,63,speed);put(&mut after,108,32767);after[96]|=0x80;
 //Friendcount0 selects5FDCD, not the distinct three-drawCDE branch.
 let distance=256+2*i16::from(draws[4].value);let heading=((i16::from(enemy_sector)<<7)+i16::from(draws[5].value%128)+1024)&2047;
 Ok(Plan{after,rng_after:rng,heading,distance,query_weight,confidence,enemy_sector,draws})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]
 #[ignore="requires GENEWARS_NEUTRAL_HIT_ESCAPE_CASES literal source cases"]
 fn literal_escape_prefix_records_rng_and_heading(){
  let data=std::fs::read(std::env::var_os("GENEWARS_NEUTRAL_HIT_ESCAPE_CASES").expect("set cases.bin path")).unwrap();assert_eq!(data.len(),4*390);
  let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");let a=std::fs::read(root.join("private/integration/game-runtime/physics/angle.bin")).unwrap();let b=std::fs::read(root.join("private/integration/game-runtime/physics/bounce.bin")).unwrap();let physics=PhysicsTables::from_mapped_bytes(&a,&b).unwrap();
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for(index,c)in data.chunks_exact(390).enumerate(){let current:CreatureRecord=c[..124].try_into().unwrap();let enemy:CreatureRecord=c[124..248].try_into().unwrap();let r=rng(&c[248..255]);let p=plan(&current,&enemy,r,&physics).unwrap();assert_eq!(p.after.as_slice(),&c[255..379],"case{index} record");assert_eq!(p.rng_after,rng(&c[379..386]));assert_eq!(p.heading,i16::from_le_bytes(c[386..388].try_into().unwrap()));assert_eq!(p.distance,i16::from_le_bytes(c[388..390].try_into().unwrap()));assert_eq!(p.confidence,194);assert_eq!(p.query_weight,1024);assert_eq!(p.enemy_sector,3);
   for(at,value)in[(94,21),(120,7),(83,0),(0x57,0)]{let mut bad=current;bad[at]=value;let saved=bad;assert!(plan(&bad,&enemy,r,&physics).is_err());assert_eq!(bad,saved);}
   assert_eq!(plan(&current,&enemy,BlueprintRng{seed:0,..r},&physics),Err(Error::UnsupportedDisposition));
  }
 }
}

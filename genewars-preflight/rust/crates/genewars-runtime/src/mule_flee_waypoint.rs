//! Bounded original707A0..70959 waypoint search, not the caller zero20 AI.
//! Evidence private/ghidra/mule-flee-194/instructions-000707a0.txt;
//! countdown-expiry/instructions-00013e60.txt proves ANDbyte16,7F;RET.
//!74CC0 terrain acceptance is an explicit fallible port, not inferred here.
use crate::{actor_activation::TrigTables,physics_tables::PhysicsTables,blueprint::BlueprintRng,creature_world::CreatureRecord};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum Error<E>{Identity,Parameters,InvalidRng,Acceptance(E)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct Candidate{pub heading:i16,pub xy:[i16;2],pub accepted:bool}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct Plan{pub after:CreatureRecord,pub rng_after:BlueprintRng,pub candidates:Vec<Candidate>,pub accepted:bool,pub journal:u16,pub draw:u8}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
/// Caller supplies original EDX heading, lowword EBX distance, CL boolean.
/// Acceptance receives exact signed-word candidate coordinates and byte53 mask.
/// One5EC draw chooses +128 versus-128 search order; attempts reset from
/// original position each time. No caller AI/state/timer/speed/RNG is invented.
pub fn plan<E,F>(before:&CreatureRecord,initial_angle:i16,distance:i16,cl_flag:bool,rng_before:BlueprintRng,trig:&TrigTables,physics:&PhysicsTables,mut accept:F)->Result<Plan,Error<E>>
where F:FnMut([i16;2],u8)->Result<bool,E>{
 if before[6]!=1||before[24]>4||before[122]!=4||!(0..300).contains(&word(before,15)){return Err(Error::Identity);}
 if !(0..2048).contains(&initial_angle)||distance<0{return Err(Error::Parameters);}
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut rng=rng_before;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
 let draw=(rng.seed>>8)as u8;let mut heading=initial_angle;let mut candidates=Vec::with_capacity(16);let mut after=*before;
 for _ in 0..16{
  let index=(i32::from(heading)-512).rem_euclid(2048)as usize;
  let dx=((i64::from(trig.cosine()[index])*i64::from(distance))>>16)as i16;
  let dy=((i64::from(trig.sine()[index])*i64::from(distance))>>16)as i16;
  let xy=[word(before,7).wrapping_add(dx),word(before,9).wrapping_add(dy)];
  let accepted=accept(xy,before[0x53]).map_err(Error::Acceptance)?;candidates.push(Candidate{heading,xy,accepted});
  if accepted{
   after[11..13].copy_from_slice(&xy[0].to_le_bytes());after[13..15].copy_from_slice(&xy[1].to_le_bytes());after[0x54]=20;
   let flags=(word(before,0x49)as u16|0x14)&0xeffc;after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
   after[0x16]&=0x3f; //708C4 BF then literal13E60 7F.
   after[0x4f]=0;after[0x52]=0;
   let doubled=[xy[0].wrapping_mul(2).wrapping_sub(word(before,7).wrapping_mul(2)),xy[1].wrapping_mul(2).wrapping_sub(word(before,9).wrapping_mul(2))];
   let yaw=crate::actor_motion::angle(i32::from(doubled[0]),i32::from(doubled[1]),physics.angles())as i16;
   after[0x39..0x3b].copy_from_slice(&yaw.to_le_bytes());if !cl_flag{after[0x4a]|=0x10;}
   return Ok(Plan{after,rng_after:rng,candidates,accepted:true,journal:0x5ec,draw});
  }
  heading=((i32::from(heading)+if draw%2!=0{128}else{1920})&2047)as i16;
 }
 let flags=(word(before,0x49)as u16&0xeebc)|0x140;after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());after[0x16]&=0x7f;
 Ok(Plan{after,rng_after:rng,candidates,accepted:false,journal:0x5ec,draw})
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]
 #[ignore="requires GENEWARS_FLEE_WAYPOINT_CASES literal source fixtures"]
 fn literal_waypoint_records_rng_and_candidate_order(){
  let data=std::fs::read(std::env::var_os("GENEWARS_FLEE_WAYPOINT_CASES").expect("set cases.bin path")).unwrap();assert_eq!(data.len(),8*335);
  let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");let angles=std::fs::read(root.join("private/integration/game-runtime/physics/angle.bin")).unwrap();let bounce=std::fs::read(root.join("private/integration/game-runtime/physics/bounce.bin")).unwrap();let physics=PhysicsTables::from_mapped_bytes(&angles,&bounce).unwrap();let trig=TrigTables::generate_pinned().unwrap();
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for(index,c)in data.chunks_exact(335).enumerate(){
   let current:CreatureRecord=c[..124].try_into().unwrap();let angle=i16::from_le_bytes(c[131..133].try_into().unwrap());let distance=i16::from_le_bytes(c[133..135].try_into().unwrap());let mask=u16::from_le_bytes(c[269..271].try_into().unwrap());let mut count=0;
   let p=plan(&current,angle,distance,c[135]!=0,rng(&c[124..131]),&trig,&physics,|xy,terrain_mask|{
    assert_eq!(terrain_mask,current[0x53]);let at=271+count*4;assert_eq!(xy,[i16::from_le_bytes(c[at..at+2].try_into().unwrap()),i16::from_le_bytes(c[at+2..at+4].try_into().unwrap())],"case{index} candidate{count}");let yes=mask&(1<<count)!=0;count+=1;Ok::<_,()>(yes)
   }).unwrap();
   assert_eq!(p.after.as_slice(),&c[136..260],"case{index} fullrecord");assert_eq!(p.rng_after,rng(&c[260..267]));assert_eq!(p.accepted,c[267]!=0);assert_eq!(count,usize::from(c[268]));assert_eq!(p.journal,0x5ec);
   let saved=current;assert_eq!(plan(&current,angle,distance,false,p.rng_after,&trig,&physics,|_,_|Err::<bool,_>("blocked")),Err(Error::Acceptance("blocked")));assert_eq!(current,saved);
  }
 }
}

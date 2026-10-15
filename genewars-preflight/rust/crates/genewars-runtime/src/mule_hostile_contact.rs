//! Bounded false-contact turn in state1's live-hostile tail65D8A..65DCE.
//! Source6EDA0(self,target,256) computes73970 desired yaw, then exact7A6C0
//! shortest signed delta(period2048), accepting contact iff abs(delta)<=256.
//! False sets only word4B through73970; true invokes virtual38/5C840 attack,
//! which is deliberately rejected here. Source73770 close range is inclusive.
//! Evidence: private/ghidra/mule-contact-191/instructions-0006eda0.txt,
//! player-creation/instructions-0005ed20.txt65D8A..65DD2 and
//! primitives/instructions-0007a6c0.txt. Caller owns physics, AI phase,
//! timer-abandon gates, target binding, reference refresh and common tail.
use crate::{creature_world::CreatureRecord,physics_tables::PhysicsTables};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum HostileContactError { Identity, Current, Target, NotClose, UnsupportedAttack }
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn dword(r:&CreatureRecord,at:usize)->i32{i32::from_le_bytes(r[at..at+4].try_into().unwrap())}
///7A6C0 preserves the direct difference on the exact half-period tie.
fn shortest_delta(current:i16,target:i16)->i32{
 let direct=i32::from(target)-i32::from(current);
 let alternative=direct-2048*direct.signum();
 if direct.abs()<=alternative.abs(){direct}else{alternative}
}
/// Returns an exact record after the FALSE-contact local tail, changing only
///4B. Aligned contact, dead targets and unsupported ownership/reference shapes
/// reject before any mutation. No RNG, damage, animation, linking or factories.
pub fn plan_false_contact(current:&CreatureRecord,target:&CreatureRecord,physics:&PhysicsTables)
 ->Result<CreatureRecord,HostileContactError>{
 use HostileContactError as Error;
 let id=word(current,15);let target_id=word(target,15);
 if !(0..300).contains(&id)||!(0..300).contains(&target_id)||id==target_id{return Err(Error::Identity);}
 let neutral=current[24]==4&&current[0x61]==0x40&&current[0x78]==0&&current[0x76]<=90;
 let owned=current[24]==0&&current[0x61]==0&&current[0x78]==7&&current[0x5a]==1&&word(current,0x58)==target_id;
 if current[6]!=1||current[122]!=4||current[94]!=1||!(neutral||owned)
  ||!matches!(current[0x60],0x48|0xc8)||current[0x4f]!=0||current[0x5d]!=0
  ||current[0x52]!=1||current[0x57]!=1||word(current,0x50)!=target_id||word(current,0x55)!=target_id
  ||word(current,0x6a)<=0||!(0..2048).contains(&word(current,0x39))
 {return Err(Error::Current);}
 if target[6]!=1||target[122]!=4||!matches!(target[94],1|8|21)
  ||target[24]!=(if neutral{0}else{4})||word(target,0x6a)<=0
  ||word(target,0x49)as u16&0x400!=0{return Err(Error::Target);}
 let xy=genewars_core::cd_candidate::distance_squared_wrapping_15(
  [word(current,7),word(current,9)],[word(target,7),word(target,9)])as i32;
 let z=dword(target,0x27).wrapping_sub(dword(current,0x27))>>3;
 let distance=xy.wrapping_add(z.wrapping_mul(z));
 if !(0..=16384).contains(&distance){return Err(Error::NotClose);}
 //73970 doubles coordinates as signed lowwords before AE028. Do not use
 //unwrapped Cartesian deltas across the signed15 terrain seam.
 let dx=word(target,7).wrapping_mul(2).wrapping_sub(word(current,7).wrapping_mul(2));
 let dy=word(target,9).wrapping_mul(2).wrapping_sub(word(current,9).wrapping_mul(2));
 let desired=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles())as i16;
 if shortest_delta(word(current,0x39),desired).abs()<=256{return Err(Error::UnsupportedAttack);}
 let mut after=*current;after[0x4b..0x4d].copy_from_slice(&desired.to_le_bytes());Ok(after)
}
#[cfg(test)]mod tests{
 use super::*;
 fn fixture()->(CreatureRecord,CreatureRecord){
  let mut a=[0xa5;124];let mut b=[0x5a;124];
  for(r,id,owner,state)in[(&mut a,5i16,4,1),(&mut b,1,0,8)]{r[6]=1;r[24]=owner;r[122]=4;r[94]=state;r[15..17].copy_from_slice(&id.to_le_bytes());r[0x6a..0x6c].copy_from_slice(&67i16.to_le_bytes());r[0x27..0x2b].fill(0);r[0x49..0x4b].fill(0);r[7..11].fill(0);}
  a[0x60]=0xc8;a[0x61]=0x40;a[0x78]=0;a[0x76]=50;a[0x4f]=0;a[0x5d]=0;
  a[0x50..0x53].copy_from_slice(&[1,0,1]);a[0x55..0x58].copy_from_slice(&[1,0,1]);
  b[7..9].copy_from_slice(&64i16.to_le_bytes());(a,b)
 }
 #[test]fn contact_threshold_is_inclusive_and_false_turn_preserves_all_other_bytes(){
  let(a,b)=fixture();let physics=PhysicsTables::fixture();
  for(yaw,aligned)in[(255i16,false),(256,true),(257,true),(768,true),(769,false)]{
   let mut current=a;current[0x39..0x3b].copy_from_slice(&yaw.to_le_bytes());let saved=current;
   if aligned{assert_eq!(plan_false_contact(&current,&b,&physics),Err(HostileContactError::UnsupportedAttack));}
   else{let after=plan_false_contact(&current,&b,&physics).unwrap();assert_eq!(word(&after,0x4b),512);for i in 0..124{if i!=0x4b&&i!=0x4c{assert_eq!(after[i],saved[i],"offset{i:x}");}}}
   assert_eq!(current,saved);
  }
 }
 #[test]fn angle_and_position_seams_use_original_period_and_reject_attack(){
  let(mut a,mut b)=fixture();let physics=PhysicsTables::fixture();
  a[7..9].copy_from_slice(&32760i16.to_le_bytes());b[7..9].copy_from_slice(&8i16.to_le_bytes());a[0x39..0x3b].copy_from_slice(&255i16.to_le_bytes());assert_eq!(word(&plan_false_contact(&a,&b,&physics).unwrap(),0x4b),512);
  a[7..11].fill(0);b[7..9].fill(0);b[9..11].copy_from_slice(&(-64i16).to_le_bytes());
  for(yaw,aligned)in[(2047i16,true),(1792,true),(1791,false),(256,true),(257,false),(1024,false)]{
   a[0x39..0x3b].copy_from_slice(&yaw.to_le_bytes());
   if aligned{assert_eq!(plan_false_contact(&a,&b,&physics),Err(HostileContactError::UnsupportedAttack));}
   else{assert_eq!(word(&plan_false_contact(&a,&b,&physics).unwrap(),0x4b),0);}
  }
  assert_eq!(shortest_delta(0,1024),1024);assert_eq!(shortest_delta(1024,0),-1024);
 }
 #[test]fn unsupported_references_target_and_far_range_leave_inputs_unchanged(){
  let(mut a,b)=fixture();a[0x39..0x3b].copy_from_slice(&255i16.to_le_bytes());let physics=PhysicsTables::fixture();let saved=(a,b);
  let mut bad=a;bad[0x52]=0;assert_eq!(plan_false_contact(&bad,&b,&physics),Err(HostileContactError::Current));
  let mut dead=b;dead[94]=24;assert_eq!(plan_false_contact(&a,&dead,&physics),Err(HostileContactError::Target));
  let mut far=b;far[7..9].copy_from_slice(&129i16.to_le_bytes());assert_eq!(plan_false_contact(&a,&far,&physics),Err(HostileContactError::NotClose));
  assert_eq!((a,b),saved);
 }

 #[test] #[ignore="requires private literal6EDA0/65DBE original-source fixtures"]
 fn hostile_false_contact_literal_cases(){
  let path=std::env::var("GENEWARS_HOSTILE_CONTACT_CASES").expect("private cases.bin path required");
  let data=std::fs::read(path).unwrap();assert_eq!(data.len(),13*373);
  let resources=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime/physics");
  let physics=PhysicsTables::from_mapped_bytes(&std::fs::read(resources.join("angle.bin")).unwrap(),&std::fs::read(resources.join("bounce.bin")).unwrap()).unwrap();let mut false_count=0;
  for (i,c)in data.chunks_exact(373).enumerate(){
   let current:CreatureRecord=c[..124].try_into().unwrap();let target:CreatureRecord=c[124..248].try_into().unwrap();
   if c[248]!=0 {let expected:CreatureRecord=c[249..373].try_into().unwrap();assert_eq!(plan_false_contact(&current,&target,&physics).unwrap(),expected,"literalcase{i}");false_count+=1;}
   else{assert_eq!(plan_false_contact(&current,&target,&physics),Err(HostileContactError::UnsupportedAttack),"alignedcase{i}");}
  }
  assert_eq!(false_count,7);
 }
}

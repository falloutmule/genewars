//! Exact bounded true-contact virtual38/5C840 entry, before65DB5/common tail.
//! Calls1EE50 stop, turns desired yaw to the live target and consumes one6E1
//! draw. It does NOT apply damage, clear low60bit40, advance animation, or
//! implement later state21 combat. Caller owns those original continuations.
//! Evidence private/ghidra/mule-contact-191/instructions-0005c840.txt and
//! construction-command/instructions-0001ee50.txt. Mule table104A6Cbyte4
//! selects1060A4byte5B (signed-positive channel91); mapped object3 SHA256
//! aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87.
use crate::{blueprint::BlueprintRng,creature_world::CreatureRecord,physics_tables::PhysicsTables};
use crate::mule_hostile_contact::{self,HostileContactError};
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum AttackEntryError { Contact(HostileContactError), NotAligned, InvalidRng }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct AttackEntryDraw { pub journal:u16,pub seed_before:u16,pub seed_after:u16,pub value:u8 }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct AttackEntryPlan {
 pub after:CreatureRecord,pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,
 pub draw:AttackEntryDraw,
 ///5C8E6..5C90A virtual44: channel91, EBX2, ECX0, stack[127,100].
 pub pending_audio:Option<u8>,
}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
/// Requires the same bounded live state1 owners/references/range as the false
/// contact planner, and original6EDA0 must return true(absdelta<=256).
/// Neither input is mutated. Returned timer32767 precedes the shared tail;
/// low60bit40 remains unchanged until the separate source caller65DB5.
pub fn plan_entry(current:&CreatureRecord,target:&CreatureRecord,rng_before:BlueprintRng,physics:&PhysicsTables)
 ->Result<AttackEntryPlan,AttackEntryError>{
 match mule_hostile_contact::plan_false_contact(current,target,physics){
  Err(HostileContactError::UnsupportedAttack)=>{},
  Ok(_)=>return Err(AttackEntryError::NotAligned),
  Err(error)=>return Err(AttackEntryError::Contact(error)),
 }
 if rng_before.ring>=32{return Err(AttackEntryError::InvalidRng);}
 let dx=word(target,7).wrapping_mul(2).wrapping_sub(word(current,7).wrapping_mul(2));
 let dy=word(target,9).wrapping_mul(2).wrapping_sub(word(current,9).wrapping_mul(2));
 let yaw=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles())as i16;
 let mut after=*current;
 after[0x5e]=21;after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());
 //Exact virtual84=1EE50. Current speed41 and retained target references survive.
 after[0x3f..0x41].fill(0);
 after[0x49..0x4b].copy_from_slice(&(word(current,0x49)as u16&0xefe8).to_le_bytes());
 after[0x54]=0;after[0x16]&=0x3f;after[0x4b..0x4d].copy_from_slice(&yaw.to_le_bytes());
 let mut rng=rng_before;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
 let draw=AttackEntryDraw{journal:0x6e1,seed_before:rng_before.seed,seed_after:rng.seed,value:(rng.seed>>8)as u8};
 let pending_audio=if draw.value%3==1{Some(0x5b)}else{None};
 Ok(AttackEntryPlan{after,rng_before,rng_after:rng,draw,pending_audio})
}
#[cfg(test)]mod tests{
 use super::*;
 fn fixture()->(CreatureRecord,CreatureRecord){
  let mut a=[0xa5;124];let mut b=[0x5a;124];
  for(r,id,owner)in[(&mut a,5i16,4),(&mut b,1i16,0)]{r[6]=1;r[24]=owner;r[122]=4;r[94]=1;r[15..17].copy_from_slice(&id.to_le_bytes());r[0x6a..0x6c].copy_from_slice(&67i16.to_le_bytes());r[0x27..0x2b].fill(0);r[0x49..0x4b].fill(0);r[7..11].fill(0);}
  a[0x60]=0xc8;a[0x61]=0x40;a[0x78]=0;a[0x76]=50;a[0x4f]=0;a[0x5d]=0;a[0x16]=0x82;
  a[0x50..0x53].copy_from_slice(&[1,0,1]);a[0x55..0x58].copy_from_slice(&[1,0,1]);a[0x39..0x3b].fill(0);a[0x49..0x4b].copy_from_slice(&0x3377u16.to_le_bytes());
  b[9..11].copy_from_slice(&(-64i16).to_le_bytes());(a,b)
 }
 #[test]fn source_stop_and_state_entry_preserve_current_speed_refs_and_opaque_fields(){
  let(a,b)=fixture();let physics=PhysicsTables::fixture();let rng=BlueprintRng{seed:3,ring:31,draw_count:u32::MAX};let p=plan_entry(&a,&b,rng,&physics).unwrap();
  assert_eq!(p.after[94],21);assert_eq!(word(&p.after,0x3f),0);assert_eq!(word(&p.after,0x6c),32767);assert_eq!(word(&p.after,0x4b),0);assert_eq!(word(&p.after,0x49)as u16,0x3377&0xefe8);
  assert_eq!(p.after[0x60],0xc8);assert_eq!(&p.after[0x50..0x53],&a[0x50..0x53]);assert_eq!(&p.after[0x55..0x5e],&a[0x55..0x5e]);
  assert_eq!(p.draw.journal,0x6e1);assert_eq!(p.draw.value,178);assert_eq!(p.pending_audio,Some(0x5b));assert_eq!(p.rng_after.ring,0);assert_eq!(p.rng_after.draw_count,0);
  for i in 0..124{if !matches!(i,0x16|0x3f|0x40|0x49..=0x4c|0x54|0x5e|0x6c|0x6d){assert_eq!(p.after[i],a[i],"offset{i:x}");}}
 }
 #[test]fn no_audio_and_unaligned_or_invalid_rng_reject_without_input_mutation(){
  let(a,b)=fixture();let physics=PhysicsTables::fixture();let rng=BlueprintRng{seed:0,ring:0,draw_count:0};let saved=(a,b);
  let p=plan_entry(&a,&b,rng,&physics).unwrap();assert_eq!(p.pending_audio,None);assert_eq!(p.draw.value,0);
  let mut bad=a;bad[0x39..0x3b].copy_from_slice(&257i16.to_le_bytes());assert_eq!(plan_entry(&bad,&b,rng,&physics),Err(AttackEntryError::NotAligned));
  let mut invalid=rng;invalid.ring=32;assert_eq!(plan_entry(&a,&b,invalid,&physics),Err(AttackEntryError::InvalidRng));assert_eq!((a,b),saved);
 }
 #[test]
 #[ignore = "requires GENEWARS_HOSTILE_ATTACK_CASES literal source fixture"]
 fn literal_attack_entry_cases_match_all_record_rng_audio_bytes() {
  let path=std::env::var_os("GENEWARS_HOSTILE_ATTACK_CASES").expect("set literal cases.bin path");
  let bytes=std::fs::read(path).unwrap();assert_eq!(bytes.len(),10*387);
  let repository=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
  let angle=std::fs::read(repository.join("private/integration/game-runtime/physics/angle.bin")).unwrap();
  let bounce=std::fs::read(repository.join("private/integration/game-runtime/physics/bounce.bin")).unwrap();
  let physics=PhysicsTables::from_mapped_bytes(&angle,&bounce).unwrap();
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for (index,case) in bytes.chunks_exact(387).enumerate() {
   let current:CreatureRecord=case[..124].try_into().unwrap();
   let target:CreatureRecord=case[124..248].try_into().unwrap();
   let before=(current,target);
   let plan=plan_entry(&current,&target,rng(&case[248..255]),&physics).unwrap();
   assert_eq!(plan.after.as_slice(),&case[255..379],"case {index}: complete record");
   assert_eq!(plan.rng_after,rng(&case[379..386]),"case {index}: RNG");
   assert_eq!(plan.pending_audio,if case[386]==255{None}else{Some(case[386])},"case {index}: audio");
   assert_eq!((current,target),before,"case {index}: immutable inputs");
  }
 }
}

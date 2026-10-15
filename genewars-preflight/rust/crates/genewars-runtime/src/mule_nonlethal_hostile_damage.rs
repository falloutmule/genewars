//! Bounded nonself5B9F0: owned task7 Mule hits neutral pursuing Mule.
//! Actual5876 source call and10 literal cases: private/review/
//! mule-nonlethal-hostile-damage-proof/08e6b7761e5a4b12973568c567cc09cc.
//! Source construction-command/instructions-0005b9f0.txt, mission-array-refs/
//! instructions-0005cc60.txt. Pinned object3 SHA256
//! aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87:
//! Mule1046B0 defense+5=6;104A6C class04,04. No self-water substitution.
use crate::blueprint::BlueprintRng;
use crate::creature_world::CreatureRecord;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Error{Identity,State,References,SpeedContinuation,DamageDomain,InvalidRng}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct Draw{pub journal:u16,pub seed_before:u16,pub seed_after:u16,pub value:u8}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Plan{pub target_after:CreatureRecord,pub effective_damage:i16,pub rng_after:BlueprintRng,pub pending_audio:u8,pub draws:Vec<Draw>}
fn word(r:&CreatureRecord,at:usize)->i16{i16::from_le_bytes([r[at],r[at+1]])}
fn draw(rng:&mut BlueprintRng,draws:&mut Vec<Draw>)->u8{
 let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
 let value=(rng.seed>>8)as u8;draws.push(Draw{journal:0x685,seed_before:before,seed_after:rng.seed,value});value
}
/// Caller computes incoming damage and proves live typed Mule bindings. This
/// packet includes only nonflight, nonlethal effective1..4, no blood or death.
/// It excludes the5BC63 virtual84 stop branch by requiring attacker speed less
/// than target speed. All inputs remain immutable on every return.
pub fn plan(target:&CreatureRecord,attacker:&CreatureRecord,incoming:i16,rng_before:BlueprintRng)->Result<Plan,Error>{
 let ti=word(target,15);let ai=word(attacker,15);
 if target[6]!=1||attacker[6]!=1||target[24]!=4||attacker[24]!=0||target[122]!=4||attacker[122]!=4||!(0..300).contains(&ti)||!(0..300).contains(&ai)||ti==ai{return Err(Error::Identity);}
 if target[94]!=1||target[0x61]!=0x40||!matches!(target[0x60],0x48|0xc8)||target[0x78]!=0||target[0x76]>90
 ||attacker[94]!=21||attacker[0x61]!=0||!matches!(attacker[0x60],8|0x88)||attacker[0x78]!=7||attacker[0x76]>90
 ||word(target,0x49)as u16&0x400!=0||word(attacker,0x49)as u16&0x400!=0||word(attacker,0x6a)<=0{return Err(Error::State);}
 let tref=[ai as u8,(ai>>8)as u8,1];let aref=[ti as u8,(ti>>8)as u8,1];
 if target[0x4f]!=0||target[0x5d]!=0||attacker[0x4f]!=0||attacker[0x5d]!=0
 ||target[0x50..0x53]!=tref||target[0x55..0x58]!=tref
 ||[0x50,0x55,0x58].iter().any(|&at|attacker[at..at+3]!=aref){return Err(Error::References);}
 if word(attacker,0x41)>=word(target,0x41){return Err(Error::SpeedContinuation);}
 let percent=50+50*i32::from(target[0x76])/100;let defense=6*percent/100;
 let effective=(i32::from(incoming)-defense).max(1);
 if incoming<=0||effective>4||i32::from(word(target,0x6a))<=effective{return Err(Error::DamageDomain);}
 if rng_before.ring>=32{return Err(Error::InvalidRng);}
 let mut after=*target;
 //5CC60 state1 dispatch5CC73 releases only kind2 plant reservations; our
 //kind1 target has no external release.5CCA0 local cleanup preserves50/58.
 after[0x15]&=0xdf;after[0x33..0x37].fill(0);
 after[0x60..0x62].copy_from_slice(&((word(target,0x60)as u16)&0xd71a).to_le_bytes());
 after[0x5d]=0;after[0x57]=1; //5C3CD..5C3DB reinstalls identical attacker ref55.
 after[0x6a..0x6c].copy_from_slice(&(word(target,0x6a)-effective as i16).to_le_bytes());
 after[94]=20;after[0x6c..0x6e].copy_from_slice(&32767i16.to_le_bytes());
 let mut rng=rng_before;let mut draws=Vec::with_capacity(2);
 //5C404 and5C48E BOTH journal685; the latter is not journal686.
 let pending_audio=if draw(&mut rng,&mut draws)%2!=0{8}else{28+draw(&mut rng,&mut draws)%3};
 Ok(Plan{target_after:after,effective_damage:effective as i16,rng_after:rng,pending_audio,draws})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]
 #[ignore="requires GENEWARS_HOSTILE_DAMAGE_CASES literal source cases"]
 fn literal_nonlethal_damage_matches_records_rng_audio(){
  let data=std::fs::read(std::env::var_os("GENEWARS_HOSTILE_DAMAGE_CASES").expect("set cases.bin path")).unwrap();assert_eq!(data.len(),10*389);
  let rng=|b:&[u8]|BlueprintRng{seed:u16::from_le_bytes(b[..2].try_into().unwrap()),ring:b[2],draw_count:u32::from_le_bytes(b[3..7].try_into().unwrap())};
  for(index,c)in data.chunks_exact(389).enumerate(){
   let t:CreatureRecord=c[..124].try_into().unwrap();let a:CreatureRecord=c[124..248].try_into().unwrap();let damage=i16::from_le_bytes(c[248..250].try_into().unwrap());
   let p=plan(&t,&a,damage,rng(&c[250..257])).unwrap();assert_eq!(p.target_after.as_slice(),&c[257..381],"record case{index}");assert_eq!(p.rng_after,rng(&c[381..388]));assert_eq!(p.pending_audio,c[388]);
  }
 }
 #[test]fn unsupported_domains_preserve_inputs(){
  let mut t=[0xa5;124];let mut a=[0x5a;124];for(r,id,owner,state)in[(&mut t,5i16,4,1),(&mut a,1i16,0,21)]{r[6]=1;r[24]=owner;r[122]=4;r[94]=state;r[15..17].copy_from_slice(&id.to_le_bytes());r[0x49..0x4b].fill(0);r[0x4f]=0;r[0x5d]=0;r[0x76]=50;r[0x6a..0x6c].copy_from_slice(&67i16.to_le_bytes());}
  t[0x60]=0xc8;t[0x61]=0x40;t[0x78]=0;t[0x41..0x43].copy_from_slice(&277i16.to_le_bytes());a[0x41..0x43].fill(0);a[0x60]=0x88;a[0x61]=0;a[0x78]=7;
  for at in[0x50,0x55]{t[at..at+3].copy_from_slice(&[1,0,1]);}for at in[0x50,0x55,0x58]{a[at..at+3].copy_from_slice(&[5,0,1]);}
  let rng=BlueprintRng{seed:0,ring:31,draw_count:u32::MAX};let saved=(t,a);let p=plan(&t,&a,8,rng).unwrap();assert_eq!(p.effective_damage,4);assert_eq!(p.rng_after.ring,1);assert_eq!(p.rng_after.draw_count,1);assert_eq!((t,a),saved);
  for(offset,value)in[(94,8),(0x57,0),(0x60,0xca),(0x76,91)]{let mut bad=t;bad[offset]=value;let before=bad;assert!(plan(&bad,&a,8,rng).is_err());assert_eq!(bad,before);}
  for incoming in[0,9,i16::MAX]{assert_eq!(plan(&t,&a,incoming,rng),Err(Error::DamageDomain));}
  let mut lethal=t;lethal[0x6a..0x6c].copy_from_slice(&4i16.to_le_bytes());assert_eq!(plan(&lethal,&a,8,rng),Err(Error::DamageDomain));
 }
}

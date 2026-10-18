//! Bounded child-effect20 damage and reaction (4F6A0 -> 5B9F0 -> 5C7E0).
//! The caller owns query traversal, factory/map links, player statistics and
//! effect audio. This pure plan does not enable firing or advance the world.
use crate::blueprint::BlueprintRng;
use crate::landing::{LandingAudioRequest,LandingRecord};
pub type ImpactRecipient=[u8;124];
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ImpactError{UnsupportedRecipient,UnsupportedEffect,InvalidRng}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ImpactPlan{
 pub before:ImpactRecipient,pub after:ImpactRecipient,
 pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,
 pub audio:Vec<LandingAudioRequest>,
 /// Apply one wrapping increment to the source player's counter at0x6FD.
 pub kill_credit:Option<u8>,
 /// Health reached zero, but source still enters reaction18 afterward.
 pub lethal:bool,
}
fn word(a:&[u8],i:usize)->u16{u16::from_le_bytes([a[i],a[i+1]])}
fn put(a:&mut[u8],i:usize,v:u16){a[i..i+2].copy_from_slice(&v.to_le_bytes())}
fn cancel(a:&mut[u8]){put(a,51,0);put(a,53,0);a[21]&=!32;put(a,96,word(a,96)&0xd71a);a[93]=0;a[87]=0;put(a,108,0)}
fn draw(rng:&mut BlueprintRng)->u16{rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);rng.seed>>8}
fn audio(channel:u8)->LandingAudioRequest{LandingAudioRequest{channel,mode:2,parameter:0,stack_byte:127,stack_dword:100}}
/// Admits an unlinked neutral idle Mule and a building-origin child20.
/// Query RNG has already been consumed. Preserve opaque fields and every map
/// link. Other species, states and linked work must be recovered separately.
pub fn plan_neutral_mule_impact(before:&ImpactRecipient,effect:&LandingRecord,rng:BlueprintRng)->Result<ImpactPlan,ImpactError>{
 if rng.ring>=32{return Err(ImpactError::InvalidRng)}
 if before[6]!=1||before[24]!=4||before[122]!=4||before[94]!=8||word(before,96)&2!=0||(word(before,106)as i16)<1{return Err(ImpactError::UnsupportedRecipient)}
 if effect[6]!=5||effect[69]!=20||effect[73]!=4||word(effect,71)>=256||effect[24]>4||word(effect,15)>=200{return Err(ImpactError::UnsupportedEffect)}
 let mut a=*before;let mut rng_after=rng;
 // Incoming damage is1; the source's armor subtraction floors it to1.
 cancel(&mut a);put(&mut a,106,word(before,106).wrapping_sub(1));a[94]=20;put(&mut a,108,32767);put(&mut a,85,word(effect,15));a[87]=5;
 let channel=if draw(&mut rng_after)%2!=0{8}else{28+(draw(&mut rng_after)%3)as u8};let mut events=vec![audio(channel)];
 let lethal=word(&a,106)==0;
 if lethal{
  a[94]=24;cancel(&mut a);let flags=word(&a,21)&0xf7df;put(&mut a,21,flags);a[120]=0;
  let flags=word(&a,73)&0x3bef;put(&mut a,73,flags);let flags=word(&a,21)&0x3fdf;put(&mut a,21,flags);a[96]&=0xef;put(&mut a,102,0);events.push(audio(31));
 }
 // 4F6A0 calls reaction even after lethal damage. Do not replace this with
 // an assumed immediate corpse state; the next full creature update does so.
 cancel(&mut a);a[97]&=127;put(&mut a,63,0);let flags=word(&a,73)&0xefe8;put(&mut a,73,flags);a[84]=0;a[22]&=63;a[94]=18;put(&mut a,108,64);a[74]&=0x3b;a[21]&=!32;events.push(audio(57));
 Ok(ImpactPlan{before:*before,after:a,rng_before:rng,rng_after,audio:events,kill_credit:if lethal&&effect[24]<4{Some(effect[24])}else{None},lethal})
}

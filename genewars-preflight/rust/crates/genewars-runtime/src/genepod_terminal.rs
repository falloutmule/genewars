//! Terminal Genepod payment/quality prefix45470..4568F/459E8 only.
//! Continuations are NOT commit-ready: caller must complete factory/failure
//! effects, research, shared animation/power tail in one world transaction.
use crate::{blueprint::BlueprintRng,genepod_positive_work::{self,WorkPlan,WorkStatus,WorkError}};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum TerminalError{Work(WorkError),NotTerminal,Rng}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum TerminalRequest{
 ///5A210 owner=payer,species4,quality lowbyte,nullparent; sound1A precedes it.
 Factory{owner:u8,species:u8,quality:u8,position:[i16;2],sound:u8},
 ///459E8 continuation, not mission loss: sentinel301,sound8, randomized
 ///sound1C..1E and3..8 effect20 factories, then capped research increase.
 FailedQuality,
}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct TerminalContinuation{
 pub building_staged:[u8;67],pub players_staged:[[u8;1805];4],pub rng_after:BlueprintRng,
 pub payer:u8,pub quality:i32,pub roll:u8,pub request:TerminalRequest,
}
#[derive(Debug,Clone,PartialEq,Eq)]pub enum TerminalPlan{Waiting(WorkPlan),Continue(TerminalContinuation)}
pub fn plan_terminal(building:&[u8;67],players:&[[u8;1805];4],frame:u32,mut rng:BlueprintRng)->Result<TerminalPlan,TerminalError>{
 if i32::from_le_bytes(building[54..58].try_into().unwrap())!=1{return Err(TerminalError::NotTerminal);}
 if rng.ring>=32{return Err(TerminalError::Rng);}
 //Reuse the identical cadence/affordability/debit prefix. The work2 local
 //scratch avoids its intentional terminal rejection; no world is mutated.
 let mut scratch=*building;scratch[54..58].copy_from_slice(&2i32.to_le_bytes());
 let mut work=genepod_positive_work::plan_work(&scratch,players,frame).map_err(TerminalError::Work)?;
 let payer=match work.status{WorkStatus::Paid{payer,..}=>payer,_=>{work.building_after[54..58].copy_from_slice(&1i32.to_le_bytes());return Ok(TerminalPlan::Waiting(work));}};
 let a=&mut work.building_after;a[54..58].fill(0);a[27]=1;
 let mut quality=i32::from_le_bytes(work.players_after[payer as usize][0x62a..0x62e].try_into().unwrap())>>11;if a[46]<2{quality=quality.min(100);}
 rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);let roll=((rng.seed>>8)%100)as u8;
 let request=if i32::from(roll)<=quality.wrapping_mul(2){TerminalRequest::Factory{owner:payer,species:4,quality:quality as u8,position:[i16::from_le_bytes([a[7],a[8]]),i16::from_le_bytes([a[9],a[10]])],sound:0x1a}}else{TerminalRequest::FailedQuality};
 Ok(TerminalPlan::Continue(TerminalContinuation{building_staged:work.building_after,players_staged:work.players_after,rng_after:rng,payer,quality,roll,request}))
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct EjectionRequest{pub slot:u16,pub yaw:u16,pub pitch:u16,pub speed:u16,pub height_delta:i32}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct TerminalAfterRequests{pub building_staged:[u8;67],pub players_staged:[[u8;1805];4],pub rng_after:BlueprintRng,pub ejection:Option<EjectionRequest>}
fn draw(r:&mut BlueprintRng)->u16{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=(r.ring+1)&31;r.draw_count=r.draw_count.wrapping_add(1);r.seed>>8}
fn research(p:&mut[u8;1805],increase:i32){let cap=(50+25*i32::from(p[0x6e1]))<<11;let old=i32::from_le_bytes(p[0x62a..0x62e].try_into().unwrap());if old<cap{let next=old.wrapping_add(increase);p[0x62a..0x62e].copy_from_slice(&(if cap>next{next}else{cap}).to_le_bytes());}}
/// Resume ONLY after real5A210 result, using its returned shared RNG. Ejection
/// is an explicit world obligation: original velocity is built from these
/// angles/speed and newborn physicalheight is increased by3840.
pub fn plan_factory_return(c:&TerminalContinuation,newborn_slot:Option<u16>,players_after_factory:&[[u8;1805];4],mut rng:BlueprintRng)->Result<TerminalAfterRequests,TerminalError>{
 if !matches!(c.request,TerminalRequest::Factory{..})||newborn_slot.is_some_and(|s|s>=300)||c.payer>=4{return Err(TerminalError::NotTerminal);}
 if rng.ring>=32{return Err(TerminalError::Rng);}
 let mut a=c.building_staged;let mut p=*players_after_factory;let grade=i32::from(a[46]);let ejection=if let Some(slot)=newborn_slot{
  let speed=3840+64*(draw(&mut rng)%100);let pitch=2*(draw(&mut rng)%50);let yaw=8*draw(&mut rng);a[58..60].copy_from_slice(&slot.to_le_bytes());research(&mut p[c.payer as usize],(grade+1)<<11);
  Some(EjectionRequest{slot,yaw,pitch,speed,height_delta:3840})
 }else{a[58..60].copy_from_slice(&301u16.to_le_bytes());None};
 research(&mut p[c.payer as usize],grade<<6);Ok(TerminalAfterRequests{building_staged:a,players_staged:p,rng_after:rng,ejection})
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct FailureEffectRequest{pub code:u8,pub position:[i16;2],pub physical_height:i32,pub yaw:u16,pub pitch:u16,pub speed:u16}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct FailureCursor{continuation:TerminalContinuation,pub remaining:u8,pub rng_after:BlueprintRng,pub randomized_sound:u8,pub randomized_parameter:u16}
///Sound8 precedes this slice. Returned randomized sound uses mode2,priority1,
///volume127 and the explicit randomized parameter; no device playback implied.
pub fn begin_failure(c:&TerminalContinuation)->Result<FailureCursor,TerminalError>{
 if c.request!=TerminalRequest::FailedQuality||c.payer>=4{return Err(TerminalError::NotTerminal);}
 let mut continuation=c.clone();continuation.building_staged[58..60].copy_from_slice(&301u16.to_le_bytes());let mut rng=c.rng_after;
 let randomized_parameter=50+draw(&mut rng)%100;let randomized_sound=(28+draw(&mut rng)%3)as u8;let remaining=(3+draw(&mut rng)%6)as u8;
 Ok(FailureCursor{continuation,remaining,rng_after:rng,randomized_sound,randomized_parameter})
}
/// Caller executes EACH48790 before requesting the next effect, passing back
/// its real RNG. This preserves factory draws even on allocation failure.
pub fn next_failure_effect(cursor:&FailureCursor,mut rng:BlueprintRng)->Result<(FailureCursor,FailureEffectRequest),TerminalError>{
 if cursor.remaining==0{return Err(TerminalError::NotTerminal);}if rng.ring>=32{return Err(TerminalError::Rng);}
 let speed=8192+64*(draw(&mut rng)%128);let pitch=2*(draw(&mut rng)%150);let yaw=8*draw(&mut rng);let b=&cursor.continuation.building_staged;
 let request=FailureEffectRequest{code:0x20,position:[i16::from_le_bytes([b[7],b[8]]),i16::from_le_bytes([b[9],b[10]])],physical_height:i32::from_le_bytes(b[17..21].try_into().unwrap()).wrapping_add(3840),yaw,pitch,speed};
 let mut after=cursor.clone();after.remaining-=1;after.rng_after=rng;Ok((after,request))
}
/// After the last real effect factory; common animation/power tail still pending.
pub fn finish_failure(cursor:&FailureCursor,rng_after_last_factory:BlueprintRng)->Result<TerminalAfterRequests,TerminalError>{
 if cursor.remaining!=0{return Err(TerminalError::NotTerminal);}if rng_after_last_factory.ring>=32{return Err(TerminalError::Rng);}
 let mut players=cursor.continuation.players_staged;research(&mut players[cursor.continuation.payer as usize],2048);
 Ok(TerminalAfterRequests{building_staged:cursor.continuation.building_staged,players_staged:players,rng_after:rng_after_last_factory,ejection:None})
}

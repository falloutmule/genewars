//!62270..624AC neutral Mule birth continuation, before the common callback tail.
//! No5CC60 reset occurs: paired references and opaque flags are preserved.
//! Caller supplies each original5B100 result against the SAME saved current AI
//! cell and the post-factory world.73C20 samples directly; it has no74D90 query.
use crate::{actor_activation::TrigTables,blueprint::BlueprintRng};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum BirthTailError<E>{Record,Cell,Rng,Predicate(E)}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct BirthTailPlan{pub current_after:[u8;124],pub peer_after:[u8;124],pub rng_after:BlueprintRng,pub relocated:[bool;2]}
/// Predicate order is peer then current, with state already8. The boolean array
/// follows that order. The caller stages any environment services transactionally.
pub fn plan_tail<E>(current:&[u8;124],peer:&[u8;124],saved_cell:u16,mut rng:BlueprintRng,trig:&TrigTables,mut predicate:impl FnMut(&[u8;124],u16)->Result<bool,E>)->Result<BirthTailPlan,BirthTailError<E>>{
 let word=|r:&[u8;124],at|i16::from_le_bytes([r[at],r[at+1]]);
 if saved_cell>=16384{return Err(BirthTailError::Cell);}
 if rng.ring>=32{return Err(BirthTailError::Rng);}
 if word(current,15)==word(peer,15){return Err(BirthTailError::Record);}
 for(r,other)in[(current,peer),(peer,current)]{
  if r[6]!=1||r[24]!=4||r[0x7a]!=4||r[0x5e]!=10||r[0x57]!=1||word(r,0x55)!=word(other,15)||!(0..300).contains(&word(r,15)){return Err(BirthTailError::Record);}
 }
 let mut a=*current;let mut b=*peer;let mut relocated=[false;2];
 for(index,r)in [&mut b,&mut a].into_iter().enumerate(){
  rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
  let delay=(rng.seed>>8)%20+10;r[0x5e]=8;
  relocated[index]=predicate(r,saved_cell).map_err(BirthTailError::Predicate)?;
  if !relocated[index]{r[0x6c..0x6e].copy_from_slice(&delay.to_le_bytes());continue;}
  let sample=crate::construction_work::sample_construction_work_point([word(r,7),word(r,9)],rng.seed,trig.cosine().try_into().unwrap(),trig.sine().try_into().unwrap());
  rng.seed=sample.seed_after;rng.ring=(rng.ring+2)&31;rng.draw_count=rng.draw_count.wrapping_add(2);
  for axis in 0..2{r[11+axis*2..13+axis*2].copy_from_slice(&sample.point[axis].to_le_bytes());}
  r[0x54]=20;r[0x16]&=0x3f;let flags=(u16::from_le_bytes([r[0x49],r[0x4a]])|0x14)&0xeffc;r[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());r[0x4f]=0;r[0x52]=0;
  let speed=crate::creature_world::mule_requested_speed(r);r[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());r[0x6c..0x6e].copy_from_slice(&0x7fffu16.to_le_bytes());r[0x60]|=0x80;
 }
 Ok(BirthTailPlan{current_after:a,peer_after:b,rng_after:rng,relocated})
}

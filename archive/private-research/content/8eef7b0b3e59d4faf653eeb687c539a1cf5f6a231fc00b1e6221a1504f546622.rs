//! Source5B260 waypoint branch after caller proves virtual10=false and5B100=true.
//! The supplied query must implement74D90 with record53, self, and associated building.
use crate::{actor_activation::TrigTables,blueprint::BlueprintRng};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum CellEntryError<E>{Record,Cell,Rng,Query(E)}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CellEntryPlan{pub record_after:[u8;124],pub rng_after:BlueprintRng,pub queried_points:Vec<[i16;2]>,pub accepted:bool}
pub fn plan_waypoint<E>(before:&[u8;124],new_cell:u16,mut rng:BlueprintRng,trig:&TrigTables,mut accepts:impl FnMut([i16;2])->Result<bool,E>)->Result<CellEntryPlan,CellEntryError<E>>{
 if before[6]!=1||!matches!(before[0x5e],1|8){return Err(CellEntryError::Record);}
 if new_cell>=16384{return Err(CellEntryError::Cell);}
 if rng.ring>=32{return Err(CellEntryError::Rng);}
 let word=|at|i16::from_le_bytes([before[at],before[at+1]]);
 rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
 let step=if (rng.seed>>8)&1!=0{256i32}else{-256};let mut heading=i32::from(word(0x4b));let mut point=[0;2];let mut queried_points=Vec::new();let mut accepted=false;
 for _ in 0..8{heading=(heading+2048+step)&2047;let index=((heading+1536)&2047)as usize;
  for (axis,table)in [trig.cosine(),trig.sine()].into_iter().enumerate(){let delta=table[index].wrapping_mul(512)>>16;let doubled=word(7+axis*2).wrapping_mul(2).wrapping_add(delta as i16);point[axis]=(doubled>>1)&0x7fff;}
  queried_points.push(point);accepted=accepts(point).map_err(CellEntryError::Query)?;if accepted{break;}
 }
 let mut a=*before;a[0x45..0x47].copy_from_slice(&point[0].to_le_bytes());a[0x47..0x49].copy_from_slice(&point[1].to_le_bytes());
 let goal_cell=(((word(13) as u16>>8)&127)*128)+((word(11) as u16>>8)&127);
 if goal_cell==new_cell{a[0x49]|=1;a[0x54]=80;a[0x4a]&=0xef;}else{a[0x49]|=1;a[0x4a]|=0x10;}
 Ok(CellEntryPlan{record_after:a,rng_after:rng,queried_points,accepted})
}

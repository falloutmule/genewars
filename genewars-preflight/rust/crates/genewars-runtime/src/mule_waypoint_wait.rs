//! Source6FC5C..6FD67 waiting prefix; caller restores saved3F after steering/physics.
use crate::physics_tables::PhysicsTables;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum WaitError<E>{Record,Association,Goal(E)}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct WaitPlan{pub record_after:[u8;124],pub saved_target_speed:i16}
pub fn plan_wait<E>(before:&[u8;124],frame:u32,physics:&PhysicsTables,mut goal_predicate:impl FnMut([i16;2])->Result<bool,E>)->Result<WaitPlan,WaitError<E>>{
 if before[6]!=1||before[0x49]&3!=3{return Err(WaitError::Record);}
 if before[0x52]!=0{return Err(WaitError::Association);}
 let word=|at|i16::from_le_bytes([before[at],before[at+1]]);
 let phase=frame.wrapping_add(i32::from(word(15))as u32);let goal=[word(11),word(13)];let mut a=*before;
 let mut flags=u16::from_le_bytes([a[0x49],a[0x4a]]);
 if phase&3==3&&!goal_predicate(goal).map_err(WaitError::Goal)?{a[0x54]=20;flags&=0xeffc;a[0x16]&=0x7f;}
 else{a[0x54]=a[0x54].wrapping_sub(1);if a[0x54]==0{a[0x54]=20;flags=(flags&0xeebc)|0x140;a[0x16]&=0x7f;}}
 a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
 if phase&1!=0{let dx=goal[0].wrapping_sub(word(7)).wrapping_mul(2);let dy=goal[1].wrapping_sub(word(9)).wrapping_mul(2);let heading=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles());a[0x4b..0x4d].copy_from_slice(&heading.to_le_bytes());}
 let saved_target_speed=word(0x3f);a[0x3f..0x41].fill(0);
 Ok(WaitPlan{record_after:a,saved_target_speed})
}

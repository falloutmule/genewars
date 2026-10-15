//!69D9C..69DC8 pair stop, including both virtual84=1EE50 calls.
//! Caller proves original physical-distance/action branch; this pure planner
//! does not choose a peer or advance lifecycle. Both timers are32 before tail.
#[derive(Debug,Clone,PartialEq,Eq)]pub enum PairStopError{Record,Identity}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct PairStopPlan{pub current_after:[u8;124],pub peer_after:[u8;124]}
pub fn plan_stop(current:&[u8;124],peer:&[u8;124])->Result<PairStopPlan,PairStopError>{
 let word=|r:&[u8;124],at|i16::from_le_bytes([r[at],r[at+1]]);
 if !(0..300).contains(&word(current,15))||!(0..300).contains(&word(peer,15))||word(current,15)==word(peer,15){return Err(PairStopError::Identity);}
 for (r,other)in[(current,peer),(peer,current)]{
  if r[6]!=1||r[24]!=4||r[0x7a]!=4||r[0x5e]!=3||(word(r,0x60)as u16)&0x2004!=0x2004||r[0x57]!=1||word(r,0x55)!=word(other,15){return Err(PairStopError::Record);}
 }
 let stop=|before:&[u8;124]|{let mut a=*before;a[0x3f..0x41].fill(0);let flags=(word(before,0x49)as u16)&0xefe8;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x54]=0;a[0x16]&=0x3f;a[0x5e]=10;a[0x6c..0x6e].copy_from_slice(&32i16.to_le_bytes());a};
 Ok(PairStopPlan{current_after:stop(current),peer_after:stop(peer)})
}

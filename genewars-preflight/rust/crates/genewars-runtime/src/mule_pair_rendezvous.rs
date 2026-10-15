//!69A17..69B8D atomic ground-Mule rendezvous; caller owns source branch selection.
//! Saved cell is the post-physics AI cell (5EF1F / EBP+EA). This bounded planner
//! requires both current and peer positions in it; no RNG, plants or lifecycle tail.
#[derive(Debug,Clone,PartialEq,Eq)]pub enum RendezvousError{Record,Identity,Cell}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct RendezvousPlan{pub current_after:[u8;124],pub peer_after:[u8;124]}
pub fn plan_rendezvous(current:&[u8;124],peer:&[u8;124],saved_cell:u16)->Result<RendezvousPlan,RendezvousError>{
 let word=|r:&[u8;124],at|i16::from_le_bytes([r[at],r[at+1]]);
 if !(0..300).contains(&word(current,15))||!(0..300).contains(&word(peer,15))||word(current,15)==word(peer,15){return Err(RendezvousError::Identity);}
 for (r,other) in [(current,peer),(peer,current)]{
  if r[6]!=1||r[24]!=4||r[0x7a]!=4||r[0x5e]!=3||r[0x61]&0x20!=0||r[0x57]!=1||word(r,0x55)!=word(other,15){return Err(RendezvousError::Record);}
  if saved_cell>=16384||word(r,7)<0||word(r,9)<0||((word(r,9)as u16>>8)*128+(word(r,7)as u16>>8))!=saved_cell{return Err(RendezvousError::Cell);}
 }
 if ((word(peer,9)as u16>>8)*128+(word(peer,7)as u16>>8))!=saved_cell{return Err(RendezvousError::Cell);}
 let x=word(current,7)&!255;let y=word(current,9)&!255;
 let finish=|before:&[u8;124],goal:[i16;2]|{let mut a=*before;a[0x61]|=0x20;a[11..13].copy_from_slice(&goal[0].to_le_bytes());a[13..15].copy_from_slice(&goal[1].to_le_bytes());
  let flags=(u16::from_le_bytes([a[0x49],a[0x4a]])|0x14)&0xeffc;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x54]=20;a[0x16]&=0x3f;a[0x4f]=0;a[0x52]=0;
  let speed=crate::creature_world::mule_requested_speed(before)>>1;a[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());a[0x6c..0x6e].copy_from_slice(&0x7fffi16.to_le_bytes());a[0x60]|=0x80;a};
 Ok(RendezvousPlan{current_after:finish(current,[x+64,y+192]),peer_after:finish(peer,[x+192,y+64])})
}

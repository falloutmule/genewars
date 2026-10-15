//!693AE..694D3 pair initialization only: caller proves eligibility and source-selected peer.
//! No RNG is consumed here. Caller validates/releases returned plant reservations
//! (plant2E bit0) and commits BOTH records atomically. Neutral no60bit2 makes
//!5C920 selection cleanup a no-op. Lifecycle/timer tail is deliberately separate.
#[derive(Debug,Clone,PartialEq,Eq)]pub enum PairInitError{Record,Identity,Plant}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct PairInitPlan{pub current_after:[u8;124],pub peer_after:[u8;124],pub release_plant_reservations:Vec<u16>}
pub fn plan_pair_init(current:&[u8;124],peer:&[u8;124])->Result<PairInitPlan,PairInitError>{
 let id=|r:&[u8;124]|i16::from_le_bytes([r[15],r[16]]);
 if !(0..300).contains(&id(current))||!(0..300).contains(&id(peer))||id(current)==id(peer){return Err(PairInitError::Identity);}
 if current[0x5e]!=7||peer[0x5e]!=1{return Err(PairInitError::Record);}
 let mut releases=Vec::new();
 for r in [current,peer]{
  if r[6]!=1||r[24]!=4||r[0x7a]!=4||r[0x60]&2!=0||!matches!(r[0x57],0|2){return Err(PairInitError::Record);}
  if r[0x57]==2{let plant=i16::from_le_bytes([r[0x55],r[0x56]]);if !(0..512).contains(&plant){return Err(PairInitError::Plant);}releases.push(plant as u16);}
 }
 let speed=crate::creature_world::mule_requested_speed(current);
 let finish=|before:&[u8;124],other:&[u8;124]|{
  let mut a=*before;a[0x15]&=0xdf;a[0x33..0x37].fill(0);a[0x5d]=0;
  let flags=u16::from_le_bytes([a[0x60],a[0x61]])&0xd71a;a[0x60..0x62].copy_from_slice(&flags.to_le_bytes());
  a[0x5e]=3;a[0x55..0x57].copy_from_slice(&other[15..17]);a[0x57]=1;
  let route=(u16::from_le_bytes([a[0x49],a[0x4a]])|0x14)&0xeffc;a[0x49..0x4b].copy_from_slice(&route.to_le_bytes());a[0x54]=20;a[0x16]&=0x3f;
  a[0x4f]=0;a[0x50..0x52].copy_from_slice(&other[15..17]);a[0x52]=1;
  a[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());a[0x6c..0x6e].copy_from_slice(&0x7fffi16.to_le_bytes());a[0x60]|=0x80;a
 };
 Ok(PairInitPlan{current_after:finish(current,peer),peer_after:finish(peer,current),release_plant_reservations:releases})
}

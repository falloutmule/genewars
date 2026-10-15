//!693AE..694D3 pair initialization only: caller proves eligibility and source-selected peer.
//! Source-proved feeding peer7 retains its live neutral plant reservation.
//! Peer state8 may retain an old kind1 target:5CCA0 clears it locally only.
//! Initiator eligibility/stage and peer scoring remain caller responsibilities.
//! No RNG is consumed here. Caller validates/releases returned plant reservations
//! (plant2E bit0) and commits BOTH records atomically. Neutral no60bit2 makes
//!5C920 selection cleanup a no-op. Lifecycle/timer tail is deliberately separate.
#[derive(Debug,Clone,PartialEq,Eq)]pub enum PairInitError{Record,Identity,Plant}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct PairInitPlan{pub current_after:[u8;124],pub peer_after:[u8;124],pub release_plant_reservations:Vec<u16>}
pub fn plan_pair_init(current:&[u8;124],peer:&[u8;124])->Result<PairInitPlan,PairInitError>{
 let id=|r:&[u8;124]|i16::from_le_bytes([r[15],r[16]]);
 if !(0..300).contains(&id(current))||!(0..300).contains(&id(peer))||id(current)==id(peer){return Err(PairInitError::Identity);}
 if current[0x5e]!=7||!matches!(peer[0x5e],1|7|8){return Err(PairInitError::Record);}
 // New peer7 domain is the authentic1279 feeding candidate. Peer stage79
 // is deliberately unrestricted: source selected a negative-stage feeder.
 if peer[0x5e]==7 && (peer[0x57]!=2||peer[0x60]!=8||peer[0x61]!=0x40||peer[0x77]!=255
    ||u16::from_le_bytes([peer[0x49],peer[0x4a]])&0x400!=0
    ||i16::from_le_bytes([peer[0x64],peer[0x65]])<=0){return Err(PairInitError::Record);}
 let mut releases=Vec::new();
 for (index,r) in [current,peer].into_iter().enumerate(){
  if r[6]!=1||r[24]!=4||r[0x7a]!=4||r[0x60]&2!=0||!(matches!(r[0x57],0|2)||(index==1&&r[0x5e]==8&&r[0x57]==1)){return Err(PairInitError::Record);}
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

#[cfg(test)]mod tests{
 use super::*;
 fn feeding_pair()->([u8;124],[u8;124]){
  let mut current=[0;124];let mut peer=[0;124];
  for(slot,r)in [&mut current,&mut peer].into_iter().enumerate(){r[6]=1;r[24]=4;r[122]=4;r[94]=7;r[96]=8;r[97]=0x40;r[119]=255;r[100]=67;r[15..17].copy_from_slice(&(slot as i16).to_le_bytes());}
  current[87]=0;peer[85..87].copy_from_slice(&4i16.to_le_bytes());peer[87]=2;peer[121]=253;
  (current,peer)
 }
 #[test]fn retired_food_caller_pairs_with_feeding_peer_and_releases_its_plant(){
  let(a,b)=feeding_pair();let p=plan_pair_init(&a,&b).unwrap();assert_eq!(p.release_plant_reservations,vec![4]);
  assert_eq!(p.current_after[94],3);assert_eq!(p.peer_after[94],3);assert_eq!(&p.current_after[85..88],&[1,0,1]);assert_eq!(&p.peer_after[85..88],&[0,0,1]);assert_eq!(p.peer_after[121],253);
 }
 #[test]fn feeding_peer_rejects_unproved_action_and_preserves_inputs(){
  let(a,b)=feeding_pair();for(off,value)in [(87,0),(96,10),(97,0),(119,254),(100,0),(74,4)]{let mut bad=b;bad[off]=value;assert_eq!(plan_pair_init(&a,&bad),Err(PairInitError::Record));assert_eq!(a,feeding_pair().0);}
  let mut bad=b;bad[85..87].copy_from_slice(&512i16.to_le_bytes());assert_eq!(plan_pair_init(&a,&bad),Err(PairInitError::Plant));
 }
}

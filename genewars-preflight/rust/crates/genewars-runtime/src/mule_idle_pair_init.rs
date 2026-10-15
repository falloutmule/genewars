//! Neutral idle mating at62F23..63045, distinct from feeding693AE.
//! Caller proves the radius1024 source-selected mate and empty enemy action.
//! Source5D700 returns true immediately for neutralowner4; no specialist is
//! required. Type4 Mule6BDD0 is false, so6C190 is not invoked. No RNG/audio.
#[derive(Debug,Clone,PartialEq,Eq)]pub enum IdlePairError{Identity,CurrentEligibility,PeerEligibility,UnsupportedCleanup}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct IdlePairPlan{pub current_after:[u8;124],pub peer_after:[u8;124]}
/// One admitted idle8/8 pair. Stale kind1 target cleanup is LOCAL in state8;
/// neither old target nor an external plant/association is modified here.
/// Both requested speeds are derived from current, exactly6300B..63010.
pub fn plan_pair_init(current:&[u8;124],peer:&[u8;124])->Result<IdlePairPlan,IdlePairError>{
 let word=|r:&[u8;124],o|i16::from_le_bytes([r[o],r[o+1]]);
 let id=|r:&[u8;124]|word(r,15);
 if !(0..300).contains(&id(current))||!(0..300).contains(&id(peer))||id(current)==id(peer){return Err(IdlePairError::Identity);}
 if current[6]!=1||current[24]!=4||current[122]!=4||current[94]!=8||current[119]!=255||(current[121]as i8)<=0||word(current,104)<=word(current,98)>>1{return Err(IdlePairError::CurrentEligibility);}
 if peer[6]!=1||peer[24]!=4||peer[122]!=4||peer[94]!=8||peer[119]!=255||word(peer,73)as u16&0x400!=0{return Err(IdlePairError::PeerEligibility);}
 for r in [current,peer]{
  if r[96]&2!=0||r[82]!=0||!matches!(r[87],0|1)||r[87]==1&&!(0..300).contains(&word(r,85)){return Err(IdlePairError::UnsupportedCleanup);}
 }
 let speed=crate::creature_world::mule_requested_speed(current);
 let finish=|before:&[u8;124],other:&[u8;124]|{
  let mut a=*before;a[21]&=0xdf;a[51..55].fill(0);a[93]=0;
  a[96..98].copy_from_slice(&(word(before,96)as u16&0xd71a).to_le_bytes());
  a[94]=3;a[85..87].copy_from_slice(&other[15..17]);a[87]=1;
  a[73..75].copy_from_slice(&((word(before,73)as u16|0x14)&0xeffc).to_le_bytes());a[84]=20;a[22]&=0x3f;
  a[79]=0;a[80..82].copy_from_slice(&other[15..17]);a[82]=1;
  a[63..65].copy_from_slice(&speed.to_le_bytes());a[108..110].copy_from_slice(&32767i16.to_le_bytes());a[96]|=0x80;a
 };
 Ok(IdlePairPlan{current_after:finish(current,peer),peer_after:finish(peer,current)})
}
#[cfg(test)]mod tests{
 use super::*;
 fn pair()->([u8;124],[u8;124]){let mut a=[0;124];let mut b=[0;124];for(slot,r)in [&mut a,&mut b].into_iter().enumerate(){r[6]=1;r[24]=4;r[122]=4;r[94]=8;r[119]=255;r[15..17].copy_from_slice(&(slot as i16).to_le_bytes());r[98..100].copy_from_slice(&143i16.to_le_bytes());r[96]=0x88;r[123]=255;}a[121]=3;a[104..106].copy_from_slice(&90i16.to_le_bytes());b[121]=253;b[104..106].copy_from_slice(&16i16.to_le_bytes());a[118]=50;b[118]=100;(a,b)}
 #[test]fn idle_pair_uses_current_speed_and_retains_negative_peer_stage(){let(a,b)=pair();let p=plan_pair_init(&a,&b).unwrap();assert_eq!(&p.current_after[63..65],&p.peer_after[63..65]);assert_eq!(p.current_after[94],3);assert_eq!(p.peer_after[94],3);assert_eq!(&p.current_after[85..88],&[1,0,1]);assert_eq!(&p.peer_after[85..88],&[0,0,1]);assert_eq!(p.peer_after[121],253);assert_eq!(&p.peer_after[104..106],&b[104..106]);}
 #[test]fn ineligible_or_live_external_tasks_do_not_form_idle_pair(){let(a,b)=pair();for(off,v)in [(121,253),(119,254),(94,7)]{let mut bad=a;bad[off]=v;assert!(plan_pair_init(&bad,&b).is_err());}for(off,v)in [(87,2),(82,1),(96,0x8a),(74,4)]{let mut bad=b;bad[off]=v;assert!(plan_pair_init(&a,&bad).is_err());}}
}

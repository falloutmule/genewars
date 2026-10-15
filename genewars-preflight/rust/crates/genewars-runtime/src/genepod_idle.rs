//! Bounded full451F0 completed grade0 Genepod idle callback.
//! Exact BGENE animation1; production, disabled and destruction branches reject.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum GenepodIdleError { Building, Player, ProductionRequired }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct GenepodIdlePlan { pub building_after:[u8;67],pub player_after:[u8;1805],pub queue_requested:bool }
pub fn plan_completed_idle(before:&[u8;67],player:&[u8;1805])->Result<GenepodIdlePlan,GenepodIdleError>{
 let flags=u16::from_le_bytes(before[49..51].try_into().unwrap());
 if before[6]!=4||before[24]>=4||before[45]!=0||before[46]!=0||flags&!4!=1{return Err(GenepodIdleError::Building);}
 if player[1]!=before[24]||player[0x4a1]!=before[24]{return Err(GenepodIdleError::Player);}
 if i32::from_le_bytes(before[54..58].try_into().unwrap())>0{return Err(GenepodIdleError::ProductionRequired);}
 let(a,p)=shared_prefix(before,player);let a=shared_tail(before,a);
 Ok(GenepodIdlePlan{building_after:a,player_after:p,queue_requested:true})
}
///Shared grade0 power prefix, before selector work/payment.
pub(crate) fn shared_prefix(before:&[u8;67],player:&[u8;1805])->([u8;67],[u8;1805]){
 let mut a=*before;let mut p=*player;
 let mut timer=i16::from_le_bytes(before[61..63].try_into().unwrap()).wrapping_sub(i16::from(before[63]));
 if timer<=0{timer=0;a[49]&=!4;if i16::from_le_bytes(p[0x5ea..0x5ec].try_into().unwrap())==0{p[0x5ea..0x5ec].copy_from_slice(&1000i16.to_le_bytes());}}
 a[61..63].copy_from_slice(&timer.to_le_bytes());(a,p)
}
///Shared BGENE animation1/power/dirty tail; before is the callback beforeimage.
pub(crate) fn shared_tail(before:&[u8;67],mut a:[u8;67])->[u8;67]{
 a[28]=u8::from(a[27]!=0);let timer=i16::from_le_bytes(a[61..63].try_into().unwrap()).min(200);a[61..63].copy_from_slice(&timer.to_le_bytes());if a[63]!=0{a[63]-=1;}
 if before[25]!=a[25]||a[28]!=0||(before[49]^a[49])&4!=0||a[48]!=0{a[21]|=4;}
 if a[48]!=0{a[48]-=1;}a[25]=1;
 a[27]=a[27].min(3);
 if (a[30] as i8)>(a[29] as i8){a[30]=a[29];}
 let delay=a[30] as i8;a[30]=a[30].wrapping_sub(1);
 if delay<=0{a[30]=a[29];let holds=[0,0,3,0];a[26]=a[26].min(holds[a[27] as usize]);let old=a[26];a[26]=old.wrapping_sub(1);if old==0{let next=usize::from(a[27])+usize::from(a[28]);a[27]=if next>=4{0}else{next as u8};a[26]=holds[a[27] as usize];}}
 a
}

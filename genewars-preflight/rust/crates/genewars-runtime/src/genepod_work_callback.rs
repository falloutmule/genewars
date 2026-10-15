//!Full nonterminal completed grade0 BGENE callback; terminal remains atomic rejection.
use crate::{genepod_idle,genepod_positive_work::{self,WorkError,WorkStatus}};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum GenepodWorkError{Building,Player,Work(WorkError)}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct GenepodWorkPlan{pub building_after:[u8;67],pub players_after:[[u8;1805];4],pub status:WorkStatus,pub queue_requested:bool}
pub fn plan_completed_work(before:&[u8;67],players:&[[u8;1805];4],frame:u32)->Result<GenepodWorkPlan,GenepodWorkError>{
 let flags=u16::from_le_bytes([before[49],before[50]]);
 if before[6]!=4||before[24]>=4||before[45]!=0||before[46]!=0||flags&!4!=1||before[51]!=4||i32::from_le_bytes(before[54..58].try_into().unwrap())<=0{return Err(GenepodWorkError::Building);}
 let owner=before[24]as usize;if players[owner][1]!=before[24]||players[owner][0x4a1]>=4{return Err(GenepodWorkError::Player);}
 let(a,player)=genepod_idle::shared_prefix(before,&players[owner]);let mut p=*players;p[owner]=player;
 let work=genepod_positive_work::plan_work(&a,&p,frame).map_err(GenepodWorkError::Work)?;
 Ok(GenepodWorkPlan{building_after:genepod_idle::shared_tail(before,work.building_after),players_after:work.players_after,status:work.status,queue_requested:true})
}

//! Source53A33 funded debit and actor action14 setup. Cancellation remains required on insufficiency.
use genewars_core::cd_candidate::{construction_stage3_debit,ConstructionSelector,ConstructionPools,ConstructionDebitStatus};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionPaymentError{UnsupportedActor,InvalidOwner,InvalidYaw,MovementRequired,InsufficientCancellationRequired}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionPaymentPlan{pub actor_before:[u8;200],pub actor_after:[u8;200],pub player_before:[u8;1805],pub player_after:[u8;1805],pub cost:i32}
/// `refill` is the source flag at the resource block's selected owner+2.
/// `yaw` is the actual73970(actor position, task position) result, not actor facing.
pub fn plan_funded_construction_payment(actor:&[u8;200],blueprint:&[u8;36],player:&[u8;1805],refill:bool,yaw:u16)
    ->Result<ConstructionPaymentPlan,ConstructionPaymentError>{
    if actor[6]!=3||blueprint[0x16]!=0||actor[0x55]!=9||actor[0x7e]!=0||actor[0x82]!=1
        ||i16::from_le_bytes(actor[0x88..0x8a].try_into().unwrap())!=3||actor[0x83]>=14{
        return Err(ConstructionPaymentError::UnsupportedActor);
    }
    if actor[0x18]>=4||player[1]!=actor[0x18]||player[0x4a1]>=4{return Err(ConstructionPaymentError::InvalidOwner);}
    if actor[0x49]&32==0{return Err(ConstructionPaymentError::MovementRequired);}
    if yaw>=2048{return Err(ConstructionPaymentError::InvalidYaw);}
    let pools=ConstructionPools{primary:i32::from_le_bytes(player[0x4a3..0x4a7].try_into().unwrap()),secondary:i32::from_le_bytes(player[0x4a7..0x4ab].try_into().unwrap())};
    let debit=construction_stage3_debit(ConstructionSelector::try_from(actor[0x83]as i8).unwrap(),blueprint[0x17]as i8,pools,refill,true);
    if debit.status!=ConstructionDebitStatus::Paid{return Err(ConstructionPaymentError::InsufficientCancellationRequired);}
    let mut actor_after=*actor;let mut player_after=*player;
    player_after[0x4a3..0x4a7].copy_from_slice(&debit.pools.primary.to_le_bytes());player_after[0x4a7..0x4ab].copy_from_slice(&debit.pools.secondary.to_le_bytes());
    actor_after[0x88..0x8a].copy_from_slice(&4i16.to_le_bytes());actor_after[0x70]&=!16;
    actor_after[0x70]|=if !(0x300..0x700).contains(&yaw){0x84}else{0x80};
    actor_after[0x59]=14;actor_after[0x1d]=0;actor_after[0x1b]=0;actor_after[0x1c]=1;
    Ok(ConstructionPaymentPlan{actor_before:*actor,actor_after,player_before:*player,player_after,cost:debit.cost.unwrap()})
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn fresh_zero_funds_reject_and_explicit_funding_pays_exact_cost(){
  let mut a=[0;200];a[6]=3;a[0x55]=9;a[0x82]=1;a[0x83]=1;a[0x49]=32;a[0x88]=3;let mut bp=[0;36];bp[0x17]=31;let mut player=[0;1805];
  assert_eq!(plan_funded_construction_payment(&a,&bp,&player,false,0),Err(ConstructionPaymentError::InsufficientCancellationRequired));
  player[0x4a3..0x4a7].copy_from_slice(&1000i32.to_le_bytes());let q=plan_funded_construction_payment(&a,&bp,&player,false,0).unwrap();assert_eq!(q.cost,688);assert_eq!(i32::from_le_bytes(q.player_after[0x4a3..0x4a7].try_into().unwrap()),312);assert_eq!(q.actor_after[0x88],4);assert_eq!(q.actor_after[0x59],14);
 }
}

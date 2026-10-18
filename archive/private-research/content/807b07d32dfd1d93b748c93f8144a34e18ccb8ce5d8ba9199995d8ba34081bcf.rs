//! Source526C0 with52340 case1 for an architect's pending task destination.
use crate::actor_motion;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionMotionError{UnsupportedActor,UnsupportedDestination,InvalidAngleTable}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionTaskMovePlan{pub after:[u8;200],pub ready:bool}
pub fn plan_architect_task_movement(before:&[u8;200],blueprint:&[u8;36],angles:&[u16;257])
    ->Result<ConstructionTaskMovePlan,ConstructionMotionError>{
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||blueprint[0x16]!=0||blueprint[0x15]>=3{return Err(ConstructionMotionError::UnsupportedActor);}
    if angles.iter().any(|v|*v>256){return Err(ConstructionMotionError::InvalidAngleTable);}
    let mut after=*before;
    if after[0x70]&32!=0||after[0x49]&3!=0{return Ok(ConstructionTaskMovePlan{after,ready:false});}
    if after[0x49]&0x30==0{
        if after[0x5a]!=1{return Err(ConstructionMotionError::UnsupportedDestination);}
        after[11..15].copy_from_slice(&before[0x60..0x64]);
        let word=|p|i16::from_le_bytes(after[p..p+2].try_into().unwrap());
        let dx=word(11).wrapping_sub(word(7)).wrapping_mul(2);let dy=word(13).wrapping_sub(word(9)).wrapping_mul(2);
        let yaw=actor_motion::angle(i32::from(dx),i32::from(dy),angles);
        after[0x4b..0x4d].copy_from_slice(&yaw.to_le_bytes());after[0x39..0x3b].copy_from_slice(&yaw.to_le_bytes());
        after[0x54]=20;after[0x49]|=0x14;after[0x16]&=0x3f;
        let flags=u16::from_le_bytes(after[0x49..0x4b].try_into().unwrap())&0xeffc;
        after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());after[0x4f]=0;after[0x52]=0;
    }
    let ready=after[0x49]&0x60!=0;
    if ready{after[0x16]&=0x3f;after[0x54]=0;after[0x49]&=!16;after[0x70]&=!16;}
    Ok(ConstructionTaskMovePlan{after,ready})
}

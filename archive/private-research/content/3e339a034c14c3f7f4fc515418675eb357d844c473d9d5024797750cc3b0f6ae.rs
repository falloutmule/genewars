//! Source534D0 stage1/2 wait for an active terraform job.
//! Completed and abandoned jobs require their separate transition planners.
use crate::actor_construction_lifecycle::ConstructionAudioRequest;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum TerraformWaitError{UnsupportedActor,CompletedJobRequired,AbandonJobRequired}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TerraformWaitPlan{pub after:[u8;200],pub pending_audio:Option<ConstructionAudioRequest>}
///53645..536A4 active-job branch followed by53999 distance gate.
/// The caller supplies exact736F0 squared planar distance to the working point.
pub fn plan_architect_terraform_wait(before:&[u8;200],job:&[u8;176],distance_squared:u32)->Result<TerraformWaitPlan,TerraformWaitError>{
    let stage=u16::from_le_bytes(before[0x88..0x8a].try_into().unwrap());
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||!(1..=2).contains(&stage){return Err(TerraformWaitError::UnsupportedActor);}
    if i16::from_le_bytes(job[12..14].try_into().unwrap())<0{return Err(TerraformWaitError::CompletedJobRequired);}
    if distance_squared>=65536{return Err(TerraformWaitError::AbandonJobRequired);}
    Ok(TerraformWaitPlan{after:*before,pending_audio:(before[0x1b]==2&&before[0x1a]==0).then_some(ConstructionAudioRequest{channel:0x60,mode:2,parameter:0,stack_byte:127,stack_dword:100})})
}

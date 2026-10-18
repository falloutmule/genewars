//! Source534D0 mode0/stage0 terraform start. Other stages stay explicit.
use crate::construction_setup::reserve_terrain_site;
use crate::construction_terrain::{TerrainJob,TerrainJobAllocation,TerrainJobError,plan_allocate_terrain_job};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionLifecycleError{InvalidRecord,UnsupportedTask,MovementRequired,Terrain(TerrainJobError)}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ConstructionAudioRequest{pub channel:u8,pub mode:u8,pub parameter:u8,pub stack_byte:u8,pub stack_dword:i32}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TerraformStartPlan{pub before:[u8;200],pub after:[u8;200],pub map_after:Vec<u8>,pub job:Option<TerrainJobAllocation>,pub viewport_invalidations:Vec<u16>,pub pending_audio:Option<ConstructionAudioRequest>}
fn word(b:&[u8;200],p:usize)->i16{i16::from_le_bytes(b[p..p+2].try_into().unwrap())}
pub fn plan_architect_terraform_start(map:&[u8],before:&[u8;200],pool:&[TerrainJob;25],count:u8,table:&[u8;8192])->Result<TerraformStartPlan,ConstructionLifecycleError>{
 if before[6]!=3{return Err(ConstructionLifecycleError::InvalidRecord);}if before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||before[0x83]>=14||word(before,0x88)!=0{return Err(ConstructionLifecycleError::UnsupportedTask);}if before[0x49]&32==0{return Err(ConstructionLifecycleError::MovementRequired);}if map.len()!=229376{return Err(ConstructionLifecycleError::Terrain(TerrainJobError::MapLength));}
 let mut map_after=map.to_vec();let mut after=*before;after[0x59]=13;after[0x70]=(after[0x70]&!16)|128;
 let selector=usize::from(after[0x83]);let footprint=[3u8,1,2,3,2,2,1,2,1,2,3,3,2,1][selector];let position=[word(&after,11),word(&after,13)];let task_position=[word(&after,0x8c),word(&after,0x8e)];let mut viewport_invalidations=Vec::new();
 let(index,stage)=if selector==5&&position!=task_position{viewport_invalidations=reserve_terrain_site(&mut map_after,position,3,table).map_err(ConstructionLifecycleError::Terrain)?;(2,2)}else{(footprint-1,1)};
 let job=plan_allocate_terrain_job(pool,count,position,index,word(&after,0x8a),word(&after,0x84),&map_after).map_err(ConstructionLifecycleError::Terrain)?;
 if let Some(j)=&job{after[0x90..0x94].copy_from_slice(&u32::from(j.slot).to_le_bytes());after[0x88..0x8a].copy_from_slice(&(stage as u16).to_le_bytes());}
 let pending_audio=job.as_ref().map(|_|ConstructionAudioRequest{channel:0x29,mode:2,parameter:255,stack_byte:127,stack_dword:100});
 Ok(TerraformStartPlan{before:*before,after,map_after,job,viewport_invalidations,pending_audio})
}

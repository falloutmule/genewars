//! Bounded airborne6DF30 effect20 branch, preserving real obstacle queries.
use crate::landing::{self,LandingRecord,LandingError,LandingAirbornePlan};
fn word(a:&LandingRecord,o:usize)->i16{i16::from_le_bytes([a[o],a[o+1]])}
fn dword(a:&LandingRecord,o:usize)->i32{i32::from_le_bytes(a[o..o+4].try_into().unwrap())}
pub fn candidate(before:&LandingRecord)->Result<[i16;2],LandingError>{
 if before[6]!=5||before[69]!=32||before[73]!=0{return Err(LandingError::InvalidRecord);}
 let mut temporary=*before;temporary[69]=6;landing::effect6_physics_candidate(&temporary)
}
pub fn plan_airborne(before:&LandingRecord,ground:i32,shade:u8,water:i32,gravity:i16,
 obstacle_lookup_is_null:bool,angles:&[u16;257])->Result<LandingAirbornePlan,LandingError>{
 candidate(before)?;
 if before[21]&!1!=0||before[22]!=0||word(before,63)!=-1||before[51..55]!=[0;4]{return Err(LandingError::UnsupportedPhysicsState);}
 let acceleration=word(before,55).wrapping_sub(gravity);
 let next=dword(before,39).wrapping_add(i32::from(word(before,49).wrapping_add(acceleration))>>3);
 if dword(before,39)<=dword(before,17)||next<=ground{return Err(LandingError::UnsupportedPhysicsState);}
 // Water crossing invokes additional source factories and is not admitted here.
 if dword(before,39)<=water||next<=water{return Err(LandingError::EnvironmentalCallbackRequired);}
 let mut temporary=*before;temporary[69]=6;temporary[21]|=32;temporary[55..57].copy_from_slice(&acceleration.to_le_bytes());
 let mut plan=landing::plan_effect6_flight(&temporary,ground,shade,water,obstacle_lookup_is_null,[0;4],0,angles)?;
 plan.before=*before;plan.after[69]=32;plan.after[21]=before[21];plan.after[55..57].copy_from_slice(&before[55..57]);Ok(plan)
}

//! Bounded airborne6DF30 effect20 branch; caller proves collision no-op.
use crate::landing::{self,LandingRecord,LandingError,LandingAirbornePlan};
fn word(a:&LandingRecord,o:usize)->i16{i16::from_le_bytes([a[o],a[o+1]])}
fn dword(a:&LandingRecord,o:usize)->i32{i32::from_le_bytes(a[o..o+4].try_into().unwrap())}
pub fn candidate(before:&LandingRecord)->Result<[i16;2],LandingError>{
 if before[6]!=5||before[69]!=32||before[73]!=0{return Err(LandingError::InvalidRecord);}
 let mut temporary=*before;temporary[69]=6;landing::effect6_physics_candidate(&temporary)
}
pub fn plan_airborne(before:&LandingRecord,ground:i32,shade:u8,water:i32,gravity:i16,
 collision_has_no_numeric_effect:bool,angles:&[u16;257])->Result<LandingAirbornePlan,LandingError>{
 candidate(before)?;
 if before[21]&!1!=0||before[22]!=0||word(before,63)!=-1||before[51..55]!=[0;4]{return Err(LandingError::UnsupportedPhysicsState);}
 let acceleration=word(before,55).wrapping_sub(gravity);
 let next=dword(before,39).wrapping_add(i32::from(word(before,49).wrapping_add(acceleration))>>3);
 if dword(before,39)<=dword(before,17)||next<=ground{return Err(LandingError::UnsupportedPhysicsState);}
 // Water crossing invokes additional source factories and is not admitted here.
 if dword(before,39)<=water||next<=water{return Err(LandingError::EnvironmentalCallbackRequired);}
 let mut temporary=*before;temporary[69]=6;temporary[21]|=32;temporary[55..57].copy_from_slice(&acceleration.to_le_bytes());
 let mut plan=landing::plan_effect6_flight(&temporary,ground,shade,water,collision_has_no_numeric_effect,[0;4],0,angles)?;
 plan.before=*before;plan.after[69]=32;plan.after[21]=before[21];plan.after[55..57].copy_from_slice(&before[55..57]);Ok(plan)
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn collision_noop_does_not_admit_water_crossing(){
  let mut before=[0u8;81];before[6]=5;before[69]=0x20;
  before[17..21].copy_from_slice(&1000i32.to_le_bytes());before[39..43].copy_from_slice(&2100i32.to_le_bytes());
  before[49..51].copy_from_slice(&(-1024i16).to_le_bytes());before[63..65].copy_from_slice(&(-1i16).to_le_bytes());
  //Candidate1844 is airborne above ground1000 but crosses water2000.
  assert_eq!(plan_airborne(&before,1000,0,2000,1024,true,&[0;257]),Err(LandingError::EnvironmentalCallbackRequired));
 }
}

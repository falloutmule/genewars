//! Bounded source opcode5 architect assignment, before507D0 initialization.
//! Category0 mode0 encodes the building selector in the high argument byte.
pub type ConstructionActorRecord=[u8;200];
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ConstructionCommandError{Inactive,WrongCategory,UnsupportedMode,UnsupportedReinitialization,UnsupportedRunningState,InvalidSelector(u8)}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ArchitectAssignmentPlan{pub before:ConstructionActorRecord,pub after:ConstructionActorRecord,pub building_selector:u8,pub panel_refresh_pending:bool}
///1C120 opcode5 ->81080 ->51960, admitted idle category0/new task.
///Caller binds the blueprint and source selected-player/current-actor context.
pub fn plan_architect_assignment(before:&ConstructionActorRecord,blueprint:&[u8;36],packet_mode_word:u16,packet_argument:u16,owner_is_selected:bool,actor_is_selected:bool)->Result<ArchitectAssignmentPlan,ConstructionCommandError>{
 if before[6]!=3{return Err(ConstructionCommandError::Inactive);}
 if blueprint[0x16]!=0{return Err(ConstructionCommandError::WrongCategory);}
 if packet_mode_word>>8!=0{return Err(ConstructionCommandError::UnsupportedMode);}
 if before[0x82]!=0{return Err(ConstructionCommandError::UnsupportedReinitialization);}
 if before[0x55]!=1{return Err(ConstructionCommandError::UnsupportedRunningState);}
 let selector=(packet_argument>>8)as u8;if selector>=14{return Err(ConstructionCommandError::InvalidSelector(selector));}
 let mut after=*before;after[0x57]|=1;after[0x7e]=0;after[0x83]=selector;after[0x82]=1;
 Ok(ArchitectAssignmentPlan{before:*before,after,building_selector:selector,panel_refresh_pending:owner_is_selected&&actor_is_selected})
}

//! Source82590 and victory-view7DBC0/7DDD0 numeric transition plans.
//! Presentation services and72E50 world cleanup must be fulfilled explicitly;
//! these plans alone do not load a mission or establish campaign continuity.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ResultTransitionError{InvalidPlayer,NotCampaignVictory,AlternateMode,VideoLockState,EndingSequenceRequired}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ResultViewRequest{Victory1863DC,Campaign186F19}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct VictoryEntryPlan{pub players_before:[[u8;1805];4],pub players_after:[[u8;1805];4],pub flags_after:u8,pub mode_after:u8,pub frame_after:u32,pub saved_frame:u32,pub stage_after:u8,pub service_calls:Vec<u32>,pub exit_cleanup_72e50_required:bool}
/// The original Enter handler queues packet21; its82590 receiver calls this
/// transition only after the original terminal bit is set.
pub fn continue_after_win(flags:u8,mode:u8)->Result<Option<ResultViewRequest>,ResultTransitionError>{
 if flags&0x40==0{return Ok(None)}
 if flags&0x20==0{return Err(ResultTransitionError::NotCampaignVictory)}
 let _=mode;Ok(Some(ResultViewRequest::Victory1863DC))
}
/// Bounded7DBC0 entry with initially unlocked video surface and successful
/// native presentation service. Stage5 and increment happen on view ENTRY,
/// not on72ED0 victory. Calls remain ordered obligations before commit.
pub fn enter_victory(players:&[[u8;1805];4],selected:u8,flags:u8,mode:u8,frame:u32,video_lock_depth:i8)->Result<VictoryEntryPlan,ResultTransitionError>{
 if selected>=4{return Err(ResultTransitionError::InvalidPlayer)}
 if flags&0x60!=0x60{return Err(ResultTransitionError::NotCampaignVictory)}
 if mode&1!=0{return Err(ResultTransitionError::AlternateMode)}
 if video_lock_depth!=0{return Err(ResultTransitionError::VideoLockState)}
 let mut after=*players;after[selected as usize][0x6e7]=after[selected as usize][0x6e7].wrapping_add(1);
 let mut calls=Vec::new();if mode&8==0{calls.push(0x8e780)}
 calls.extend([0x154b0,0x154f0,0xa4480,0xa1340,0x2f0c0,0x1be50,0xa1610,0xaea78,0xa1610,0x1f490,0x715e0,0xa2fb8]);
 Ok(VictoryEntryPlan{players_before:*players,players_after:after,flags_after:flags&!4,mode_after:mode|2,frame_after:0,saved_frame:frame,stage_after:5,service_calls:calls,exit_cleanup_72e50_required:true})
}
/// Source7DDD0 tests runtime118779 bit1. Its input-producer binding must be
/// preserved by the host; this is not an automatic next-mission transition.
pub fn advance_victory(runtime_input_flags:u8,flags:u8,progress:u8)->Result<Option<ResultViewRequest>,ResultTransitionError>{
 if runtime_input_flags&1==0{return Ok(None)}
 if flags&0x20==0{return Err(ResultTransitionError::NotCampaignVictory)}
 if progress==24{return Err(ResultTransitionError::EndingSequenceRequired)}
 Ok(Some(ResultViewRequest::Campaign186F19))
}

/// Shared1BCE0..1BCFF after old-view exit and new-view entry.
/// Campaign186F19 module flags6=0 is independently constructor-bound.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ViewResumePlan{pub mode_after:u8,pub frame_after:u32,pub resumed:bool}
pub fn resume_after_view_entry(mode:u8,module_flags:u8,frame:u32,saved_frame:u32)->ViewResumePlan{
    let resumed=mode&2!=0&&module_flags&1==0;
    ViewResumePlan{mode_after:if resumed{mode&!2}else{mode},frame_after:if resumed{saved_frame}else{frame},resumed}
}
#[cfg(test)]mod resume_tests{
    use super::*;
    #[test]fn source_dispatch_resume_preserves_other_bits_and_paused_modules(){
        for (mode,module,expected_mode,expected_frame,resumed) in [(0,0,0,17,false),(2,0,0,4630,true),(3,0,1,4630,true),(2,1,2,17,false),(0x82,0,0x80,4630,true)]{
            assert_eq!(resume_after_view_entry(mode,module,17,4630),ViewResumePlan{mode_after:expected_mode,frame_after:expected_frame,resumed});
        }
    }
}

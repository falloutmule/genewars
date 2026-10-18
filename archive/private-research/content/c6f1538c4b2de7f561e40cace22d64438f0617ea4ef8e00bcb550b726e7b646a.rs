//! Ordinary mode0 loss routing:1BC8E,7DEA0 and7E060.
//! Presentation and72E50 cleanup are explicit host obligations.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LossViewRequest { Loss186400, Title185008 }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum LossTransitionError { LossNotLatched, AlternateMode, VideoLockState }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LossEntryPlan {
    pub flags_after:u8,
    pub mode_after:u8,
    pub frame_after:u32,
    pub saved_frame:u32,
    pub stage_after:u8,
    pub service_calls:Vec<u32>,
    pub exit_cleanup_72e50_required:bool,
}
/// The main loop automatically consumes the mode0 loss latch. No Return
/// packet or intermediate victory-style widget is required by this branch.
pub fn pending_loss(flags:u8,mode:u8)->Option<LossViewRequest> {
    (flags&0x80!=0 && mode&1==0).then_some(LossViewRequest::Loss186400)
}
/// Successful initially-unlocked7DEA0 presentation entry. Player/campaign
/// bytes are untouched; unlike victory, frame is not reset and progress
/// is not incremented. The loss latch clears only after presentation calls.
pub fn enter_loss(flags:u8,mode:u8,frame:u32,video_lock_depth:i8)->Result<LossEntryPlan,LossTransitionError> {
    if flags&0x80==0{return Err(LossTransitionError::LossNotLatched)}
    if mode&1!=0{return Err(LossTransitionError::AlternateMode)}
    if video_lock_depth!=0{return Err(LossTransitionError::VideoLockState)}
    let mut service_calls=Vec::new();if mode&8==0{service_calls.push(0x8e780)}
    service_calls.extend([0x154b0,0x154f0,0xa4480,0xa1340,0x2f0c0,0x1be50,0xa1610,0xaea78,0xa1610,0xa2fb8]);
    Ok(LossEntryPlan{flags_after:flags&!0x84,mode_after:mode|2,frame_after:frame,saved_frame:frame,stage_after:5,service_calls,exit_cleanup_72e50_required:true})
}
/// Source7E060 tests118779 bit1 and queues the title view, not a retry or
/// next mission. Input-production binding remains the host's responsibility.
pub fn advance_loss(runtime_input_flags:u8)->Option<LossViewRequest> {
    (runtime_input_flags&1!=0).then_some(LossViewRequest::Title185008)
}

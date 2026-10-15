//! Bounded81C60 ->5DF50 owned Mule move, with5CC60's no-reference cleanup.
//! The caller validates pool identity and supplies the actual virtual68 result.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct CommandContext {
    pub caller_owner:u8,
    pub virtual_dead:bool,
    pub caller_is_selected:bool,
    /// Exact5E053..5E09C selected-creature-panel predicate, evaluated by host.
    pub selected_panel_refresh:bool,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum CommandError { Identity, Ownership, UnsupportedCleanup }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum CommandOutcome { IgnoredIneligible, Installed }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CommandPlan {
    pub before:[u8;124],pub after:[u8;124],pub outcome:CommandOutcome,
    pub feedback_requested:bool,pub panel_refresh_requested:bool,
}
/// Target is the ordinary source terrain projection. No RNG is consumed.
/// Admits source-created idle/newborn state8 or repeated state0 commands only.
/// Ejection/attachment state2 and task/reference cleanup require separate ports.
pub fn plan_move(before:&[u8;124],target:[i16;2],context:CommandContext)->Result<CommandPlan,CommandError>{
    let slot=i16::from_le_bytes([before[15],before[16]]);
    if before[6]!=1||before[0x7a]!=4||!(0..300).contains(&slot)||before[24]>=4{return Err(CommandError::Identity);}
    if context.caller_owner!=before[24]{return Err(CommandError::Ownership);}
    let mut result=CommandPlan{before:*before,after:*before,outcome:CommandOutcome::IgnoredIneligible,feedback_requested:false,panel_refresh_requested:false};
    if matches!(before[0x5e],0x12|0x0f)||context.virtual_dead||before[0x61]&1!=0{return Ok(result);}
    if !matches!(before[0x5e],0|8)||before[0x60]&2!=0||before[0x57]!=0{return Err(CommandError::UnsupportedCleanup);}
    let a=&mut result.after;
    //5C920 has no effects with attachment flag2 clear.5CC60 idle has no peer.
    a[0x33..0x37].fill(0);a[0x15]&=!0x20;
    let flags=u16::from_le_bytes([a[0x60],a[0x61]])&0xd71a;
    a[0x60..0x62].copy_from_slice(&flags.to_le_bytes());a[0x5d]=0;a[0x57]=0;a[0x6c..0x6e].fill(0);
    a[0x5e]=0;
    for offset in [0x6e,0x72]{a[offset..offset+2].copy_from_slice(&target[0].to_le_bytes());a[offset+2..offset+4].copy_from_slice(&target[1].to_le_bytes());}
    a[0x60]|=0x18;a[0x61]=(a[0x61]|0x80)&!0x40;a[0x78]=7;a[0x5f]=0;
    result.outcome=CommandOutcome::Installed;result.feedback_requested=context.caller_is_selected;result.panel_refresh_requested=context.selected_panel_refresh;
    Ok(result)
}

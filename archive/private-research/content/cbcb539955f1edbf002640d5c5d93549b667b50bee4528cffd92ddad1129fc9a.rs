//! Code15 rising water effect and code16 child, with explicit source physics.
//! Full-pool replacement and world links are owned by the transactional adapter.
use crate::landing::{self, LandingError, LandingEvent, LandingFactoryPlan, LandingRecord, ResolvedCellHead};

fn word(a:&LandingRecord,o:usize)->i16 { i16::from_le_bytes(a[o..o+2].try_into().unwrap()) }
fn dword(a:&LandingRecord,o:usize)->i32 { i32::from_le_bytes(a[o..o+4].try_into().unwrap()) }
fn put_word(a:&mut LandingRecord,o:usize,v:i16) { a[o..o+2].copy_from_slice(&v.to_le_bytes()); }

pub fn plan_factory(code:u8,pool:&[LandingRecord;200],count:i16,owner:u8,
    at:[i16;2],ground:i32,shade:u8,height:i32,angles:[u16;3],head:Option<ResolvedCellHead>,
    sine:&[i32;2048],cosine:&[i32;2048])->Result<LandingFactoryPlan,LandingError> {
    if !matches!(code,0x15|0x16) { return Err(LandingError::InvalidRecord); }
    if angles[0]>2048||angles[1]>2048||angles[2]>32767 { return Err(LandingError::UnsupportedPhysicsState); }
    let mut plan=landing::plan_landing_factory(pool,count,owner,0,at,ground,shade,head)?;
    let a=&mut plan.after_nonpointer;
    a[39..43].copy_from_slice(&height.to_le_bytes());
    a[69]=code;a[73]=plan.before[73];a[76]=plan.before[76];
    for (axis,angle) in angles.into_iter().enumerate() { put_word(a,57+2*axis,angle as i16); }
    let shift=|angle:u16|if angle<512 {usize::from(angle)+1536}else{usize::from(angle)-512};
    let horizontal=((i64::from(cosine[shift(angles[1])])*i64::from(angles[2]))>>16)as i32;
    put_word(a,45,((i64::from(cosine[shift(angles[0])])*i64::from(horizontal))>>16)as i16);
    put_word(a,47,((i64::from(sine[shift(angles[0])])*i64::from(horizontal))>>16)as i16);
    put_word(a,49,((-i64::from(sine[shift(angles[1])])*i64::from(angles[2]))>>16)as i16);
    plan.animation_index=u16::from(code);
    plan.events.retain(|e|matches!(e,LandingEvent::SetCellHead{..}|LandingEvent::SetOldHeadPrevious{..}));
    Ok(plan)
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct WaterEffectUpdatePlan {
    pub after:LandingRecord,
    pub retirement_required:bool,
    /// Source4B3B0 must create code16 before retiring code15; creation may fail.
    pub create_child:bool,
}

/// Full animation/lifetime prefix and selector15 tail after source6DF30.
/// Raw water is the signed word11A9C3, unlike the doubled physics water level.
pub fn plan_animation_after_physics(before:&LandingRecord,raw_water_height:i16)
    ->Result<WaterEffectUpdatePlan,LandingError> {
    if before[6]!=5||!matches!(before[69],0x15|0x16) { return Err(LandingError::InvalidRecord); }
    let frames=if before[69]==0x15 {2}else{3};
    if before[28]!=1||before[29]!=0 { return Err(LandingError::UnsupportedAnimationState); }
    let mut after=*before;
    let timer=word(&after,74);if timer>=0 {put_word(&mut after,74,timer.wrapping_sub(1));}
    // Both bound animations have zero holds and delay0: signed counter
    // clamping/expiry advances every call, including negative prior counters.
    after[26]=0;after[30]=0;after[27]=(after[27].min(frames-1)+1)%frames;
    let create_child=before[69]==0x15&&i32::from(raw_water_height)<=dword(&after,39);
    let retirement_required=if before[69]==0x15 {
        create_child||(word(&after,61)<=64&&dword(&after,39)<=dword(&after,17))
    }else{word(&after,74)<0&&after[27]==frames-1};
    Ok(WaterEffectUpdatePlan{after,retirement_required,create_child})
}

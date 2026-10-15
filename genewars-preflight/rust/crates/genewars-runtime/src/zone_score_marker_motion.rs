//! Source kind5 code3C: common6DF30 physics, EFF60 animation and49D30 cleanup.
//! Descriptor[0,0,11,FF,0,0]; six zero-hold frames; no selector damping/RNG.
//! Registry movement, actual resource binding and render enqueue are host work.
use crate::landing::{self, LandingAirbornePlan, LandingError, LandingRecord};
use crate::landing_effect_lifecycle::{self as lifecycle, CleanupRecord, EffectRetirementPlan, LifecycleError, RetirementUiState};
fn as_common(before: &LandingRecord) -> Result<LandingRecord, LandingError> {
    if before[6] != 5 || before[69] != 0x3c { return Err(LandingError::InvalidRecord); }
    let mut temporary = *before;
    temporary[69] = 6;
    Ok(temporary)
}
fn restore(before: &LandingRecord, mut plan: LandingAirbornePlan) -> LandingAirbornePlan {
    plan.before = *before;
    plan.after[69] = 0x3c;
    plan
}
pub fn physics_candidate(before: &LandingRecord) -> Result<[i16; 2], LandingError> {
    landing::effect6_physics_candidate(&as_common(before)?)
}
#[allow(clippy::too_many_arguments)]
// Source4B090 virtual70 returns false for3C, so6DF30 skips obstacles.
pub fn plan_flight(before: &LandingRecord, ground: i32, shade: u8, water_height: i32,
    terrain_height_words: [i16; 4], bounce_coefficient: i16,
    angle_table: &[u16; 257]) -> Result<LandingAirbornePlan, LandingError> {
    Ok(restore(before, landing::plan_effect6_flight(&as_common(before)?, ground, shade,
        water_height, true, terrain_height_words, bounce_coefficient, angle_table)?))
}
pub fn grounded_candidate(before: &LandingRecord, ground: i32, angles: &[u16; 257],
    sine: &[i32; 2048], cosine: &[i32; 2048]) -> Result<LandingRecord, LandingError> {
    let mut candidate = landing::effect6_grounded_candidate(&as_common(before)?, ground, angles, sine, cosine)?;
    candidate[69] = 0x3c;
    Ok(candidate)
}
pub fn plan_grounded_after_second_sample(before: &LandingRecord, candidate: &LandingRecord,
    ground: i32, shade: u8, water_height: i32)
    -> Result<LandingAirbornePlan, LandingError> {
    Ok(restore(before, landing::plan_effect6_grounded_after_second_sample(&as_common(before)?,
        &as_common(candidate)?, ground, shade, water_height, true)?))
}
/// Requires actual source6DF30 output. EFF60 andEFF47 share all six zero-hold
/// frames/flags0; class0 retirement and the selector return are identical.
pub fn plan_animation_after_physics(before: &LandingRecord) -> Result<landing::Effect6UpdatePlan, LandingError> {
    as_common(before)?;
    if before[26] != 0 || before[27] > 5 || before[28] != 1 || !(1..=3).contains(&before[29]) {
        return Err(LandingError::UnsupportedAnimationState);
    }
    let mut after = *before;
    let timer = i16::from_le_bytes(before[74..76].try_into().unwrap());
    if timer >= 0 { after[74..76].copy_from_slice(&timer.wrapping_sub(1).to_le_bytes()); }
    //7AC00 compares the preserved byte counter as SIGNED. A stale positive
    //counter clamps to delay; a negative counter expires immediately.
    let delay = before[29];
    let counter = (before[30] as i8).min(delay as i8);
    after[30] = (counter as u8).wrapping_sub(1);
    if counter <= 0 {
        after[30] = delay;
        after[27] = (before[27] + 1) % 6;
    }
    let retirement_required = i16::from_le_bytes(after[74..76].try_into().unwrap()) < 0
        && after[27] == 5 && after[30] == 0;
    Ok(landing::Effect6UpdatePlan { after, retirement_required })
}
/// Descriptor childFF has no retirement factory. Source49D30/724C0 is the
/// existing generic effect cleanup; supply every validated source pool/UI slot.
/// This plans required map unlink; it does not execute the registry mutation.
pub fn plan_retirement(before: &LandingRecord, count: i16, records: &[CleanupRecord<'_>],
    ui: &RetirementUiState) -> Result<EffectRetirementPlan, LifecycleError> {
    if before[6] != 5 || before[69] != 0x3c { return Err(LifecycleError::InvalidDeadRecord); }
    let mut temporary = *before;
    temporary[69] = 6;
    let mut plan = lifecycle::plan_effect6_retirement(&temporary, count, records, ui)?;
    plan.before = *before;
    plan.after[69] = 0x3c;
    Ok(plan)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn marker() -> LandingRecord { let mut a=[0;81];a[6]=5;a[69]=0x3c;a[28]=1;a[29]=2;a[74..76].copy_from_slice(&(-1i16).to_le_bytes());a[45..47].copy_from_slice(&(-1001i16).to_le_bytes());a }
    #[test] fn six_frame_lifetime_preserves_velocity_and_requests_retirement_once() {
        let mut a=marker();let velocity=a[45..51].to_vec();let mut calls=0;
        loop {let p=plan_animation_after_physics(&a).unwrap();calls+=1;assert_eq!(&p.after[45..51],velocity.as_slice());assert_eq!(p.after[69],0x3c);a=p.after;if p.retirement_required{break;}assert!(calls<24);}
        assert_eq!(calls,15);assert_eq!((a[27],a[30]),(5,0));
    }
    #[test] fn stale_factory_counter_clamps_or_expires_as_signed_byte() {
        let mut a=marker();a[30]=61;let clamped=plan_animation_after_physics(&a).unwrap();assert_eq!((clamped.after[27],clamped.after[30]),(0,1));
        for counter in [128,255] {a[30]=counter;let expired=plan_animation_after_physics(&a).unwrap();assert_eq!((expired.after[27],expired.after[30]),(1,2));}
    }
    #[test] fn foreign_selector_and_unproved_animation_reject_without_mutation() {
        let mut a=marker();a[69]=6;let before=a;assert_eq!(plan_animation_after_physics(&a),Err(LandingError::InvalidRecord));assert_eq!(a,before);
        a[69]=0x3c;a[28]=255;assert_eq!(plan_animation_after_physics(&a),Err(LandingError::UnsupportedAnimationState));
    }
}

//! Source 53645 completed-job branch for ordinary construction selectors.
//! The caller commits the actor and shared RNG together, then services audio/UI requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerraformCompletionError { UnsupportedActor, JobStillActive, MovementRequired }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerraformCompletionPlan {
    pub before: [u8; 200],
    pub after: [u8; 200],
    pub seed_after: u16,
    pub rng_journals: [u16; 2],
    pub stop_audio_channel: u8,
    pub stop_audio_mode: u8,
    pub pending_audio_60: bool,
    pub pending_message_16: bool,
}
/// `desired_speed` is the checked virtual80 result (51930) for this blueprint.
/// This slice admits the ready 526C0 tail and archetypes 0..2; selector5's second site is separate.
pub fn plan_architect_completed_terraform_prefix(
    before: &[u8; 200], blueprint: &[u8; 36], job: &[u8; 176],
    seed: u16, desired_speed: i16,
) -> Result<TerraformCompletionPlan, TerraformCompletionError> {
    let stage = i16::from_le_bytes(before[0x88..0x8a].try_into().unwrap());
    if before[6] != 3 || before[0x55] != 9 || before[0x7e] != 0
        || before[0x82] != 1 || before[0x83] >= 14 || before[0x83] == 5
        || !(stage == 1 || stage == 2) || blueprint[0x16] != 0 || blueprint[0x15] >= 3 {
        return Err(TerraformCompletionError::UnsupportedActor);
    }
    if i16::from_le_bytes(job[12..14].try_into().unwrap()) >= 0 {
        return Err(TerraformCompletionError::JobStillActive);
    }
    let first = seed.wrapping_mul(0xe62d);
    let second = first.wrapping_mul(0xe62d);
    let offset: [i16; 2] = if (second >> 8) % 10 < 5 { [-256, 256] } else { [256, -256] };
    let mut after = *before;
    for (axis, delta) in offset.into_iter().enumerate() {
        let p = 0x8c + axis * 2;
        let value = i16::from_le_bytes(before[p..p+2].try_into().unwrap()).wrapping_add(delta);
        after[0x60+axis*2..0x62+axis*2].copy_from_slice(&value.to_le_bytes());
    }
    after[0x5a] = 1;
    after[0x3f..0x41].copy_from_slice(&desired_speed.to_le_bytes());
    after[0x88..0x8a].copy_from_slice(&3i16.to_le_bytes());
    //53844 clears working;53973 sets movement before the526C0 tail.
    after[0x70] = (after[0x70] & !0x80) | 0x10;
    Ok(TerraformCompletionPlan {
        before: *before, after, seed_after: second, rng_journals: [0x78b, 0x7b2],
        stop_audio_channel: 0x29, stop_audio_mode: 4,
        pending_audio_60: before[0x1b] == 2 && before[0x1a] == 0,
        pending_message_16: (first >> 8) % 10 < 5,
    })
}

/// Completed-job continuation with the previously admitted ready526C0 tail.
pub fn plan_architect_completed_terraform(before:&[u8;200],blueprint:&[u8;36],job:&[u8;176],seed:u16,desired_speed:i16)
    ->Result<TerraformCompletionPlan,TerraformCompletionError>{
    let mut plan=plan_architect_completed_terraform_prefix(before,blueprint,job,seed,desired_speed)?;
    if before[0x49] & 0x23 != 0x20 || before[0x70] & 0x20 != 0 {
        return Err(TerraformCompletionError::MovementRequired);
    }
    plan.after[0x70]&=!0x10;plan.after[0x16]&=0x3f;plan.after[0x54]=0;plan.after[0x49]&=!0x10;
    Ok(plan)
}

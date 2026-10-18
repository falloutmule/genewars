//! Explicit-input family-1 scheduling and first-counter lifecycle plans.
//!
//! These helpers model only the pinned candidate's proved first scheduled
//! plant slice. They do not execute callbacks, mutate cell/entity pointers,
//! perform UI/audio cleanup, run creation passes, or simulate a whole tick.

use core::fmt;
use std::vec::Vec;

pub const PLANT_SLOT_LIMIT: usize = 512;
pub const FAMILY1_PLANT_TYPE: u8 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantLifecycleError {
    InvalidCursor(u16),
    TooManySlots(usize),
    InvalidRemainingCount(u16),
    InsufficientEligibleSlots { expected: u16, found: u16 },
    InvalidActiveType { index: usize, value: u8 },
    MissingCallbackState { index: usize },
    DuplicateCallbackState { index: usize },
    UnmatchedCallbackState { index: usize },
    InvalidPlantType(u8),
    UnsupportedFlags15(u8),
    UnsupportedFlags2e(u8),
    UnsupportedUpdateCounter(u8),
    InvalidGrowth(i16),
    InvalidCellCounter(u8),
    InvalidCheckThreshold(u8),
    InvalidSpreadPeriod(u8),
    InvalidOwner(u8),
    NegativeGrowthAfterWrap(i16),
    AllocationFailed,
}

impl fmt::Display for PlantLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}

impl std::error::Error for PlantLifecycleError {}

/// Minimal state examined by the ordinary family-1 scheduler at one slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantScheduleSlot {
    /// Byte +6. For the admitted plant family this is zero or two.
    pub active_type_at_6: u8,
    /// Byte +0x15. Bit zero suppresses the slot without consuming the count.
    pub flags_at_15: u8,
    /// Byte +0x2E. Bit 0x10 suppresses queue insertion after a scheduled call.
    pub flags_at_2e: u8,
}

/// Caller-supplied state after the scheduled virtual callback. The scheduler
/// never runs that callback; callers can derive this state with
/// [`plan_first_counter_plant_update`] or supply a separately observed state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantCallbackState {
    pub active_type_at_6: u8,
    pub flags_at_2e: u8,
    /// Caller-supplied per-callback diagnostic count. The scheduler sums these
    /// values only to flag that an additional creation pass may be pending.
    /// This is not a claim about the width or aggregation behavior of an
    /// original global counter; the admitted first-counter callback slice
    /// creates zero plants, and the subsequent pass is not modeled here.
    pub created_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantSchedulePlan {
    /// Indices receiving virtual +0x64 requests, in ascending scan order.
    pub callback_indices: Vec<usize>,
    /// First-pass family-1 queue candidates after caller-supplied callback
    /// state is applied. These are indices only; rendering is separate.
    pub queued_indices: Vec<usize>,
    pub cursor_after: u16,
    pub eligible_slots_consumed: u16,
    /// Saturating u16 sum of the explicit per-callback diagnostic inputs.
    /// This is not an inferred native global-counter width or live count.
    pub created_count: u16,
    /// True means caller-supplied callback states report a nonzero creation
    /// count. The corresponding subsequent pass was not simulated.
    pub second_pass_pending: bool,
}

/// Advance the explicit scheduler cursor by eight. The wrap branch uses the
/// proven `(sum + 1) & 7` operation when the unsigned admitted sum exceeds 512.
pub fn advance_plant_schedule_cursor(cursor: u16) -> Result<u16, PlantLifecycleError> {
    if cursor > 512 { return Err(PlantLifecycleError::InvalidCursor(cursor)); }
    let next = cursor + 8;
    Ok(if next > 512 { (next + 1) & 7 } else { next })
}

/// Plan the ordinary scheduler over a caller-supplied contiguous family-1
/// slot slice. `remaining_count` is the number of eligible records the
/// original loop must consume, not the total array length. Slots skipped for
/// inactive type or flags+0x15 bit zero do not consume it. Callback results
/// are explicit post-call inputs keyed by slot index; this function does not
/// invoke plant update code or the subsequent creation pass.
pub fn plan_family1_plant_schedule(
    initial_cursor: u16,
    remaining_count: u16,
    slots: &[PlantScheduleSlot],
    callback_states: &[(usize, PlantCallbackState)],
) -> Result<PlantSchedulePlan, PlantLifecycleError> {
    if initial_cursor > 512 { return Err(PlantLifecycleError::InvalidCursor(initial_cursor)); }
    if slots.len() > PLANT_SLOT_LIMIT { return Err(PlantLifecycleError::TooManySlots(slots.len())); }
    if remaining_count > PLANT_SLOT_LIMIT as u16 {
        return Err(PlantLifecycleError::InvalidRemainingCount(remaining_count));
    }
    let mut callback_indices = Vec::new();
    callback_indices.try_reserve_exact(2).map_err(|_| PlantLifecycleError::AllocationFailed)?;
    let mut queued_indices = Vec::new();
    queued_indices.try_reserve_exact(remaining_count as usize).map_err(|_| PlantLifecycleError::AllocationFailed)?;
    let mut used_callback_states = Vec::new();
    used_callback_states.try_reserve_exact(2).map_err(|_| PlantLifecycleError::AllocationFailed)?;

    // The cursor's first +8 step happens even with zero remaining records.
    let mut cursor = advance_plant_schedule_cursor(initial_cursor)?;
    let mut remaining = remaining_count;
    let mut created_count = 0u16;
    for (index, slot) in slots.iter().copied().enumerate() {
        if remaining == 0 { break; }
        if slot.active_type_at_6 != 0 && slot.active_type_at_6 != FAMILY1_PLANT_TYPE {
            return Err(PlantLifecycleError::InvalidActiveType { index, value: slot.active_type_at_6 });
        }
        if slot.active_type_at_6 == 0 || slot.flags_at_15 & 0x01 != 0 {
            continue;
        }

        let mut active_after = slot.active_type_at_6;
        let mut flags2e_after = slot.flags_at_2e;
        if callback_indices.len() < 2 && index == usize::from(cursor) {
            callback_indices.push(index);
            let state = callback_states.iter().find(|(candidate, _)| *candidate == index)
                .map(|(_, state)| *state)
                .ok_or(PlantLifecycleError::MissingCallbackState { index })?;
            if used_callback_states.contains(&index) {
                return Err(PlantLifecycleError::DuplicateCallbackState { index });
            }
            used_callback_states.push(index);
            if state.active_type_at_6 != 0 && state.active_type_at_6 != FAMILY1_PLANT_TYPE {
                return Err(PlantLifecycleError::InvalidActiveType { index, value: state.active_type_at_6 });
            }
            active_after = state.active_type_at_6;
            flags2e_after = state.flags_at_2e;
            created_count = created_count.saturating_add(u16::from(state.created_count));
            cursor = advance_plant_schedule_cursor(cursor)?;
        }
        if active_after != 0 && flags2e_after & 0x10 == 0 {
            queued_indices.push(index);
        }
        remaining -= 1;
    }
    if remaining != 0 {
        return Err(PlantLifecycleError::InsufficientEligibleSlots {
            expected: remaining_count, found: remaining_count - remaining,
        });
    }
    for (index, _) in callback_states {
        if used_callback_states.contains(index) {
            // Duplicates are rejected below by counting occurrences.
            if callback_states.iter().filter(|(candidate, _)| candidate == index).count() > 1 {
                return Err(PlantLifecycleError::DuplicateCallbackState { index: *index });
            }
        } else {
            return Err(PlantLifecycleError::UnmatchedCallbackState { index: *index });
        }
    }
    Ok(PlantSchedulePlan { callback_indices, queued_indices, cursor_after: cursor,
        eligible_slots_consumed: remaining_count, created_count, second_pass_pending: created_count != 0 })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstCounterPlantInput {
    pub active_type_at_6: u8,
    pub flags_at_15: u8,
    pub owner_at_18: u8,
    pub animation_at_19: u8,
    pub frame_at_1b: u8,
    pub growth_i16: i16,
    pub flags_at_2e: u8,
    pub update_counter_at_2f: u8,
    pub retirement_parameter_at_1d: u8,
    /// Runtime byte +0x30. It is untouched on the positive-growth path and
    /// initialized from the resource parameter only when retirement starts.
    pub retirement_countdown_at_30: u8,
    /// Nine-cell positive-growth byte sum. The caller supplies the already
    /// wrapped u8 sum; neighborhood traversal is not performed here.
    pub positive_growth_neighbors_u8: u8,
    pub crowding_limit_at_1a: u8,
    pub check_threshold_at_1b: u8,
    pub spread_period_at_18: u8,
    pub signed_growth_delta_i8: i8,
    pub cell_counter_at_13: u8,
    pub plant_count_u16: u16,
    /// Present only when owner is in the admitted range 0..=3.
    pub owner_count_u16: Option<u16>,
    /// Caller-computed gate for the old-frame virtual dirty call.
    pub old_frame_inside_current_view: bool,
    /// Null cached frame suppresses the body of the old-frame dirty helper.
    pub cached_frame_present: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FirstCounterPhase {
    Grow,
    DelayedRetirement,
    Retired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingPlantCleanup {
    /// Request to unlink the typed reference from its cell's linked list.
    pub unlink_cell_reference: bool,
    /// Whether the old-frame virtual dirty hook is requested by the current
    /// view gate, and whether its body runs with the cached-frame pointer.
    pub old_frame_dirty_virtual_requested: bool,
    pub old_frame_dirty_body_runs: bool,
    /// Opaque request to execute the proved 0x724C0 reference/UI/audio helper.
    pub bookkeeping_724c0_requested: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FirstCounterPlantPlan {
    pub phase: FirstCounterPhase,
    pub active_type_at_6_after: u8,
    pub flags_at_15_after: u8,
    pub owner_at_18: u8,
    pub animation_at_19_after: u8,
    pub frame_at_1b_after: u8,
    pub growth_i16_after: i16,
    pub flags_at_2e_after: u8,
    pub update_counter_at_2f_after: u8,
    pub retirement_countdown_at_30_after: u8,
    pub cell_counter_at_13_after: u8,
    pub plant_count_u16_after: u16,
    pub owner_count_u16_after: Option<u16>,
    pub created_count: u8,
    pub cleanup: PendingPlantCleanup,
}

/// Plan the admitted first-counter growth/retirement slice from explicit
/// fields. It does not inspect a map, count neighbors, unlink pointers, call
/// the old-frame renderer, or perform 0x724C0's UI/audio/reference cleanup.
pub fn plan_first_counter_plant_update(
    input: FirstCounterPlantInput,
) -> Result<FirstCounterPlantPlan, PlantLifecycleError> {
    if input.active_type_at_6 != FAMILY1_PLANT_TYPE {
        return Err(PlantLifecycleError::InvalidPlantType(input.active_type_at_6));
    }
    if input.flags_at_15 & 0x01 != 0 { return Err(PlantLifecycleError::UnsupportedFlags15(input.flags_at_15)); }
    if input.flags_at_2e & 0x9A != 0 { return Err(PlantLifecycleError::UnsupportedFlags2e(input.flags_at_2e)); }
    if input.update_counter_at_2f != 0 {
        return Err(PlantLifecycleError::UnsupportedUpdateCounter(input.update_counter_at_2f));
    }
    if input.growth_i16 <= 0 { return Err(PlantLifecycleError::InvalidGrowth(input.growth_i16)); }
    if input.cell_counter_at_13 == 0 { return Err(PlantLifecycleError::InvalidCellCounter(input.cell_counter_at_13)); }
    if input.check_threshold_at_1b == 0 {
        return Err(PlantLifecycleError::InvalidCheckThreshold(input.check_threshold_at_1b));
    }
    if input.spread_period_at_18 <= 1 { return Err(PlantLifecycleError::InvalidSpreadPeriod(input.spread_period_at_18)); }
    if input.owner_at_18 > 4 { return Err(PlantLifecycleError::InvalidOwner(input.owner_at_18)); }
    let owner_count = if input.owner_at_18 < 4 {
        Some(input.owner_count_u16.ok_or(PlantLifecycleError::InvalidOwner(input.owner_at_18))?)
    } else { None };

    let mut growth = input.growth_i16;
    if input.positive_growth_neighbors_u8 <= input.crowding_limit_at_1a {
        growth = growth.wrapping_add(i16::from(input.signed_growth_delta_i8));
        if growth > 32 { growth = 32; }
    } else {
        growth = 0;
    }
    let should_retire = growth <= 0;
    let mut animation = input.animation_at_19;
    let mut flags15 = input.flags_at_15;
    let mut flags2e = input.flags_at_2e;
    let mut active_type = input.active_type_at_6;
    let mut retirement_countdown = input.retirement_countdown_at_30;
    let mut cell_counter = input.cell_counter_at_13;
    let mut plant_count = input.plant_count_u16;
    let mut owner_count_after = owner_count;
    let mut cleanup = PendingPlantCleanup {
        unlink_cell_reference: false,
        old_frame_dirty_virtual_requested: false,
        old_frame_dirty_body_runs: false,
        bookkeeping_724c0_requested: false,
    };
    let phase;
    if should_retire {
        retirement_countdown = input.retirement_parameter_at_1d;
        animation = animation.wrapping_add(6);
        flags15 |= 0x04;
        flags2e |= 0x80;
        if retirement_countdown >= 4 {
            retirement_countdown -= 4;
            phase = FirstCounterPhase::DelayedRetirement;
        } else {
            phase = FirstCounterPhase::Retired;
            active_type = 0;
            if cell_counter < 64 { cell_counter = cell_counter.wrapping_add(2); }
            plant_count = plant_count.wrapping_sub(1);
            if let Some(value) = owner_count_after.as_mut() { *value = value.wrapping_sub(1); }
            cleanup = PendingPlantCleanup {
                unlink_cell_reference: true,
                old_frame_dirty_virtual_requested: input.old_frame_inside_current_view,
                old_frame_dirty_body_runs: input.old_frame_inside_current_view && input.cached_frame_present,
                bookkeeping_724c0_requested: true,
            };
        }
        growth = 0;
    } else {
        phase = FirstCounterPhase::Grow;
        let target_animation = core::cmp::min(growth >> 3, 2) as u8;
        animation = core::cmp::max(animation, target_animation);
        if animation != input.animation_at_19 { flags15 |= 0x04; }
    }

    Ok(FirstCounterPlantPlan {
        phase,
        active_type_at_6_after: active_type,
        flags_at_15_after: flags15,
        owner_at_18: input.owner_at_18,
        animation_at_19_after: animation,
        frame_at_1b_after: input.frame_at_1b,
        growth_i16_after: growth,
        flags_at_2e_after: flags2e,
        update_counter_at_2f_after: 1,
        retirement_countdown_at_30_after: retirement_countdown,
        cell_counter_at_13_after: cell_counter,
        plant_count_u16_after: plant_count,
        owner_count_u16_after: owner_count_after,
        created_count: 0,
        cleanup,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot() -> PlantScheduleSlot {
        PlantScheduleSlot { active_type_at_6: 2, flags_at_15: 4, flags_at_2e: 0 }
    }

    fn first_input() -> FirstCounterPlantInput {
        FirstCounterPlantInput {
            active_type_at_6: 2, flags_at_15: 4, owner_at_18: 4,
            animation_at_19: 0, frame_at_1b: 7, growth_i16: 22,
            flags_at_2e: 0, update_counter_at_2f: 0,
            retirement_parameter_at_1d: 3, positive_growth_neighbors_u8: 5,
            retirement_countdown_at_30: 0,
            crowding_limit_at_1a: 3, check_threshold_at_1b: 7,
            spread_period_at_18: 4, signed_growth_delta_i8: 3,
            cell_counter_at_13: 63, plant_count_u16: 40,
            owner_count_u16: None, old_frame_inside_current_view: true,
            cached_frame_present: false,
        }
    }

    #[test]
    fn scheduled_cursor_and_two_callbacks_match_loaded_forty_slot_slice() {
        let slots = vec![slot(); 40];
        let states = [
            (8, PlantCallbackState { active_type_at_6: 2, flags_at_2e: 0, created_count: 0 }),
            (16, PlantCallbackState { active_type_at_6: 2, flags_at_2e: 0, created_count: 0 }),
        ];
        let plan = plan_family1_plant_schedule(0, 40, &slots, &states).unwrap();
        assert_eq!(plan.callback_indices, [8, 16]);
        assert_eq!(plan.cursor_after, 24);
        assert_eq!(plan.queued_indices.len(), 40);
        assert_eq!(plan.eligible_slots_consumed, 40);
        assert!(!plan.second_pass_pending);
    }

    #[test]
    fn skipped_slots_do_not_consume_budget_and_retired_callback_is_not_queued() {
        let mut slots = vec![slot(); 20];
        slots[0].active_type_at_6 = 0;
        slots[1].flags_at_15 |= 1;
        let states = [
            (8, PlantCallbackState { active_type_at_6: 0, flags_at_2e: 0x80, created_count: 0 }),
            (16, PlantCallbackState { active_type_at_6: 2, flags_at_2e: 0x10, created_count: 1 }),
        ];
        let plan = plan_family1_plant_schedule(0, 18, &slots, &states).unwrap();
        assert_eq!(plan.callback_indices, [8, 16]);
        assert!(!plan.queued_indices.contains(&8));
        assert!(!plan.queued_indices.contains(&16));
        assert_eq!(plan.eligible_slots_consumed, 18);
        assert_eq!(plan.created_count, 1);
        assert!(plan.second_pass_pending);
    }

    #[test]
    fn cursor_vectors_cover_512_boundary() {
        for (input, expected) in [(0, 8), (504, 512), (505, 2), (511, 0), (512, 1)] {
            assert_eq!(advance_plant_schedule_cursor(input).unwrap(), expected);
        }
        assert_eq!(advance_plant_schedule_cursor(513), Err(PlantLifecycleError::InvalidCursor(513)));
        let plan = plan_family1_plant_schedule(512, 0, &[], &[]).unwrap();
        assert_eq!(plan.cursor_after, 1); // first +8 is retained even with count zero
    }

    #[test]
    fn first_update_growth_wraps_then_caps_and_preserves_frame() {
        let mut input = first_input();
        input.growth_i16 = 32767;
        input.positive_growth_neighbors_u8 = 0;
        input.signed_growth_delta_i8 = 1;
        input.animation_at_19 = 1;
        let plan = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(plan.phase, FirstCounterPhase::Retired); // signed word wraps to -32768
        assert_eq!(plan.growth_i16_after, 0);
        assert_eq!(plan.animation_at_19_after, 7);
        assert_eq!(plan.frame_at_1b_after, 7);
    }

    #[test]
    fn growth_caps_at_32_and_only_marks_animation_change() {
        let mut input = first_input();
        input.growth_i16 = 31;
        input.positive_growth_neighbors_u8 = 0;
        input.signed_growth_delta_i8 = 8;
        input.retirement_countdown_at_30 = 29;
        let plan = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(plan.phase, FirstCounterPhase::Grow);
        assert_eq!(plan.growth_i16_after, 32);
        assert_eq!(plan.animation_at_19_after, 2);
        assert_eq!(plan.flags_at_15_after, 4);
        assert_eq!(plan.frame_at_1b_after, 7);
        assert_eq!(plan.created_count, 0);
        assert_eq!(plan.retirement_countdown_at_30_after, 29);
    }

    #[test]
    fn crowding_is_strictly_greater_than_threshold() {
        let mut input = first_input();
        input.growth_i16 = 15;
        input.positive_growth_neighbors_u8 = input.crowding_limit_at_1a;
        input.signed_growth_delta_i8 = 1;
        let plan = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(plan.phase, FirstCounterPhase::Grow);
        assert_eq!(plan.growth_i16_after, 16);
        assert_eq!(plan.animation_at_19_after, 2);
    }

    #[test]
    fn delayed_retirement_and_immediate_cleanup_are_explicit_and_wrapping() {
        let mut input = first_input();
        input.retirement_parameter_at_1d = 4;
        let delayed = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(delayed.phase, FirstCounterPhase::DelayedRetirement);
        assert_eq!(delayed.retirement_countdown_at_30_after, 0);
        assert_eq!(delayed.active_type_at_6_after, 2);
        assert_eq!(delayed.cell_counter_at_13_after, 63);
        assert_eq!(delayed.plant_count_u16_after, 40);
        assert!(!delayed.cleanup.bookkeeping_724c0_requested);

        input.retirement_parameter_at_1d = 3;
        input.owner_at_18 = 0;
        input.owner_count_u16 = Some(0);
        let retired = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(retired.phase, FirstCounterPhase::Retired);
        assert_eq!(retired.active_type_at_6_after, 0);
        assert_eq!(retired.cell_counter_at_13_after, 65); // guarded increment is not u8 wrap
        assert_eq!(retired.plant_count_u16_after, 39);
        assert_eq!(retired.owner_count_u16_after, Some(u16::MAX));
        assert!(retired.cleanup.unlink_cell_reference);
        assert!(retired.cleanup.old_frame_dirty_virtual_requested);
        assert!(!retired.cleanup.old_frame_dirty_body_runs);
        assert!(retired.cleanup.bookkeeping_724c0_requested);

        input.plant_count_u16 = 0;
        input.cell_counter_at_13 = 64;
        input.retirement_parameter_at_1d = 3;
        let wrapped = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(wrapped.plant_count_u16_after, u16::MAX);
        assert_eq!(wrapped.cell_counter_at_13_after, 64);
    }

    #[test]
    fn owner_four_skips_player_counter_and_hidden_dirty_gate_is_retained() {
        let mut input = first_input();
        input.positive_growth_neighbors_u8 = 4;
        input.owner_at_18 = 4;
        input.owner_count_u16 = None;
        input.old_frame_inside_current_view = false;
        input.cached_frame_present = true;
        let plan = plan_first_counter_plant_update(input).unwrap();
        assert_eq!(plan.phase, FirstCounterPhase::Retired);
        assert_eq!(plan.owner_count_u16_after, None);
        assert!(!plan.cleanup.old_frame_dirty_virtual_requested);
        assert!(!plan.cleanup.old_frame_dirty_body_runs);
    }

    #[test]
    fn rejects_unproven_lifecycle_inputs_and_missing_schedule_callback_state() {
        let mut input = first_input();
        input.flags_at_2e = 0x02;
        assert_eq!(plan_first_counter_plant_update(input), Err(PlantLifecycleError::UnsupportedFlags2e(2)));
        input = first_input();
        input.growth_i16 = 0;
        assert_eq!(plan_first_counter_plant_update(input), Err(PlantLifecycleError::InvalidGrowth(0)));
        assert_eq!(plan_family1_plant_schedule(0, 40, &vec![slot(); 40], &[]),
            Err(PlantLifecycleError::MissingCallbackState { index: 8 }));
        let states = [
            (8, PlantCallbackState { active_type_at_6: 2, flags_at_2e: 0, created_count: 0 }),
            (16, PlantCallbackState { active_type_at_6: 2, flags_at_2e: 0, created_count: 0 }),
        ];
        assert!(matches!(plan_family1_plant_schedule(0, 41, &vec![slot(); 40], &states),
            Err(PlantLifecycleError::InsufficientEligibleSlots { .. })));
    }
}

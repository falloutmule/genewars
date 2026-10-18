//! Explicit plans for CD 0x801C0 category selection and 0x81310 allocation.
//!
//! Evidence: `research/ghidra/player-creation-evidence.json`. These helpers do
//! not generate random blueprints, initialize resource pointers, execute an
//! actor callback, reveal terrain, or simulate a complete mission tick.

pub const PLAYER_BLUEPRINT_CAPACITY: usize = 32;
pub const PLAYER_ACTOR_SLOTS: usize = 5;
pub const FAMILY2_ACTOR_TYPE: u8 = 3;

/// Fields read by selection from one 36-byte player metadata record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorBlueprintCandidate {
    /// Record byte +0. Any nonzero value is active; names remain opaque.
    pub active_name_byte: u8,
    /// Record byte +0x16. Values above three are ignored by the source.
    pub category: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerCreationError {
    InvalidPlayer(u8),
    InvalidStoredCount(i8),
    /// A free native slot exists, but appending its ID would exceed the
    /// admitted five-byte player ID list. The original does not check this.
    StoredListFull,
}

impl core::fmt::Display for PlayerCreationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PlayerCreationError {}

/// Requests made only after the allocator finds an inactive native slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorAllocationRequest {
    pub local_slot: u8,
    /// Native family-2 slot ID: `5 * player + local_slot`, in 0..20.
    pub global_slot_id: u8,
    pub owner: u8,
    pub active_type: u8,
    /// Player-relative destination byte for appending the ID, in +4..+8.
    pub player_id_list_offset: u8,
    /// OR this mask into blueprint byte +0x20; preserve other bits/bytes.
    pub blueprint_flags20_or: u8,
    /// Native byte +0x74 receives the selected blueprint index.
    pub blueprint_index_at74: u8,
    /// 0x4FD30's final stores, before the separate 0x80830 placement call.
    pub flags15_word: u16,
    pub flags70_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorCreationAttempt {
    pub blueprint_index: u8,
    pub category: u8,
    /// None means all five native slots were occupied. The category attempt
    /// quota is still consumed, so a later record cannot retry that category.
    pub allocation: Option<ActorAllocationRequest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerActorCreationPlan {
    /// At most four attempts, in ascending metadata-record scan order.
    /// The used prefix has `attempt_count` entries; its remainder is None.
    pub attempts: [Option<ActorCreationAttempt>; 4],
    pub attempt_count: u8,
    pub created_count: u8,
    pub player_count_after: u8,
    /// A snapshot with successful allocations marked type3. Existing nonzero
    /// active bytes are preserved. Applying initialization remains external.
    pub slot_types_after: [u8; PLAYER_ACTOR_SLOTS],
    /// 0x801C0 finally requests player+9 := target_table[player], including
    /// when there were no attempts. The table/target is caller supplied.
    pub target_reset_player: u8,
}

/// Source category gate, including signed-byte campaign progress thresholds.
pub fn actor_category_is_enabled(category: u8, mask: u8, progress: i8, campaign: bool) -> bool {
    match category {
        0 => mask & 1 != 0,
        1 => mask & 2 != 0 && (!campaign || progress >= 1),
        2 => mask & 4 != 0,
        3 => mask & 8 != 0 && (!campaign || progress >= 4),
        _ => false,
    }
}

/// Plan selection and first-free allocations without applying native writes.
///
/// `active_slots` corresponds to the player's five 200-byte native records at
/// `0x16360E + 1000*player`. A zero active byte +6 is free; any other byte is
/// occupied. `stored_count` is the signed player byte +3, safely restricted to
/// 0..=5. The source appends through this count without checking its bounds.
///
/// Callers must supply live metadata/slot/count inputs. Raw CONQ records alone
/// cannot establish retained selected-player state. A successful request
/// still needs 0x4FD30-equivalent initialization and later explicit placement;
/// this plan neither executes those callbacks nor invents resource pointers.
/// Errors return no partial plan and leave all caller inputs unchanged.
pub fn plan_player_actor_creation(
    blueprints: &[ActorBlueprintCandidate; PLAYER_BLUEPRINT_CAPACITY],
    mask: u8,
    progress: i8,
    campaign: bool,
    player: u8,
    active_slots: [u8; PLAYER_ACTOR_SLOTS],
    stored_count: i8,
) -> Result<PlayerActorCreationPlan, PlayerCreationError> {
    if player >= 4 {
        return Err(PlayerCreationError::InvalidPlayer(player));
    }
    if !(0..=5).contains(&stored_count) {
        return Err(PlayerCreationError::InvalidStoredCount(stored_count));
    }
    let mut result = PlayerActorCreationPlan {
        attempts: [None; 4],
        attempt_count: 0,
        created_count: 0,
        player_count_after: stored_count as u8,
        slot_types_after: active_slots,
        target_reset_player: player,
    };
    let mut attempted_categories = 0_u8;
    for (index, blueprint) in blueprints.iter().enumerate() {
        if blueprint.active_name_byte == 0
            || !actor_category_is_enabled(blueprint.category, mask, progress, campaign)
        {
            continue;
        }
        let category_bit = 1 << blueprint.category;
        if attempted_categories & category_bit != 0 {
            continue;
        }
        // The source sets this quota before calling 0x81310, even on failure.
        attempted_categories |= category_bit;
        let allocation = if let Some(local_slot) = result.slot_types_after.iter().position(|&v| v == 0) {
            if result.player_count_after >= PLAYER_ACTOR_SLOTS as u8 {
                return Err(PlayerCreationError::StoredListFull);
            }
            let request = ActorAllocationRequest {
                local_slot: local_slot as u8,
                global_slot_id: player * 5 + local_slot as u8,
                owner: player,
                active_type: FAMILY2_ACTOR_TYPE,
                player_id_list_offset: 4 + result.player_count_after,
                blueprint_flags20_or: 1,
                blueprint_index_at74: index as u8,
                flags15_word: 0x0280,
                flags70_word: 0,
            };
            result.slot_types_after[local_slot] = FAMILY2_ACTOR_TYPE;
            result.player_count_after += 1;
            result.created_count += 1;
            Some(request)
        } else {
            None
        };
        result.attempts[result.attempt_count as usize] = Some(ActorCreationAttempt {
            blueprint_index: index as u8,
            category: blueprint.category,
            allocation,
        });
        result.attempt_count += 1;
    }
    Ok(result)
}

/// 0x57A10's proved early return: clear byte+0x70 bit0 means zero created
/// count, no mutation, no callback and no queue request. False means the full
/// ordinary update is unresolved, rather than permission to approximate it.
pub fn family2_ordinary_update_is_suppressed(flags70_byte: u8) -> bool {
    flags70_byte & 1 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn records(categories: &[u8]) -> [ActorBlueprintCandidate; 32] {
        let mut result = [ActorBlueprintCandidate { active_name_byte: 0, category: 0 }; 32];
        for (record, &category) in result.iter_mut().zip(categories) {
            *record = ActorBlueprintCandidate { active_name_byte: 1, category };
        }
        result
    }

    #[test]
    fn gated_first_campaign_attempts_zero_and_ten_in_first_free_slots() {
        let generated = core::array::from_fn(|i| ActorBlueprintCandidate {
            active_name_byte: u8::from(i < 20), category: (i / 5) as u8,
        });
        let plan = plan_player_actor_creation(&generated, 5, 0, true, 0, [0; 5], 0).unwrap();
        assert_eq!((plan.attempt_count, plan.created_count, plan.player_count_after), (2, 2, 2));
        for (attempt, index) in plan.attempts[..2].iter().zip([0, 10]) {
            let attempt = attempt.unwrap();
            let allocation = attempt.allocation.unwrap();
            assert_eq!(attempt.blueprint_index, index);
            assert_eq!(allocation.blueprint_index_at74, index);
            assert_eq!(allocation.global_slot_id, u8::from(index == 10));
            assert_eq!((allocation.flags15_word, allocation.flags70_word), (0x280, 0));
        }
        assert_eq!(plan.slot_types_after, [3, 3, 0, 0, 0]);
        assert_eq!(plan.target_reset_player, 0);
    }

    #[test]
    fn signed_progress_thresholds_and_noncampaign_bypass_are_exact() {
        for progress in i8::MIN..=i8::MAX {
            assert!(actor_category_is_enabled(0, 15, progress, true));
            assert!(actor_category_is_enabled(2, 15, progress, true));
            assert_eq!(actor_category_is_enabled(1, 15, progress, true), progress >= 1);
            assert_eq!(actor_category_is_enabled(3, 15, progress, true), progress >= 4);
            assert!(actor_category_is_enabled(1, 15, progress, false));
            assert!(actor_category_is_enabled(3, 15, progress, false));
        }
        for category in 4..=u8::MAX {
            assert!(!actor_category_is_enabled(category, 255, 127, false));
        }
    }

    #[test]
    fn allocation_failure_consumes_category_quota_without_native_writes() {
        let plan = plan_player_actor_creation(&records(&[0, 0, 2, 2]), 5, 0, true, 1, [3; 5], 5).unwrap();
        assert_eq!(plan.attempt_count, 2);
        assert_eq!(plan.attempts[0].unwrap().blueprint_index, 0);
        assert_eq!(plan.attempts[1].unwrap().blueprint_index, 2);
        assert!(plan.attempts[..2].iter().all(|a| a.unwrap().allocation.is_none()));
        assert_eq!((plan.created_count, plan.player_count_after), (0, 5));
        assert_eq!(plan.slot_types_after, [3; 5]);
    }

    #[test]
    fn holes_are_filled_ascending_and_ids_stay_in_player_partition() {
        for player in 0..4 {
            let plan = plan_player_actor_creation(&records(&[3, 2, 1, 0]), 15, 4, true, player, [7, 0, 8, 0, 0], 2).unwrap();
            for (attempt, local) in plan.attempts[..3].iter().zip([1, 3, 4]) {
                let allocation = attempt.unwrap().allocation.unwrap();
                assert_eq!(allocation.local_slot, local);
                assert_eq!(allocation.global_slot_id, player * 5 + local);
            }
            assert_eq!(plan.attempts[3].unwrap().allocation, None);
            assert_eq!(plan.slot_types_after, [7, 3, 8, 3, 3]);
            assert_eq!(plan.player_count_after, 5);
        }
    }

    #[test]
    fn bounds_errors_do_not_mutate_inputs_and_empty_scan_still_resets_target() {
        let blueprints = records(&[0]); let slots = [0; 5];
        assert_eq!(plan_player_actor_creation(&blueprints, 1, 0, true, 4, slots, 0), Err(PlayerCreationError::InvalidPlayer(4)));
        for count in [-128, -1, 6, 127] {
            assert_eq!(plan_player_actor_creation(&blueprints, 1, 0, true, 0, slots, count), Err(PlayerCreationError::InvalidStoredCount(count)));
        }
        assert_eq!(plan_player_actor_creation(&blueprints, 1, 0, true, 0, slots, 5), Err(PlayerCreationError::StoredListFull));
        assert_eq!(slots, [0; 5]);
        let empty = plan_player_actor_creation(&records(&[]), 255, 127, false, 3, slots, 0).unwrap();
        assert_eq!((empty.attempt_count, empty.created_count, empty.target_reset_player), (0, 0, 3));
        for flags in 0..=255 {
            assert_eq!(family2_ordinary_update_is_suppressed(flags), flags % 2 == 0);
        }
        assert!(family2_ordinary_update_is_suppressed(0x0400_u16 as u8));
    }
}

//! Bounded opcode-3 move-order setter over one explicit native actor record.
//!
//! The caller supplies the exact output of the complete target resolver. This
//! helper does not run that resolver, validate a point against terrain, or
//! establish the outer packet/player/actor eligibility gates. It applies only
//! the closed setter writes and refuses the setter's transitive reinitializing
//! states without changing the record.
//!
//! Instruction and vector evidence is pinned in
//! `research/ghidra/move-order-evidence.json` and
//! `research/ghidra/move-order-bridge.md` (candidate SHA-256
//! `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`).
//! The global actor table begins at VA `0x16360E`, with 20 records at decimal
//! stride 200 (`0xC8`). Opcode3 target `0x1C528` dispatches to setter `0x80FC0`.

use core::fmt;

pub const ACTOR_RECORD_BYTES: usize = 0xC8;
pub const ACTOR_SLOT_COUNT: i16 = 20;

const FLAGS_71: usize = 0x71;
const FLAGS_57: usize = 0x57;
const STATE_82: usize = 0x82;
const BYTE_5B: usize = 0x5B;
const TARGET_5C: usize = 0x5C;
const DESTINATION_0B: usize = 0x0B;

/// The exact result returned by a separately executed, fully admitted target
/// resolver. Its provenance is deliberately explicit: this module does not
/// check distance, terrain class, obstacles, blockers, or search steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrecomputedResolvedTarget {
    pub position: [i16; 2],
}

impl PrecomputedResolvedTarget {
    pub const fn new(position: [i16; 2]) -> Self {
        Self { position }
    }

    pub const fn position(self) -> [i16; 2] {
        self.position
    }
}

/// Explicit environmental results used by the resolver's bounded
/// accepted-request case. This type supplies values; it never queries world
/// state or invokes obstacle, terrain, or blocker helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolverEnvironment {
    pub terrain_class: u8,
    pub actor_avoidance_mask: u8,
    pub obstacle_present: bool,
    /// Result from the bounded cell-blocker helper; must be nonnegative.
    pub blocker_weight: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolverError {
    InvalidTerrainClass(u8),
    NegativeBlockerWeight(i32),
    BackwardSearchRequired,
}

/// Implement only the resolver outcomes closed by the instruction proof.
/// Signed-15 torus distance at most 4096 snaps to `current`, regardless of
/// environment predicates. Farther points return `requested` only when the
/// obstacle is absent, its terrain class is not avoided and blocker weight is
/// below 10. Other farther cases require the original backward search.
pub fn resolve_bounded_move_target(
    current: [i16; 2],
    requested: [i16; 2],
    environment: ResolverEnvironment,
) -> Result<PrecomputedResolvedTarget, ResolverError> {
    if environment.terrain_class > 7 {
        return Err(ResolverError::InvalidTerrainClass(environment.terrain_class));
    }
    if environment.blocker_weight < 0 {
        return Err(ResolverError::NegativeBlockerWeight(environment.blocker_weight));
    }
    let dx = ((i32::from(current[0]) - i32::from(requested[0]) + 16_384) & 32_767) - 16_384;
    let dy = ((i32::from(current[1]) - i32::from(requested[1]) + 16_384) & 32_767) - 16_384;
    let distance_squared = dx * dx + dy * dy;
    if distance_squared <= 4_096 {
        return Ok(PrecomputedResolvedTarget::new(current));
    }

    let terrain_bit = 1u8 << environment.terrain_class;
    if !environment.obstacle_present
        && environment.actor_avoidance_mask & terrain_bit == 0
        && environment.blocker_weight < 10
    {
        Ok(PrecomputedResolvedTarget::new(requested))
    } else {
        Err(ResolverError::BackwardSearchRequired)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode3MoveOrderResult {
    /// Source flag +0x71 mask0x10 returns without touching the actor.
    NoopFlag71,
    /// The setter stages `requested`, then the resolver's returned point
    /// replaces +0x5C and is also copied into the destination at +0x0B.
    Applied {
        requested: [i16; 2],
        resolved: [i16; 2],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Opcode3MoveOrderError {
    InvalidActorId(i16),
    ActorRecordLength { expected: usize, actual: usize },
    ReinitializationRequired { flags_at_57: u8, state_at_82: u8 },
}

impl fmt::Display for Opcode3MoveOrderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Opcode3MoveOrderError {}

/// Apply the opcode-3 setter's proved record writes to a single actor slot.
///
/// `actor_id` is the signed packet word, used as an index into the 20-record
/// global array. Outer command eligibility (owner/player, active type, ready
/// bit and state value) is the caller's responsibility. `resolved` must be
/// supplied from a separately validated invocation of the complete resolver.
///
/// Errors leave `record` byte-for-byte unchanged. A `NoopFlag71` is a proved
/// source no-op and also leaves the record unchanged.
pub fn apply_opcode3_move_order(
    actor_id: i16,
    record: &mut [u8],
    requested: [i16; 2],
    resolved: PrecomputedResolvedTarget,
) -> Result<Opcode3MoveOrderResult, Opcode3MoveOrderError> {
    if !(0..ACTOR_SLOT_COUNT).contains(&actor_id) {
        return Err(Opcode3MoveOrderError::InvalidActorId(actor_id));
    }
    if record.len() != ACTOR_RECORD_BYTES {
        return Err(Opcode3MoveOrderError::ActorRecordLength {
            expected: ACTOR_RECORD_BYTES,
            actual: record.len(),
        });
    }

    // This source branch precedes the 0x74A50 resolver path.
    if record[FLAGS_71] & 0x10 != 0 {
        return Ok(Opcode3MoveOrderResult::NoopFlag71);
    }

    let old_flags_57 = record[FLAGS_57];
    let state_82 = record[STATE_82];
    if old_flags_57 & 0x01 != 0 || state_82 != 0 {
        return Err(Opcode3MoveOrderError::ReinitializationRequired {
            flags_at_57: old_flags_57,
            state_at_82: state_82,
        });
    }

    // All error gates precede mutation. Preserve all opaque record bytes and
    // retain the source's intermediate requested-target staging in the write
    // sequence, even though the resolved point is the final +0x5C value.
    record[FLAGS_57] = old_flags_57 | 0x02;
    write_point(record, TARGET_5C, requested);
    record[BYTE_5B] = 0;
    write_point(record, TARGET_5C, resolved.position);
    write_point(record, DESTINATION_0B, resolved.position);

    Ok(Opcode3MoveOrderResult::Applied {
        requested,
        resolved: resolved.position,
    })
}

fn write_point(record: &mut [u8], offset: usize, point: [i16; 2]) {
    record[offset..offset + 2].copy_from_slice(&point[0].to_le_bytes());
    record[offset + 2..offset + 4].copy_from_slice(&point[1].to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> [u8; ACTOR_RECORD_BYTES] {
        let mut bytes = [0xA5; ACTOR_RECORD_BYTES];
        bytes[FLAGS_71] = 0;
        bytes[FLAGS_57] = 0x40;
        bytes[STATE_82] = 0;
        bytes
    }

    #[test]
    fn applies_only_proved_fields_and_returns_staged_and_resolved_points() {
        let mut bytes = record();
        let before = bytes;
        let result = apply_opcode3_move_order(
            19,
            &mut bytes,
            [120, -41],
            PrecomputedResolvedTarget::new([118, -38]),
        )
        .unwrap();
        assert_eq!(
            result,
            Opcode3MoveOrderResult::Applied {
                requested: [120, -41],
                resolved: [118, -38],
            }
        );
        assert_eq!(bytes[FLAGS_57], before[FLAGS_57] | 0x02);
        assert_eq!(bytes[BYTE_5B], 0);
        assert_eq!(
            read_point(&bytes, TARGET_5C),
            [118, -38],
            "resolved point replaces the staged request"
        );
        assert_eq!(read_point(&bytes, DESTINATION_0B), [118, -38]);
        for index in 0..bytes.len() {
            if index == FLAGS_57
                || index == BYTE_5B
                || (TARGET_5C..TARGET_5C + 4).contains(&index)
                || (DESTINATION_0B..DESTINATION_0B + 4).contains(&index)
            {
                continue;
            }
            assert_eq!(bytes[index], before[index], "unexpected write at {index:#x}");
        }
    }

    #[test]
    fn flag71_noop_precedes_reinitialization_gate_and_preserves_record() {
        let mut bytes = record();
        bytes[FLAGS_71] = 0x10;
        bytes[FLAGS_57] |= 1;
        bytes[STATE_82] = 3;
        let before = bytes;
        assert_eq!(
            apply_opcode3_move_order(
                0,
                &mut bytes,
                [4, 5],
                PrecomputedResolvedTarget::new([6, 7]),
            ),
            Ok(Opcode3MoveOrderResult::NoopFlag71)
        );
        assert_eq!(bytes, before);
    }

    #[test]
    fn reinitialization_and_bad_actor_inputs_are_atomic() {
        for (flags_57, state_82) in [(1, 0), (0, 9)] {
            let mut bytes = record();
            bytes[FLAGS_57] = flags_57;
            bytes[STATE_82] = state_82;
            let before = bytes;
            assert!(matches!(
                apply_opcode3_move_order(
                    3,
                    &mut bytes,
                    [1, 2],
                    PrecomputedResolvedTarget::new([3, 4]),
                ),
                Err(Opcode3MoveOrderError::ReinitializationRequired { .. })
            ));
            assert_eq!(bytes, before);
        }

        let mut bytes = record();
        let before = bytes;
        assert_eq!(
            apply_opcode3_move_order(
                -1,
                &mut bytes,
                [0, 0],
                PrecomputedResolvedTarget::new([0, 0]),
            ),
            Err(Opcode3MoveOrderError::InvalidActorId(-1))
        );
        assert_eq!(bytes, before);
        assert!(matches!(
            apply_opcode3_move_order(
                1,
                &mut bytes[..ACTOR_RECORD_BYTES - 1],
                [0, 0],
                PrecomputedResolvedTarget::new([0, 0]),
            ),
            Err(Opcode3MoveOrderError::ActorRecordLength { .. })
        ));
        assert_eq!(bytes, before);
    }

    #[test]
    fn bounded_resolver_matches_radius_boundary_and_clear_far_point() {
        let blocked = ResolverEnvironment {
            terrain_class: 7,
            actor_avoidance_mask: 0xFF,
            obstacle_present: true,
            blocker_weight: 255,
        };
        assert_eq!(
            resolve_bounded_move_target([0, 0], [64, 0], blocked).unwrap().position(),
            [0, 0],
            "distance squared 4096 snaps before accepted-destination predicates"
        );
        let clear = ResolverEnvironment {
            terrain_class: 2,
            actor_avoidance_mask: 0,
            obstacle_present: false,
            blocker_weight: 0,
        };
        assert_eq!(
            resolve_bounded_move_target([0, 0], [65, 0], clear).unwrap().position(),
            [65, 0]
        );
        assert_eq!(
            resolve_bounded_move_target(
                [0, 0],
                [64, 1],
                ResolverEnvironment { blocker_weight: 9, ..clear },
            )
            .unwrap()
            .position(),
            [64, 1],
            "distance squared 4097 is outside the snap radius"
        );
    }

    #[test]
    fn bounded_resolver_wraps_signed15_coordinates_and_rejects_search_cases() {
        let blocked_but_near = ResolverEnvironment {
            terrain_class: 0,
            actor_avoidance_mask: 0xFF,
            obstacle_present: true,
            blocker_weight: 255,
        };
        assert_eq!(
            resolve_bounded_move_target(
                [32767, -32768],
                [-32768, 32767],
                blocked_but_near,
            )
            .unwrap()
            .position(),
            [32767, -32768]
        );
        for environment in [
            ResolverEnvironment { terrain_class: 2, actor_avoidance_mask: 4, obstacle_present: false, blocker_weight: 0 },
            ResolverEnvironment { terrain_class: 2, actor_avoidance_mask: 0, obstacle_present: false, blocker_weight: 10 },
            ResolverEnvironment { terrain_class: 2, actor_avoidance_mask: 0, obstacle_present: true, blocker_weight: 0 },
        ] {
            assert_eq!(
                resolve_bounded_move_target([0, 0], [65, 0], environment),
                Err(ResolverError::BackwardSearchRequired)
            );
        }
        assert_eq!(
            resolve_bounded_move_target(
                [0, 0],
                [65, 0],
                ResolverEnvironment { terrain_class: 8, ..blocked_but_near },
            ),
            Err(ResolverError::InvalidTerrainClass(8))
        );
        assert_eq!(
            resolve_bounded_move_target(
                [0, 0],
                [65, 0],
                ResolverEnvironment { blocker_weight: -1, ..blocked_but_near },
            ),
            Err(ResolverError::NegativeBlockerWeight(-1))
        );
    }

    fn read_point(record: &[u8], offset: usize) -> [i16; 2] {
        [
            i16::from_le_bytes([record[offset], record[offset + 1]]),
            i16::from_le_bytes([record[offset + 2], record[offset + 3]]),
        ]
    }
}

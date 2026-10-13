//! Explicit-seed placement and bounded ready-state activation plans.
//!
//! This module models only the proved `0x73E00` candidate generator and the
//! accepted-point branch of `0x80AE0`. It does not recreate landing physics,
//! earlier shared-RNG consumers, the complete retry loop, or an initial
//! controllable actor. See `research/ghidra/actor-activation-evidence.json`
//! and `research/ghidra/actor-activation-bridge.md` for the pinned contract.

use core::fmt;

pub const TRIG_TABLE_ENTRIES: usize = 2048;
pub const TRIG_TABLE_BYTES: usize = TRIG_TABLE_ENTRIES * 4;
const ACTOR_BYTES: usize = 0xC8;
const ACTOR_PARTITION_SLOTS: usize = 5;
const BLUEPRINT_COUNT: usize = 32;
const BLUEPRINT_BYTES: usize = 36;
const MAP_BYTES: usize = 128 * 128 * 14;
const MAP_CELL_BYTES: usize = 14;
const MAP_HEAD_OFFSET: usize = 6;
pub const MAX_CANDIDATE_ATTEMPTS: usize = 4096;
pub const SINE_TABLE_SHA256: &str = "84dfe6902e4c8947b75f4ff0a54d82fced478dd9682255eaa973a88cbf76cf3c";
pub const COSINE_TABLE_SHA256: &str = "688e629369dc1c5379bae04b5e11100a4fd0aabd89745f2c0b5b6309ce0261c9";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorActivationError {
    TrigTableLength { name: &'static str, expected: usize, actual: usize },
    TrigTableHash { name: &'static str, expected: &'static str, actual: String },
    ActorRecordLength { expected: usize, actual: usize },
    BlueprintTableLength { expected: usize, actual: usize },
    MapLength { expected: usize, actual: usize },
    InvalidSlotIndex(usize),
    InvalidActorId(i16),
    ActorIdMismatch { parameter: i16, record: i16 },
    ActorSlotMismatch { actor_id: i16, slot_index: usize },
    UnsupportedActorType(u8),
    AssociatedType5RouteUnsupported,
    InvalidBlueprintIndex(i8),
    InvalidCenter([u16; 2]),
    InvalidTerrainClass(u8),
    NegativeBlockerWeight(i32),
    CandidateListExhausted { attempts: usize },
    InvalidReference { index: i16, kind: u8 },
    MissingResolvedCellHead { index: i16, kind: u8 },
    UnexpectedResolvedCellHead,
    ResolvedCellHeadMismatch { raw_index: i16, raw_kind: u8, record_id: i16, record_kind: u8 },
    AllocationFailed,
}

impl fmt::Display for ActorActivationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for ActorActivationError {}

/// Both 2048-entry signed 16.16 tables, admitted only by the exact pinned
/// mapped-object hashes from the activation proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrigTables {
    sine: Vec<i32>,
    cosine: Vec<i32>,
}

impl TrigTables {
    pub fn from_mapped_bytes(sine: &[u8], cosine: &[u8]) -> Result<Self, ActorActivationError> {
        verify_table("sine", sine, SINE_TABLE_SHA256)?;
        verify_table("cosine", cosine, COSINE_TABLE_SHA256)?;
        let sine = sine.chunks_exact(4).map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect();
        let cosine = cosine.chunks_exact(4).map(|chunk| i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])).collect();
        Ok(Self { sine, cosine })
    }

    /// Generate the table values from the proven truncation rule and admit
    /// them only if the result exactly matches the pinned executable tables.
    /// If the host math library differs at a truncation boundary, this returns
    /// a hash error instead of silently using different placement geometry.
    pub fn generate_pinned() -> Result<Self, ActorActivationError> {
        let mut sine = Vec::new();
        let mut cosine = Vec::new();
        sine.try_reserve_exact(TRIG_TABLE_BYTES).map_err(|_| ActorActivationError::AllocationFailed)?;
        cosine.try_reserve_exact(TRIG_TABLE_BYTES).map_err(|_| ActorActivationError::AllocationFailed)?;
        let tau = core::f64::consts::TAU;
        for index in 0..TRIG_TABLE_ENTRIES {
            let angle = index as f64 * tau / TRIG_TABLE_ENTRIES as f64;
            sine.extend_from_slice(&((65_536.0 * angle.sin()) as i32).to_le_bytes());
            cosine.extend_from_slice(&((65_536.0 * angle.cos()) as i32).to_le_bytes());
        }
        Self::from_mapped_bytes(&sine, &cosine)
    }

    pub fn sine(&self) -> &[i32] { &self.sine }
    pub fn cosine(&self) -> &[i32] { &self.cosine }
}

fn verify_table(name: &'static str, bytes: &[u8], expected_hash: &'static str) -> Result<(), ActorActivationError> {
    if bytes.len() != TRIG_TABLE_BYTES {
        return Err(ActorActivationError::TrigTableLength { name, expected: TRIG_TABLE_BYTES, actual: bytes.len() });
    }
    let actual = crate::identity::sha256_hex(bytes);
    if actual != expected_hash {
        return Err(ActorActivationError::TrigTableHash { name, expected: expected_hash, actual });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementCandidate {
    pub seed_before: u16,
    pub seed_after: u16,
    pub angle_index: u16,
    pub position_u15: [u16; 2],
}

/// Consume exactly one shared 16-bit RNG draw and sample one point at radius
/// 512 around an explicit u15 center. No ambient or serialized seed is read.
pub fn sample_placement_candidate(
    seed_before: u16,
    center_u15: [u16; 2],
    tables: &TrigTables,
) -> Result<PlacementCandidate, ActorActivationError> {
    if center_u15.iter().any(|&value| value > 0x7FFF) {
        return Err(ActorActivationError::InvalidCenter(center_u15));
    }
    let seed_after = seed_before.wrapping_mul(0xE62D);
    let angle_index = ((seed_after >> 8) * 8) as u16;
    let cosine_index = angle_index.wrapping_sub(512) & 2047;
    let dx = ((1024i32 * tables.cosine[usize::from(cosine_index)]) >> 16) as i16;
    let dy = ((1024i32 * tables.sine[usize::from(cosine_index)]) >> 16) as i16;
    let x_doubled = (center_u15[0] as i16).wrapping_mul(2);
    let y_doubled = (center_u15[1] as i16).wrapping_mul(2);
    let x = (x_doubled.wrapping_add(dx) >> 1) as u16 & 0x7FFF;
    let y = (y_doubled.wrapping_add(dy) >> 1) as u16 & 0x7FFF;
    Ok(PlacementCandidate { seed_before, seed_after, angle_index, position_u15: [x, y] })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateEnvironment {
    pub terrain_class: u8,
    /// Result of the source's cell-plant blocker helper; no world query is
    /// performed by this module.
    pub blocker_weight: i32,
    /// Identity read from the resolved old-head entity record at the sampled
    /// candidate cell. This is required only for the accepted candidate when
    /// the raw cell head is nonnull; it is checked against the raw triple.
    pub resolved_cell_head: Option<ResolvedCellHead>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedCellHead {
    pub record_id: i16,
    pub record_type: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateAttemptReceipt {
    pub candidate: PlacementCandidate,
    pub environment: CandidateEnvironment,
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivationPlanningOutcome {
    Ineligible { actor_id: i16, slot_index: usize },
    Planned(ActorActivationPlan),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellLinkPlan {
    pub cell_index: u16,
    pub expected_old_head: [u8; 3],
    pub actor_id: i16,
    pub actor_type: u8,
    pub actor_previous_after: [u8; 3],
    pub actor_next_after: [u8; 3],
    pub new_cell_head: [u8; 3],
    pub old_head_previous_after: Option<[u8; 3]>,
}

/// Complete per-slot activation writes as after-images. The caller must
/// validate these expected snapshots before applying them to its world and
/// must separately finish the five-slot player batch (+0x49D increment).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorActivationPlan {
    pub actor_id: i16,
    pub slot_index: usize,
    pub candidate_attempts: Vec<CandidateAttemptReceipt>,
    pub seed_before: u16,
    pub seed_after: u16,
    pub actor_before: Vec<u8>,
    pub actor_after: Vec<u8>,
    pub blueprint_index: usize,
    pub blueprint_before: [u8; BLUEPRINT_BYTES],
    pub blueprint_after: [u8; BLUEPRINT_BYTES],
    pub link: CellLinkPlan,
}

/// Plan one ready-state actor slot from explicit candidate-environment
/// results. Candidate draws are consumed in order until the class mask and
/// blocker weight admit a point. The source loop is unbounded; this safe port
/// accepts at most 4096 candidate results per call. No actor, blueprint, map,
/// or RNG state is mutated here; exhaustion returns an error and no partial
/// plan.
///
/// `associated_type5_absent` must be true to select the proved no-associated
/// unit route. The associated-type5 resolver route is intentionally excluded.
/// `map` is the exact 128x128x14-byte terrain-map region. The returned link
/// plan carries the old head bytes so the world owner can revalidate before
/// applying its raw-reference-preserving writes. For a nonnull head, the
/// caller must resolve the family array, validate its family bound, and pass
/// the record's +0x0F ID and +6 type in the accepted `CandidateEnvironment`;
/// kind/index plausibility alone is insufficient.
pub fn plan_actor_activation(
    actor_id: i16,
    slot_index: usize,
    actor_record: &[u8],
    blueprint_table: &[u8],
    map: &[u8],
    center_u15: [u16; 2],
    seed_before: u16,
    tables: &TrigTables,
    candidate_environments: &[CandidateEnvironment],
    associated_type5_absent: bool,
) -> Result<ActivationPlanningOutcome, ActorActivationError> {
    if !(0..20).contains(&actor_id) {
        return Err(ActorActivationError::InvalidActorId(actor_id));
    }
    if slot_index >= ACTOR_PARTITION_SLOTS {
        return Err(ActorActivationError::InvalidSlotIndex(slot_index));
    }
    if actor_record.len() != ACTOR_BYTES {
        return Err(ActorActivationError::ActorRecordLength { expected: ACTOR_BYTES, actual: actor_record.len() });
    }

    let flags70 = u16::from_le_bytes([actor_record[0x70], actor_record[0x71]]);
    if actor_record[6] == 0 || flags70 & 0x0400 == 0 {
        return Ok(ActivationPlanningOutcome::Ineligible { actor_id, slot_index });
    }
    if actor_record[6] != 3 {
        return Err(ActorActivationError::UnsupportedActorType(actor_record[6]));
    }
    if usize::try_from(actor_id).expect("actor ID was range checked") % ACTOR_PARTITION_SLOTS != slot_index {
        return Err(ActorActivationError::ActorSlotMismatch { actor_id, slot_index });
    }
    let record_actor_id = i16::from_le_bytes([actor_record[0x0F], actor_record[0x10]]);
    if record_actor_id != actor_id {
        return Err(ActorActivationError::ActorIdMismatch { parameter: actor_id, record: record_actor_id });
    }
    if !associated_type5_absent {
        return Err(ActorActivationError::AssociatedType5RouteUnsupported);
    }
    if blueprint_table.len() != BLUEPRINT_COUNT * BLUEPRINT_BYTES {
        return Err(ActorActivationError::BlueprintTableLength {
            expected: BLUEPRINT_COUNT * BLUEPRINT_BYTES,
            actual: blueprint_table.len(),
        });
    }
    if map.len() != MAP_BYTES {
        return Err(ActorActivationError::MapLength { expected: MAP_BYTES, actual: map.len() });
    }
    if center_u15.iter().any(|&value| value > 0x7FFF) {
        return Err(ActorActivationError::InvalidCenter(center_u15));
    }
    if candidate_environments.len() > MAX_CANDIDATE_ATTEMPTS {
        return Err(ActorActivationError::CandidateListExhausted { attempts: MAX_CANDIDATE_ATTEMPTS });
    }

    let blueprint_index_i8 = actor_record[0x74] as i8;
    if !(0..BLUEPRINT_COUNT as i8).contains(&blueprint_index_i8) {
        return Err(ActorActivationError::InvalidBlueprintIndex(blueprint_index_i8));
    }
    let blueprint_index = usize::from(blueprint_index_i8 as u8);
    let blueprint_start = blueprint_index * BLUEPRINT_BYTES;
    let blueprint_before: [u8; BLUEPRINT_BYTES] = blueprint_table[blueprint_start..blueprint_start + BLUEPRINT_BYTES]
        .try_into().expect("validated fixed blueprint record extent");
    let avoidance_mask = actor_record[0x53];
    let mut seed = seed_before;
    let mut attempts = Vec::new();
    attempts.try_reserve_exact(candidate_environments.len()).map_err(|_| ActorActivationError::AllocationFailed)?;
    let mut accepted = None;
    for &environment in candidate_environments {
        if environment.terrain_class > 7 {
            return Err(ActorActivationError::InvalidTerrainClass(environment.terrain_class));
        }
        if environment.blocker_weight < 0 {
            return Err(ActorActivationError::NegativeBlockerWeight(environment.blocker_weight));
        }
        let candidate = sample_placement_candidate(seed, center_u15, tables)?;
        seed = candidate.seed_after;
        let class_bit = 1u8 << environment.terrain_class;
        let is_accepted = avoidance_mask & class_bit == 0 && environment.blocker_weight < 10;
        attempts.push(CandidateAttemptReceipt { candidate, environment, accepted: is_accepted });
        if is_accepted {
            accepted = Some((candidate, environment));
            break;
        }
    }
    let Some((candidate, accepted_environment)) = accepted else {
        return Err(ActorActivationError::CandidateListExhausted { attempts: candidate_environments.len() });
    };

    let x = candidate.position_u15[0];
    let y = candidate.position_u15[1];
    let map_x = usize::from((x >> 8) & 127);
    let map_y = usize::from((y >> 8) & 127);
    let cell_index = (map_y * 128 + map_x) as u16;
    let cell_start = usize::from(cell_index) * MAP_CELL_BYTES;
    let old_head = [map[cell_start + MAP_HEAD_OFFSET], map[cell_start + MAP_HEAD_OFFSET + 1], map[cell_start + MAP_HEAD_OFFSET + 2]];
    let old_head_index = i16::from_le_bytes([old_head[0], old_head[1]]);
    let old_head_kind = old_head[2];
    let resolved_head = match (old_head_kind == 0, accepted_environment.resolved_cell_head) {
        (true, None) => None,
        (true, Some(_)) => return Err(ActorActivationError::UnexpectedResolvedCellHead),
        (false, None) => return Err(ActorActivationError::MissingResolvedCellHead { index: old_head_index, kind: old_head_kind }),
        (false, Some(resolved)) => {
            if old_head_index < 0 || !(1..=8).contains(&old_head_kind) {
                return Err(ActorActivationError::InvalidReference { index: old_head_index, kind: old_head_kind });
            }
            if resolved.record_id != old_head_index || resolved.record_type != old_head_kind {
                return Err(ActorActivationError::ResolvedCellHeadMismatch {
                    raw_index: old_head_index, raw_kind: old_head_kind,
                    record_id: resolved.record_id, record_kind: resolved.record_type,
                });
            }
            Some(resolved)
        }
    };

    let mut actor_before = Vec::new();
    actor_before.try_reserve_exact(ACTOR_BYTES).map_err(|_| ActorActivationError::AllocationFailed)?;
    actor_before.extend_from_slice(actor_record);
    let mut actor_after = actor_before.clone();
    let mut blueprint_after = blueprint_before;
    let actor_type = actor_after[6];
    let actor_ref = { let id = actor_id.to_le_bytes(); [id[0], id[1], actor_type] };
    let cell_id = cell_index.to_le_bytes();
    let actor_previous_after = [cell_id[0], cell_id[1], 0];
    let (actor_next_after, old_head_previous_after) = if let Some(resolved) = resolved_head {
        let id = resolved.record_id.to_le_bytes();
        ([id[0], id[1], resolved.record_type], Some(actor_ref))
    } else {
        ([actor_after[3], actor_after[4], 0], None)
    };
    actor_after[0..3].copy_from_slice(&actor_previous_after);
    actor_after[3..6].copy_from_slice(&actor_next_after);
    let new_cell_head = actor_ref;

    let flags_after = (flags70 & 0xFBFE) | 0x0001;
    actor_after[0x70..0x72].copy_from_slice(&flags_after.to_le_bytes());
    actor_after[0x16] &= !0x08;
    actor_after[0x07..0x0B].copy_from_slice(&[
        candidate.position_u15[0].to_le_bytes()[0], candidate.position_u15[0].to_le_bytes()[1],
        candidate.position_u15[1].to_le_bytes()[0], candidate.position_u15[1].to_le_bytes()[1],
    ]);

    let blueprint_category = blueprint_before[0x16];
    let blueprint_count = u32::from_le_bytes([
        blueprint_before[0x1C], blueprint_before[0x1D], blueprint_before[0x1E], blueprint_before[0x1F],
    ]).wrapping_add(1);
    // 0x80AE0 follows the actor's native C0 blueprint handle and writes the
    // blueprint's byte+18 back to blueprint+1B. The actor's own frame byte
    // remains untouched.
    blueprint_after[0x1B] = blueprint_before[0x18];
    blueprint_after[0x20] |= 0x02;
    blueprint_after[0x1C..0x20].copy_from_slice(&blueprint_count.to_le_bytes());
    if blueprint_count == 1 && blueprint_category == 1 { actor_after[0x72] = 40; }

    let link = CellLinkPlan {
        cell_index,
        expected_old_head: old_head,
        actor_id,
        actor_type,
        actor_previous_after,
        actor_next_after,
        new_cell_head,
        old_head_previous_after,
    };
    Ok(ActivationPlanningOutcome::Planned(ActorActivationPlan {
        actor_id,
        slot_index,
        candidate_attempts: attempts,
        seed_before,
        seed_after: candidate.seed_after,
        actor_before,
        actor_after,
        blueprint_index,
        blueprint_before,
        blueprint_after,
        link,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_map() -> Vec<u8> { vec![0; MAP_BYTES] }

    fn eligible_actor(actor_id: i16, selector: u8, avoid_mask: u8) -> Vec<u8> {
        let mut actor = vec![0xA5; ACTOR_BYTES];
        actor[6] = 3;
        actor[0x0F..0x11].copy_from_slice(&actor_id.to_le_bytes());
        actor[0x53] = avoid_mask;
        actor[0x70..0x72].copy_from_slice(&0x0400u16.to_le_bytes());
        actor[0x74] = selector;
        actor
    }

    fn blueprint_table() -> Vec<u8> {
        let mut table = vec![0; BLUEPRINT_COUNT * BLUEPRINT_BYTES];
        table[0x16] = 1;
        table[0x18] = 9;
        table
    }

    #[test]
    fn generated_tables_are_hash_admitted_and_candidate_vectors_match() {
        let tables = TrigTables::generate_pinned().expect("pinned trig tables");
        assert_eq!(tables.sine().len(), 2048);
        assert_eq!(tables.cosine().len(), 2048);
        for (seed, expected_after, expected_angle, expected) in [
            (0, 0, 0, [20736, 13056]),
            (1, 58925, 1840, [20431, 13156]),
            (5428, 29220, 912, [20908, 14050]),
            (65535, 6611, 200, [21030, 13149]),
        ] {
            let candidate = sample_placement_candidate(seed, [20736, 13568], &tables).unwrap();
            assert_eq!(candidate.seed_after, expected_after);
            assert_eq!(candidate.angle_index, expected_angle);
            assert_eq!(candidate.position_u15, expected);
        }
    }

    #[test]
    fn mapped_table_admission_rejects_wrong_length_and_hash() {
        let tables = TrigTables::generate_pinned().unwrap();
        let mut sine = Vec::with_capacity(8192);
        for value in tables.sine() { sine.extend_from_slice(&value.to_le_bytes()); }
        let mut cosine = Vec::with_capacity(8192);
        for value in tables.cosine() { cosine.extend_from_slice(&value.to_le_bytes()); }
        assert_eq!(TrigTables::from_mapped_bytes(&sine[..8191], &cosine).unwrap_err(), ActorActivationError::TrigTableLength {
            name: "sine", expected: 8192, actual: 8191,
        });
        sine[0] ^= 1;
        assert!(matches!(TrigTables::from_mapped_bytes(&sine, &cosine), Err(ActorActivationError::TrigTableHash { name: "sine", .. })));
    }

    #[test]
    fn activation_retries_explicitly_then_emits_raw_link_and_metadata_plan() {
        let tables = TrigTables::generate_pinned().unwrap();
        let actor = eligible_actor(4, 0, 1);
        let blueprint = blueprint_table();
        let map = zero_map();
        let outcome = plan_actor_activation(
            4, 4, &actor, &blueprint, &map, [20736, 13568], 1, &tables,
            &[
                CandidateEnvironment { terrain_class: 0, blocker_weight: 0, resolved_cell_head: None },
                CandidateEnvironment { terrain_class: 1, blocker_weight: 9, resolved_cell_head: None },
            ], true,
        ).unwrap();
        let ActivationPlanningOutcome::Planned(plan) = outcome else { panic!("eligible actor must plan") };
        assert_eq!(plan.candidate_attempts.len(), 2);
        assert!(!plan.candidate_attempts[0].accepted);
        assert!(plan.candidate_attempts[1].accepted);
        assert_eq!(plan.seed_before, 1);
        assert_eq!(plan.seed_after, 58345);
        assert_eq!(&plan.actor_before, &actor);
        assert_eq!(&plan.actor_after[0x07..0x0B], &[20401u16.to_le_bytes(), 13180u16.to_le_bytes()].concat());
        assert_eq!(u16::from_le_bytes([plan.actor_after[0x70], plan.actor_after[0x71]]), 1);
        assert_eq!(plan.actor_after[0x16] & 8, 0);
        assert_eq!(plan.actor_after[0x11], 0xA5); // height remains opaque
        assert_eq!(plan.actor_after[0x17], 0xA5); // shade remains opaque
        assert_eq!(plan.actor_after[0x1B], 0xA5); // actor frame remains opaque
        assert_eq!(plan.blueprint_after[0x1B], 9);
        assert_eq!(plan.actor_after[0x72], 40);
        assert_eq!(plan.blueprint_after[0x20] & 2, 2);
        assert_eq!(u32::from_le_bytes(plan.blueprint_after[0x1C..0x20].try_into().unwrap()), 1);
        assert_eq!(plan.link.actor_id, 4);
        assert_eq!(plan.link.actor_type, 3);
        assert_eq!(plan.link.expected_old_head, [0, 0, 0]);
        assert_eq!(plan.link.actor_next_after, [0xA5, 0xA5, 0]); // preserve null index bytes
        assert_eq!(plan.link.new_cell_head, [4, 0, 3]);
    }

    #[test]
    fn nonempty_cell_head_is_retained_and_null_next_kind_only_is_cleared() {
        let tables = TrigTables::generate_pinned().unwrap();
        let actor = eligible_actor(0, 0, 0);
        let blueprint = blueprint_table();
        let mut map = zero_map();
        let candidate = sample_placement_candidate(0, [20736, 13568], &tables).unwrap();
        let cell = usize::from((candidate.position_u15[1] >> 8) * 128 + (candidate.position_u15[0] >> 8));
        let start = cell * MAP_CELL_BYTES + MAP_HEAD_OFFSET;
        map[start..start + 3].copy_from_slice(&[7, 0, 2]);
        let outcome = plan_actor_activation(0, 0, &actor, &blueprint, &map, [20736, 13568], 0, &tables,
            &[CandidateEnvironment {
                terrain_class: 0,
                blocker_weight: 0,
                resolved_cell_head: Some(ResolvedCellHead { record_id: 7, record_type: 2 }),
            }], true).unwrap();
        let ActivationPlanningOutcome::Planned(plan) = outcome else { panic!("expected plan") };
        assert_eq!(plan.link.expected_old_head, [7, 0, 2]);
        assert_eq!(plan.link.actor_next_after, [7, 0, 2]);
        assert_eq!(plan.link.old_head_previous_after, Some([0, 0, 3]));
        assert_eq!(plan.link.actor_previous_after, [(cell as u16).to_le_bytes()[0], (cell as u16).to_le_bytes()[1], 0]);
    }

    #[test]
    fn nonnull_head_requires_a_resolved_identity_matching_the_raw_cell_head() {
        let tables = TrigTables::generate_pinned().unwrap();
        let actor = eligible_actor(0, 0, 0);
        let blueprint = blueprint_table();
        let mut map = zero_map();
        let candidate = sample_placement_candidate(0, [20736, 13568], &tables).unwrap();
        let cell = usize::from((candidate.position_u15[1] >> 8) * 128 + (candidate.position_u15[0] >> 8));
        let start = cell * MAP_CELL_BYTES + MAP_HEAD_OFFSET;
        map[start..start + 3].copy_from_slice(&[7, 0, 2]);
        let env = |head| CandidateEnvironment { terrain_class: 0, blocker_weight: 0, resolved_cell_head: head };
        let missing = plan_actor_activation(0, 0, &actor, &blueprint, &map, [20736, 13568], 0, &tables,
            &[env(None)], true).unwrap_err();
        assert_eq!(missing, ActorActivationError::MissingResolvedCellHead { index: 7, kind: 2 });
        let mismatch = plan_actor_activation(0, 0, &actor, &blueprint, &map, [20736, 13568], 0, &tables,
            &[env(Some(ResolvedCellHead { record_id: 8, record_type: 2 }))], true).unwrap_err();
        assert_eq!(mismatch, ActorActivationError::ResolvedCellHeadMismatch {
            raw_index: 7, raw_kind: 2, record_id: 8, record_kind: 2,
        });
    }

    #[test]
    fn ineligible_and_unsupported_routes_do_not_emit_partial_plans() {
        let tables = TrigTables::generate_pinned().unwrap();
        let mut actor = eligible_actor(0, 0, 0);
        actor[6] = 0;
        let outcome = plan_actor_activation(0, 0, &actor, &[], &[], [0, 0], 0, &tables, &[], true).unwrap();
        assert_eq!(outcome, ActivationPlanningOutcome::Ineligible { actor_id: 0, slot_index: 0 });

        actor[6] = 3;
        assert_eq!(plan_actor_activation(0, 0, &actor, &[], &[], [0, 0], 0, &tables, &[], false).unwrap_err(), ActorActivationError::AssociatedType5RouteUnsupported);

        assert_eq!(plan_actor_activation(20, 0, &actor, &[], &[], [0, 0], 0, &tables, &[], true).unwrap_err(), ActorActivationError::InvalidActorId(20));
    }

    #[test]
    fn candidate_exhaustion_is_transactionally_empty() {
        let tables = TrigTables::generate_pinned().unwrap();
        let actor = eligible_actor(0, 0, 1);
        let blueprint = blueprint_table();
        let map = zero_map();
        let before = (actor.clone(), blueprint.clone(), map.clone());
        let result = plan_actor_activation(0, 0, &actor, &blueprint, &map, [1, 1], 1, &tables,
            &[CandidateEnvironment { terrain_class: 0, blocker_weight: 0, resolved_cell_head: None }], true);
        assert_eq!(result.unwrap_err(), ActorActivationError::CandidateListExhausted { attempts: 1 });
        assert_eq!((actor, blueprint, map), before);
    }
}

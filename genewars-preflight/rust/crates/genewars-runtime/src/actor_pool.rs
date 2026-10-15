//! Owned native family-2 actor records for bounded runtime transactions.
//!
//! The admitted CONQ source stores twenty 140-byte records. The 0x57540
//! loader copies bytes [0,31) to native [0,31) and [31,140) to [39,148), leaving
//! native ranges [31,39) and [148,200) to the caller. Records with a nonzero
//! source byte +6 take a separate virtual initialization route and are
//! rejected here. Evidence: `research/ghidra/mission-load-bridge.md` and
//! `research/ghidra/player-creation-bridge.md`.

use core::fmt;
use genewars_assets::ConqMissionSource;

use crate::actor_initialization::{
    plan_no_unit_actor_initialization, ActorInitializationPlan,
    ActorResourceTableKey, BlueprintRecordHandle, ACTOR_HANDLE_BYTE_RANGES,
    ACTOR_INITIALIZATION_BYTES, ACTOR_PARTITION_SLOTS, PLAYER_SLOTS,
};
use crate::player_state::{PlayerState, PlayerStateOrigin, PreparedPlayerActors};

pub const ACTOR_POOL_SLOTS: usize = 20;
pub const NATIVE_ACTOR_BYTES: usize = ACTOR_INITIALIZATION_BYTES;
pub const PACKED_ACTOR_BYTES: usize = 140;
pub const PACKED_ACTOR_GROUP_BYTES: usize = ACTOR_POOL_SLOTS * PACKED_ACTOR_BYTES;
const ACTOR_ID_OFFSET: usize = 0x0F;
const ACTOR_TYPE_OFFSET: usize = 0x06;
const ACTOR_OWNER_OFFSET: usize = 0x18;
const ACTOR_SELECTOR_OFFSET: usize = 0x74;

/// Logical handles for native pointer fields; no process address is stored.
/// The values are copied from an initializer plan or explicitly supplied for
/// records whose native pointers were constructed elsewhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActorHandleBindings {
    pub blueprint: Option<BlueprintRecordHandle>,
    pub resource_database: Option<ActorResourceTableKey>,
    pub animation_database: Option<ActorResourceTableKey>,
    pub internal_actor_offset: Option<u8>,
}

impl ActorHandleBindings {
    pub const fn empty() -> Self {
        Self {
            blueprint: None,
            resource_database: None,
            animation_database: None,
            internal_actor_offset: None,
        }
    }

    fn from_plan(plan: &ActorInitializationPlan) -> Self {
        Self {
            blueprint: Some(BlueprintRecordHandle {
                player_id: plan.player_id,
                blueprint_index: plan.blueprint_index,
            }),
            resource_database: Some(plan.resource_key),
            animation_database: Some(plan.resource_key),
            internal_actor_offset: Some(0xC4),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorPoolError {
    PackedGroupLength { expected: usize, actual: usize },
    MissionSource(genewars_assets::MissionSourceError),
    ActivePackedRecord { slot: usize, active_type: u8 },
    SlotOutOfBounds(usize),
    PlayerOutOfBounds(u8),
    InvalidActorIdentity { id: u16, owner: u8 },
    StaleBeforeImage(usize),
    ActorIdentityChanged(usize),
    PointerBytesChanged { slot: usize, offset: usize },
    InvalidBindings(usize),
    PlayerMismatch { expected: u8, actual: u8 },
    PlayerBeforeImageMismatch,
    PlayerOriginRequired,
    PartitionBeforeImageMismatch(usize),
    PreparedSelectionMismatch,
    InvalidAllocation,
    DuplicateAllocation(usize),
    InitializationMismatch(usize),
    ActorAfterImageMismatch(usize),
    PlayerAfterImageMismatch,
}

impl fmt::Display for ActorPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ActorPoolError {}

impl From<genewars_assets::MissionSourceError> for ActorPoolError {
    fn from(value: genewars_assets::MissionSourceError) -> Self { Self::MissionSource(value) }
}

/// Twenty private native records and their separately typed pointer bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorPool {
    records: [[u8; NATIVE_ACTOR_BYTES]; ACTOR_POOL_SLOTS],
    bindings: [ActorHandleBindings; ACTOR_POOL_SLOTS],
}

impl ActorPool {
    /// Test-only callback replay; caller validates identities, bindings, links,
    /// counts and exact captured-buffer roundtrip before invoking gameplay.
    #[cfg(test)]
    pub(crate) fn from_captured_records(records: [[u8; 200]; 20], bindings: [ActorHandleBindings; 20]) -> Self {
        Self { records, bindings }
    }

    /// Load the family-2 group from an already admitted mission source.
    /// `native_prior` supplies the two destination ranges the file does not
    /// serialize; `bindings` similarly supplies any pre-existing logical
    /// pointer handles. Neither is guessed or cleared by the loader.
    pub fn from_mission_source(
        source: &ConqMissionSource<'_>,
        native_prior: &[[u8; NATIVE_ACTOR_BYTES]; ACTOR_POOL_SLOTS],
        bindings: [ActorHandleBindings; ACTOR_POOL_SLOTS],
    ) -> Result<Self, ActorPoolError> {
        let group = source.group(2)?;
        Self::from_packed_inactive(group.bytes(), native_prior, bindings)
    }

    /// Apply the exact copy shape of 0x57540 after requiring all twenty packed
    /// records to stay on the inactive/no-virtual-initializer path.
    pub fn from_packed_inactive(
        packed: &[u8],
        native_prior: &[[u8; NATIVE_ACTOR_BYTES]; ACTOR_POOL_SLOTS],
        bindings: [ActorHandleBindings; ACTOR_POOL_SLOTS],
    ) -> Result<Self, ActorPoolError> {
        if packed.len() != PACKED_ACTOR_GROUP_BYTES {
            return Err(ActorPoolError::PackedGroupLength {
                expected: PACKED_ACTOR_GROUP_BYTES,
                actual: packed.len(),
            });
        }
        let mut records = *native_prior;
        for slot in 0..ACTOR_POOL_SLOTS {
            let source_start = slot * PACKED_ACTOR_BYTES;
            let source = &packed[source_start..source_start + PACKED_ACTOR_BYTES];
            if source[ACTOR_TYPE_OFFSET] != 0 {
                return Err(ActorPoolError::ActivePackedRecord {
                    slot,
                    active_type: source[ACTOR_TYPE_OFFSET],
                });
            }
            records[slot][..31].copy_from_slice(&source[..31]);
            records[slot][39..148].copy_from_slice(&source[31..140]);
        }
        Ok(Self { records, bindings })
    }

    pub fn record(&self, id: usize) -> Result<&[u8; NATIVE_ACTOR_BYTES], ActorPoolError> {
        self.records.get(id).ok_or(ActorPoolError::SlotOutOfBounds(id))
    }

    pub fn bindings(&self, id: usize) -> Result<ActorHandleBindings, ActorPoolError> {
        self.bindings.get(id).copied().ok_or(ActorPoolError::SlotOutOfBounds(id))
    }

    pub fn records(&self) -> &[[u8; NATIVE_ACTOR_BYTES]; ACTOR_POOL_SLOTS] { &self.records }

    pub fn partition(&self, player: u8) -> Result<[[u8; NATIVE_ACTOR_BYTES]; ACTOR_PARTITION_SLOTS], ActorPoolError> {
        if player >= PLAYER_SLOTS as u8 { return Err(ActorPoolError::PlayerOutOfBounds(player)); }
        let start = usize::from(player) * ACTOR_PARTITION_SLOTS;
        Ok(core::array::from_fn(|i| self.records[start + i]))
    }

    /// Replace a record only when its complete before-image still matches.
    /// This is used for proved link-byte changes. Actor identity and all four
    /// native pointer placeholders must remain stable; typed bindings are
    /// retained unchanged. Initialization uses the atomic partition commit
    /// below to install new typed handles.
    pub(crate) fn replace_record_checked(
        &mut self,
        id: usize,
        expected: &[u8; NATIVE_ACTOR_BYTES],
        after: &[u8; NATIVE_ACTOR_BYTES],
    ) -> Result<(), ActorPoolError> {
        let current = self.records.get(id).ok_or(ActorPoolError::SlotOutOfBounds(id))?;
        if current != expected { return Err(ActorPoolError::StaleBeforeImage(id)); }
        validate_identity(id, expected)?;
        validate_identity(id, after)?;
        if expected[ACTOR_TYPE_OFFSET] != after[ACTOR_TYPE_OFFSET]
            || expected[ACTOR_ID_OFFSET..ACTOR_ID_OFFSET + 2] != after[ACTOR_ID_OFFSET..ACTOR_ID_OFFSET + 2]
            || expected[ACTOR_OWNER_OFFSET] != after[ACTOR_OWNER_OFFSET]
            || expected[ACTOR_SELECTOR_OFFSET] != after[ACTOR_SELECTOR_OFFSET]
        {
            return Err(ActorPoolError::ActorIdentityChanged(id));
        }
        for (start, end) in ACTOR_HANDLE_BYTE_RANGES {
            if expected[start..end] != after[start..end] {
                return Err(ActorPoolError::PointerBytesChanged { slot: id, offset: start });
            }
        }
        self.records[id] = *after;
        Ok(())
    }

    /// Atomically commit the verified no-associated-unit initialization for
    /// one fresh player's five-slot partition. Both player and actor inputs
    /// are explicit beforeimages, and all actor/player afterimages are
    /// independently reconstructed before either owned object is mutated.
    pub fn commit_prepared_partition(
        &mut self,
        player: &mut PlayerState,
        expected_player: &PlayerState,
        expected_partition: &[[u8; NATIVE_ACTOR_BYTES]; ACTOR_PARTITION_SLOTS],
        prepared: &PreparedPlayerActors,
    ) -> Result<Vec<u8>, ActorPoolError> {
        let player_id = expected_player.player_id();
        if player_id >= PLAYER_SLOTS as u8 { return Err(ActorPoolError::PlayerOutOfBounds(player_id)); }
        if player.player_id() != player_id { return Err(ActorPoolError::PlayerMismatch { expected: player_id, actual: player.player_id() }); }
        if *player != *expected_player { return Err(ActorPoolError::PlayerBeforeImageMismatch); }
        if expected_player.origin() == PlayerStateOrigin::PreparedFreshActors
            || prepared.player.origin() != PlayerStateOrigin::PreparedFreshActors
        { return Err(ActorPoolError::PlayerOriginRequired); }

        let first_global = usize::from(player_id) * ACTOR_PARTITION_SLOTS;
        for local in 0..ACTOR_PARTITION_SLOTS {
            if self.records[first_global + local] != expected_partition[local] {
                return Err(ActorPoolError::PartitionBeforeImageMismatch(local));
            }
        }

        validate_prepared(player_id, expected_player, expected_partition, prepared)?;

        // Stage the entire postimage, including logical handles. Nothing below
        // this point can fail, so the two owned states change together.
        let mut next_records = self.records;
        let mut next_bindings = self.bindings;
        let mut committed_ids = Vec::new();
        for init in &prepared.initializations {
            let local = usize::from(init.local_slot);
            let global = first_global + local;
            if self.bindings[global] != ActorHandleBindings::empty() {
                return Err(ActorPoolError::InvalidBindings(global));
            }
            next_records[global] = prepared.actors[local];
            next_bindings[global] = ActorHandleBindings::from_plan(init);
            committed_ids.push(global as u8);
        }
        self.records = next_records;
        self.bindings = next_bindings;
        *player = prepared.player.clone();
        Ok(committed_ids)
    }
}

fn validate_identity(id: usize, record: &[u8; NATIVE_ACTOR_BYTES]) -> Result<(), ActorPoolError> {
    let actor_id = u16::from_le_bytes([record[ACTOR_ID_OFFSET], record[ACTOR_ID_OFFSET + 1]]);
    let owner = record[ACTOR_OWNER_OFFSET];
        if id >= ACTOR_POOL_SLOTS || actor_id as usize != id || owner >= PLAYER_SLOTS as u8
            || record[ACTOR_TYPE_OFFSET] == 0
        || usize::from(owner) * ACTOR_PARTITION_SLOTS > id
        || id >= (usize::from(owner) + 1) * ACTOR_PARTITION_SLOTS
    {
        return Err(ActorPoolError::InvalidActorIdentity { id: actor_id, owner });
    }
    Ok(())
}

fn validate_prepared(
    player_id: u8,
    expected_player: &PlayerState,
    expected_partition: &[[u8; NATIVE_ACTOR_BYTES]; ACTOR_PARTITION_SLOTS],
    prepared: &PreparedPlayerActors,
) -> Result<(), ActorPoolError> {
    let plan = &prepared.selection;
    if prepared.player.player_id() != player_id || plan.target_reset_player != player_id
        || !prepared.landing_factory_pending
        || plan.created_count as usize != prepared.initializations.len()
        || plan.player_count_after as i8 != prepared.player.actor_count()
        || expected_player.creation_mode() != 0 || expected_player.actor_count() != 0
        || (expected_player.progress() != 0 && !(expected_player.progress()==1&&expected_player.origin()==PlayerStateOrigin::RestoredForMission))
    {
        return Err(ActorPoolError::PreparedSelectionMismatch);
    }
    if expected_partition.iter().any(|record| record[ACTOR_TYPE_OFFSET] != 0) {
        return Err(ActorPoolError::PreparedSelectionMismatch);
    }

    // The requested mask is not stored in the public prepared value. Derive
    // the smallest mask that admits its listed categories, then independently
    // rerun the proved selector. This rejects fabricated allocation, slot,
    // ID, owner and blueprint afterimages before any record is committed.
    let mut selection_mask = 0u8;
    for attempt in plan.attempts.iter().flatten() {
        if attempt.category > 3 || attempt.blueprint_index >= 32 {
            return Err(ActorPoolError::PreparedSelectionMismatch);
        }
        let bp = expected_player.blueprint(usize::from(attempt.blueprint_index))
            .map_err(|_| ActorPoolError::PreparedSelectionMismatch)?;
        if bp[0] == 0 || bp[0x16] != attempt.category {
            return Err(ActorPoolError::PreparedSelectionMismatch);
        }
        selection_mask |= 1 << attempt.category;
    }
    let selector = expected_player.creation_plan(selection_mask, true, [0; ACTOR_PARTITION_SLOTS])
        .map_err(|_| ActorPoolError::PreparedSelectionMismatch)?;
    if selector != *plan { return Err(ActorPoolError::PreparedSelectionMismatch); }

    let expected_origin = prepared.initializations.first().map(|p| (p.temporary_origin, p.sampled_origin_height, p.sampled_origin_shade));
    if prepared.initializations.iter().any(|p| {
        p.final_target != prepared.player.target()
            || Some((p.temporary_origin, p.sampled_origin_height, p.sampled_origin_shade)) != expected_origin
    }) {
        return Err(ActorPoolError::InitializationMismatch(usize::MAX));
    }

    let mut next_player = *expected_player.bytes();
    next_player[9..11].copy_from_slice(&prepared.player.target()[0].to_le_bytes());
    next_player[11..13].copy_from_slice(&prepared.player.target()[1].to_le_bytes());
    let mut allocated = [false; ACTOR_PARTITION_SLOTS];
    let mut expected_inits = Vec::new();
    let mut attempts_seen = 0usize;
    for attempt in plan.attempts.iter().flatten() {
        attempts_seen += 1;
        let Some(allocation) = attempt.allocation else { continue; };
        let local = usize::from(allocation.local_slot);
        let list_offset = usize::from(allocation.player_id_list_offset);
        if local >= ACTOR_PARTITION_SLOTS || allocated[local]
            || allocation.owner != player_id || allocation.active_type != 3
            || allocation.global_slot_id as usize != usize::from(player_id) * ACTOR_PARTITION_SLOTS + local
            || !(4..9).contains(&list_offset)
            || list_offset != 4 + next_player[3] as usize
            || expected_partition[local][ACTOR_TYPE_OFFSET] != 0
        { return Err(ActorPoolError::InvalidAllocation); }
        allocated[local] = true;
        let bp = usize::from(allocation.blueprint_index_at74);
        if bp >= 32 { return Err(ActorPoolError::InvalidAllocation); }
        let bp_start = 0x16 + bp * 36;
        next_player[bp_start + 0x20] |= allocation.blueprint_flags20_or;
        next_player[list_offset] = allocation.global_slot_id;
        next_player[3] = next_player[3].wrapping_add(1);

        let expected_init = plan_no_unit_actor_initialization(
            &expected_partition[local], player_id, local, &next_player,
            allocation.blueprint_index_at74, prepared.initializations.get(expected_inits.len())
                .ok_or(ActorPoolError::InitializationMismatch(local))?.temporary_origin,
            prepared.initializations[expected_inits.len()].sampled_origin_height,
            prepared.initializations[expected_inits.len()].sampled_origin_shade,
            prepared.initializations[expected_inits.len()].final_target,
        ).map_err(|_| ActorPoolError::InitializationMismatch(local))?;
        let actual = prepared.initializations.get(expected_inits.len())
            .ok_or(ActorPoolError::InitializationMismatch(local))?;
        if actual.local_slot as usize != local || actual.player_id != player_id
            || *actual != expected_init
        { return Err(ActorPoolError::InitializationMismatch(local)); }
        expected_inits.push(expected_init);
    }
    if attempts_seen != usize::from(plan.attempt_count) { return Err(ActorPoolError::PreparedSelectionMismatch); }
    let mut expected_slot_types = [0u8; ACTOR_PARTITION_SLOTS];
    for local in 0..ACTOR_PARTITION_SLOTS {
        if allocated[local] { expected_slot_types[local] = 3; }
    }
    if plan.slot_types_after != expected_slot_types { return Err(ActorPoolError::PreparedSelectionMismatch); }
    if expected_inits.len() != prepared.initializations.len() { return Err(ActorPoolError::PreparedSelectionMismatch); }
    next_player[9..11].copy_from_slice(&prepared.player.target()[0].to_le_bytes());
    next_player[11..13].copy_from_slice(&prepared.player.target()[1].to_le_bytes());
    if next_player.as_slice() != prepared.player.bytes().as_slice() {
        return Err(ActorPoolError::PlayerAfterImageMismatch);
    }
    for local in 0..ACTOR_PARTITION_SLOTS {
        let expected_actor = if allocated[local] {
            expected_inits.iter().find(|p| usize::from(p.local_slot) == local)
                .ok_or(ActorPoolError::InitializationMismatch(local))?.actor_after_nonpointer.as_slice()
        } else {
            expected_partition[local].as_slice()
        };
        if expected_actor != prepared.actors[local].as_slice() {
            return Err(ActorPoolError::ActorAfterImageMismatch(local));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actor_initialization::{ActorResourceTableKey, BlueprintRecordHandle};
    use crate::player_state::{ActorOriginSample, PlayerState};

    fn inactive_pool(prior: [[u8; 200]; 20]) -> ActorPool {
        let mut packed = vec![0x5A; PACKED_ACTOR_GROUP_BYTES];
        for slot in 0..ACTOR_POOL_SLOTS { packed[slot * PACKED_ACTOR_BYTES + ACTOR_TYPE_OFFSET] = 0; }
        ActorPool::from_packed_inactive(&packed, &prior,
            [ActorHandleBindings::empty(); ACTOR_POOL_SLOTS]).unwrap()
    }

    fn fresh(player: u8) -> PlayerState {
        PlayerState::fresh_campaign(player, [20000, 12000], b"TEST\0").unwrap()
    }

    fn ready_player(player_id: u8, style: u8, category: u8) -> PlayerState {
        let mut player = fresh(player_id);
        let mut bytes = *player.bytes();
        let bp = 0x16;
        bytes[bp] = b'P';
        bytes[bp + 0x15] = style;
        bytes[bp + 0x16] = category;
        player = PlayerState::from_record(&bytes).unwrap();
        player
    }

    #[test]
    fn loader_copies_two_ranges_and_preserves_both_native_gaps() {
        let mut prior = [[0xD3; 200]; 20];
        let mut packed = vec![0x61; PACKED_ACTOR_GROUP_BYTES];
        for slot in 0..20 {
            packed[slot * 140 + 6] = 0;
            packed[slot * 140 + 15..slot * 140 + 17].copy_from_slice(&(slot as u16).to_le_bytes());
            prior[slot][6] = 0;
        }
        let pool = ActorPool::from_packed_inactive(&packed, &prior, [ActorHandleBindings::empty();20]).unwrap();
        for slot in 0..20 {
            let r = pool.record(slot).unwrap();
            assert_eq!(&r[..31], &packed[slot*140..slot*140+31]);
            assert_eq!(&r[31..39], &[0xD3; 8]);
            assert_eq!(&r[39..148], &packed[slot*140+31..slot*140+140]);
            assert_eq!(&r[148..], &[0xD3;52]);
        }
    }

    #[test]
    fn loader_rejects_active_packed_slot_and_bad_extent() {
        let prior = [[0;200];20];
        let mut packed = vec![0;PACKED_ACTOR_GROUP_BYTES];
        packed[13*140+6] = 2;
        assert_eq!(ActorPool::from_packed_inactive(&packed,&prior,[ActorHandleBindings::empty();20]).unwrap_err(),
            ActorPoolError::ActivePackedRecord { slot:13, active_type:2 });
        assert_eq!(ActorPool::from_packed_inactive(&packed[..packed.len()-1],&prior,[ActorHandleBindings::empty();20]).unwrap_err(),
            ActorPoolError::PackedGroupLength { expected: PACKED_ACTOR_GROUP_BYTES, actual: PACKED_ACTOR_GROUP_BYTES-1 });
    }

    #[test]
    fn replacement_is_beforeimage_bound_and_preserves_identity_handles() {
        let mut pool = inactive_pool([[0u8;200];20]);
        let mut before = [0u8;200];
        before[6]=3; before[15..17].copy_from_slice(&0u16.to_le_bytes());
        before[18]=0; before[0x74]=2;
        pool.records[0]=before;
        pool.bindings[0]=ActorHandleBindings {
            blueprint:Some(BlueprintRecordHandle { player_id:0, blueprint_index:2 }),
            resource_database:Some(ActorResourceTableKey { style:0,category:0,flat_index:0 }),
            animation_database:Some(ActorResourceTableKey { style:0,category:0,flat_index:0 }),
            internal_actor_offset:Some(0xC4),
        };
        let mut after = before; after[0] ^= 1;
        pool.replace_record_checked(0,&before,&after).unwrap();
        assert_eq!(pool.record(0).unwrap(),&after);
        assert_eq!(pool.bindings(0).unwrap().blueprint,Some(BlueprintRecordHandle { player_id:0, blueprint_index:2 }));
        let mut bad=after; bad[0x74]=3;
        assert_eq!(pool.replace_record_checked(0,&after,&bad).unwrap_err(),ActorPoolError::ActorIdentityChanged(0));
        let mut bad=after; bad[0xC4]=1;
        assert_eq!(pool.replace_record_checked(0,&after,&bad).unwrap_err(),ActorPoolError::PointerBytesChanged { slot:0,offset:0xC4 });
        let mut bad=after; bad[0x0F]=1;
        assert!(matches!(pool.replace_record_checked(0,&after,&bad),Err(ActorPoolError::InvalidActorIdentity{..})));
    }

    #[test]
    fn prepared_commit_is_atomic_and_partitioned_for_each_owner() {
        for owner in 0..4u8 {
            let mut native = [[0xA7;200];20];
            for rec in &mut native { rec[6]=0; }
            let mut pool = inactive_pool(native);
            let prior = pool.partition(owner).unwrap();
            let expected = ready_player(owner, owner % 4, (owner+1)%4);
            let prepared = expected.prepare_fresh_actor_partition(&prior, 1, ActorOriginSample {
                position:[321,-456], height:1234, shade:owner*7,
            }, [777,-888]).unwrap();
            let mut current = expected.clone();
            let committed = pool.commit_prepared_partition(&mut current,&expected,&prior,&prepared).unwrap();
            assert_eq!(current.bytes(),prepared.player.bytes());
            assert_eq!(committed.len(),prepared.initializations.len());
            for init in &prepared.initializations {
                let global=usize::from(init.global_actor_id);
                assert_eq!(pool.record(global).unwrap(), &prepared.actors[usize::from(init.local_slot)]);
                let binding=pool.bindings(global).unwrap();
                assert_eq!(binding.blueprint,Some(BlueprintRecordHandle { player_id:owner, blueprint_index:init.blueprint_index }));
                assert_eq!(binding.resource_database,Some(ActorResourceTableKey { style:owner%4,category:(owner+1)%4,flat_index:(owner%4)*4+(owner+1)%4 }));
            }
            let original = current.clone();
            let post_partition = pool.partition(owner).unwrap();
            assert_eq!(pool.commit_prepared_partition(&mut current,&expected,&prior,&prepared).unwrap_err(), ActorPoolError::PlayerBeforeImageMismatch);
            assert_eq!(current,original);
            assert_eq!(pool.partition(owner).unwrap(),post_partition);
        }
    }

    #[test]
    fn second_initialization_failure_does_not_commit_first_slot_or_player() {
        let mut pool = inactive_pool([[0u8;200];20]);
        let mut expected=ready_player(1,0,0);
        let mut bytes=*expected.bytes();
        bytes[0x16+36]=b'Q'; bytes[0x16+36+0x15]=1; bytes[0x16+36+0x16]=2;
        expected=PlayerState::from_record(&bytes).unwrap();
        let active_prior=pool.partition(1).unwrap();
        // Make two metadata rows eligible while keeping an explicit empty actor partition.
        let mut prepared=expected.prepare_fresh_actor_partition(&active_prior,5,ActorOriginSample { position:[0,0],height:0,shade:0 },[1,2]).unwrap();
        assert!(prepared.initializations.len()>=2);
        prepared.actors[usize::from(prepared.initializations[1].local_slot)][0] ^= 1;
        let before_pool=pool.clone(); let mut current=expected.clone();
        assert!(pool.commit_prepared_partition(&mut current,&expected,&active_prior,&prepared).is_err());
        assert_eq!(pool,before_pool); assert_eq!(current,expected);
    }

    #[test]
    fn public_prepared_values_cannot_forge_allocator_or_target_writes() {
        let mut pool = inactive_pool([[0u8;200];20]);
        let expected = ready_player(2,1,2);
        let partition = pool.partition(2).unwrap();
        let mut current = expected.clone();
        let good = expected.prepare_fresh_actor_partition(&partition,4,ActorOriginSample {
            position:[32,64],height:700,shade:19,
        },[900,-1200]).unwrap();

        let mut forged = good.clone();
        forged.selection.attempts[0].as_mut().unwrap().allocation.as_mut().unwrap().blueprint_flags20_or = 0x80;
        let before_pool = pool.clone();
        assert_eq!(pool.commit_prepared_partition(&mut current,&expected,&partition,&forged).unwrap_err(),
            ActorPoolError::PreparedSelectionMismatch);
        assert_eq!(pool,before_pool); assert_eq!(current,expected);

        let mut forged = good;
        forged.initializations[0].final_target = [1,2];
        assert!(pool.commit_prepared_partition(&mut current,&expected,&partition,&forged).is_err());
        assert_eq!(pool,before_pool); assert_eq!(current,expected);
    }

    #[test]
    fn prepared_commit_refuses_untracked_old_handles_on_reused_inactive_slot() {
        let mut pool = inactive_pool([[0u8;200];20]);
        let expected = ready_player(0,0,0);
        let partition = pool.partition(0).unwrap();
        let prepared = expected.prepare_fresh_actor_partition(&partition,1,ActorOriginSample {
            position:[0,0],height:0,shade:0,
        },[10,20]).unwrap();
        let global = usize::from(prepared.initializations[0].global_actor_id);
        pool.bindings[global] = ActorHandleBindings {
            blueprint: Some(BlueprintRecordHandle { player_id:0, blueprint_index:31 }),
            ..ActorHandleBindings::empty()
        };
        let before_pool = pool.clone();
        let mut current = expected.clone();
        assert_eq!(pool.commit_prepared_partition(&mut current,&expected,&partition,&prepared).unwrap_err(),
            ActorPoolError::InvalidBindings(global));
        assert_eq!(pool,before_pool); assert_eq!(current,expected);
    }
}

//! Owned player records for the fresh-campaign and mission-restore boundary.
//!
//! CD candidate 0x80460 runs after the menu clears the selected record;
//! 0x80590 later restores only two ranges over the mission's serialized record.
//! This is a startup prerequisite, not an initialized world or a ready actor.

use genewars_core::cd_candidate::{
    plan_player_actor_creation, ActorBlueprintCandidate, PlayerActorCreationPlan,
    PlayerCreationError,
};

pub const PLAYER_RECORD_BYTES: usize = 0x70D;
pub const BLUEPRINT_BYTES: usize = 36;
pub const BLUEPRINT_COUNT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerStateError {
    RecordLength(usize),
    InvalidPlayer(u8),
    PlayerMismatch { expected: u8, actual: u8 },
    /// The caller supplies the original bounded NUL string, including its NUL.
    InvalidCampaignLabel,
    BlueprintIndex(usize),
    Creation(PlayerCreationError),
    Generation(crate::blueprint::BlueprintError),
    Initialization(crate::actor_initialization::ActorInitializationError),
    FreshCreationStateRequired,
    ActorPartitionOccupied(usize),
    StaleBlueprint(usize),
    StaleLandingReference,
    StaleBookkeeping,
}

impl core::fmt::Display for PlayerStateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for PlayerStateError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerStateOrigin {
    /// A caller-supplied native/serialized record; no startup history implied.
    ExplicitRecord,
    /// Zeroed selected record followed by the fresh byte+2==0 path of 0x80460.
    FreshCampaign,
    /// Source record overlaid with 0x80590's two retained-player ranges.
    RestoredForMission,
    /// Selection/allocation/initial placement prepared; landing still pending.
    PreparedFreshActors,
}

/// Terrain samples at the temporary initializer origin, before final placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorOriginSample {
    pub position: [i16; 2],
    pub height: i32,
    pub shade: u8,
}

/// One atomic startup after-image. Native pointer fields are typed bindings in
/// the initialization plans. No map insertion, landing factory or ready-state
/// activation is executed by this transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedPlayerActors {
    pub player: PlayerState,
    pub actors: [[u8; 200]; 5],
    pub selection: PlayerActorCreationPlan,
    pub initializations: Vec<crate::actor_initialization::ActorInitializationPlan>,
    /// External request under the caller's special-branch-dword==0 contract;
    /// not an unconditional observation of every original 0x80830 route.
    pub landing_factory_pending: bool,
}

/// Unknown fields remain byte-exact. Accessors interpret only traced fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    bytes: [u8; PLAYER_RECORD_BYTES],
    origin: PlayerStateOrigin,
}

impl PlayerState {
    pub fn from_record(record: &[u8]) -> Result<Self, PlayerStateError> {
        let bytes = record.try_into().map_err(|_| PlayerStateError::RecordLength(record.len()))?;
        Ok(Self { bytes, origin: PlayerStateOrigin::ExplicitRecord })
    }

    /// Explicit target and label inputs are not guessed from the mission file.
    /// The admitted label occupies +0x6E9..+0x6F7, including its final NUL.
    /// The original has an unbounded copy; this safe port rejects longer data.
    pub fn fresh_campaign(
        player: u8, initial_target: [i16; 2], label_nul: &[u8],
    ) -> Result<Self, PlayerStateError> {
        if player >= 4 { return Err(PlayerStateError::InvalidPlayer(player)); }
        if label_nul.is_empty() || label_nul.len() > 14 || label_nul.last() != Some(&0)
            || label_nul[..label_nul.len() - 1].contains(&0) {
            return Err(PlayerStateError::InvalidCampaignLabel);
        }
        let mut bytes = [0; PLAYER_RECORD_BYTES];
        bytes[0] = 2;
        for offset in [1, 0x4A1, 0x6E2] { bytes[offset] = player; }
        bytes[9..11].copy_from_slice(&initial_target[0].to_le_bytes());
        bytes[11..13].copy_from_slice(&initial_target[1].to_le_bytes());
        bytes[0x67E..0x682].copy_from_slice(&1_u32.to_le_bytes());
        // With the record cleared, the existing-entry scan is empty. Append
        // value4, increment its byte count and the separate dword at +0x705.
        bytes[0x6AE] = 4;
        bytes[0x6E0] = 1;
        bytes[0x705..0x709].copy_from_slice(&1_u32.to_le_bytes());
        bytes[0x62A..0x62E].copy_from_slice(&0x11800_u32.to_le_bytes());
        bytes[0x6E9..0x6E9 + label_nul.len()].copy_from_slice(label_nul);
        Ok(Self { bytes, origin: PlayerStateOrigin::FreshCampaign })
    }

    /// Apply the exact retained ranges to a separately loaded source record.
    /// The chosen slot must agree with the saved record's player ID. Fields
    /// outside these ranges (including +0x4A1 and +0x705) come from the source.
    pub fn restored_for_mission(
        source_record: &[u8], selected_player: u8, retained: &Self,
    ) -> Result<Self, PlayerStateError> {
        if selected_player >= 4 { return Err(PlayerStateError::InvalidPlayer(selected_player)); }
        if retained.player_id() != selected_player {
            return Err(PlayerStateError::PlayerMismatch { expected: selected_player, actual: retained.player_id() });
        }
        let mut result = Self::from_record(source_record)?;
        for range in [0..0x4A1, 0x61A..0x6F7] {
            result.bytes[range.clone()].copy_from_slice(&retained.bytes[range]);
        }
        result.origin = PlayerStateOrigin::RestoredForMission;
        Ok(result)
    }

    pub fn bytes(&self) -> &[u8; PLAYER_RECORD_BYTES] { &self.bytes }
    pub(crate) fn replace_bookkeeping_checked(&mut self,before:&[u8;PLAYER_RECORD_BYTES],after:&[u8;PLAYER_RECORD_BYTES])->Result<(),PlayerStateError> {
        if &self.bytes!=before {return Err(PlayerStateError::StaleBookkeeping);}
        if after[1]!=before[1] {return Err(PlayerStateError::PlayerMismatch{expected:before[1],actual:after[1]});}
        self.bytes=*after;Ok(())
    }
    /// Code31 phase3 completion preserves the raw null index and origin.
    pub(crate) fn retire_landing_reference(&mut self,slot:u16)->Result<(),PlayerStateError> {
        if self.bytes[0x496..0x499]!=[slot as u8,(slot>>8) as u8,5] { return Err(PlayerStateError::StaleLandingReference); }
        self.bytes[0x498]=0;self.bytes[0x499..0x49D].copy_from_slice(&700u32.to_le_bytes());Ok(())
    }
    pub fn origin(&self) -> PlayerStateOrigin { self.origin }
    pub fn player_id(&self) -> u8 { self.bytes[1] }
    pub fn creation_mode(&self) -> u8 { self.bytes[2] }
    pub fn actor_count(&self) -> i8 { self.bytes[3] as i8 }
    pub fn metadata_count(&self) -> u8 { self.bytes[0x15] }
    pub fn progress(&self) -> i8 { self.bytes[0x6E7] as i8 }
    pub fn target(&self) -> [i16; 2] {
        [i16::from_le_bytes([self.bytes[9], self.bytes[10]]),
         i16::from_le_bytes([self.bytes[11], self.bytes[12]])]
    }
    pub fn blueprint(&self, index: usize) -> Result<&[u8; BLUEPRINT_BYTES], PlayerStateError> {
        if index >= BLUEPRINT_COUNT { return Err(PlayerStateError::BlueprintIndex(index)); }
        let start = 0x16 + index * BLUEPRINT_BYTES;
        Ok(self.bytes[start..start + BLUEPRINT_BYTES].try_into().unwrap())
    }
    pub(crate) fn commit_activation_blueprint(
        &mut self, index: usize, expected: &[u8; BLUEPRINT_BYTES], after: &[u8; BLUEPRINT_BYTES],
    ) -> Result<(), PlayerStateError> {
        if self.blueprint(index)? != expected { return Err(PlayerStateError::StaleBlueprint(index)); }
        let start = 0x16 + index * BLUEPRINT_BYTES;
        self.bytes[start..start + BLUEPRINT_BYTES].copy_from_slice(after);
        Ok(())
    }
    pub fn activation_count(&self) -> u32 {
        u32::from_le_bytes(self.bytes[0x49D..0x4A1].try_into().unwrap())
    }
    pub(crate) fn set_landing_reference(&mut self, index: u16) {
        self.bytes[0x496..0x498].copy_from_slice(&index.to_le_bytes());
        self.bytes[0x498] = 5;
    }
    pub(crate) fn finish_activation_batch(&mut self) {
        let count = self.activation_count().wrapping_add(1);
        self.bytes[0x49D..0x4A1].copy_from_slice(&count.to_le_bytes());
    }
    pub fn creation_plan(
        &self, mask: u8, campaign: bool, active_slots: [u8; 5],
    ) -> Result<PlayerActorCreationPlan, PlayerStateError> {
        let blueprints = core::array::from_fn(|index| {
            let record = self.blueprint(index).unwrap();
            ActorBlueprintCandidate { active_name_byte: record[0], category: record[0x16] }
        });
        plan_player_actor_creation(&blueprints, mask, self.progress(), campaign,
            self.player_id(), active_slots, self.actor_count()).map_err(PlayerStateError::Creation)
    }

    /// Execute the explicitly requested metadata regeneration transaction.
    /// Caller supplies the live RNG and original reusable scratch bytes. Only
    /// metadata count/table are committed; actor allocation remains separate.
    pub fn regenerate_blueprints(
        &mut self, scratch: [u8; BLUEPRINT_BYTES], names: &crate::blueprint::BlueprintNames,
        rng: crate::blueprint::BlueprintRng, campaign: bool,
    ) -> Result<crate::blueprint::BlueprintGeneration, PlayerStateError> {
        let prior = core::array::from_fn(|index| *self.blueprint(index).unwrap());
        let generated = crate::blueprint::generate_blueprints(&prior, scratch, names, rng, campaign)
            .map_err(PlayerStateError::Generation)?;
        for (index, record) in generated.records.iter().enumerate() {
            let start = 0x16 + index * BLUEPRINT_BYTES;
            self.bytes[start..start + BLUEPRINT_BYTES].copy_from_slice(record);
        }
        self.bytes[0x15] = generated.metadata_count;
        Ok(generated)
    }

    /// Compose 0x801C0 selection, 0x81310 allocation and the new-actor-only
    /// 0x80830 placement pass. This bounded path requires an empty five-slot
    /// partition and fresh campaign counters; existing-actor placement and the
    /// special immediate-activation branch are outside this transaction.
    /// Caller must establish global0x11AC22==0. With that dword nonzero and
    /// tick0, the original may activate immediately instead of requesting a
    /// landing object. This partial API never evaluates that global branch.
    pub fn prepare_fresh_actor_partition(
        &self, prior_actors: &[[u8; 200]; 5], mask: u8,
        origin: ActorOriginSample, final_target: [i16; 2],
    ) -> Result<PreparedPlayerActors, PlayerStateError> {
        if self.creation_mode() != 0 || self.actor_count() != 0 || self.progress() != 0 {
            return Err(PlayerStateError::FreshCreationStateRequired);
        }
        if let Some(slot) = prior_actors.iter().position(|record| record[6] != 0) {
            return Err(PlayerStateError::ActorPartitionOccupied(slot));
        }
        let selection = self.creation_plan(mask, true, [0;5])?;
        let mut player = self.clone();
        let mut actors = *prior_actors;
        let mut initializations = Vec::new();
        for attempt in selection.attempts.iter().flatten() {
            if let Some(allocation) = attempt.allocation {
                let index = usize::from(allocation.blueprint_index_at74);
                player.bytes[0x16 + index * 36 + 0x20] |= allocation.blueprint_flags20_or;
                let initialized = crate::actor_initialization::plan_no_unit_actor_initialization(
                    &actors[usize::from(allocation.local_slot)], self.player_id(),
                    usize::from(allocation.local_slot), &player.bytes,
                    allocation.blueprint_index_at74, origin.position, origin.height, origin.shade,
                    final_target,
                ).map_err(PlayerStateError::Initialization)?;
                actors[usize::from(allocation.local_slot)].copy_from_slice(&initialized.actor_after_nonpointer);
                player.bytes[usize::from(allocation.player_id_list_offset)] = allocation.global_slot_id;
                player.bytes[3] += 1;
                initializations.push(initialized);
            }
        }
        player.bytes[9..11].copy_from_slice(&final_target[0].to_le_bytes());
        player.bytes[11..13].copy_from_slice(&final_target[1].to_le_bytes());
        player.origin = PlayerStateOrigin::PreparedFreshActors;
        Ok(PreparedPlayerActors { player, actors, selection, initializations, landing_factory_pending: true })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_campaign_has_no_invented_blueprints_or_actors() {
        for player in 0..4 {
            let state = PlayerState::fresh_campaign(player, [20736, 13568], b"fixture\0").unwrap();
            assert_eq!(state.origin(), PlayerStateOrigin::FreshCampaign);
            assert_eq!(state.player_id(), player);
            assert_eq!(state.target(), [20736, 13568]);
            assert_eq!((state.creation_mode(), state.actor_count(), state.metadata_count(), state.progress()), (0,0,0,0));
            assert_eq!(state.creation_plan(5, true, [0;5]).unwrap().created_count, 0);
            assert_eq!(&state.bytes()[0x62A..0x62E], &0x11800_u32.to_le_bytes());
            assert_eq!(&state.bytes()[0x705..0x709], &1_u32.to_le_bytes());
            assert_eq!((state.bytes()[0x6AE], state.bytes()[0x6E0]), (4,1));
        }
    }

    #[test]
    fn restoration_preserves_every_byte_outside_both_ranges() {
        let mut retained = core::array::from_fn::<_, PLAYER_RECORD_BYTES, _>(|i| (i as u8).wrapping_mul(37));
        retained[1] = 2;
        let source = core::array::from_fn::<_, PLAYER_RECORD_BYTES, _>(|i| !(i as u8));
        let saved = PlayerState::from_record(&retained).unwrap();
        let restored = PlayerState::restored_for_mission(&source, 2, &saved).unwrap();
        for i in 0..PLAYER_RECORD_BYTES {
            assert_eq!(restored.bytes()[i], if i < 0x4A1 || (0x61A..0x6F7).contains(&i) { retained[i] } else { source[i] }, "offset {i:x}");
        }
        assert_eq!(saved.bytes(), &retained);
        assert_eq!(restored.origin(), PlayerStateOrigin::RestoredForMission);
    }

    #[test]
    fn owned_blueprints_feed_existing_selection_with_signed_progress() {
        let mut bytes = [0; PLAYER_RECORD_BYTES];
        for i in 0..20 { bytes[0x16+i*36] = b'A'; bytes[0x16+i*36+0x16] = (i/5) as u8; }
        bytes[0x15] = 20;
        let state = PlayerState::from_record(&bytes).unwrap();
        let plan = state.creation_plan(5, true, [0;5]).unwrap();
        assert_eq!(plan.created_count, 2);
        assert_eq!(plan.attempts[0].unwrap().blueprint_index, 0);
        assert_eq!(plan.attempts[1].unwrap().blueprint_index, 10);
        assert_eq!(state.bytes(), &bytes); // Planning does not allocate actors.
    }

    #[test]
    fn malformed_input_and_slot_mismatch_are_rejected() {
        for label in [&b""[..], &b"missing terminator"[..], &b"a\0b\0"[..], &[b'a';15][..]] {
            assert_eq!(PlayerState::fresh_campaign(0,[0;2],label), Err(PlayerStateError::InvalidCampaignLabel));
        }
        assert_eq!(PlayerState::fresh_campaign(4,[0;2],b"\0"), Err(PlayerStateError::InvalidPlayer(4)));
        assert_eq!(PlayerState::from_record(&[0;10]), Err(PlayerStateError::RecordLength(10)));
        let saved = PlayerState::fresh_campaign(0,[0;2],b"\0").unwrap();
        assert!(matches!(PlayerState::restored_for_mission(&[0;PLAYER_RECORD_BYTES],1,&saved), Err(PlayerStateError::PlayerMismatch{..})));
        for index in [32,usize::MAX] { assert_eq!(saved.blueprint(index),Err(PlayerStateError::BlueprintIndex(index))); }
    }

    #[test]
    fn generation_commits_only_metadata_and_rejection_preserves_player() {
        use crate::blueprint::{BlueprintNames, BlueprintRng};
        let strings: Vec<_> = (0..128).map(|i| format!("test{i:03}").into_bytes()).collect();
        let names = BlueprintNames::new(&strings.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap();
        let mut player = PlayerState::fresh_campaign(0,[20736,13568],b"test\0").unwrap();
        let before = player.clone();
        assert!(player.regenerate_blueprints([0x95;36], &names, BlueprintRng{seed:7,ring:32,draw_count:0},true).is_err());
        assert_eq!(player,before);
        let report = player.regenerate_blueprints([0x95;36], &names, BlueprintRng{seed:7,ring:31,draw_count:0},true).unwrap();
        assert_eq!(report.draws.len(),64);
        assert_eq!(player.metadata_count(),20);
        assert_eq!(player.creation_plan(5,true,[0;5]).unwrap().created_count,2);
        for i in 0..PLAYER_RECORD_BYTES {
            if !(0x15..0x496).contains(&i) { assert_eq!(player.bytes()[i],before.bytes()[i],"offset {i:x}"); }
        }
    }

    #[test]
    fn generated_player_prepares_two_owned_actors_without_forcing_readiness() {
        use crate::blueprint::{BlueprintNames, BlueprintRng};
        let strings: Vec<_> = (0..128).map(|i| format!("test{i:03}").into_bytes()).collect();
        let names = BlueprintNames::new(&strings.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap();
        for owner in 0..4 {
            let mut player = PlayerState::fresh_campaign(owner,[1,2],b"test\0").unwrap();
            player.regenerate_blueprints([0x7D;36],&names,BlueprintRng{seed:0,ring:0,draw_count:0},true).unwrap();
            let before = player.clone();
            let mut prior = [[0xA5;200];5];
            for actor in &mut prior { actor[6]=0; }
            let origin = ActorOriginSample { position:[0,0], height:-12345, shade:57 };
            let prepared = player.prepare_fresh_actor_partition(&prior,5,origin,[20736,13568]).unwrap();
            assert_eq!(player,before);
            assert_eq!(prepared.player.actor_count(),2);
            assert_eq!(&prepared.player.bytes()[4..6], &[5*owner,5*owner+1]);
            assert_eq!(prepared.player.target(),[20736,13568]);
            assert_eq!(prepared.initializations.len(),2);
            assert!(prepared.landing_factory_pending);
            for (slot,blueprint) in [(0,0),(1,10)] {
                assert_eq!(prepared.actors[slot][6],3);
                assert_eq!(&prepared.actors[slot][0x70..0x72],&[0,4]);
                assert_eq!(prepared.actors[slot][0x74],blueprint);
                assert_eq!(&prepared.actors[slot][0x11..0x15],&(-12345_i32).to_le_bytes());
                assert_eq!(prepared.player.blueprint(blueprint as usize).unwrap()[0x20]&1,1);
            }
            assert_eq!(&prepared.actors[2..],&prior[2..]);
        }
    }

    #[test]
    fn invalid_second_actor_rejects_entire_creation_after_image() {
        let mut bytes = [0; PLAYER_RECORD_BYTES];
        for (index,category,style) in [(0,0,0),(10,2,255)] {
            let at = 0x16 + index*36;
            bytes[at]=b'A'; bytes[at+0x16]=category; bytes[at+0x15]=style;
        }
        let player=PlayerState::from_record(&bytes).unwrap();
        let prior=[[0;200];5];
        let result=player.prepare_fresh_actor_partition(&prior,5,ActorOriginSample{position:[0,0],height:0,shade:0},[1,2]);
        assert!(matches!(result,Err(PlayerStateError::Initialization(_))));
        assert_eq!(player.bytes(),&bytes);
        assert_eq!(prior,[[0;200];5]);
    }
}

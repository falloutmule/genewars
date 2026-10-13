//! Owned actor startup, ordered activation and bounded pending commands.
//!
//! The packed backing retains the map and non-actor common link headers;
//! kind3 resolves exclusively through the native actor pool. The backing is
//! therefore NOT a save file: native actors and player after-images live in
//! their owned overlays. This stage does not run the original scheduler.
//! Live RNG, native gaps, player state and environmental query results are
//! explicit inputs. In particular, the no-associated activation route is not
//! the normal code31 landing route, which retains an associated type5 object.

use crate::actor_activation::{self, ActivationPlanningOutcome, ActorActivationError,
    ActorActivationPlan, CandidateEnvironment, PlacementCandidate, ResolvedCellHead, TrigTables};
use crate::actor_command::{self, Opcode3MoveOrderError, Opcode3MoveOrderResult, ResolverEnvironment, ResolverError};
use crate::actor_pool::{ActorPool, ActorPoolError, ActorHandleBindings};
use crate::blueprint::{BlueprintRng, BlueprintNames, BlueprintGeneration};
use crate::player_state::{ActorOriginSample, PlayerState, PlayerStateError};
use genewars_assets::{ConqMissionSource, MissionRecordGroupLayout};
use std::collections::HashSet;

const MISSION_HASH: &str = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2";
const MAP_START: usize = 0x355E;
const MAP_BYTES: usize = 128 * 128 * 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorWorldError {
    MissionHash(String), InvalidPlayer(u8), PlayerIdentity { slot: usize, actual: u8 },
    InvalidRngRing(u8), Pool(ActorPoolError), Player(PlayerStateError),
    Activation(ActorActivationError), Command(Opcode3MoveOrderError), Resolver(ResolverError),
    InvalidReference([u8; 3]), EntityIdentity([u8; 3]), BrokenPrevious([u8; 3]),
    DuplicateLink([u8; 3]), AlreadyLinked(i16), MissingActorLink(i16),
    ActorOwner(i16), ActorNotCommandable(i16), AssociatedUnitUnsupported,
    EnvironmentUnavailable, StaleLink,
    InvalidPortraitState { current: i8, count: i8 },
}
impl core::fmt::Display for ActorWorldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ActorWorldError {}
impl From<ActorPoolError> for ActorWorldError { fn from(e: ActorPoolError) -> Self { Self::Pool(e) } }
impl From<PlayerStateError> for ActorWorldError { fn from(e: PlayerStateError) -> Self { Self::Player(e) } }
impl From<ActorActivationError> for ActorWorldError { fn from(e: ActorActivationError) -> Self { Self::Activation(e) } }

/// Values returned by separately admitted terrain/blocker helpers at the
/// supplied sampled point. The actor avoidance mask always comes from the
/// owned record, and map-head identity always comes from this world's store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlacementEnvironment { pub terrain_class: u8, pub blocker_weight: i32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssociatedPlacementEnvironment {
    pub placement: PlacementEnvironment,
    /// Additional resolver lookup at an accepted candidate. Rejected
    /// placement attempts and flag71 no-ops do not consume this value.
    pub obstacle_present: bool,
}

/// Ground-control branch inputs after module/hit dispatch and874B0 world
/// projection. These are source packet/state fields, not invented UI events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GroundMoveInput {
    pub buttons: u8,
    pub modifiers: u8,
    pub pause_flags: u8,
    pub group_word_526: u16,
    pub group_byte_52f: u8,
    pub current_portrait: i8,
    pub selected: [u8; 3],
    pub projected_point: [i16; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroundMoveOutcome {
    Ignored,
    Dispatched { actor_id: i16, opcode: u8, result: Opcode3MoveOrderResult },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationBatchReceipt {
    pub player_id: u8,
    pub rng_before: BlueprintRng,
    pub rng_after: BlueprintRng,
    pub count_before: u32,
    pub count_after: u32,
    pub activations: Vec<ActorActivationPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWorld {
    backing: Vec<u8>,
    layouts: [MissionRecordGroupLayout; 8],
    actors: ActorPool,
    players: [PlayerState; 4],
    rng: BlueprintRng,
}

impl ActorWorld {
    /// Admit the pinned source only. Native prior bytes/handles are explicit;
    /// no zeroed native memory, fresh player metadata or live seed is inferred.
    pub fn from_mission(
        source: &ConqMissionSource<'_>, prior_native: &[[u8; 200]; 20],
        prior_bindings: [ActorHandleBindings; 20], players: [PlayerState; 4], rng: BlueprintRng,
    ) -> Result<Self, ActorWorldError> {
        let actual = crate::identity::sha256_hex(source.bytes());
        if actual != MISSION_HASH { return Err(ActorWorldError::MissionHash(actual)); }
        Self::from_parts(source, prior_native, prior_bindings, players, rng)
    }

    fn from_parts(
        source: &ConqMissionSource<'_>, prior_native: &[[u8; 200]; 20],
        prior_bindings: [ActorHandleBindings; 20], players: [PlayerState; 4], rng: BlueprintRng,
    ) -> Result<Self, ActorWorldError> {
        if rng.ring >= 32 { return Err(ActorWorldError::InvalidRngRing(rng.ring)); }
        for (slot, player) in players.iter().enumerate() {
            if player.player_id() != slot as u8 {
                return Err(ActorWorldError::PlayerIdentity { slot, actual: player.player_id() });
            }
        }
        let groups = source.groups();
        let actors = ActorPool::from_packed_inactive(groups[2].bytes(), prior_native, prior_bindings)?;
        let result = Self { backing: source.bytes().to_vec(), layouts: groups.map(|g| g.layout()), actors, players, rng };
        result.validate_links()?;
        Ok(result)
    }

    pub fn actors(&self) -> &ActorPool { &self.actors }
    pub fn player(&self, player: u8) -> Result<&PlayerState, ActorWorldError> {
        self.players.get(usize::from(player)).ok_or(ActorWorldError::InvalidPlayer(player))
    }
    pub fn rng(&self) -> BlueprintRng { self.rng }
    pub fn map(&self) -> &[u8] { &self.backing[MAP_START..MAP_START + MAP_BYTES] }
    /// Partial backing only; serializing this omits owned actors and players.
    pub fn backing_bytes(&self) -> &[u8] { &self.backing }

    pub fn regenerate_player_blueprints(
        &mut self, player: u8, scratch: [u8; 36], names: &BlueprintNames, campaign: bool,
    ) -> Result<BlueprintGeneration, ActorWorldError> {
        self.player(player)?;
        // Once actors have handles into metadata, regeneration would invalidate
        // their resource keys. The broader rebind path is not implemented.
        if self.actors.partition(player)?.iter().any(|a| a[6] != 0) {
            return Err(ActorWorldError::ActorNotCommandable(i16::from(player) * 5));
        }
        let generated = self.players[usize::from(player)].regenerate_blueprints(scratch, names, self.rng, campaign)?;
        self.rng = generated.rng_after;
        Ok(generated)
    }

    /// Commit fresh selection and typed initialization as one transaction.
    /// Caller still establishes the proved special-global==0 branch; this
    /// returns with the landing factory pending and actor readiness clear.
    pub fn prepare_fresh_player(
        &mut self, player: u8, mask: u8, origin: ActorOriginSample, target: [i16; 2],
    ) -> Result<(), ActorWorldError> {
        let before = self.player(player)?.clone();
        let partition = self.actors.partition(player)?;
        let prepared = before.prepare_fresh_actor_partition(&partition, mask, origin, target)?;
        self.actors.commit_prepared_partition(&mut self.players[usize::from(player)], &before,
            &partition, &prepared)?;
        Ok(())
    }

    /// Execute the complete five-slot no-associated-unit batch on a clone.
    /// Query results are caller evidence, not inferred world predicates. The
    /// query sees earlier accepted mutations in this batch. Any failure rolls
    /// back all world/player/map/handle/RNG state; external query side effects
    /// are not owned by the transaction. At most4096 attempts per slot.
    pub fn activate_without_associated_unit<F>(
        &mut self, player: u8, tables: &TrigTables, mut query: F,
    ) -> Result<ActivationBatchReceipt, ActorWorldError>
    where F: FnMut(&Self, usize, PlacementCandidate) -> Result<PlacementEnvironment, ActorWorldError> {
        let state = self.player(player)?;
        if state.bytes()[0x496 + 2] == 5 { return Err(ActorWorldError::AssociatedUnitUnsupported); }
        self.activate_batch(player,tables,false,|world,slot,candidate| {
            Ok(AssociatedPlacementEnvironment { placement: query(world,slot,candidate)?, obstacle_present:false })
        })
    }

    /// Associated-type5 route with actual reference resolution. This is an
    /// activation transaction only; the caller must establish the landing
    /// contact event and prior shared-RNG schedule. It does not create a
    /// landing object or advance its physics/contact state.
    pub fn activate_associated_unit<F>(
        &mut self, player: u8, tables: &TrigTables, query: F,
    ) -> Result<ActivationBatchReceipt, ActorWorldError>
    where F: FnMut(&Self, usize, PlacementCandidate) -> Result<AssociatedPlacementEnvironment, ActorWorldError> {
        let raw=reference(self.player(player)?.bytes(),0x496);
        if raw[2]!=5 { return Err(ActorWorldError::AssociatedUnitUnsupported); }
        self.entity(raw)?;
        self.activate_batch(player,tables,true,query)
    }

    fn activate_batch<F>(
        &mut self, player: u8, tables: &TrigTables, associated: bool, mut query: F,
    ) -> Result<ActivationBatchReceipt, ActorWorldError>
    where F: FnMut(&Self, usize, PlacementCandidate) -> Result<AssociatedPlacementEnvironment, ActorWorldError> {
        let state=self.player(player)?;
        let center = if associated {
            let record=self.entity(reference(state.bytes(),0x496))?;
            [word(record,7) as u16,word(record,9) as u16]
        } else { state.target().map(|n|n as u16) };
        let rng_before = self.rng;
        let count_before = state.activation_count();
        let mut staged = self.clone();
        let mut linked = staged.validate_links()?;
        let mut activations = Vec::new();
        for slot in 0..5 {
            let id = i16::from(player) * 5 + slot as i16;
            let before = *staged.actors.record(id as usize)?;
            if before[6] == 0 || u16::from_le_bytes([before[0x70], before[0x71]]) & 0x400 == 0 { continue; }
            if before[0x18] != player { return Err(ActorWorldError::ActorOwner(id)); }
            if linked.contains(&(3, id)) { return Err(ActorWorldError::AlreadyLinked(id)); }
            // Original80AE0 writes these flags before its placement queries.
            // Expose that staged prefix to read-only environmental callbacks.
            let mut prefixed = before;
            let flags = u16::from_le_bytes([before[0x70],before[0x71]]);
            prefixed[0x70..0x72].copy_from_slice(&((flags & 0xFBFE)|1).to_le_bytes());
            prefixed[0x16] &= 0xF7;
            staged.actors.replace_record_checked(id as usize,&before,&prefixed)?;
            let mut environments = Vec::new();
            let mut obstacle_present = false;
            let seed_before_slot = staged.rng.seed;
            let mut seed = seed_before_slot;
            for _ in 0..actor_activation::MAX_CANDIDATE_ATTEMPTS {
                let candidate = actor_activation::sample_placement_candidate(seed, center, tables)?;
                seed = candidate.seed_after;
                staged.rng.seed = seed;
                staged.rng.ring = (staged.rng.ring+1)&31;
                staged.rng.draw_count = staged.rng.draw_count.wrapping_add(1);
                let queried = query(&staged, slot, candidate)?;
                let environment = queried.placement;
                if environment.terrain_class > 7 {
                    return Err(ActorActivationError::InvalidTerrainClass(environment.terrain_class).into());
                }
                if environment.blocker_weight < 0 {
                    return Err(ActorActivationError::NegativeBlockerWeight(environment.blocker_weight).into());
                }
                let accepted = before[0x53] & (1 << environment.terrain_class) == 0 && environment.blocker_weight < 10;
                let resolved_cell_head = if accepted {
                    obstacle_present = queried.obstacle_present;
                    let cell = cell_for(if associated { center } else { candidate.position_u15 });
                    let raw = reference(staged.map(), cell * 14 + 6);
                    if raw[2] == 0 { None } else {
                        let record = staged.entity(raw)?;
                        Some(ResolvedCellHead { record_id: word(record, 15), record_type: record[6] })
                    }
                } else { None };
                environments.push(CandidateEnvironment { terrain_class: environment.terrain_class,
                    blocker_weight: environment.blocker_weight, resolved_cell_head });
                if accepted { break; }
            }
            let state = &staged.players[usize::from(player)];
            let planned = if associated {
                actor_activation::plan_associated_actor_activation(id, slot, &before,
                    &state.bytes()[0x16..0x496], staged.map(), center, seed_before_slot, tables, &environments, obstacle_present)?
            } else {
                actor_activation::plan_actor_activation(id, slot, &before,
                    &state.bytes()[0x16..0x496], staged.map(), center, seed_before_slot, tables, &environments, true)?
            };
            let ActivationPlanningOutcome::Planned(plan) = planned else { unreachable!("eligible slot above"); };
            // The proved planner uses the original before-image. Restore that
            // prefix privately, then commit its complete validated after-image.
            staged.actors.replace_record_checked(id as usize,&prefixed,&before)?;
            staged.commit_activation(player, &plan)?;
            linked.insert((3, id));
            activations.push(plan);
        }
        staged.players[usize::from(player)].finish_activation_batch();
        staged.validate_links()?;
        let receipt = ActivationBatchReceipt { player_id: player, rng_before, rng_after: staged.rng,
            count_before, count_after: staged.players[usize::from(player)].activation_count(), activations };
        *self = staged;
        Ok(receipt)
    }

    fn commit_activation(&mut self, player: u8, plan: &ActorActivationPlan) -> Result<(), ActorWorldError> {
        let cell_offset = MAP_START + usize::from(plan.link.cell_index) * 14 + 6;
        if reference(&self.backing, cell_offset) != plan.link.expected_old_head { return Err(ActorWorldError::StaleLink); }
        if let Some(previous) = plan.link.old_head_previous_after {
            self.replace_previous(plan.link.expected_old_head, previous)?;
        }
        self.actors.replace_record_checked(plan.actor_id as usize, plan.actor_before.as_slice().try_into().unwrap(),
            plan.actor_after.as_slice().try_into().unwrap())?;
        self.players[usize::from(player)].commit_activation_blueprint(plan.blueprint_index,
            &plan.blueprint_before, &plan.blueprint_after)?;
        self.backing[cell_offset..cell_offset + 3].copy_from_slice(&plan.link.new_cell_head);
        Ok(())
    }

    /// Apply a pending opcode3 order to an owned, linked, ready actor. This
    /// checks actor-local gates and the bounded resolver; the caller supplies
    /// an already projected world point after the separate controller gates.
    /// No movement, full path search, UI packet production or scheduler runs.
    pub fn apply_bounded_move_order(
        &mut self, player: u8, id: i16, requested: [i16; 2], terrain_class: u8,
        obstacle_present: bool, blocker_weight: i32,
    ) -> Result<Opcode3MoveOrderResult, ActorWorldError> {
        self.player(player)?;
        let before = *self.actors.record(id as usize)?;
        if before[6] != 3 || before[0x18] != player || id / 5 != i16::from(player) {
            return Err(ActorWorldError::ActorOwner(id));
        }
        if before[0x70] & 1 == 0 || (2..=5).contains(&(before[0x55] as i8)) {
            return Err(ActorWorldError::ActorNotCommandable(id));
        }
        if !self.validate_links()?.contains(&(3, id)) { return Err(ActorWorldError::MissingActorLink(id)); }
        // Preserve the setter's no-op/reinitialization branch order before
        // invoking any resolver that could fail or request backward search.
        let mut after = before;
        let noop = actor_command::apply_opcode3_move_order(id, &mut after, requested,
            actor_command::PrecomputedResolvedTarget::new(requested)).map_err(ActorWorldError::Command)?;
        if noop == Opcode3MoveOrderResult::NoopFlag71 { return Ok(noop); }
        let resolved = actor_command::resolve_bounded_move_target([word(&before, 7), word(&before, 9)], requested,
            ResolverEnvironment { terrain_class, actor_avoidance_mask: before[0x53], obstacle_present, blocker_weight })
            .map_err(ActorWorldError::Resolver)?;
        let result = actor_command::apply_opcode3_move_order(id, &mut after, requested, resolved).map_err(ActorWorldError::Command)?;
        self.actors.replace_record_checked(id as usize, &before, &after)?;
        Ok(result)
    }

    /// Source2C64C eligibility followed by the bounded opcode3 dispatcher.
    /// Feedback/audio and the outer1C120 activity/tick gates are external;
    /// this method neither projects a screen point nor runs actor movement.
    pub fn dispatch_ground_move(
        &mut self, player: u8, input: GroundMoveInput, environment: PlacementEnvironment,
        obstacle_present: bool,
    ) -> Result<GroundMoveOutcome, ActorWorldError> {
        let count=self.player(player)?.actor_count();
        if !(0..=5).contains(&count) || !(0..=4).contains(&input.current_portrait) {
            return Err(ActorWorldError::InvalidPortraitState { current:input.current_portrait,count });
        }
        if input.buttons&8==0 || input.modifiers&0x0A!=0 || input.pause_flags&2!=0
            || input.group_word_526!=0 || input.group_byte_52f!=0
            || input.current_portrait>=count || input.selected[2]!=3 {
            return Ok(GroundMoveOutcome::Ignored);
        }
        let selected=self.entity(input.selected)?;
        if selected[0x18]!=player || selected[0x70]&1==0 || (2..=5).contains(&(selected[0x55]as i8)) {
            return Ok(GroundMoveOutcome::Ignored);
        }
        let id=word(selected,15);
        let result=self.apply_bounded_move_order(player,id,input.projected_point,environment.terrain_class,
            obstacle_present,environment.blocker_weight)?;
        Ok(GroundMoveOutcome::Dispatched { actor_id:id,opcode:3,result })
    }

    /// Common prefix lookup. Type3 always uses the native overlay; other
    /// families use their proved serialized common first31 bytes only.
    pub fn entity(&self, raw: [u8; 3]) -> Result<&[u8], ActorWorldError> {
        let id = i16::from_le_bytes([raw[0], raw[1]]);
        if !(1..=8).contains(&raw[2]) || id < 0 { return Err(ActorWorldError::InvalidReference(raw)); }
        let layout = self.layouts[usize::from(raw[2] - 1)];
        if id as usize >= layout.count { return Err(ActorWorldError::InvalidReference(raw)); }
        let record: &[u8] = if raw[2] == 3 { self.actors.record(id as usize)? } else {
            let start = layout.file_offset + id as usize * layout.packed_stride;
            &self.backing[start..start + 31]
        };
        if record[6] != raw[2] || word(record, 15) != id { return Err(ActorWorldError::EntityIdentity(raw)); }
        Ok(record)
    }

    fn replace_previous(&mut self, raw: [u8; 3], previous: [u8; 3]) -> Result<(), ActorWorldError> {
        self.entity(raw)?;
        let id = i16::from_le_bytes([raw[0], raw[1]]);
        if raw[2] == 3 {
            let before = *self.actors.record(id as usize)?;
            let mut after = before;
            after[..3].copy_from_slice(&previous);
            self.actors.replace_record_checked(id as usize, &before, &after)?;
        } else {
            let layout = self.layouts[usize::from(raw[2] - 1)];
            let start = layout.file_offset + id as usize * layout.packed_stride;
            self.backing[start..start + 3].copy_from_slice(&previous);
        }
        Ok(())
    }

    fn validate_links(&self) -> Result<HashSet<(u8, i16)>, ActorWorldError> {
        let mut seen = HashSet::new();
        for cell in 0..128 * 128 {
            let mut raw = reference(self.map(), cell * 14 + 6);
            // A head's kind0 previous triple encodes its owning cell index.
            // It is not the opaque null index used by a terminal next link.
            let cell_bytes=(cell as u16).to_le_bytes();
            let mut previous = [cell_bytes[0],cell_bytes[1],0];
            while raw[2] != 0 {
                let record = self.entity(raw)?;
                let id = i16::from_le_bytes([raw[0], raw[1]]);
                if !seen.insert((raw[2], id)) { return Err(ActorWorldError::DuplicateLink(raw)); }
                let actual = reference(record, 0);
                if actual != previous {
                    return Err(ActorWorldError::BrokenPrevious(raw));
                }
                previous = raw;
                raw = reference(record, 3);
            }
        }
        Ok(seen)
    }
}

fn reference(bytes: &[u8], offset: usize) -> [u8; 3] { bytes[offset..offset + 3].try_into().unwrap() }
fn word(bytes: &[u8], offset: usize) -> i16 { i16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap()) }
fn cell_for(point: [u16; 2]) -> usize { usize::from(point[1] >> 8) * 128 + usize::from(point[0] >> 8) }

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> ActorWorld {
        let mut bytes = vec![0; 343_958];
        bytes[..4].copy_from_slice(&0x55336_u32.to_le_bytes());
        bytes[337_054..337_058].copy_from_slice(&0x6BE_u32.to_le_bytes());
        let players = core::array::from_fn(|p| PlayerState::fresh_campaign(p as u8, [4096,4096], b"fixture\0").unwrap());
        let priors = core::array::from_fn(|id| core::array::from_fn(|offset| ((id * 29 + offset) & 255) as u8));
        ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(), &priors,
            core::array::from_fn(|_| ActorHandleBindings::default()), players,
            BlueprintRng { seed: 0, ring: 31, draw_count: u32::MAX }).unwrap()
    }
    fn prepare(world: &mut ActorWorld, player: u8) {
        let owned: Vec<_> = (0..128).map(|i| format!("name{i}")).collect();
        let borrowed: Vec<_> = owned.iter().map(|s| s.as_bytes()).collect();
        let names = BlueprintNames::new(&borrowed).unwrap();
        let generated = world.regenerate_player_blueprints(player, [0xA5;36], &names, true).unwrap();
        assert_eq!(generated.draws.len(),64);
        world.prepare_fresh_player(player, 5, ActorOriginSample { position:[0,0], height:1234, shade:67 }, [4096,4096]).unwrap();
    }
    fn accept(_: &ActorWorld, _: usize, _: PlacementCandidate) -> Result<PlacementEnvironment, ActorWorldError> {
        Ok(PlacementEnvironment { terrain_class: 1, blocker_weight: 0 })
    }

    #[test]
    fn owned_startup_activation_and_pending_command_for_every_player() {
        let tables = TrigTables::generate_pinned().unwrap();
        for player in 0..4 {
            let mut world = fixture();
            prepare(&mut world, player);
            let id = i16::from(player) * 5;
            let before = world.clone();
            assert!(matches!(world.apply_bounded_move_order(player,id,[5000,5000],1,false,0), Err(ActorWorldError::ActorNotCommandable(_))));
            assert_eq!(world,before);
            let receipt = world.activate_without_associated_unit(player,&tables,accept).unwrap();
            assert_eq!(receipt.activations.iter().map(|a| a.actor_id).collect::<Vec<_>>(),[id,id+1]);
            assert_eq!(receipt.rng_after.draw_count, receipt.rng_before.draw_count.wrapping_add(2));
            assert_eq!(receipt.rng_after.ring, (receipt.rng_before.ring+2)&31);
            assert_eq!(receipt.count_after,1);
            let cell = usize::from(receipt.activations[0].link.cell_index);
            assert_eq!(reference(world.map(),cell*14+6),[id as u8+1,0,3]);
            assert_eq!(reference(world.actors.record((id+1) as usize).unwrap(),3),[id as u8,0,3]);
            assert_eq!(reference(world.actors.record(id as usize).unwrap(),0),[id as u8+1,0,3]);
            let actor_before = *world.actors.record(id as usize).unwrap();
            let result = world.apply_bounded_move_order(player,id,[5000,5000],1,false,0).unwrap();
            assert!(matches!(result,Opcode3MoveOrderResult::Applied { resolved:[5000,5000],.. }));
            assert_eq!(&world.actors.record(id as usize).unwrap()[7..11],&actor_before[7..11],"pending order does not move actor");
            assert_eq!(word(world.actors.record(id as usize).unwrap(),11),5000);
            assert_eq!(&world.actors.record(id as usize).unwrap()[0x11..0x15],&1234_i32.to_le_bytes());
            assert_eq!(world.actors.record(id as usize).unwrap()[0x17],67);
            assert_eq!(world.validate_links().unwrap().len(),2);
        }
    }

    #[test]
    fn later_slot_failure_rolls_back_every_owned_component() {
        let tables = TrigTables::generate_pinned().unwrap();
        let mut world = fixture();
        prepare(&mut world,0);
        let before = world.clone();
        let mut saw_staged = false;
        let error = world.activate_without_associated_unit(0,&tables,|staged,slot,_| {
            if slot == 1 {
                assert_eq!(staged.actors.record(0).unwrap()[0x70]&1,1);
                assert_eq!(staged.rng.draw_count,before.rng.draw_count.wrapping_add(2));
                saw_staged = true;
                Err(ActorWorldError::EnvironmentUnavailable)
            } else { accept(staged,slot,actor_activation::sample_placement_candidate(0,[0,0],&tables).unwrap()) }
        }).unwrap_err();
        assert_eq!(error,ActorWorldError::EnvironmentUnavailable);
        assert!(saw_staged);
        assert_eq!(world,before);
    }

    #[test]
    fn retry_cap_invalid_environment_and_associated_route_are_atomic() {
        let tables = TrigTables::generate_pinned().unwrap();
        let mut world = fixture();
        prepare(&mut world,0);
        for environment in [PlacementEnvironment{terrain_class:1,blocker_weight:10},
            PlacementEnvironment{terrain_class:8,blocker_weight:0},PlacementEnvironment{terrain_class:1,blocker_weight:-1}] {
            let before=world.clone();
            assert!(world.activate_without_associated_unit(0,&tables,|_,_,_|Ok(environment)).is_err());
            assert_eq!(world,before);
        }
        let mut player=*world.players[0].bytes();
        player[0x498]=5;
        world.players[0]=PlayerState::from_record(&player).unwrap();
        let before=world.clone();
        assert_eq!(world.activate_without_associated_unit(0,&tables,accept).unwrap_err(),ActorWorldError::AssociatedUnitUnsupported);
        assert_eq!(world,before);
    }

    #[test]
    fn no_eligible_slots_still_increment_wrapping_counter_without_draws() {
        let tables = TrigTables::generate_pinned().unwrap();
        let mut world=fixture();
        let mut player=*world.players[0].bytes();
        player[0x49D..0x4A1].copy_from_slice(&u32::MAX.to_le_bytes());
        world.players[0]=PlayerState::from_record(&player).unwrap();
        let receipt=world.activate_without_associated_unit(0,&tables,|_,_,_|panic!("no eligible slots")).unwrap();
        assert_eq!(receipt.count_after,0);
        assert_eq!(receipt.rng_before,receipt.rng_after);
        assert!(receipt.activations.is_empty());
    }

    #[test]
    fn non_actor_heads_are_resolved_and_relinked_without_payload_changes() {
        let tables=TrigTables::generate_pinned().unwrap();
        for kind in [1,2,4,5,6,7,8] {
            let mut world=fixture();
            prepare(&mut world,0);
            let point=actor_activation::sample_placement_candidate(world.rng.seed,[4096,4096],&tables).unwrap().position_u15;
            let cell=cell_for(point);
            let layout=world.layouts[usize::from(kind-1)];
            let start=layout.file_offset;
            world.backing[start..start+layout.packed_stride].fill(0xA5);
            let cell_bytes=(cell as u16).to_le_bytes();
            world.backing[start..start+3].copy_from_slice(&[cell_bytes[0],cell_bytes[1],0]);
            world.backing[start+5]=0;
            world.backing[start+6]=kind;
            world.backing[start+15..start+17].fill(0);
            world.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[0,0,kind]);
            let before=world.backing.clone();
            world.activate_without_associated_unit(0,&tables,accept).unwrap();
            assert_eq!(&world.backing[start..start+3],&[0,0,3]);
            assert_eq!(&world.backing[start+3..start+layout.packed_stride],&before[start+3..start+layout.packed_stride]);
            assert_eq!(world.validate_links().unwrap().len(),3);
        }
    }

    #[test]
    fn corrupt_and_duplicate_heads_fail_before_any_query() {
        let tables=TrigTables::generate_pinned().unwrap();
        for raw in [[0,0,1],[255,127,2],[0,0,9]] {
            let mut world=fixture();
            prepare(&mut world,0);
            world.backing[MAP_START+6..MAP_START+9].copy_from_slice(&raw);
            let before=world.clone();
            assert!(world.activate_without_associated_unit(0,&tables,|_,_,_|panic!("invalid links")).is_err());
            assert_eq!(world,before);
        }
        let mut world=fixture();
        prepare(&mut world,0);
        world.activate_without_associated_unit(0,&tables,accept).unwrap();
        world.backing[MAP_START+6..MAP_START+9].copy_from_slice(&[1,0,3]);
        assert!(world.validate_links().is_err());
    }

    #[test]
    fn blocked_move_preserves_world_and_live_seed() {
        let tables=TrigTables::generate_pinned().unwrap();
        let mut world=fixture();
        prepare(&mut world,0);
        world.activate_without_associated_unit(0,&tables,accept).unwrap();
        let before=world.clone();
        assert_eq!(world.apply_bounded_move_order(0,0,[5000,5000],1,true,0).unwrap_err(),
            ActorWorldError::Resolver(ResolverError::BackwardSearchRequired));
        assert_eq!(world,before);
        assert!(world.apply_bounded_move_order(1,0,[5000,5000],1,false,0).is_err());
        assert_eq!(world,before);
    }

    fn associated_fixture() -> ActorWorld {
        let mut world=fixture();
        prepare(&mut world,0);
        let mut player=*world.players[0].bytes();
        player[0x496..0x499].copy_from_slice(&[0,0,5]);
        world.players[0]=PlayerState::from_record(&player).unwrap();
        let start=world.layouts[4].file_offset;
        world.backing[start+6]=5;
        world.backing[start+7..start+11].copy_from_slice(&[0,8,0,32]);
        let cell=cell_for([2048,8192]);
        let cell_bytes=(cell as u16).to_le_bytes();
        world.backing[start..start+3].copy_from_slice(&[cell_bytes[0],cell_bytes[1],0]);
        world.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[0,0,5]);
        world
    }

    #[test]
    fn associated_activation_links_at_object_and_orders_toward_candidate() {
        let tables=TrigTables::generate_pinned().unwrap();
        let mut world=associated_fixture();
        let before=world.clone();
        let receipt=world.activate_associated_unit(0,&tables,|_,_,_|Ok(AssociatedPlacementEnvironment {
            placement:PlacementEnvironment{terrain_class:1,blocker_weight:0},obstacle_present:false
        })).unwrap();
        assert_eq!(receipt.activations.len(),2);
        for id in [0,1] {
            let actor=world.actors.record(id).unwrap();
            assert_eq!([word(actor,7),word(actor,9)],[2048,8192]);
            assert_eq!([word(actor,11),word(actor,13)],[2048,7680]);
            assert_eq!(actor[0x57]&2,2);
        }
        assert_eq!(reference(world.map(),cell_for([2048,8192])*14+6),[1,0,3]);
        assert_eq!(reference(world.actors.record(0).unwrap(),3),[0,0,5]);
        assert_eq!(&world.players[0].bytes()[0x496..0x499],&before.players[0].bytes()[0x496..0x499]);
        assert_eq!(world.validate_links().unwrap().len(),3);
    }

    #[test]
    fn associated_later_obstacle_failure_and_inhibited_order_branches() {
        let tables=TrigTables::generate_pinned().unwrap();
        let mut world=associated_fixture();
        let before=world.clone();
        assert!(world.activate_associated_unit(0,&tables,|_,slot,_|Ok(AssociatedPlacementEnvironment {
            placement:PlacementEnvironment{terrain_class:1,blocker_weight:0},obstacle_present:slot==1
        })).is_err());
        assert_eq!(world,before);
        for id in [0,1] {
            let prior=*world.actors.record(id).unwrap();
            let mut after=prior;
            after[0x71]|=0x10;
            after[0x57]|=1;
            after[0x82]=9;
            world.actors.replace_record_checked(id,&prior,&after).unwrap();
        }
        let before=world.clone();
        world.activate_associated_unit(0,&tables,|_,_,_|Ok(AssociatedPlacementEnvironment {
            placement:PlacementEnvironment{terrain_class:1,blocker_weight:0},obstacle_present:true
        })).unwrap();
        for id in [0,1] {
            let actor=world.actors.record(id).unwrap();
            assert_eq!(&actor[11..15],&before.actors.record(id).unwrap()[11..15]);
            assert_eq!(actor[0x57],before.actors.record(id).unwrap()[0x57]);
            assert_eq!(actor[0x70]&1,1);
        }
    }

    #[test]
    fn ground_controller_gates_before_dispatch_and_uses_owned_actor() {
        let tables=TrigTables::generate_pinned().unwrap();
        let mut world=fixture(); prepare(&mut world,0);
        world.activate_without_associated_unit(0,&tables,accept).unwrap();
        let input=GroundMoveInput { buttons:8,modifiers:0,pause_flags:0,group_word_526:0,group_byte_52f:0,
            current_portrait:0,selected:[0,0,3],projected_point:[5000,5000] };
        let env=PlacementEnvironment{terrain_class:1,blocker_weight:0};
        let before=world.clone();
        for blocked in [GroundMoveInput{buttons:0,..input},GroundMoveInput{modifiers:2,..input},
            GroundMoveInput{pause_flags:2,..input},GroundMoveInput{group_word_526:1,..input},
            GroundMoveInput{group_byte_52f:1,..input},GroundMoveInput{current_portrait:2,..input},
            GroundMoveInput{selected:[0,0,2],..input}] {
            assert_eq!(world.dispatch_ground_move(0,blocked,env,false).unwrap(),GroundMoveOutcome::Ignored);
            assert_eq!(world,before);
        }
        assert!(matches!(world.dispatch_ground_move(0,input,env,false).unwrap(),
            GroundMoveOutcome::Dispatched { actor_id:0,opcode:3,result:Opcode3MoveOrderResult::Applied {..} }));
        assert_eq!(word(world.actors.record(0).unwrap(),11),5000);
    }

    #[test]
    fn head_previous_kind_zero_still_requires_owning_cell_index() {
        let mut world=associated_fixture();
        let start=world.layouts[4].file_offset;
        world.backing[start+3..start+6].copy_from_slice(&[0xA5,0x5A,0]);
        assert!(world.validate_links().is_ok(),"terminal next index stays opaque");
        world.backing[start]^=1;
        assert_eq!(world.validate_links().unwrap_err(),ActorWorldError::BrokenPrevious([0,0,5]));
    }
}

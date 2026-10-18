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
use crate::world::{MissionWorld, PlantStorage, PlantParameterTable, PlantTickReport, RuntimeError};
use crate::effect_pool::{EffectPool, EffectPoolError, EffectAnimationBinding};
use crate::creature_world::{CreaturePool, CreatureRecord, CreatureError};
use crate::family8_pool::{Family8Pool, AnimationBinding as Family8AnimationBinding};
use crate::family8_creation::Family8Record;
use crate::building_pool::BuildingPool;
use crate::landing::{self, LandingRecord, LandingError, LandingEvent};
use genewars_assets::{ConqMissionSource, MissionRecordGroupLayout};
use std::collections::HashSet;

mod effect_updates;
mod creature_updates;
mod creature_retirement;
pub use creature_retirement::{MuleRetirementReceipt,MuleUiCleanupRequest};
mod actor_updates;
mod bookkeeping_updates;
mod mission_funds;
mod mission_status;
mod family_scheduler;
mod plant_updates;
mod family8_updates;
mod building_updates;
mod terrain_updates;
pub use terrain_updates::{TerrainUpdateError, TerrainUpdateReceipt};
mod building_storage;
mod building_scheduler;
mod admitted_buildings;
mod extractor_updates;
pub use extractor_updates::ExtractorWorldReceipt;
pub use admitted_buildings::{AdmittedBuildingCallback,AdmittedBuildingPass};
pub use building_scheduler::{IncompleteBuildingPass,Selector1BuildingPass,Selector1BuildingCallback};
mod construction_scheduler;
pub use construction_scheduler::ConstructionAwareActorCallback;
mod family8_retirement;
pub use family8_retirement::Family8RetirementReceipt;
pub use building_updates::{ConstructionWorldError, ArchitectStage0Receipt, ArchitectMotionReceipt};
pub use family8_updates::{Family8CreationReceipt, Family8StepReceipt, Family8StepPlan, Family8PassReceipt};
pub use family_scheduler::{CreaturePassReceipt, CreatureUiAnimationRequest};
pub use actor_updates::ActorMovingReceipt;
mod reveal;
pub use reveal::RevealReceipt;
pub use effect_updates::{LandingPhysicsContext, LandingStepReceipt, EffectStepReceipt, LandingEffectCallback, LandingEffectPassReceipt};

const MISSION_HASH: &str = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2";
const MAP_START: usize = 0x355E;
const MAP_BYTES: usize = 128 * 128 * 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorWorldError {
    MissionHash(String), InvalidPlayer(u8), PlayerIdentity { slot: usize, actual: u8 },
    MissionFunds(crate::mission_player_funds::MissionFundsError),
    MissionStatus(crate::mission_status::MissionStatusError),
    InvalidRngRing(u8), Pool(ActorPoolError), Player(PlayerStateError),
    Activation(ActorActivationError), Command(Opcode3MoveOrderError), Resolver(ResolverError),
    InvalidReference([u8; 3]), EntityIdentity([u8; 3]), BrokenPrevious([u8; 3]),
    DuplicateLink([u8; 3]), AlreadyLinked(i16), MissingActorLink(i16),
    ActorOwner(i16), ActorNotCommandable(i16), AssociatedUnitUnsupported,
    EnvironmentUnavailable, StaleLink,
    Plants(RuntimeError), PlantsNotAdmitted,
    MissingBlueprintBinding(i16), InvalidBlueprintCategory(u8),
    UnsupportedBlockerFamily(u8), UnsupportedBuildingQuery(usize),
    Effects(EffectPoolError), EffectsNotAdmitted, EffectsAlreadyAdmitted,
    Creatures(CreatureError), CreaturesAlreadyAdmitted,
    EffectCountMismatch { declared: i16, active: usize }, Landing(LandingError),
    LandingAlreadyAssociated(u8), Terrain(String),
    Physics(crate::physics_tables::PhysicsTableError),
    UnsupportedEffectEvent, MissingEffectLink(u16), EffectBinding(u16),
    Lifecycle(crate::landing_effect_lifecycle::LifecycleError),
    BuildingEffectLifecycle(crate::building_effect_lifecycle::BuildingEffectLifecycleError),
    CleanupBeforeImage { kind:u8, slot:u16, offset:u16 },
    InvalidPortraitState { current: i8, count: i8 },
    UnsupportedRevealFamily(u8),
    ActorMotion(crate::actor_motion::ActorMotionError),
    Bookkeeping(crate::bookkeeping::BookkeepingError),
    UnsupportedBookkeepingFamily(u8),
    UnsupportedCreatureSchedule,
    CreatureCountMismatch { declared: i16, active: usize },
    TargetSearchLimit,
    BuildingPool(crate::building_pool::BuildingPoolError),
    BuildingFactory(crate::building::BuildingError),
    BuildingProgress(crate::building_progress::BuildingProgressError),
    BuildingCallback(crate::building_callback::BuildingCallbackError),
    QuietBuilding(crate::completed_building::QuietBuildingError),
    ExtractorCallback(crate::extractor_production::ExtractorCallbackError),
    BuildingsNotAdmitted, BuildingsAlreadyAdmitted,
    BuildingCountMismatch{owner:u8,declared:i8,active:usize},
    Family8Pool(crate::family8_pool::PoolError),
    Family8Creation(crate::family8_creation::CreationError),
    Family8NotAdmitted, Family8AlreadyAdmitted,
    Family8CountMismatch { declared: u8, active: usize },
    Family8Motion(crate::family8_motion::Family8MotionError),
    MissingFamily8Link(u16), Family8Binding(u16),
}
impl From<crate::building_pool::BuildingPoolError> for ActorWorldError { fn from(e:crate::building_pool::BuildingPoolError)->Self{Self::BuildingPool(e)} }
impl From<crate::building::BuildingError> for ActorWorldError { fn from(e:crate::building::BuildingError)->Self{Self::BuildingFactory(e)} }
impl core::fmt::Display for ActorWorldError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for ActorWorldError {}
impl From<crate::family8_pool::PoolError> for ActorWorldError {
    fn from(e: crate::family8_pool::PoolError) -> Self { Self::Family8Pool(e) }
}
impl From<crate::family8_creation::CreationError> for ActorWorldError {
    fn from(e: crate::family8_creation::CreationError) -> Self { Self::Family8Creation(e) }
}
impl From<crate::family8_motion::Family8MotionError> for ActorWorldError {
    fn from(e: crate::family8_motion::Family8MotionError) -> Self { Self::Family8Motion(e) }
}
impl From<ActorPoolError> for ActorWorldError { fn from(e: ActorPoolError) -> Self { Self::Pool(e) } }
impl From<PlayerStateError> for ActorWorldError { fn from(e: PlayerStateError) -> Self { Self::Player(e) } }
impl From<ActorActivationError> for ActorWorldError { fn from(e: ActorActivationError) -> Self { Self::Activation(e) } }
impl From<RuntimeError> for ActorWorldError { fn from(e: RuntimeError) -> Self { Self::Plants(e) } }
impl From<EffectPoolError> for ActorWorldError { fn from(e: EffectPoolError) -> Self { Self::Effects(e) } }
impl From<CreatureError> for ActorWorldError { fn from(e: CreatureError) -> Self { Self::Creatures(e) } }
impl From<crate::actor_motion::ActorMotionError> for ActorWorldError {
    fn from(e:crate::actor_motion::ActorMotionError)->Self {Self::ActorMotion(e)}
}
impl From<crate::bookkeeping::BookkeepingError> for ActorWorldError {
    fn from(e:crate::bookkeeping::BookkeepingError)->Self {Self::Bookkeeping(e)}
}
impl From<LandingError> for ActorWorldError { fn from(e: LandingError) -> Self { Self::Landing(e) } }
impl From<crate::physics_tables::PhysicsTableError> for ActorWorldError {
    fn from(e: crate::physics_tables::PhysicsTableError) -> Self { Self::Physics(e) }
}
impl From<crate::landing_effect_lifecycle::LifecycleError> for ActorWorldError {
    fn from(e:crate::landing_effect_lifecycle::LifecycleError)->Self { Self::Lifecycle(e) }
}

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

/// Ground-control branch inputs after module/hit dispatch and its admitted
/// target producer.874B0 applies only to the minimap control, not terrain.
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
pub struct LandingCreationReceipt {
    pub slot: u16,
    pub player: u8,
    pub position: [i16; 2],
    pub ground_height: i32,
    pub shade: u8,
    /// Factory audio and caller reveal operations retain source order.
    /// These are still pending; this receipt does not mark startup complete.
    pub pending_events: Vec<LandingEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorWorld {
    backing: Vec<u8>,
    layouts: [MissionRecordGroupLayout; 8],
    actors: ActorPool,
    players: [PlayerState; 4],
    rng: BlueprintRng,
    plants: Option<PlantSubsystem>,
    effects: Option<Box<EffectPool>>,
    creatures: Option<Box<CreaturePool>>,
    family8: Option<Box<Family8Pool>>,
    buildings: Option<Box<BuildingPool>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlantSubsystem { parameters: PlantParameterTable, tick_number: u64 }

/// A temporary borrow of the world's sole backing and actor pool. Plant
/// callbacks read and rewrite actor links directly in the native records;
/// no packed actor mirror or second persistent map exists.
struct SharedPlantStorage<'a> { backing: &'a mut Vec<u8>, actors: &'a mut ActorPool, effects: Option<&'a mut EffectPool>, creatures: Option<&'a mut CreaturePool>, family8: Option<&'a mut Family8Pool>, buildings: Option<&'a mut BuildingPool> }
impl core::ops::Deref for SharedPlantStorage<'_> {
    type Target = [u8];
    fn deref(&self) -> &[u8] { self.backing }
}
impl core::ops::DerefMut for SharedPlantStorage<'_> {
    fn deref_mut(&mut self) -> &mut [u8] { self.backing }
}
impl PlantStorage for SharedPlantStorage<'_> {
    fn common_record(&self, kind: u8, index: u16) -> Result<&[u8], RuntimeError> {
        if kind == 4 {
            if let Some(pool) = self.buildings.as_ref() {
                return pool.record(usize::from(index)).map(|r| &r[..31])
                    .map_err(|_| RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind });
            }
        }
        if kind == 8 {
            if let Some(pool) = self.family8.as_ref() {
                return pool.record(usize::from(index)).map(|r| &r[..31])
                    .map_err(|_| RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind });
            }
        }
        if kind == 1 {
            if let Some(pool) = self.creatures.as_ref() {
                return pool.record(usize::from(index)).map(|r| &r[..31])
                    .map_err(|_| RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind });
            }
        }
        if kind == 3 {
            return self.actors.record(usize::from(index)).map(|r| &r[..31])
                .map_err(|_| RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind });
        }
        if kind == 5 {
            if let Some(pool) = self.effects.as_ref() {
                return pool.record(usize::from(index)).map(|r| &r[..31])
                    .map_err(|_| RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind });
            }
        }
        self.backing.common_record(kind, index)
    }
    fn write_common_reference(&mut self, kind: u8, index: u16, offset: usize, value: [u8; 3]) -> Result<(), RuntimeError> {
        if offset != 0 && offset != 3 { return Err(RuntimeError::StorageInvariant("invalid actor link offset")); }
        if kind == 4 {
            if let Some(pool) = self.buildings.as_mut() {
                let before = *pool.record(usize::from(index)).map_err(|_| RuntimeError::StorageInvariant("invalid buildings slot"))?;
                let mut after = before;
                after[offset..offset + 3].copy_from_slice(&value);
                return pool.replace_record_checked(usize::from(index), &before, &after)
                    .map_err(|_| RuntimeError::StorageInvariant("buildings link replacement rejected"));
            }
        }
        if kind == 8 {
            if let Some(pool) = self.family8.as_mut() {
                let before = *pool.record(usize::from(index)).map_err(|_| RuntimeError::StorageInvariant("invalid family8 slot"))?;
                let mut after = before;
                after[offset..offset + 3].copy_from_slice(&value);
                return pool.replace_record_checked(usize::from(index), &before, &after)
                    .map_err(|_| RuntimeError::StorageInvariant("family8 link replacement rejected"));
            }
        }
        if kind == 1 {
            if let Some(pool) = self.creatures.as_mut() {
                let before = *pool.record(usize::from(index)).map_err(|_| RuntimeError::StorageInvariant("invalid creature slot"))?;
                let mut after = before;
                after[offset..offset + 3].copy_from_slice(&value);
                return pool.replace_record_checked(usize::from(index), &before, &after)
                    .map_err(|_| RuntimeError::StorageInvariant("creature link replacement rejected"));
            }
        }
        if kind == 5 {
            if let Some(pool) = self.effects.as_mut() {
                let before = *pool.record(usize::from(index)).map_err(|_| RuntimeError::StorageInvariant("invalid effect slot"))?;
                let mut after = before;
                after[offset..offset + 3].copy_from_slice(&value);
                return pool.replace_record_checked(usize::from(index), &before, &after)
                    .map_err(|_| RuntimeError::StorageInvariant("effect link replacement rejected"));
            }
        }
        if kind != 3 { return self.backing.write_common_reference(kind, index, offset, value); }
        let before = *self.actors.record(usize::from(index))
            .map_err(|_| RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind })?;
        let mut after = before;
        after[offset..offset + 3].copy_from_slice(&value);
        self.actors.replace_record_checked(usize::from(index), &before, &after)
            .map_err(|_| RuntimeError::StorageInvariant("actor link replacement rejected"))
    }
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
        let result = Self { backing: source.bytes().to_vec(), layouts: groups.map(|g| g.layout()), actors, players, rng, plants: None, effects: None, creatures: None, family8: None, buildings: None };
        result.validate_links()?;
        Ok(result)
    }

    pub fn actors(&self) -> &ActorPool { &self.actors }
    pub fn effects(&self) -> Option<&EffectPool> { self.effects.as_deref() }
    pub fn creatures(&self) -> Option<&CreaturePool> { self.creatures.as_deref() }
    pub fn family8(&self) -> Option<&Family8Pool> { self.family8.as_deref() }
    pub fn buildings(&self)->Option<&BuildingPool>{self.buildings.as_deref()}

    /// Promote the current packed kind1 records into their native overlay.
    /// Existing common links are copied exactly. This admits storage only;
    /// creature AI remains a separate source-checked update boundary.
    pub fn admit_creatures(&mut self, prior: &[CreatureRecord; 300]) -> Result<(), ActorWorldError> {
        if self.creatures.is_some() { return Err(ActorWorldError::CreaturesAlreadyAdmitted); }
        let group = self.layouts[0];
        let pool = CreaturePool::from_packed(&self.backing[group.file_offset..group.file_offset + group.count * group.packed_stride], prior)?;
        self.validate_links()?;
        let mut staged = self.clone();staged.creatures = Some(Box::new(pool));staged.validate_links()?;
        *self = staged;Ok(())
    }

    /// Load the inactive family4 packed records into owned native storage,
    /// keeping the caller's constructor/resource gap explicit.
    pub fn admit_effects(&mut self, prior: &[LandingRecord; 200], bindings: [Option<EffectAnimationBinding>; 200]) -> Result<(), ActorWorldError> {
        if self.effects.is_some() { return Err(ActorWorldError::EffectsAlreadyAdmitted); }
        let group = self.layouts[4];
        let pool = EffectPool::from_packed_inactive(&self.backing[group.file_offset..group.file_offset + group.count * group.packed_stride], prior, bindings)?;
        let declared = word(&self.backing, 0x3554);
        let active = pool.records().iter().filter(|r| r[6] != 0).count();
        if declared < 0 || declared as usize != active { return Err(ActorWorldError::EffectCountMismatch { declared, active }); }
        self.validate_links()?;
        self.effects = Some(Box::new(pool));
        Ok(())
    }

    /// Commit the proved factory's record, count, map links and player
    /// reference. Audio/reveal operations remain explicit pending events.
    /// No descent/contact, actor readiness, RNG draw or full tick is implied.
    pub fn create_landing_core(&mut self, player: u8, selected_player: u8) -> Result<LandingCreationReceipt, ActorWorldError> {
        let state = self.player(player)?;
        if state.bytes()[0x498] != 0 { return Err(ActorWorldError::LandingAlreadyAssociated(player)); }
        let position = state.target();
        self.validate_links()?;
        let ground = genewars_core::cd_candidate::interpolate_height(self.map(), position[0], position[1]).map_err(|e| ActorWorldError::Terrain(e.to_string()))?;
        let shade = genewars_core::cd_candidate::interpolate_entity_shade(self.map(), position[0], position[1]).map_err(|e| ActorWorldError::Terrain(e.to_string()))?;
        let cell = floor_cell(position);
        let old_raw = reference(self.map(), cell * 14 + 6);
        let old_head = if old_raw[2] == 0 { None } else {
            self.entity(old_raw)?;
            Some(landing::ResolvedCellHead { identity: landing::LandingReference { index: word(&old_raw,0), kind: old_raw[2] } })
        };
        let pool = self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        let declared = word(&self.backing, 0x3554);
        let active = pool.records().iter().filter(|r| r[6] != 0).count();
        if declared < 0 || declared as usize != active { return Err(ActorWorldError::EffectCountMismatch { declared, active }); }
        let plan = landing::plan_landing_factory(pool.records(), declared, player, selected_player, position, ground, shade, old_head)?;
        let mut staged = self.clone();
        staged.effects.as_mut().unwrap().commit_landing_factory(&plan)?;
        let raw = [(plan.slot & 255) as u8, (plan.slot >> 8) as u8, 5];
        if old_raw[2] != 0 { staged.replace_previous(old_raw, raw)?; }
        staged.backing[MAP_START + cell * 14 + 6..MAP_START + cell * 14 + 9].copy_from_slice(&raw);
        staged.backing[0x3554..0x3556].copy_from_slice(&plan.pool_count_after.to_le_bytes());
        staged.players[usize::from(player)].set_landing_reference(plan.slot);
        staged.validate_links()?;
        let pending_events = plan.events.into_iter().filter(|event| matches!(event, LandingEvent::PlayAudio(_) | LandingEvent::Reveal { .. })).collect();
        *self = staged;
        Ok(LandingCreationReceipt { slot: plan.slot, player, position, ground_height: ground, shade, pending_events })
    }

    /// Attach hash-admitted plant parameters to this existing world. This
    /// does not reload the source, reset plants or replace any map links.
    pub fn admit_plants(&mut self, parameter_bytes: &[u8]) -> Result<(), ActorWorldError> {
        let parameters = PlantParameterTable::parse(parameter_bytes)?;
        self.validate_links()?;
        let tick_number = self.plants.as_ref().map_or(0, |p| p.tick_number);
        MissionWorld::with_shared_storage(SharedPlantStorage { backing: &mut self.backing, actors: &mut self.actors, effects: self.effects.as_deref_mut(), creatures: self.creatures.as_deref_mut(), family8: self.family8.as_deref_mut(), buildings: self.buildings.as_deref_mut() }, parameters.clone(), tick_number)?;
        self.plants = Some(PlantSubsystem { parameters, tick_number });
        Ok(())
    }

    pub fn plant_tick_number(&self) -> Option<u64> { self.plants.as_ref().map(|p| p.tick_number) }

    pub fn plant_render_records(&self) -> Result<Vec<crate::world::PlantRenderRecord>, ActorWorldError> {
        Ok(crate::world::plant_render_records(&self.backing)?)
    }

    /// Run only the admitted plant scheduling pass against the shared map.
    /// A rejected callback rolls back actor links, packed records and all
    /// subsystem state together. This is not the full-family game scheduler.
    pub fn tick_plants(&mut self) -> Result<PlantTickReport, ActorWorldError> {
        let subsystem = self.plants.as_ref().ok_or(ActorWorldError::PlantsNotAdmitted)?.clone();
        self.validate_links()?;
        let mut staged = self.clone();
        let mut view = MissionWorld::with_shared_storage(SharedPlantStorage { backing: &mut staged.backing, actors: &mut staged.actors, effects: staged.effects.as_deref_mut(), creatures: staged.creatures.as_deref_mut(), family8: staged.family8.as_deref_mut(), buildings: staged.buildings.as_deref_mut() }, subsystem.parameters, subsystem.tick_number)?;
        let report = view.tick_plants_inner()?;
        staged.plants.as_mut().unwrap().tick_number = report.tick_number;
        staged.validate_links()?;
        *self = staged;
        Ok(report)
    }
    pub fn player(&self, player: u8) -> Result<&PlayerState, ActorWorldError> {
        self.players.get(usize::from(player)).ok_or(ActorWorldError::InvalidPlayer(player))
    }
    pub fn rng(&self) -> BlueprintRng { self.rng }
    pub fn map(&self) -> &[u8] { &self.backing[MAP_START..MAP_START + MAP_BYTES] }

    pub fn terrain_class_at(&self, point: [i16; 2]) -> Result<u8, ActorWorldError> {
        Ok(crate::world::map_terrain_class(self.map(), point)?)
    }

    /// Source74270/74160 mask0, negative-height movement query. Reserved
    /// cells without a linked building return NULL. Positive-height queries
    /// need separately admitted animation bounds and do not use this API.
    pub fn move_obstacle_at(&self, point: [i16; 2]) -> Result<bool, ActorWorldError> {
        self.move_obstacle_with_flags(point,0,None)
    }
    ///74D90 nominated-building exception, preserving74270's first-result
    /// query order. The caller supplies an already resolved type4 identity.
    pub fn move_obstacle_at_excluding(&self,point:[i16;2],nominated:Option<u16>)->Result<bool,ActorWorldError> {
        self.move_obstacle_with_flags(point,1,nominated)
    }
    fn move_obstacle_with_flags(&self,point:[i16;2],required_flags:u8,nominated:Option<u16>)->Result<bool,ActorWorldError> {
        if let Some(slot)=nominated{self.entity([slot as u8,(slot>>8)as u8,4])?;}
        crate::building_obstacle::move_obstacle_filtered(self.map(),point,required_flags,nominated,|cell|{
            let mut raw=reference(self.map(),cell*14+6);let mut seen=HashSet::new();
            while raw[2]!=0 {
                if !seen.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
                let record=self.entity(raw)?;
                if raw[2]==4 {
                    if self.buildings.is_none(){return Err(ActorWorldError::BuildingsNotAdmitted);}
                    return Ok(Some(record.try_into().map_err(|_|ActorWorldError::UnsupportedBuildingQuery(cell))?));
                }
                raw=reference(record,3);
            }
            Ok(None)
        }).map_err(|e|match e{
            crate::building_obstacle::BuildingObstacleError::Query(e)=>e,
            _=>ActorWorldError::UnsupportedBuildingQuery(floor_cell(point)),
        })
    }

    /// Source8B130 for an owned type3 actor. The only weighted objects in
    /// the admitted domain are plants; kinds1/3 have pure skip predicates.
    /// Native plant+2E is serialized at packed+26, not packed+2E.
    pub fn cell_plant_blocker(&self, id: i16, point: [i16; 2]) -> Result<i32, ActorWorldError> {
        let own = [id as u8, (id as u16 >> 8) as u8, 3];
        let actor = self.entity(own)?;
        if actor[0x4A] & 8 != 0 { return Ok(0); }
        let handle = self.actors.bindings(id as usize)?.blueprint.ok_or(ActorWorldError::MissingBlueprintBinding(id))?;
        if handle.player_id != actor[0x18] { return Err(ActorWorldError::MissingBlueprintBinding(id)); }
        let category = self.player(handle.player_id)?.blueprint(usize::from(handle.blueprint_index))?[0x16];
        if category > 3 { return Err(ActorWorldError::InvalidBlueprintCategory(category)); }
        let cell = floor_cell(point);
        let nominated = if category == 1 && actor[0x81] == 2 {
            let raw = reference(actor, 0x7F);
            let plant = self.entity(raw)?;
            if floor_cell([word(plant,7),word(plant,9)]) == cell { return Ok(0); }
            Some(raw)
        } else { None };
        let mut raw = reference(self.map(), cell * 14 + 6);
        let mut previous = [(cell & 255) as u8, (cell >> 8) as u8, 0];
        let mut seen = HashSet::new();
        let mut weight = 0;
        while raw[2] != 0 {
            if !seen.insert(raw) { return Err(ActorWorldError::DuplicateLink(raw)); }
            let record = self.entity(raw)?;
            if reference(record, 0) != previous { return Err(ActorWorldError::BrokenPrevious(raw)); }
            if raw != own && Some(raw) != nominated {
                match raw[2] {
                    // Building vtable +0x10 is 1F410 (false); 8B130 adds weight only for plants.
                    1 | 3 | 4 | 5 | 8 => (),
                    2 => {
                        let index = word(&raw,0) as usize;
                        let offset = self.layouts[1].file_offset + index * self.layouts[1].packed_stride;
                        if self.backing[offset + 0x26] & 8 == 0 { weight += 10; }
                    },
                    kind => return Err(ActorWorldError::UnsupportedBlockerFamily(kind)),
                }
            }
            previous = raw;
            raw = reference(record, 3);
        }
        Ok(weight)
    }

    pub fn placement_environment(&self, id: i16, point: [i16; 2]) -> Result<PlacementEnvironment, ActorWorldError> {
        let actor = self.entity([id as u8, (id as u16 >> 8) as u8, 3])?;
        let terrain_class = self.terrain_class_at(point)?;
        // The source acceptance predicate short-circuits before8B130.
        let blocker_weight = if actor[0x53] & (1 << terrain_class) != 0 { 0 } else { self.cell_plant_blocker(id, point)? };
        Ok(PlacementEnvironment { terrain_class, blocker_weight })
    }

    pub fn activate_without_associated_unit_owned(&mut self, player: u8, tables: &TrigTables) -> Result<ActivationBatchReceipt, ActorWorldError> {
        self.activate_without_associated_unit(player, tables, |world, slot, candidate| {
            world.placement_environment(i16::from(player) * 5 + slot as i16, candidate.position_u15.map(|v| v as i16))
        })
    }

    pub fn activate_associated_unit_owned(&mut self, player: u8, tables: &TrigTables) -> Result<ActivationBatchReceipt, ActorWorldError> {
        self.activate_associated_unit(player, tables, |world, slot, candidate| {
            let id = i16::from(player) * 5 + slot as i16;
            let point = candidate.position_u15.map(|v| v as i16);
            let placement = world.placement_environment(id, point)?;
            let actor = world.actors.record(id as usize)?;
            let accepted = actor[0x53] & (1 << placement.terrain_class) == 0 && placement.blocker_weight < 10;
            let obstacle_present = if accepted && actor[0x71] & 0x10 == 0 { world.move_obstacle_at(point)? } else { false };
            Ok(AssociatedPlacementEnvironment { placement, obstacle_present })
        })
    }
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
        self.apply_move_with_query(player, id, requested, |_, _| {
            Ok(ResolverEnvironment { terrain_class, actor_avoidance_mask: 0, obstacle_present, blocker_weight })
        })
    }

    /// Resolve an order from current owned map/list fields. Setter no-op and
    /// reinit gates precede queries. A near request queries the snapped current
    /// point before returning it, preserving the source's helper call order.
    pub fn apply_bounded_move_order_owned(&mut self, player: u8, id: i16, requested: [i16; 2]) -> Result<Opcode3MoveOrderResult, ActorWorldError> {
        self.apply_move_with_query(player, id, requested, |world, actor| {
            let current = [word(actor,7), word(actor,9)];
            let delta = std::array::from_fn::<_,2,_>(|i| ((i32::from(current[i]) - i32::from(requested[i]) + 16384) & 32767) - 16384);
            let mut environment = ResolverEnvironment { terrain_class: 0, actor_avoidance_mask: actor[0x53], obstacle_present: false, blocker_weight: 0 };
            let probe = if delta[0] * delta[0] + delta[1] * delta[1] <= 4096 { current } else { requested };
            environment.obstacle_present = world.move_obstacle_at(probe)?;
            environment.terrain_class = world.terrain_class_at(probe)?;
            if actor[0x53] & (1 << environment.terrain_class) == 0 {
                environment.blocker_weight = world.cell_plant_blocker(id, probe)?;
            }
            Ok(environment)
        })
    }

    fn apply_move_with_query(
        &mut self, player: u8, id: i16, requested: [i16; 2],
        query: impl FnOnce(&Self, &[u8; 200]) -> Result<ResolverEnvironment, ActorWorldError>,
    ) -> Result<Opcode3MoveOrderResult, ActorWorldError> {
        self.apply_move_with_resolver(player,id,requested,|world,before| {
            let mut environment=query(world,before)?;
            environment.actor_avoidance_mask=before[0x53];
            actor_command::resolve_bounded_move_target([word(before,7),word(before,9)],requested,environment)
                .map_err(ActorWorldError::Resolver)
        })
    }

    /// Source74A50 backward search using the same owned actor query domain
    /// as bounded commands. No-op and reinitialization gates precede queries.
    pub fn apply_searched_move_order_owned(&mut self,player:u8,id:i16,requested:[i16;2],
        physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables)->Result<Opcode3MoveOrderResult,ActorWorldError> {
        self.apply_move_with_resolver(player,id,requested,|world,before| {
            let result=crate::target_resolver::resolve_target([word(before,7),word(before,9)],requested,physics,trig,|point| {
                // Source74270 obstacle query precedes terrain, even when a
                // later predicate rejects before the cell-blocker query.
                let obstacle=world.move_obstacle_at(point)?;
                let class=world.terrain_class_at(point)?;
                if before[0x53]&(1<<class)!=0||obstacle {return Ok(false);}
                Ok(world.cell_plant_blocker(id,point)?<10)
            });
            match result {
                Ok(resolved)=>Ok(actor_command::PrecomputedResolvedTarget::new(resolved.point)),
                Err(crate::target_resolver::TargetResolverError::Query(error))=>Err(error),
                Err(crate::target_resolver::TargetResolverError::IterationLimit)=>Err(ActorWorldError::TargetSearchLimit),
            }
        })
    }

    fn apply_move_with_resolver(&mut self,player:u8,id:i16,requested:[i16;2],
        resolve:impl FnOnce(&Self,&[u8;200])->Result<actor_command::PrecomputedResolvedTarget,ActorWorldError>)
        ->Result<Opcode3MoveOrderResult,ActorWorldError> {
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
        let resolved=resolve(self,&before)?;
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
        let Some(id) = self.ground_move_actor(player, input)? else { return Ok(GroundMoveOutcome::Ignored); };
        let result=self.apply_bounded_move_order(player,id,input.projected_point,environment.terrain_class,
            obstacle_present,environment.blocker_weight)?;
        Ok(GroundMoveOutcome::Dispatched { actor_id:id,opcode:3,result })
    }

    pub fn dispatch_ground_move_owned(&mut self, player: u8, input: GroundMoveInput) -> Result<GroundMoveOutcome, ActorWorldError> {
        let Some(id) = self.ground_move_actor(player, input)? else { return Ok(GroundMoveOutcome::Ignored); };
        let result = self.apply_bounded_move_order_owned(player, id, input.projected_point)?;
        Ok(GroundMoveOutcome::Dispatched { actor_id:id,opcode:3,result })
    }

    pub fn dispatch_ground_move_searched(&mut self,player:u8,input:GroundMoveInput,
        physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables)->Result<GroundMoveOutcome,ActorWorldError> {
        let Some(id)=self.ground_move_actor(player,input)? else{return Ok(GroundMoveOutcome::Ignored);};
        let result=self.apply_searched_move_order_owned(player,id,input.projected_point,physics,trig)?;
        Ok(GroundMoveOutcome::Dispatched{actor_id:id,opcode:3,result})
    }

    fn ground_move_actor(&self, player: u8, input: GroundMoveInput) -> Result<Option<i16>, ActorWorldError> {
        let count=self.player(player)?.actor_count();
        if !(0..=5).contains(&count) || !(0..=4).contains(&input.current_portrait) {
            return Err(ActorWorldError::InvalidPortraitState { current:input.current_portrait,count });
        }
        if input.buttons&8==0 || input.modifiers&0x0A!=0 || input.pause_flags&2!=0
            || input.group_word_526!=0 || input.group_byte_52f!=0
            || input.current_portrait>=count || input.selected[2]!=3 {
            return Ok(None);
        }
        let selected=self.entity(input.selected)?;
        if selected[0x18]!=player || selected[0x70]&1==0 || (2..=5).contains(&(selected[0x55]as i8)) {
            return Ok(None);
        }
        Ok(Some(word(selected,15)))
    }

    /// Common prefix lookup. Type3 always uses the native overlay; other
    /// families use their proved serialized common first31 bytes only.
    pub fn entity(&self, raw: [u8; 3]) -> Result<&[u8], ActorWorldError> {
        let id = i16::from_le_bytes([raw[0], raw[1]]);
        if !(1..=8).contains(&raw[2]) || id < 0 { return Err(ActorWorldError::InvalidReference(raw)); }
        let layout = self.layouts[usize::from(raw[2] - 1)];
        if id as usize >= layout.count { return Err(ActorWorldError::InvalidReference(raw)); }
        let record: &[u8] = if raw[2] == 3 { self.actors.record(id as usize)? }
        else if raw[2] == 1 && self.creatures.is_some() { self.creatures.as_ref().unwrap().record(id as usize)? }
        else if raw[2] == 5 && self.effects.is_some() { self.effects.as_ref().unwrap().record(id as usize)? }
        else if raw[2] == 4 && self.buildings.is_some() { self.buildings.as_ref().unwrap().record(id as usize)? }
        else if raw[2] == 8 && self.family8.is_some() { self.family8.as_ref().unwrap().record(id as usize)? } else {
            let start = layout.file_offset + id as usize * layout.packed_stride;
            &self.backing[start..start + 31]
        };
        if record[6] != raw[2] || word(record, 15) != id { return Err(ActorWorldError::EntityIdentity(raw)); }
        Ok(record)
    }

    fn replace_previous(&mut self, raw: [u8; 3], previous: [u8; 3]) -> Result<(), ActorWorldError> {
        self.entity(raw)?;
        let id = i16::from_le_bytes([raw[0], raw[1]]);
        if raw[2] == 1 && self.creatures.is_some() {
            let pool = self.creatures.as_mut().unwrap();let before = *pool.record(id as usize)?;let mut after = before;
            after[..3].copy_from_slice(&previous);pool.replace_record_checked(id as usize,&before,&after)?;
        } else if raw[2] == 4 && self.buildings.is_some() {
            let pool = self.buildings.as_mut().unwrap();let before = *pool.record(id as usize)?;let mut after = before;
            after[..3].copy_from_slice(&previous);pool.replace_record_checked(id as usize,&before,&after)?;
        } else if raw[2] == 8 && self.family8.is_some() {
            let pool = self.family8.as_mut().unwrap();let before = *pool.record(id as usize)?;let mut after = before;
            after[..3].copy_from_slice(&previous);pool.replace_record_checked(id as usize,&before,&after)?;
        } else if raw[2] == 3 {
            let before = *self.actors.record(id as usize)?;
            let mut after = before;
            after[..3].copy_from_slice(&previous);
            self.actors.replace_record_checked(id as usize, &before, &after)?;
        } else if raw[2] == 5 && self.effects.is_some() {
            let pool = self.effects.as_mut().unwrap();
            let before = *pool.record(id as usize)?;
            let mut after = before;
            after[..3].copy_from_slice(&previous);
            pool.replace_record_checked(id as usize, &before, &after)?;
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
fn floor_cell(point: [i16; 2]) -> usize { usize::from((point[1] as u16 >> 8) & 127) * 128 + usize::from((point[0] as u16 >> 8) & 127) }

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

    fn shared_plant_fixture(order: &[[u8; 3]]) -> ActorWorld {
        let mut world = fixture();
        prepare(&mut world, 0);
        world.activate_without_associated_unit(0, &TrigTables::generate_pinned().unwrap(), accept).unwrap();
        let actor = *world.actors.record(0).unwrap();
        let cell = cell_for([word(&actor, 7) as u16, word(&actor, 9) as u16]);
        let cell_ref = [(cell & 255) as u8, (cell >> 8) as u8, 0];
        let mut count = 0u16;
        for (index, raw) in order.iter().copied().enumerate() {
            let previous = if index == 0 { cell_ref } else { order[index - 1] };
            let next = order.get(index + 1).copied().unwrap_or([0xA5, 0x5A, 0]);
            if raw[2] == 3 {
                let before = *world.actors.record(raw[0] as usize).unwrap();
                let mut after = before;
                after[..3].copy_from_slice(&previous);
                after[3..6].copy_from_slice(&next);
                world.actors.replace_record_checked(raw[0] as usize, &before, &after).unwrap();
            } else {
                let layout = world.layouts[usize::from(raw[2]-1)];
                let start = layout.file_offset + usize::from(raw[0]) * layout.packed_stride;
                let record = &mut world.backing[start..start + layout.packed_stride];
                record[..3].copy_from_slice(&previous);
                record[3..6].copy_from_slice(&next);
                record[6] = raw[2];
                record[7..11].copy_from_slice(&actor[7..11]);
                record[15..17].copy_from_slice(&[raw[0], 0]);
                record[24] = 4;
                if raw[2] == 2 { count += 1; }
                else if raw[2] == 1 { record[0x7A-8] = 4; }
            }
        }
        world.backing[MAP_START + cell * 14 + 6..MAP_START + cell * 14 + 9].copy_from_slice(&order[0]);
        world.backing[0x3552..0x3554].copy_from_slice(&count.to_le_bytes());
        world.plants = Some(PlantSubsystem { parameters: PlantParameterTable::fixture_for_shared_test(), tick_number: 0 });
        world.validate_links().unwrap();
        world
    }

    #[test]
    fn family8_factory_preserves_native_links_and_null_exhaustion() {
        let mut world = shared_plant_fixture(&[[8,0,2],[0,0,1],[1,0,3],[0,0,3]]);
        world.admit_creatures(&[[0;124];300]).unwrap();
        world.admit_family8(&[[0xA5;102];16],[None;16]).unwrap();
        let point = [word(world.entity([8,0,2]).unwrap(),7),word(world.entity([8,0,2]).unwrap(),9)];
        let before = world.clone();
        for slot in 0..16 {
            let receipt = world.create_family8(slot%3,slot%4,point,25600).unwrap().unwrap();
            assert_eq!(receipt.slot,slot as u16);
            assert_eq!(&world.family8().unwrap().record(slot as usize).unwrap()[31..39],&[0xA5;8]);
            assert_eq!(world.entity([slot,0,8]).unwrap().len(),102);
        }
        assert_eq!(world.backing[0x3559],16);
        assert_eq!(world.validate_links().unwrap().len(),20);
        assert_eq!(world.rng,before.rng);
        assert_eq!(world.players,before.players);
        let group = world.layouts[7];
        assert_eq!(&world.backing[group.file_offset..group.file_offset+group.count*group.packed_stride],
            &before.backing[group.file_offset..group.file_offset+group.count*group.packed_stride]);
        let full=world.clone();assert_eq!(world.create_family8(0,0,point,25600).unwrap(),None);assert_eq!(world,full);
        assert_eq!(world.step_empty_family8_countdown(0,0),Err(ActorWorldError::UnsupportedBookkeepingFamily(8)));
        assert_eq!(world,full);
    }

    #[test]
    fn family8_expiry_visits_newborns_and_rolls_back_later_unsupported_callback() {
        let mut world=fixture();world.admit_family8(&[[0;102];16],[None;16]).unwrap();
        world.backing[0x355c..0x355e].copy_from_slice(&1u16.to_le_bytes());
        let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let animation=crate::family8_motion::Family8Animation::fixture();
        let directions=crate::family8_motion::Family8DirectionalAnimations::fixture();
        let receipt=world.step_family8_callbacks(0,4,0,6001,0,&physics,&trig,&animation,&directions).unwrap();
        assert!(receipt.countdown.expired);assert_eq!(receipt.expiry_attempts.len(),4);
        assert_eq!(receipt.callbacks.iter().map(|c|c.slot).collect::<Vec<_>>(),vec![0,1,2,3]);
        assert_eq!(word(&world.backing,0x355c),0);
        for record in world.family8().unwrap().records().iter().take(4){assert_eq!(record[0x1b],1);}
        world.create_family8(2,0,[4000,4000],25000).unwrap();let before=world.clone();
        assert!(matches!(world.step_family8_callbacks(0,4,0,6002,0,&physics,&trig,&animation,&directions),Err(ActorWorldError::Family8Motion(_))));
        assert_eq!(world,before);
    }

    #[test]
    fn family8_storage_admission_and_stale_factory_failures_are_atomic() {
        let mut world=fixture();world.backing[0x3559]=1;let before=world.clone();
        assert_eq!(world.admit_family8(&[[0;102];16],[None;16]),Err(ActorWorldError::Family8CountMismatch{declared:1,active:0}));
        assert_eq!(world,before);world.backing[0x3559]=0;
        let mut bindings=[None;16];bindings[0]=Some(Family8AnimationBinding::Offset80);
        world.admit_family8(&[[0;102];16],bindings).unwrap();let before=world.clone();
        assert_eq!(world.create_family8(0,0,[4096,4096],25600),Err(ActorWorldError::Family8Pool(crate::family8_pool::PoolError::StaleBinding(0))));
        assert_eq!(world,before);
        assert_eq!(world.admit_family8(&[[0;102];16],[None;16]),Err(ActorWorldError::Family8AlreadyAdmitted));
        assert_eq!(world,before);
    }

    #[test]
    fn family8_plant_retirement_relinks_and_cleans_targets_with_late_rollback() {
        let directions_bytes:Vec<u8>=[[0i16,-256],[256,-256],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[0,0]]
            .into_iter().flatten().flat_map(i16::to_le_bytes).collect();
        let directions=crate::plant_spread::SpreadDirections::from_mapped_bytes(&directions_bytes).unwrap();
        for late_failure in [false,true] {
            let mut world=shared_plant_fixture(&[[8,0,2],[16,0,2],[0,0,1],[1,0,3],[0,0,3]]);
            world.admit_creatures(&[[0;124];300]).unwrap();world.admit_family8(&[[0;102];16],[None;16]).unwrap();
            let plant=world.entity([8,0,2]).unwrap();let point=[word(plant,7),word(plant,9)];
            world.create_family8(0,0,point,25600).unwrap();
            let pool=world.family8.as_mut().unwrap();let prior=*pool.record(0).unwrap();let mut after=prior;
            after[0x59..0x5c].copy_from_slice(&[8,0,2]);pool.replace_record_checked(0,&prior,&after).unwrap();
            if late_failure {world.backing[world.layouts[1].file_offset+16*41+24]=0;}
            let before=world.clone();let result=world.tick_plants_with_reproduction(&directions);
            if late_failure {assert!(result.is_err());assert_eq!(world,before);}
            else {
                assert_eq!(result.unwrap().active_plant_count_after,0);
                assert_eq!(reference(world.entity([0,0,8]).unwrap(),3),[0,0,1]);
                assert_eq!(reference(world.entity([0,0,1]).unwrap(),0),[0,0,8]);
                assert_eq!(&world.family8().unwrap().record(0).unwrap()[0x59..0x5c],&[8,0,0]);
                assert_eq!(world.rng,before.rng);assert_eq!(world.validate_links().unwrap().len(),4);
            }
        }
    }

    #[test]
    fn reveal_updates_only_map_factors_dirty_bits_and_eligible_plant_shades() {
        for protected in [false,true] {
            let mut world=shared_plant_fixture(&[[1,0,3],[8,0,2],[0,0,1],[0,0,3],[0,0,5]]);
            for cell in 0..128*128 {world.backing[MAP_START+cell*14..MAP_START+cell*14+2].copy_from_slice(&16384i16.to_le_bytes());}
            let plant=world.layouts[1].file_offset+8*world.layouts[1].packed_stride;
            world.backing[plant+0x15]=if protected{2}else{0};
            world.backing[plant+0x17]=3;
            let at=[word(world.entity([8,0,2]).unwrap(),7),word(world.entity([8,0,2]).unwrap(),9)];
            let before=world.clone();
            let receipt=world.apply_reveal_4096(at).unwrap();
            assert!(!receipt.changes.is_empty());
            assert_eq!(receipt.cell_visits,4*receipt.changes.len());
            assert_eq!(receipt.pending_presentation_vertices.len(),receipt.changes.len());
            assert_eq!(world.actors,before.actors);assert_eq!(world.players,before.players);assert_eq!(world.rng,before.rng);
            if protected {assert_eq!(receipt.plant_shade_writes,0);assert_eq!(world.backing[plant+0x17],3);}
            else {assert!(receipt.plant_shade_writes>0);assert_eq!(world.backing[plant+0x17],64);}
            for (i,(&old,&new))in before.backing.iter().zip(&world.backing).enumerate() {
                if old!=new {
                    if (MAP_START..MAP_START+MAP_BYTES).contains(&i) {
                        match (i-MAP_START)%14 {11=>assert_eq!(new,old|12),12=>assert!(new>old),_=>panic!("unexpected map write {i}")}
                    }else {assert_eq!(i,plant+0x17);}
                }
            }
            let stable=world.clone();let repeated=world.apply_reveal_4096(at).unwrap();
            assert!(repeated.changes.is_empty());assert_eq!(world,stable);
        }
    }

    #[test]
    fn reveal_unknown_virtual_predicate_rolls_back_all_prior_writes() {
        let mut world=shared_plant_fixture(&[[1,0,3],[8,0,2],[0,0,4],[0,0,3]]);
        let actor=world.actors.record(0).unwrap();let at=[word(actor,7),word(actor,9)];
        let before=world.clone();
        assert_eq!(world.apply_reveal_4096(at).unwrap_err(),ActorWorldError::UnsupportedRevealFamily(4));
        assert_eq!(world,before);
    }

    #[test]
    fn shared_plant_retirement_rewrites_native_actor_neighbors_in_one_store() {
        for order in [vec![[8,0,2],[1,0,3],[0,0,3]], vec![[1,0,3],[8,0,2],[0,0,3]], vec![[1,0,3],[0,0,3],[8,0,2]]] {
            let mut world = shared_plant_fixture(&order);
            let before = world.clone();
            let report = world.tick_plants().unwrap();
            assert_eq!(report.callbacks, [8]);
            assert_eq!(report.active_plant_count_after, 0);
            assert_eq!(world.plant_tick_number(), Some(1));
            assert_eq!(world.validate_links().unwrap().len(), 2);
            assert_eq!(reference(world.actors.record(1).unwrap(), 3), [0,0,3]);
            assert_eq!(reference(world.actors.record(0).unwrap(), 0), [1,0,3]);
            for id in [0,1] {
                assert_eq!(&world.actors.record(id).unwrap()[6..], &before.actors.record(id).unwrap()[6..]);
                assert_eq!(world.actors.bindings(id), before.actors.bindings(id));
            }
            let group = world.layouts[2];
            assert_eq!(&world.backing[group.file_offset..group.file_offset + group.count * group.packed_stride],
                       &before.backing[group.file_offset..group.file_offset + group.count * group.packed_stride], "no packed actor mirror is written");
            assert_eq!(world.players, before.players);
            assert_eq!(world.rng, before.rng);
        }
    }

    #[test]
    fn shared_plant_retirement_uses_native_creature_links_without_a_packed_mirror() {
        for order in [vec![[1,0,3],[0,0,1],[8,0,2],[0,0,3]],vec![[1,0,3],[8,0,2],[0,0,1],[0,0,3]]] {
            let mut world=shared_plant_fixture(&order);world.admit_creatures(&[[0xA5;124];300]).unwrap();
            let before=world.clone();world.tick_plants().unwrap();
            let creature=world.creatures().unwrap().record(0).unwrap();
            assert_eq!(reference(creature,0),[1,0,3]);assert_eq!(reference(creature,3),[0,0,3]);
            assert_eq!(&creature[6..],&before.creatures().unwrap().record(0).unwrap()[6..]);
            assert_eq!(world.entity([0,0,1]).unwrap().len(),124);
            let group=world.layouts[0];
            assert_eq!(&world.backing[group.file_offset..group.file_offset+group.count*group.packed_stride],
                &before.backing[group.file_offset..group.file_offset+group.count*group.packed_stride]);
            assert_eq!(world.validate_links().unwrap().len(),3);
            let snapshot=world.clone();assert_eq!(world.admit_creatures(&[[0;124];300]),Err(ActorWorldError::CreaturesAlreadyAdmitted));assert_eq!(world,snapshot);
        }
    }

    #[test]
    fn shared_plant_second_failure_rolls_back_first_retirement_and_actor_links() {
        let mut world = shared_plant_fixture(&[[1,0,3],[8,0,2],[16,0,2],[0,0,3]]);
        let second = world.layouts[1].file_offset + 16 * 41;
        world.backing[second + 24] = 0;
        let before = world.clone();
        assert_eq!(world.tick_plants().unwrap_err(), ActorWorldError::Plants(RuntimeError::UnsupportedOwnerCounter { index: 16, owner: 0 }));
        assert_eq!(world, before);
    }

    #[test]
    fn reproduction_route_clears_retired_plant_targets_and_rolls_back_late_failure() {
        let directions_bytes:Vec<u8>=[[0i16,-256],[256,-256],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[0,0]]
            .into_iter().flatten().flat_map(i16::to_le_bytes).collect();
        let directions=crate::plant_spread::SpreadDirections::from_mapped_bytes(&directions_bytes).unwrap();
        for late_failure in [false,true] {
            let mut world=shared_plant_fixture(&[[1,0,3],[8,0,2],[16,0,2],[0,0,1],[0,0,3]]);
            world.admit_creatures(&[[0;124];300]).unwrap();
            let prior=*world.actors.record(0).unwrap();let mut actor=prior;actor[0x4d..0x50].copy_from_slice(&[8,0,2]);
            world.actors.replace_record_checked(0,&prior,&actor).unwrap();
            let pool=world.creatures.as_mut().unwrap();let prior=*pool.record(0).unwrap();let mut creature=prior;
            creature[0x55..0x58].copy_from_slice(&[8,0,2]);pool.replace_record_checked(0,&prior,&creature).unwrap();
            if late_failure {let second=world.layouts[1].file_offset+16*41;world.backing[second+24]=0;}
            let before=world.clone();let result=world.tick_plants_with_reproduction(&directions);
            if late_failure {assert!(result.is_err());assert_eq!(world,before);}
            else {
                assert_eq!(result.unwrap().active_plant_count_after,0);
                assert_eq!(&world.actors.record(0).unwrap()[0x4d..0x50],&[8,0,0]);
                assert_eq!(&world.creatures().unwrap().record(0).unwrap()[0x55..0x58],&[8,0,0]);
                assert_eq!(world.rng,before.rng);world.validate_links().unwrap();
            }
        }
    }

    #[test]
    fn shared_plant_growth_traverses_native_actors_without_changing_their_records() {
        let mut world = shared_plant_fixture(&[[1,0,3],[8,0,2],[0,0,3]]);
        let plant = world.layouts[1].file_offset + 8 * 41;
        world.backing[plant + 36..plant + 38].copy_from_slice(&4_i16.to_le_bytes());
        let actor = world.actors.record(0).unwrap();
        let cell = cell_for([word(actor,7) as u16,word(actor,9) as u16]);
        world.backing[MAP_START + cell * 14 + 13] = 1;
        let actors = world.actors.clone();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.active_plant_count_after, 1);
        assert_eq!(word(&world.backing, plant + 36), 5);
        assert_eq!(world.actors, actors);
    }

    #[test]
    fn owned_blocker_reads_packed_flags_and_observes_plant_retirement() {
        let mut world = shared_plant_fixture(&[[1,0,3],[8,0,2],[0,0,3]]);
        let actor = world.actors.record(0).unwrap();
        let point = [word(actor,7),word(actor,9)];
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),10);
        let plant = world.layouts[1].file_offset + 8 * 41;
        world.backing[plant + 0x26] = 0x88;
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),0);
        world.backing[plant + 0x26] = 0x80;
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),10, "retirement bit80 does not exempt a plant");
        world.tick_plants().unwrap();
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),0);
    }

    #[test]
    fn owned_blocker_applies_actor_ignore_and_category_one_plant_exemption() {
        let mut world = shared_plant_fixture(&[[1,0,3],[8,0,2],[0,0,3]]);
        let before = *world.actors.record(0).unwrap();
        let point = [word(&before,7),word(&before,9)];
        let mut after = before;
        after[0x4A] |= 8;
        world.actors.replace_record_checked(0,&before,&after).unwrap();
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),0);
        world.actors.replace_record_checked(0,&after,&before).unwrap();
        let mut player = *world.players[0].bytes();
        player[0x16 + 0x16] = 1;
        world.players[0] = PlayerState::from_record(&player).unwrap();
        after = before;
        after[0x7F..0x82].copy_from_slice(&[8,0,2]);
        world.actors.replace_record_checked(0,&before,&after).unwrap();
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),0);
        let stale = after;
        after[0x7F..0x82].copy_from_slice(&[9,0,2]);
        world.actors.replace_record_checked(0,&stale,&after).unwrap();
        assert!(matches!(world.cell_plant_blocker(0,point), Err(ActorWorldError::EntityIdentity(_))));
    }

    #[test]
    fn owned_move_queries_respect_setter_gates_and_snapped_probe_point() {
        let mut world = fixture();
        prepare(&mut world,0);
        world.activate_without_associated_unit(0,&TrigTables::generate_pinned().unwrap(),accept).unwrap();
        let actor = *world.actors.record(0).unwrap();
        let point = [word(&actor,7),word(&actor,9)];
        let near = [point[0].wrapping_sub(1),point[1]];
        assert_ne!(floor_cell(near),floor_cell(point));
        world.backing[MAP_START + floor_cell(near) * 14 + 11] |= 1;
        assert!(matches!(world.apply_bounded_move_order_owned(0,0,near).unwrap(), Opcode3MoveOrderResult::Applied { resolved, .. } if resolved == point));
        world.backing[MAP_START + floor_cell(point) * 14 + 11] |= 1;
        assert!(!world.move_obstacle_at(point).unwrap());
        assert!(matches!(world.apply_bounded_move_order_owned(0,0,near).unwrap(), Opcode3MoveOrderResult::Applied { resolved, .. } if resolved == point));
        let far = [point[0].wrapping_add(1024),point[1]];
        world.backing[MAP_START + floor_cell(far) * 14 + 11] |= 1;
        assert!(!world.move_obstacle_at(far).unwrap());
        let before=world.clone();
        assert_eq!(world.apply_bounded_move_order_owned(0,0,far).unwrap_err(),ActorWorldError::Resolver(ResolverError::BackwardSearchRequired));
        assert_eq!(world,before);
        let prior = *world.actors.record(0).unwrap();
        let mut inhibited = prior;
        inhibited[0x71] |= 0x10;
        world.actors.replace_record_checked(0,&prior,&inhibited).unwrap();
        let before = world.clone();
        assert_eq!(world.apply_bounded_move_order_owned(0,0,far).unwrap(), Opcode3MoveOrderResult::NoopFlag71);
        assert_eq!(world,before);
    }

    #[test]
    fn building_completion_cleans_live_creature_target_and_rolls_back_bad_count() {
        for bad_count in [false,true] {
            let mut world=shared_plant_fixture(&[[8,0,2],[0,0,1],[1,0,3],[0,0,3]]);
            world.admit_creatures(&[[0;124];300]).unwrap();
            world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
            let a=world.actors.record(0).unwrap();let at=[word(a,7),word(a,9)];
            let slot=world.create_building(0,1,at).unwrap().unwrap().slot;
            let before=*world.creatures.as_ref().unwrap().record(0).unwrap();let mut after=before;
            after[0x58..0x5a].copy_from_slice(&(slot as i16).to_le_bytes());after[0x5a]=4;
            after[0x78]=19;after[0x5f]=7;after[0x60]=0xff;
            world.creatures.as_mut().unwrap().replace_record_checked(0,&before,&after).unwrap();
            world.backing[0x3550..0x3552].copy_from_slice(&(if bad_count{2i16}else{1i16}).to_le_bytes());
            let snapshot=world.clone();let result=world.progress_selector1_building(slot,120,false);
            if bad_count {assert!(result.is_err());assert_eq!(world,snapshot);}
            else {
                let receipt=result.unwrap();assert!(receipt.completed);assert_eq!(receipt.creature_changes.len(),1);
                let a=world.creatures().unwrap().record(0).unwrap();
                assert_eq!((a[0x5a],a[0x78],a[0x5f],a[0x60]),(0,0,0,0xef));
                assert_eq!(world.rng(),snapshot.rng());world.validate_links().unwrap();
            }
        }
    }

    #[test]
    fn reserved_cell_query_uses_native_building_links_and_exact_extent() {
        let mut world=fixture();let point=[4096i16,3584i16];
        let cell=floor_cell(point);world.backing[MAP_START+cell*14+11]|=1;
        assert!(!world.move_obstacle_at(point).unwrap());
        world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
        let plan=world.create_building(0,1,point).unwrap().unwrap();
        assert_eq!(plan.slot,0);assert!(world.move_obstacle_at(point).unwrap());
        assert!(world.move_obstacle_at([point[0]+85,point[1]]).unwrap());
        assert!(!world.move_obstacle_at([point[0]+86,point[1]]).unwrap());
        let before=world.clone();assert!(world.move_obstacle_at(point).unwrap());assert_eq!(world,before);
    }

    #[test]
    fn owned_placement_short_circuits_rejected_terrain_before_blocker_lookup() {
        let mut world = fixture();
        prepare(&mut world,0);
        // Code31 family5 has pure false virtual10 and contributes no weight.
        let point = [0,0];
        let offset = world.layouts[4].file_offset;
        world.backing[offset + 6] = 5;
        world.backing[MAP_START + 6..MAP_START + 9].copy_from_slice(&[0,0,5]);
        assert_eq!(world.cell_plant_blocker(0,point).unwrap(),0);
        // A family with an unclosed virtual predicate rejects only if queried.
        world.backing[MAP_START + 6..MAP_START + 9].copy_from_slice(&[0,0,7]);
        let offset = world.layouts[6].file_offset;
        world.backing[offset + 6] = 7;
        assert_eq!(world.placement_environment(0,point).unwrap(), PlacementEnvironment { terrain_class:0, blocker_weight:0 });
        world.backing[MAP_START + 5] = 1;
        assert_eq!(world.placement_environment(0,point).unwrap_err(), ActorWorldError::UnsupportedBlockerFamily(7));
    }

    #[test]
    fn landing_factory_commits_native_pool_map_count_and_player_without_ready_actors() {
        let mut world = fixture();
        prepare(&mut world,0);
        world.admit_effects(&[[0xA5;81];200],[None;200]).unwrap();
        let before = world.clone();
        let receipt = world.create_landing_core(0,0).unwrap();
        assert_eq!(receipt.slot,0);
        assert_eq!(receipt.position,[4096,4096]);
        assert_eq!(world.players[0].bytes()[0x496..0x499],[0,0,5]);
        assert_eq!(world.players[0].origin(),before.players[0].origin());
        assert_eq!(word(&world.backing,0x3554),1);
        let landing = world.entity([0,0,5]).unwrap();
        assert_eq!(landing.len(),81);
        assert_eq!(word(landing,7),4096);
        assert_eq!(&landing[35..39],&[0xA5;4]);
        assert_eq!(world.effects().unwrap().binding(0).unwrap(),Some(EffectAnimationBinding::EffAnimation49));
        assert_eq!(world.actors,before.actors,"factory does not activate actors");
        assert_eq!(world.rng,before.rng,"factory code31 does not draw RNG");
        assert!(matches!(receipt.pending_events.as_slice(),[LandingEvent::PlayAudio(_),LandingEvent::Reveal{radius:4096,..}]));
        let before = world.clone();
        assert_eq!(world.create_landing_core(0,0),Err(ActorWorldError::LandingAlreadyAssociated(0)));
        assert_eq!(world,before);
    }

    #[test]
    fn landing_factory_late_stale_binding_failure_is_atomic() {
        let mut world = fixture();
        let mut bindings = [None;200];
        bindings[0] = Some(EffectAnimationBinding::EffAnimation49);
        world.admit_effects(&[[0;81];200],bindings).unwrap();
        let before = world.clone();
        assert_eq!(world.create_landing_core(0,0),Err(ActorWorldError::Effects(EffectPoolError::StaleBinding(0))));
        assert_eq!(world,before);
    }

    #[test]
    fn shared_retirement_updates_native_landing_neighbor_and_actor_link() {
        let mut world = shared_plant_fixture(&[[8,0,2],[1,0,3],[0,0,3]]);
        let mut player = *world.players[0].bytes();
        player[9..13].copy_from_slice(&world.actors.record(0).unwrap()[7..11]);
        world.players[0] = PlayerState::from_record(&player).unwrap();
        world.admit_effects(&[[0;81];200],[None;200]).unwrap();
        world.create_landing_core(0,0).unwrap();
        assert_eq!(reference(world.entity([0,0,5]).unwrap(),3),[8,0,2]);
        let before = *world.effects().unwrap().record(0).unwrap();
        world.tick_plants().unwrap();
        assert_eq!(reference(world.entity([0,0,5]).unwrap(),3),[1,0,3]);
        assert_eq!(reference(world.entity([1,0,3]).unwrap(),0),[0,0,5]);
        assert_eq!(&world.effects().unwrap().record(0).unwrap()[6..],&before[6..]);
        assert_eq!(world.validate_links().unwrap().len(),3);
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

    fn landing_update_fixture(stale_smoke_binding:bool)->ActorWorld {
        let mut world=fixture();prepare(&mut world,0);
        for cell in world.backing[MAP_START..MAP_START+MAP_BYTES].chunks_exact_mut(14) {
            cell[2..4].copy_from_slice(&2048i16.to_le_bytes());cell[5]=1;
        }
        let mut bindings=[None;200];
        if stale_smoke_binding { bindings[1]=Some(EffectAnimationBinding::EffAnimation6); }
        world.admit_effects(&[[0xA5;81];200],bindings).unwrap();
        world.create_landing_core(0,0).unwrap();world
    }

    #[test]
    fn landing_step_commits_smoke_rng_and_native_links_together() {
        let mut world=landing_update_fixture(false);
        let before=world.clone();
        let physics=crate::physics_tables::PhysicsTables::fixture();
        let trig=TrigTables::generate_pinned().unwrap();
        let event=world.step_landing_phase0(0,LandingPhysicsContext{water_height:0,gravity:1024},&physics,&trig).unwrap();
        assert!(!event.contact);assert_eq!(event.created_effect,Some(1));assert_eq!(event.draws.len(),4);
        assert_eq!(event.rng_after.draw_count,event.rng_before.draw_count.wrapping_add(4));
        assert_eq!(word(&world.backing,0x3554),2);
        let pool=world.effects().unwrap();
        assert_eq!(pool.binding(1).unwrap(),Some(EffectAnimationBinding::EffAnimation6));
        assert_eq!(pool.record(1).unwrap()[21]&1,1);
        assert_eq!(word(pool.record(1).unwrap(),55),-16);
        assert_eq!(reference(pool.record(0).unwrap(),0),[1,0,5]);
        assert_eq!(reference(pool.record(1).unwrap(),3),[0,0,5]);
        assert_eq!(reference(world.map(),floor_cell([4096,4096])*14+6),[1,0,5]);
        assert_eq!(world.actors(),before.actors());
        let packed=world.layouts[4];
        assert_eq!(&world.backing[packed.file_offset..packed.file_offset+packed.count*packed.packed_stride],
            &before.backing[packed.file_offset..packed.file_offset+packed.count*packed.packed_stride]);
    }

    #[test]
    fn landing_late_smoke_failure_rolls_back_motion_links_and_all_four_draws() {
        let mut world=landing_update_fixture(true);let before=world.clone();
        let error=world.step_landing_phase0(0,LandingPhysicsContext{water_height:0,gravity:1024},
            &crate::physics_tables::PhysicsTables::fixture(),&TrigTables::generate_pinned().unwrap()).unwrap_err();
        assert_eq!(error,ActorWorldError::Effects(EffectPoolError::StaleBinding(1)));
        assert_eq!(world,before);
    }

    #[test]
    fn landing_callback_contact_activates_associated_actors_without_forcing_ready() {
        let mut world=landing_update_fixture(false);
        let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{water_height:0,gravity:1024};
        // Deliberately only code31 callbacks: smoke remains frozen. This is
        // an integration fixture, not a claim about a complete frame sequence.
        for _ in 0..188 { assert!(!world.step_landing_phase0(0,context,&physics,&trig).unwrap().contact); }
        let seed_before=world.rng();
        let contact=world.step_landing_phase0(0,context,&physics,&trig).unwrap();
        assert!(contact.contact);assert_eq!(contact.created_effect,None);assert!(contact.draws.is_empty());
        let activation=contact.activation.unwrap();assert_eq!(activation.activations.len(),2);
        assert_eq!(activation.rng_before,seed_before);
        assert_eq!(world.player(0).unwrap().activation_count(),1);
        for slot in 0..2 {
            let actor=world.actors().record(slot).unwrap();assert_eq!(actor[0x70]&1,1);
            assert_eq!([word(actor,7),word(actor,9)],[4096,4096]);
        }
        assert_eq!(world.effects().unwrap().record(0).unwrap()[70],1);
        assert_eq!(word(world.effects().unwrap().record(0).unwrap(),74),150);
        let before=world.clone();assert!(world.step_landing_phase0(0,context,&physics,&trig).is_err());assert_eq!(world,before);
    }

    #[test]
    fn smoke_retirement_moves_unlinks_and_clears_native_and_packed_references() {
        for native_creature in [false,true] {
        let mut world=landing_update_fixture(false);
        let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{water_height:0,gravity:1024};
        world.step_landing_phase0(0,context,&physics,&trig).unwrap();
        let pool=world.effects.as_mut().unwrap();let before=*pool.record(1).unwrap();let mut smoke=before;
        smoke[27]=12;smoke[30]=1;smoke[45..47].copy_from_slice(&18000i16.to_le_bytes());
        smoke[71..74].copy_from_slice(&[1,0,5]);
        pool.replace_record_checked(1,&before,&smoke).unwrap();
        let before=*world.actors.record(0).unwrap();let mut actor=before;actor[0x7F..0x82].copy_from_slice(&[1,0,5]);
        world.actors.replace_record_checked(0,&before,&actor).unwrap();
        let start=world.layouts[0].file_offset;
        world.backing[start+6]=1;world.backing[start+15..start+17].copy_from_slice(&0i16.to_le_bytes());
        world.backing[start+0x5B-8..start+0x5E-8].copy_from_slice(&[1,0,5]);world.backing[start+0x60-8]=0xA7;
        world.backing[start+0x7A-8]=4;
        if native_creature { world.admit_creatures(&[[0xA5;124];300]).unwrap(); }
        let rng=world.rng();
        let event=world.step_effect6(1,context,&physics,&trig,&crate::landing_effect_lifecycle::RetirementUiState::native_empty()).unwrap();
        assert!(event.changed_cell);assert!(event.retired);assert_eq!(event.cleaned_reference_bytes,4);
        assert_eq!(world.rng(),rng);assert_eq!(word(&world.backing,0x3554),1);
        let retired=world.effects().unwrap().record(1).unwrap();assert_eq!(retired[6],0);assert_eq!(&retired[71..74],&[1,0,0]);
        assert_eq!(world.effects().unwrap().binding(1).unwrap(),None);
        assert_eq!(&world.actors().record(0).unwrap()[0x7F..0x82],&[1,0,0]);
        if native_creature {
            let creature=world.creatures().unwrap().record(0).unwrap();assert_eq!(&creature[0x5B..0x5E],&[1,0,0]);assert_eq!(creature[0x60],0xA5);
            assert_eq!(&world.backing[start+0x5B-8..start+0x5E-8],&[1,0,5]);assert_eq!(world.backing[start+0x60-8],0xA7);
        } else {
            assert_eq!(&world.backing[start+0x5B-8..start+0x5E-8],&[1,0,0]);assert_eq!(world.backing[start+0x60-8],0xA5);
        }
        // Singleton removal retains map head's index1, clearing kind only.
        let new_cell=floor_cell([word(retired,7),word(retired,9)]);
        assert_eq!(reference(world.map(),new_cell*14+6),[1,0,0]);
        assert_eq!(reference(world.map(),floor_cell([4096,4096])*14+6),[0,0,5]);
        assert!(world.validate_links().is_ok());
        }
    }

    #[test]
    fn smoke_late_cleanup_identity_failure_rolls_back_motion_and_unlinks() {
        let mut world=landing_update_fixture(false);let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{water_height:0,gravity:1024};world.step_landing_phase0(0,context,&physics,&trig).unwrap();
        let pool=world.effects.as_mut().unwrap();let before=*pool.record(1).unwrap();let mut smoke=before;smoke[27]=12;smoke[30]=1;
        pool.replace_record_checked(1,&before,&smoke).unwrap();
        // Unlinked active kind7 with the wrong native identity is discovered
        // only by the all-pool retirement cleanup, after motion/animation.
        let start=world.layouts[6].file_offset;world.backing[start+6]=7;world.backing[start+15..start+17].copy_from_slice(&9i16.to_le_bytes());
        let before=world.clone();let result=world.step_effect6(1,context,&physics,&trig,&crate::landing_effect_lifecycle::RetirementUiState::native_empty());
        assert_eq!(result.unwrap_err(),ActorWorldError::Lifecycle(crate::landing_effect_lifecycle::LifecycleError::RecordIdentity));assert_eq!(world,before);
    }

    #[test]
    fn landing_effect_pass_updates_newborns_same_frame_and_reuses_retired_slots() {
        let mut world=landing_update_fixture(false);let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{water_height:0,gravity:1024};
        let ui=crate::landing_effect_lifecycle::RetirementUiState::native_empty();
        let first=world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap();
        assert_eq!(first.primary.len(),1);assert_eq!(first.newborn.len(),1);
        assert!(matches!(first.newborn[0],LandingEffectCallback::Smoke(_)));
        let child=world.effects().unwrap().record(1).unwrap();assert_eq!(child[21]&1,0);assert_eq!(child[27],1);
        let mut retirements=0;
        for _ in 1..189 {
            let event=world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap();
            retirements+=event.primary.iter().chain(&event.newborn).filter(|c|matches!(c,LandingEffectCallback::Smoke(e) if e.retired)).count();
            assert!(event.queue_candidates.len()<60);
        }
        assert!(retirements>100);assert_eq!(world.effects().unwrap().record(0).unwrap()[70],1);
        assert_eq!(world.player(0).unwrap().activation_count(),1);
        for _ in 0..100 { world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap(); }
        assert_eq!(word(&world.backing,0x3554),1);assert_eq!(word(world.effects().unwrap().record(0).unwrap(),74),50);
        let phase1_rng=world.rng();let mut messages=0;let mut created=0;
        for _ in 0..50 {
            let result=world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap();
            for callback in result.primary {
                if let LandingEffectCallback::Landing(e)=callback {
                    created+=usize::from(e.created_effect.is_some());
                    messages+=e.pending_events.iter().filter(|e|matches!(e,LandingEvent::LandingMessage{owner:0,message_index:79..=86})).count();
                }
            }
        }
        assert_eq!((created,messages),(49,1));assert_eq!(world.rng().draw_count,phase1_rng.draw_count.wrapping_add(197));
        let record=world.effects().unwrap().record(0).unwrap();assert_eq!(record[70],2);assert_eq!(word(record,74),30);assert_eq!(word(record,49),512);
        let departure_rng=world.rng();
        for _ in 0..49 { world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap(); }
        let mut broken=world.clone();let start=broken.layouts[6].file_offset;broken.backing[start+6]=7;broken.backing[start+15]=9;
        let before=broken.clone();assert!(broken.step_landing_effect_callbacks(context,&physics,&trig,&ui).is_err());assert_eq!(broken,before,"late cleanup rolls back player retirement and all earlier callbacks");
        let last=world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap();
        assert!(last.primary.iter().any(|e|matches!(e,LandingEffectCallback::Landing(e) if e.retirement.is_some())));
        assert_eq!(world.rng().draw_count,departure_rng.draw_count.wrapping_add(173));
        assert_eq!(world.effects().unwrap().record(0).unwrap()[6],0);assert_eq!(world.effects().unwrap().binding(0).unwrap(),None);
        assert_eq!(reference(world.player(0).unwrap().bytes(),0x496),[0,0,0]);assert_eq!(&world.player(0).unwrap().bytes()[0x499..0x49D],&700u32.to_le_bytes());
        for _ in 0..50 { world.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap(); }
        assert_eq!(word(world.backing_bytes(),0x3554),0);assert!(world.effects().unwrap().records().iter().all(|r|r[6]==0));
    }

    #[test]
    fn unmarked_actor_smoke_joins_the_primary_effect_scan() {
        let mut world=fixture();world.admit_effects(&[[0xa5;81];200],[None;200]).unwrap();
        let trig=TrigTables::generate_pinned().unwrap();let physics=crate::physics_tables::PhysicsTables::fixture();
        let context=LandingPhysicsContext{water_height:0,gravity:1024};let initial=world.rng;
        let(slot,draw)=world.create_unmarked_smoke([4096,4096],1000,[0,512,0],context,&trig).unwrap().unwrap();
        assert_eq!(slot,0);assert_eq!(draw.journal_code,0x21d);
        assert_eq!(world.rng.draw_count,initial.draw_count.wrapping_add(1));
        assert_eq!(world.effects().unwrap().record(0).unwrap()[21]&1,0);
        let after_factory=world.rng;
        let receipt=world.step_landing_effect_callbacks(context,&physics,&trig,&crate::landing_effect_lifecycle::RetirementUiState::native_empty()).unwrap();
        assert_eq!(receipt.primary.len(),1);assert!(receipt.newborn.is_empty());
        assert!(matches!(receipt.primary[0],LandingEffectCallback::Smoke(_)));assert_eq!(world.rng,after_factory);
        world.validate_links().unwrap();
    }

    #[test]
    fn empty_bookkeeping_commits_timers_and_rejects_late_pool_or_player_changes() {
        let mut world=fixture();
        let mut player=*world.players[0].bytes();player[0x499..0x49d].copy_from_slice(&700u32.to_le_bytes());
        world.players[0]=PlayerState::from_record(&player).unwrap();
        let rng=world.rng;let backing=world.backing.clone();let actors=world.actors.clone();
        assert!(world.step_empty_building_bookkeeping(0).unwrap().is_empty());
        assert_eq!(&world.players[0].bytes()[0x499..0x49d],&699u32.to_le_bytes());
        assert_eq!(world.rng,rng);assert_eq!(world.backing,backing);assert_eq!(world.actors,actors);
        // Failure in the final player must also roll back earlier players.
        let mut player=*world.players[3].bytes();player[0x4a2]=1;world.players[3]=PlayerState::from_record(&player).unwrap();
        let before=world.clone();assert!(world.step_empty_building_bookkeeping(0).is_err());assert_eq!(world,before);
        let mut world=fixture();let group=world.layouts[3];world.backing[group.file_offset+(group.count-1)*group.packed_stride+6]=4;
        let before=world.clone();assert!(world.step_empty_building_bookkeeping(0).is_err());assert_eq!(world,before);
    }

    #[test]
    fn empty_countdown_uses_mission_period_and_rejects_expiry_without_mutation() {
        let mut world=fixture();world.backing[0x1c7a..0x1c7c].copy_from_slice(&6000u16.to_le_bytes());world.backing[0x355c..0x355e].fill(0);
        let first=world.step_empty_family8_countdown(0,0).unwrap();assert!(first.started);assert_eq!(first.after,6000);
        assert_eq!(world.step_empty_family8_countdown(0,0).unwrap().after,5999);
        world.backing[0x355c..0x355e].copy_from_slice(&1u16.to_le_bytes());let before=world.clone();
        assert!(world.step_empty_family8_countdown(0,0).is_err());assert_eq!(world,before);
        world.backing[0x355c..0x355e].fill(0);let group=world.layouts[7];world.backing[group.file_offset+6]=8;
        let before=world.clone();assert!(world.step_empty_family8_countdown(0,0).is_err());assert_eq!(world,before);
    }

    #[test]
    fn saturated_smoke_pool_commits_phase_rng_and_audio_without_a_factory_draw() {
        let mut world=landing_update_fixture(false);let physics=crate::physics_tables::PhysicsTables::fixture();let trig=TrigTables::generate_pinned().unwrap();
        let context=LandingPhysicsContext{water_height:0,gravity:1024};
        // Only landing callbacks: intentionally freeze smoke to exercise the
        // full-pool source branch, without claiming an ordinary frame order.
        for _ in 0..189 { world.step_landing_phase0(0,context,&physics,&trig).unwrap(); }
        for _ in 0..111 { world.step_landing_phase1(0,context,&trig).unwrap(); }
        assert_eq!(word(world.backing_bytes(),0x3554),200);
        let before=world.rng();let receipt=world.step_landing_phase1(0,context,&trig).unwrap();
        assert_eq!(receipt.created_effect,None);assert_eq!(receipt.draws.len(),3);
        assert_eq!(world.rng().draw_count,before.draw_count.wrapping_add(3));
        assert_eq!(receipt.pending_events.len(),2);assert_eq!(word(world.effects().unwrap().record(0).unwrap(),74),38);
        assert_eq!(word(world.backing_bytes(),0x3554),200);assert!(world.validate_links().is_ok());
    }
}

mod terrain_refresh;

mod power_updates;
pub use power_updates::PowerStepReceipt;

mod power_review{use super::*;use std::{fs,path::Path};
pub fn review_power_world(p:&Path,w:&Path){
 let mut bytes=vec![0;343958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());for c in 0..16384{bytes[MAP_START+c*14+2..MAP_START+c*14+4].copy_from_slice(&3136i16.to_le_bytes());bytes[MAP_START+c*14+5]=7;}
 let src=ConqMissionSource::parse(&bytes).unwrap();let players=std::array::from_fn(|i|PlayerState::fresh_campaign(i as u8,[4096,4096],b"fixture\0").unwrap());let mut world=ActorWorld::from_parts(&src,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:65535,ring:31,draw_count:99}).unwrap();world.admit_effects(&[[0;81];200],[None;200]).unwrap();
 let mut buildings=[[0;67];256];let b=&mut buildings[7];b[6]=4;b[15]=7;b[45]=3;b[49]=1;b[7..9].copy_from_slice(&4608i16.to_le_bytes());b[9..11].copy_from_slice(&4096i16.to_le_bytes());let cell=16*128+18;b[..2].copy_from_slice(&(cell as u16).to_le_bytes());let mut bindings=[None;256];bindings[7]=Some(crate::building::BuildingAnimationBinding{selector:3});world.buildings=Some(Box::new(BuildingPool::admit(buildings,bindings).unwrap()));world.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[7,0,4]);world.validate_links().unwrap();
 let trig=TrigTables::generate_pinned().unwrap();let t=p.join("private/review/family8-state4");let physics=crate::physics_tables::PhysicsTables::from_mapped_bytes(&fs::read(t.join("angles.bin")).unwrap(),&fs::read(t.join("bounce.bin")).unwrap()).unwrap();let req=crate::building_power::PowerTransfer33Request{destination:[7,0,4],source_position:[4096,4096],destination_position:[4608,4096],payload:50,yaw:512,pitch:512,speed:2048,height_above_ground:512};let slot=world.create_power_transfer33(&req,&trig).unwrap();assert_eq!(slot,0);
 for(name,value)in[("map",world.map().to_vec()),("effect",world.effects.as_ref().unwrap().record(0).unwrap().to_vec()),("building",world.buildings.as_ref().unwrap().record(7).unwrap().to_vec())]{fs::write(w.join(format!("world-before-{name}.bin")),value).unwrap();}
 let before=world.clone();let mut ui=crate::landing_effect_lifecycle::RetirementUiState::native_empty();ui.pointer_slots[3]=Some(landing::LandingReference{index:0,kind:5});let context=LandingPhysicsContext{water_height:0,gravity:1024};let mut steps=0;
 loop{steps+=1;let snapshot=world.clone();let receipt=world.step_power_transfer33(0,context,&physics,&ui).unwrap();let mut scheduled=snapshot.clone();let pass=scheduled.step_landing_effect_callbacks(context,&physics,&trig,&ui).unwrap();assert_eq!(scheduled,world);assert_eq!(pass.primary.len(),1);assert!(matches!(&pass.primary[0],LandingEffectCallback::Power(r) if r==&receipt.effect));assert!(pass.newborn.is_empty());if receipt.effect.retired{assert_eq!(steps,12);assert_eq!(receipt.credited_building,Some(7));assert_eq!(word(world.buildings.as_ref().unwrap().record(7).unwrap(),61),39);assert!(receipt.effect.ui_after.pointer_slots[3].is_none());let mut broken=snapshot;let group=broken.layouts[6];broken.backing[group.file_offset+6]=7;broken.backing[group.file_offset+15..group.file_offset+17].copy_from_slice(&9i16.to_le_bytes());let unchanged=broken.clone();assert!(broken.step_power_transfer33(0,context,&physics,&ui).is_err());assert_eq!(broken,unchanged);break;}assert!(steps<16);}
 assert_eq!(world.rng,before.rng);assert_eq!(world.players,before.players);assert_eq!(world.actors,before.actors);assert_eq!(world.effects.as_ref().unwrap().binding(0).unwrap(),None);assert_eq!(word(&world.backing,0x3554),0);world.validate_links().unwrap();
 for(name,value)in[("map",world.map().to_vec()),("effect",world.effects.as_ref().unwrap().record(0).unwrap().to_vec()),("building",world.buildings.as_ref().unwrap().record(7).unwrap().to_vec())]{fs::write(w.join(format!("world-{name}.bin")),value).unwrap();}
 println!("passed factory/12ordinarycallbacks/39power/mapretirement/UIcleanup/wholeworldrollback/RNG");
}
}
pub use power_review::review_power_world;

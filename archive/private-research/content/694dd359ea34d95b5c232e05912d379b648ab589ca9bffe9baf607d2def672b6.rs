//! Mutable, transactional state for the proved first-mission plant slice.
//!
//! `MissionWorld` owns an exact copy of the admitted packed mission source.
//! It validates and follows the source map's three-byte entity references,
//! preserves all packed bytes outside the proved plant/map mutations, and runs
//! family-1 callbacks through either the legacy bounded API or the explicit
//! RNG-enabled neutral reproduction API. Other-family AI, view callbacks,
//! draw-queue consumers and reference/UI/audio cleanup remain suspended here;
//! the owned ActorWorld adapter applies numeric cross-family target cleanup.

use core::fmt;
use genewars_assets::{ConqMissionSource, MissionSourceError};
use genewars_core::cd_candidate::{
    advance_plant_schedule_cursor, plan_family1_plant_schedule,
    PlantCallbackState, PlantLifecycleError, PlantScheduleSlot,
};
use std::collections::HashSet;
use std::vec::Vec;
use std::ops::{Deref, DerefMut};

mod plant_creation;
pub(crate) use plant_creation::PlantReproductionContext;

const MISSION_SHA256: &str = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2";
const PARAMETER_SHA256: &str = "297a1da0e9c30d6d4cb7c88addb8234be260e0e93b7d4b2b1c4c3ad8b7df2b7c";
const PARAMETER_COUNT: usize = 12;
const PARAMETER_STRIDE: usize = 32;
const PARAMETER_BYTES: usize = PARAMETER_COUNT * PARAMETER_STRIDE;

const MAP_OFFSET: usize = 0x355E;
const MAP_SIDE: usize = 128;
const MAP_CELLS: usize = MAP_SIDE * MAP_SIDE;
const MAP_CELL_BYTES: usize = 14;
const MAP_BYTES: usize = MAP_CELLS * MAP_CELL_BYTES;
const MAP_HEAD_OFFSET: usize = 6;
const MAP_CELL_COUNT_OFFSET: usize = 13;

const PLANT_GROUP: usize = 1;
const PLANT_FAMILY_KIND: u8 = 2;
const PLANT_COUNT_OFFSET: usize = 0x3552;
const SCHEDULE_CURSOR_OFFSET: usize = 0x355A;

const PLAYER_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantParameters {
    pub growth_deltas_i8: [i8; 8],
    pub spread_period_u8: u8,
    /// Number of RNG direction samples for an eligible reproduction attempt.
    pub random_walk_steps_u8: u8,
    pub crowding_limit_u8: u8,
    pub check_threshold_u8: u8,
    /// Source6C900 feeding value at parameter-row byte28.
    pub nutrition_u8: u8,
    pub retirement_delay_u8: u8,
}

/// Hash-bound 12-slot parameter table extracted from the pinned executable's
/// mapped object3 range at VA 0x104B80. These are independent of ANB images.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantParameterTable {
    records: [PlantParameters; PARAMETER_COUNT],
}

impl PlantParameterTable {
    #[cfg(test)]
    pub(crate) fn fixture_for_shared_test() -> Self {
        Self { records: [PlantParameters {
            growth_deltas_i8: [1; 8], spread_period_u8: 4, random_walk_steps_u8: 0,
            crowding_limit_u8: 10, check_threshold_u8: 1, nutrition_u8: 8, retirement_delay_u8: 3,
        }; PARAMETER_COUNT] }
    }

    pub fn parse(bytes: &[u8]) -> Result<Self, RuntimeError> {
        if bytes.len() != PARAMETER_BYTES {
            return Err(RuntimeError::ParameterTableLength { expected: PARAMETER_BYTES, actual: bytes.len() });
        }
        let actual = crate::identity::sha256_hex(bytes);
        if actual != PARAMETER_SHA256 {
            return Err(RuntimeError::ParameterTableHash { expected: PARAMETER_SHA256, actual });
        }
        let records = std::array::from_fn(|slot| {
            let row = &bytes[slot * PARAMETER_STRIDE..(slot + 1) * PARAMETER_STRIDE];
            PlantParameters {
                growth_deltas_i8: std::array::from_fn(|index| row[16 + index] as i8),
                spread_period_u8: row[24],
                random_walk_steps_u8: row[25],
                crowding_limit_u8: row[26],
                check_threshold_u8: row[27],
                nutrition_u8: row[28],
                retirement_delay_u8: row[29],
            }
        });
        Ok(Self { records })
    }

    pub fn slot(&self, selector: usize) -> Result<PlantParameters, RuntimeError> {
        self.records.get(selector).copied().ok_or(RuntimeError::ResourceSelectorOutOfBounds {
            selector: selector as i32, count: self.records.len(),
        })
    }

    pub fn records(&self) -> &[PlantParameters; PARAMETER_COUNT] { &self.records }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    Source(MissionSourceError),
    MissionHash { expected: &'static str, actual: String },
    ParameterTableLength { expected: usize, actual: usize },
    ParameterTableHash { expected: &'static str, actual: String },
    InvalidMapExtent { expected: usize, actual: usize },
    InvalidPlantCount { declared: u16, active: usize },
    InvalidScheduleCursor(u16),
    InvalidPlantType { index: usize, value: u8 },
    ResourceSelectorOutOfBounds { selector: i32, count: usize },
    InvalidReference { cell: usize, index: i16, kind: u8 },
    ReferenceCycle { cell: usize },
    DuplicateMapMembership { kind: u8, index: usize },
    BrokenPreviousReference { kind: u8, index: usize },
    MissingActivePlantLink(usize),
    PlantInWrongCell { index: usize, expected: [usize; 2], actual: [usize; 2] },
    MissingMapReference { kind: u8, index: usize },
    UnsupportedOwnerCounter { index: usize, owner: u8 },
    UnsupportedScheduledPlantState { index: usize, reason: &'static str },
    InsufficientEligibleSlots { expected: u16, found: u16 },
    AllocationFailed,
    StorageInvariant(&'static str),
    Lifecycle(PlantLifecycleError),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for RuntimeError {}

impl From<MissionSourceError> for RuntimeError {
    fn from(value: MissionSourceError) -> Self { Self::Source(value) }
}
impl From<PlantLifecycleError> for RuntimeError {
    fn from(value: PlantLifecycleError) -> Self { Self::Lifecycle(value) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadeBasis {
    /// This field is the shade byte copied from serialized source. Mission
    /// entry's later map-based shade refresh is not implicitly run here.
    SourceOrPreviouslyMutatedField,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlantRenderRecord {
    pub record_index: usize,
    pub position_words: [i16; 2],
    pub owner_u8: u8,
    pub resource_selector_i8: i8,
    pub animation_u8: u8,
    pub frame_u8: u8,
    pub shade_u8: u8,
    pub shade_basis: ShadeBasis,
    pub flags_at_15: u8,
    pub flags_at_2e: u8,
    pub growth_i16: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickEvent {
    Grew { record_index: usize, growth_before: i16, growth_after: i16 },
    Created { parent_index: usize, record_index: usize },
    SuppressedNoop { record_index: usize, flags_at_15: u8, flags_at_2e: u8 },
    DelayedRetirement { record_index: usize, remaining_delay: u8 },
    Retired { record_index: usize, old_cell_counter: u8, new_cell_counter: u8 },
    OldFrameDirtyCheckPending { record_index: usize },
    ReferenceUiAudioCleanupPending { record_index: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuspendedSubsystems {
    pub non_family1_ai: bool,
    pub family1_creation_pass: bool,
    pub view_callbacks: bool,
    pub queue_and_dirty_draw: bool,
    pub other_family_bookkeeping: bool,
    pub ui_audio_reference_cleanup: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantTickReport {
    pub tick_number: u64,
    pub schedule_cursor_before: u16,
    pub schedule_cursor_after: u16,
    pub active_plant_count_before: u16,
    pub active_plant_count_after: u16,
    pub callbacks: Vec<usize>,
    pub queued_indices: Vec<usize>,
    pub events: Vec<TickEvent>,
    pub suspended: SuspendedSubsystems,
    pub full_game_tick: bool,
}

const SUSPENDED: SuspendedSubsystems = SuspendedSubsystems {
    non_family1_ai: true,
    family1_creation_pass: true,
    view_callbacks: true,
    queue_and_dirty_draw: true,
    other_family_bookkeeping: true,
    ui_audio_reference_cleanup: true,
};

/// Storage operations needed by plant callbacks. An integrated world uses
/// its native actor pool for kind3 references while retaining this same map.
/// Implementations must preserve all bytes outside the requested link triple.
pub trait PlantStorage: Deref<Target = [u8]> + DerefMut {
    fn common_record(&self, kind: u8, index: u16) -> Result<&[u8], RuntimeError> {
        let source = ConqMissionSource::parse(self)?;
        let groups = source.groups();
        let layout = groups.get(usize::from(kind.wrapping_sub(1)))
            .ok_or(RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind })?.layout();
        if usize::from(index) >= layout.count {
            return Err(RuntimeError::InvalidReference { cell: usize::MAX, index: index as i16, kind });
        }
        let start = layout.file_offset + usize::from(index) * layout.packed_stride;
        Ok(&self[start..start + 31])
    }

    fn write_common_reference(&mut self, kind: u8, index: u16, offset: usize, value: [u8; 3]) -> Result<(), RuntimeError> {
        if offset != 0 && offset != 3 { return Err(RuntimeError::StorageInvariant("invalid link offset")); }
        self.common_record(kind, index)?;
        let layout = ConqMissionSource::parse(self)?.groups()[usize::from(kind - 1)].layout();
        let start = layout.file_offset + usize::from(index) * layout.packed_stride + offset;
        self[start..start + 3].copy_from_slice(&value);
        Ok(())
    }
}
impl PlantStorage for Vec<u8> {}

#[derive(Debug, Clone)]
pub struct MissionWorld<S = Vec<u8>> {
    bytes: S,
    parameters: PlantParameterTable,
    tick_number: u64,
}

impl MissionWorld {
    /// Build the bounded plant world from a verified first-mission source and
    /// the exact standalone executable-mapped parameter table. Every packed
    /// source byte is retained; only explicitly admitted plant/map fields are
    /// mutable. The table does not come from the plant ANB resources.
    pub fn from_source(
        source: &ConqMissionSource<'_>,
        parameter_bytes: &[u8],
    ) -> Result<Self, RuntimeError> {
        let source_hash = crate::identity::sha256_hex(source.bytes());
        if crate::identity::CdCampaignMission::identify(source.bytes()).is_none() {
            return Err(RuntimeError::MissionHash { expected: MISSION_SHA256, actual: source_hash });
        }
        let parameters = PlantParameterTable::parse(parameter_bytes)?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(source.bytes().len()).map_err(|_| RuntimeError::AllocationFailed)?;
        bytes.extend_from_slice(source.bytes());
        let world = Self { bytes, parameters, tick_number: 0 };
        world.validate_loaded_plant_state()?;
        Ok(world)
    }

    /// The owned plant-only route stages the complete backing before running.
    pub fn tick_plants(&mut self) -> Result<PlantTickReport, RuntimeError> {
        let mut candidate = self.clone_for_transaction()?;
        let report = candidate.tick_plants_inner()?;
        *self = candidate;
        Ok(report)
    }

    /// Explicit RNG-enabled route; legacy plant-only stepping remains bounded.
    pub fn tick_plants_with_reproduction(&mut self,rng:&mut crate::blueprint::BlueprintRng,
        directions:&crate::plant_spread::SpreadDirections,height_threshold:i16)->Result<PlantTickReport,RuntimeError> {
        let mut candidate=self.clone_for_transaction()?;let mut staged_rng=*rng;
        let mut context=PlantReproductionContext{rng:&mut staged_rng,directions,height_threshold};
        let report=candidate.tick_plants_inner_with_reproduction(&mut context)?;
        *self=candidate;*rng=staged_rng;Ok(report)
    }

    fn clone_for_transaction(&self) -> Result<Self, RuntimeError> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(self.bytes.len()).map_err(|_| RuntimeError::AllocationFailed)?;
        bytes.extend_from_slice(&self.bytes);
        Ok(Self { bytes, parameters: self.parameters.clone(), tick_number: self.tick_number })
    }
}

impl<S: PlantStorage> MissionWorld<S> {
    pub(crate) fn with_shared_storage(bytes: S, parameters: PlantParameterTable, tick_number: u64) -> Result<Self, RuntimeError> {
        let world = Self { bytes, parameters, tick_number };
        world.validate_loaded_plant_state()?;
        Ok(world)
    }

    pub fn source_bytes(&self) -> &[u8] { &self.bytes }
    pub fn parameter_table(&self) -> &PlantParameterTable { &self.parameters }
    pub fn tick_number(&self) -> u64 { self.tick_number }
    pub fn plant_count(&self) -> u16 { read_u16(&self.bytes, PLANT_COUNT_OFFSET) }
    pub fn schedule_cursor(&self) -> u16 { read_u16(&self.bytes, SCHEDULE_CURSOR_OFFSET) }
    pub fn map_cells(&self) -> &[u8] { &self.bytes[MAP_OFFSET..MAP_OFFSET + MAP_BYTES] }

    /// Active plant fields for the root-owned projection/composition layer.
    /// Shade is exposed exactly as currently stored; this method does not
    /// synthesize the later mission-entry shade refresh.
    pub fn render_records(&self) -> Result<Vec<PlantRenderRecord>, RuntimeError> {
        plant_render_records(&self.bytes)
    }

    /// Run one deterministic, transactionally committed family-1 scheduling
    /// pass over the admitted no-RNG callback paths. A reproduction/RNG path
    /// or unavailable owner-count mutation rejects the entire pass without
    /// changing bytes, cursor, or tick number.
    pub(crate) fn tick_plants_inner(&mut self) -> Result<PlantTickReport, RuntimeError> {
        self.tick_plants_inner_impl(None)
    }
    pub(crate) fn tick_plants_inner_with_reproduction(&mut self,context:&mut PlantReproductionContext<'_>)->Result<PlantTickReport,RuntimeError> {
        self.tick_plants_inner_impl(Some(context))
    }
    fn tick_plants_inner_impl(&mut self,mut reproduction:Option<&mut PlantReproductionContext<'_>>)->Result<PlantTickReport,RuntimeError> {
        let before_count = self.plant_count();
        let cursor_before = self.schedule_cursor();
        let slots = self.schedule_slots()?;
        let callback_indices = self.predict_callback_indices(cursor_before, before_count, &slots)?;
        let mut callback_states = Vec::new();
        callback_states.try_reserve_exact(callback_indices.len()).map_err(|_| RuntimeError::AllocationFailed)?;
        let mut events = Vec::new();
        events.try_reserve_exact(callback_indices.len() * 3).map_err(|_| RuntimeError::AllocationFailed)?;

        // Calls are sequential. Later callbacks see earlier map/list/growth
        // mutations when their nine-cell positive-growth count is computed.
        for &index in &callback_indices {
            let created_count=self.process_plant_callback(index, &mut events,reproduction.as_deref_mut())?;
            let layout = self.layouts()[PLANT_GROUP];
            let record_start = layout.offset + index * layout.stride;
            callback_states.push((index, PlantCallbackState {
                active_type_at_6: self.bytes[record_start + 6],
                flags_at_2e: self.bytes[record_start + 38],
                created_count,
            }));
        }

        let mut schedule = plan_family1_plant_schedule(
            cursor_before, before_count, &slots, &callback_states,
        )?;
        if schedule.second_pass_pending {
            if reproduction.is_none(){return Err(RuntimeError::StorageInvariant("newborn plants without admitted reproduction"));}
            let layout=self.layouts()[PLANT_GROUP];let mut remaining=schedule.created_count;
            for slot in 0..layout.count {
                if remaining==0{break;}
                let at=layout.offset+slot*layout.stride;
                if self.bytes[at+6]!=0&&self.bytes[at+21]&1!=0 {
                    self.bytes[at+21]&=0xfe;schedule.queued_indices.push(slot);remaining-=1;
                }
            }
            if remaining!=0{return Err(RuntimeError::StorageInvariant("missing marked newborn plant"));}
        }
        write_u16(&mut self.bytes, SCHEDULE_CURSOR_OFFSET, schedule.cursor_after);
        self.tick_number = self.tick_number.checked_add(1).ok_or(RuntimeError::AllocationFailed)?;
        Ok(PlantTickReport {
            tick_number: self.tick_number,
            schedule_cursor_before: cursor_before,
            schedule_cursor_after: schedule.cursor_after,
            active_plant_count_before: before_count,
            active_plant_count_after: self.plant_count(),
            callbacks: schedule.callback_indices,
            queued_indices: schedule.queued_indices,
            events,
            suspended: SuspendedSubsystems{family1_creation_pass:reproduction.is_none(),..SUSPENDED},
            full_game_tick: false,
        })
    }

    fn validate_loaded_plant_state(&self) -> Result<(), RuntimeError> {
        let expected_end = MAP_OFFSET.checked_add(MAP_BYTES).ok_or(RuntimeError::InvalidMapExtent {
            expected: MAP_BYTES, actual: 0,
        })?;
        if expected_end > self.bytes.len() {
            return Err(RuntimeError::InvalidMapExtent { expected: MAP_BYTES, actual: self.bytes.len().saturating_sub(MAP_OFFSET) });
        }
        let count = self.plant_count();
        let active = self.active_plant_indices()?;
        if usize::from(count) != active.len() {
            return Err(RuntimeError::InvalidPlantCount { declared: count, active: active.len() });
        }
        if self.schedule_cursor() > 512 { return Err(RuntimeError::InvalidScheduleCursor(self.schedule_cursor())); }
        self.validate_map_links(&active)
    }

    fn active_plant_indices(&self) -> Result<Vec<usize>, RuntimeError> {
        let mut active = Vec::new();
        active.try_reserve_exact(usize::from(self.plant_count())).map_err(|_| RuntimeError::AllocationFailed)?;
        let layout = self.layouts()[PLANT_GROUP];
        for index in 0..layout.count {
            let typ = self.bytes[layout.offset + index * layout.stride + 6];
            if typ == 0 { continue; }
            if typ != PLANT_FAMILY_KIND { return Err(RuntimeError::InvalidPlantType { index, value: typ }); }
            active.push(index);
        }
        Ok(active)
    }

    fn validate_map_links(&self, active_plants: &[usize]) -> Result<(), RuntimeError> {
        let layouts = self.layouts();
        let mut seen: HashSet<(u8, usize)> = HashSet::new();
        let mut plant_cells = vec![None; layouts[PLANT_GROUP].count];
        for cell_index in 0..MAP_CELLS {
            let cell_start = MAP_OFFSET + cell_index * MAP_CELL_BYTES;
            let mut current = read_reference(&self.bytes, cell_start + MAP_HEAD_OFFSET);
            let mut previous = EntityRef::NULL;
            let mut local_seen = HashSet::new();
            while let Some(reference) = current {
                if !local_seen.insert(reference) { return Err(RuntimeError::ReferenceCycle { cell: cell_index }); }
                let reference_index = usize::from(reference.index);
                if !seen.insert((reference.kind, reference_index)) {
                    return Err(RuntimeError::DuplicateMapMembership { kind: reference.kind, index: reference_index });
                }
                let Some(group_index) = reference.kind.checked_sub(1).map(usize::from) else {
                    return Err(RuntimeError::InvalidReference { cell: cell_index, index: reference.index as i16, kind: reference.kind });
                };
                let layout = *layouts.get(group_index).ok_or(RuntimeError::InvalidReference {
                    cell: cell_index, index: reference.index as i16, kind: reference.kind,
                })?;
                let index = usize::from(reference.index);
                if index >= layout.count { return Err(RuntimeError::InvalidReference {
                    cell: cell_index, index: reference.index as i16, kind: reference.kind,
                }); }
                if reference.kind == PLANT_FAMILY_KIND { plant_cells[index] = Some([cell_index % MAP_SIDE, cell_index / MAP_SIDE]); }
                let record = self.bytes.common_record(reference.kind, reference.index)?;
                let actual_previous = read_reference(record, 0);
                if actual_previous != previous {
                    return Err(RuntimeError::BrokenPreviousReference { kind: reference.kind, index });
                }
                previous = Some(reference);
                current = read_reference(record, 3);
            }
        }
        for &index in active_plants {
            if !seen.contains(&(PLANT_FAMILY_KIND, index)) { return Err(RuntimeError::MissingActivePlantLink(index)); }
            let start = layouts[PLANT_GROUP].offset + index * layouts[PLANT_GROUP].stride;
            let position = [read_i16(&self.bytes, start + 7), read_i16(&self.bytes, start + 9)];
            let expected = [usize::from(((position[0] as u16) >> 8) as u8 & 127), usize::from(((position[1] as u16) >> 8) as u8 & 127)];
            let actual = plant_cells[index].ok_or(RuntimeError::MissingActivePlantLink(index))?;
            if actual != expected { return Err(RuntimeError::PlantInWrongCell { index, expected, actual }); }
        }
        Ok(())
    }

    fn schedule_slots(&self) -> Result<Vec<PlantScheduleSlot>, RuntimeError> {
        let layout = self.layouts()[PLANT_GROUP];
        let mut slots = Vec::new();
        slots.try_reserve_exact(layout.count).map_err(|_| RuntimeError::AllocationFailed)?;
        for index in 0..layout.count {
            let start = layout.offset + index * layout.stride;
            let record = &self.bytes[start..start + layout.stride];
            slots.push(PlantScheduleSlot {
                active_type_at_6: record[6],
                flags_at_15: record[21],
                flags_at_2e: record[38],
            });
        }
        Ok(slots)
    }

    fn predict_callback_indices(
        &self,
        cursor: u16,
        remaining_count: u16,
        slots: &[PlantScheduleSlot],
    ) -> Result<Vec<usize>, RuntimeError> {
        let mut target = advance_plant_schedule_cursor(cursor)?;
        let mut remaining = remaining_count;
        let mut calls = Vec::new();
        calls.try_reserve_exact(2).map_err(|_| RuntimeError::AllocationFailed)?;
        for (index, slot) in slots.iter().copied().enumerate() {
            if remaining == 0 { break; }
            if slot.active_type_at_6 == 0 || slot.flags_at_15 & 1 != 0 { continue; }
            if calls.len() < 2 && index == usize::from(target) {
                calls.push(index);
                target = advance_plant_schedule_cursor(target)?;
            }
            remaining -= 1;
        }
        if remaining != 0 {
            return Err(RuntimeError::InsufficientEligibleSlots {
                expected: remaining_count, found: remaining_count - remaining,
            });
        }
        Ok(calls)
    }

    fn process_plant_callback(&mut self, index: usize, events: &mut Vec<TickEvent>,reproduction:Option<&mut PlantReproductionContext<'_>>) -> Result<u8, RuntimeError> {
        let layouts = self.layouts();
        let layout = layouts[PLANT_GROUP];
        if index >= layout.count { return Err(RuntimeError::UnsupportedScheduledPlantState { index, reason: "slot outside family-1 table" }); }
        let start = layout.offset + index * layout.stride;
        if self.bytes[start + 6] != PLANT_FAMILY_KIND { return Err(RuntimeError::UnsupportedScheduledPlantState { index, reason: "scheduled slot is not active family-1" }); }
        let flags15 = self.bytes[start + 21];
        let flags2e = self.bytes[start + 38];
        if flags15 & 1 != 0 || flags2e & 0x10 != 0 {
            events.push(TickEvent::SuppressedNoop { record_index: index, flags_at_15: flags15, flags_at_2e: flags2e });
            return Ok(0);
        }
        let selector = self.bytes[start + 35] as i8;
        if !(0..12).contains(&i32::from(selector)) {
            return Err(RuntimeError::ResourceSelectorOutOfBounds { selector: i32::from(selector), count: PARAMETER_COUNT });
        }
        let params = self.parameters.slot(selector as usize)?;
        let position = [read_i16(&self.bytes, start + 7), read_i16(&self.bytes, start + 9)];
        let gx = (position[0] as u16 >> 8) as u8 & 127;
        let gy = (position[1] as u16 >> 8) as u8 & 127;
        let cell_index = usize::from(gy) * MAP_SIDE + usize::from(gx);
        let cell_start = MAP_OFFSET + cell_index * MAP_CELL_BYTES;
        let cell_counter = self.bytes[cell_start + MAP_CELL_COUNT_OFFSET];
        let mut growth = read_i16(&self.bytes, start + 36);
        let mut counter = self.bytes[start + 39];
        let mut animation = self.bytes[start + 25];
        let mut flags15_after = flags15;
        let mut flags2e_after = flags2e;
        let owner = self.bytes[start + 24];
        if owner > 4 {
            return Err(RuntimeError::UnsupportedOwnerCounter { index, owner });
        }
        let mut created_count=0u8;
        let old_growth = growth;
        let old_animation = animation;
        if growth > 0 {
            let old_counter = counter;
            counter = counter.wrapping_add(1);
            if old_counter >= params.check_threshold_u8 || cell_counter == 0 {
                growth = growth.wrapping_sub(4);
            } else {
                let neighbors = self.positive_growth_neighbor_count(gx, gy)?;
                if neighbors > params.crowding_limit_u8 {
                    growth = 0;
                } else {
                    let terrain_class = self.terrain_class(position);
                    growth = growth.wrapping_add(i16::from(params.growth_deltas_i8[usize::from(terrain_class)]));
                    if growth > 32 { growth = 32; }
                    if params.spread_period_u8 == 0 {
                        return Err(RuntimeError::UnsupportedScheduledPlantState { index, reason: "zero spread period faults in the original modulo path" });
                    }
                    if counter % params.spread_period_u8 == 0 {
                        let cap = if owner == 4 { 412 } else { 512 };
                        if self.plant_count() < cap {
                            let context=reproduction.ok_or(RuntimeError::UnsupportedScheduledPlantState { index, reason: "scheduled reproduction/RNG path is not implemented" })?;
                            // Source stores growth/counter before querying its target cell.
                            self.bytes[start+36..start+38].copy_from_slice(&growth.to_le_bytes());self.bytes[start+39]=counter;
                            if let Some(slot)=self.reproduce_neutral_plant(index,params,context)? {
                                created_count=1;events.push(TickEvent::Created{parent_index:index,record_index:usize::from(slot)});
                            }
                        }
                    }
                }
            }
            let stage = core::cmp::min(growth >> 3, 2);
            if stage > i16::from(animation) { animation = stage as u8; }
        }

        // The callback runs retirement after its positive-growth branch when
        // that branch leaves growth nonpositive. For preexisting retirement,
        // it enters this same path without incrementing the counter.
        let mut active_type = PLANT_FAMILY_KIND;
        let mut retirement = self.bytes[start + 40];
        let mut cell_counter_after = cell_counter;
        let mut count_after = self.plant_count();
        if growth <= 0 {
            if flags2e & 0x88 == 0 {
                animation = animation.wrapping_add(6);
                growth = 0;
                flags2e_after |= 0x80;
                retirement = params.retirement_delay_u8;
            }
            if retirement >= 4 {
                retirement -= 4;
                events.push(TickEvent::DelayedRetirement { record_index: index, remaining_delay: retirement });
            } else {
                if owner < PLAYER_COUNT as u8 {
                    return Err(RuntimeError::UnsupportedOwnerCounter { index, owner });
                }
                active_type = 0;
                if cell_counter < 64 { cell_counter_after = cell_counter.wrapping_add(2); }
                count_after = count_after.wrapping_sub(1);
                self.unlink_plant(index, cell_start)?;
                self.bytes[cell_start + MAP_CELL_COUNT_OFFSET] = cell_counter_after;
                write_u16(&mut self.bytes, PLANT_COUNT_OFFSET, count_after);
                events.push(TickEvent::Retired { record_index: index, old_cell_counter: cell_counter, new_cell_counter: cell_counter_after });
                events.push(TickEvent::OldFrameDirtyCheckPending { record_index: index });
                events.push(TickEvent::ReferenceUiAudioCleanupPending { record_index: index });
            }
        } else {
            events.push(TickEvent::Grew { record_index: index, growth_before: old_growth, growth_after: growth });
        }
        if animation != old_animation || flags2e_after & 0x02 != 0 { flags15_after |= 0x04; }
        self.bytes[start + 6] = active_type;
        self.bytes[start + 21] = flags15_after;
        self.bytes[start + 25] = animation;
        self.bytes[start + 36..start + 38].copy_from_slice(&growth.to_le_bytes());
        self.bytes[start + 38] = flags2e_after;
        self.bytes[start + 39] = counter;
        self.bytes[start + 40] = retirement;
        Ok(created_count)
    }

    fn positive_growth_neighbor_count(&self, gx: u8, gy: u8) -> Result<u8, RuntimeError> {
        let mut count = 0u8;
        for dy in -1i16..=1 {
            for dx in -1i16..=1 {
                let x = (i16::from(gx) + dx).rem_euclid(128) as usize;
                let y = (i16::from(gy) + dy).rem_euclid(128) as usize;
                let cell_start = MAP_OFFSET + (y * MAP_SIDE + x) * MAP_CELL_BYTES;
                let mut current = read_reference(&self.bytes, cell_start + MAP_HEAD_OFFSET);
                let mut seen = HashSet::new();
                while let Some(reference) = current {
                    if !seen.insert(reference) { return Err(RuntimeError::ReferenceCycle { cell: y * MAP_SIDE + x }); }
                    if reference.kind == PLANT_FAMILY_KIND {
                        let layout = self.layouts()[PLANT_GROUP];
                        let index = usize::from(reference.index);
                        if index >= layout.count { return Err(RuntimeError::InvalidReference { cell: y * MAP_SIDE + x, index: reference.index as i16, kind: reference.kind }); }
                        let start = layout.offset + index * layout.stride;
                        if self.bytes[start + 6] == PLANT_FAMILY_KIND && read_i16(&self.bytes, start + 36) > 0 {
                            count = count.wrapping_add(1);
                        }
                    }
                    current = read_reference(self.bytes.common_record(reference.kind, reference.index)?, 3);
                }
            }
        }
        Ok(count)
    }

    fn terrain_class(&self, position: [i16; 2]) -> u8 {
        map_terrain_class(self.map_cells(), position).expect("admitted map extent")
    }

    fn unlink_plant(&mut self, index: usize, cell_start: usize) -> Result<(), RuntimeError> {
        let layout = self.layouts()[PLANT_GROUP];
        let start = layout.offset + index * layout.stride;
        let previous = read_reference(&self.bytes, start);
        let next = read_reference(&self.bytes, start + 3);
        let previous_bytes = [self.bytes[start], self.bytes[start + 1], self.bytes[start + 2]];
        let next_bytes = [self.bytes[start + 3], self.bytes[start + 4], self.bytes[start + 5]];
        if previous.is_none() {
            let head = read_reference(&self.bytes, cell_start + MAP_HEAD_OFFSET);
            if head != Some(EntityRef { index: index as u16, kind: PLANT_FAMILY_KIND }) {
                return Err(RuntimeError::MissingMapReference { kind: PLANT_FAMILY_KIND, index });
            }
            if next.is_some() {
                self.bytes[cell_start + MAP_HEAD_OFFSET..cell_start + MAP_HEAD_OFFSET + 3]
                    .copy_from_slice(&next_bytes);
            } else {
                // 8ACC0's singleton-head branch clears only the kind byte;
                // the old index word remains as a null-reference payload.
                self.bytes[cell_start + MAP_HEAD_OFFSET + 2] = 0;
            }
        } else {
            let prev = previous.unwrap();
            if read_reference(self.bytes.common_record(prev.kind, prev.index)?, 3) != Some(EntityRef { index: index as u16, kind: PLANT_FAMILY_KIND }) {
                return Err(RuntimeError::MissingMapReference { kind: PLANT_FAMILY_KIND, index });
            }
            self.bytes.write_common_reference(prev.kind, prev.index, 3, next_bytes)?;
        }
        if let Some(next_ref) = next {
            self.bytes.write_common_reference(next_ref.kind, next_ref.index, 0, previous_bytes)?;
        }
        // The original helper rewrites neighboring triples, but does not
        // clear the retired record's own previous/next bytes.
        Ok(())
    }

    fn layouts(&self) -> [GroupLayout; 8] {
        let source = ConqMissionSource::parse(&self.bytes).expect("MissionWorld retains admitted layout");
        source.groups().map(|group| {
            let layout = group.layout();
            GroupLayout {
                offset: layout.file_offset,
                count: layout.count,
                stride: layout.packed_stride,
            }
        })
    }

}

/// Read-only presentation of the packed plant family, usable with either
/// the plant-only or integrated world's sole backing store.
pub fn plant_render_records(bytes: &[u8]) -> Result<Vec<PlantRenderRecord>, RuntimeError> {
    let source = ConqMissionSource::parse(bytes)?;
    let layout = source.groups()[PLANT_GROUP].layout();
    let declared = read_u16(bytes, PLANT_COUNT_OFFSET);
    let mut records = Vec::new();
    records.try_reserve_exact(usize::from(declared)).map_err(|_| RuntimeError::AllocationFailed)?;
    for index in 0..layout.count {
        let start = layout.file_offset + index * layout.packed_stride;
        let record = &bytes[start..start + layout.packed_stride];
        if record[6] == 0 { continue; }
        if record[6] != PLANT_FAMILY_KIND { return Err(RuntimeError::InvalidPlantType { index, value: record[6] }); }
        records.push(PlantRenderRecord {
            record_index: index,
            position_words: [read_i16(record,7),read_i16(record,9)],
            owner_u8: record[24], resource_selector_i8: record[35] as i8,
            animation_u8: record[25], frame_u8: record[27], shade_u8: record[23],
            shade_basis: ShadeBasis::SourceOrPreviouslyMutatedField,
            flags_at_15: record[21], flags_at_2e: record[38], growth_i16: read_i16(record,36),
        });
    }
    if records.len() != usize::from(declared) { return Err(RuntimeError::InvalidPlantCount { declared, active: records.len() }); }
    Ok(records)
}

/// Exact8B520 terrain-class lookup, shared by plants and actor placement.
pub fn map_terrain_class(map: &[u8], position: [i16; 2]) -> Result<u8, RuntimeError> {
    if map.len() != MAP_BYTES { return Err(RuntimeError::InvalidMapExtent { expected: MAP_BYTES, actual: map.len() }); }
    let base_x = (position[0] as u16 >> 8) as u8 & 127;
    let base_y = (position[1] as u16 >> 8) as u8 & 127;
    let gx = usize::from(base_x.wrapping_add(u8::from(position[0] as u8 >= 128)) & 127);
    let gy = usize::from(base_y.wrapping_add(u8::from(position[1] as u8 >= 128)) & 127);
    let selector = map[(gy * MAP_SIDE + gx) * MAP_CELL_BYTES + 5] & 7;
    if selector != 0 { return Ok(selector); }
    for dy in -1i16..=1 {
        for dx in -1i16..=1 {
            // Eight neighbors of the ORIGINAL floor cell, not the rounded cell.
            if dx == 0 && dy == 0 { continue; }
            let x = (i16::from(base_x) + dx).rem_euclid(128) as usize;
            let y = (i16::from(base_y) + dy).rem_euclid(128) as usize;
            if map[(y * MAP_SIDE + x) * MAP_CELL_BYTES + 5] & 7 != 0 { return Ok(1); }
        }
    }
    Ok(0)
}

#[derive(Debug, Clone, Copy)]
struct GroupLayout { offset: usize, count: usize, stride: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EntityRef { index: u16, kind: u8 }

impl EntityRef { const NULL: Option<Self> = None; }

fn read_reference(bytes: &[u8], offset: usize) -> Option<EntityRef> {
    let index = read_i16(bytes, offset);
    let kind = bytes[offset + 2];
    if index < 0 || kind == 0 { None } else { Some(EntityRef { index: index as u16, kind }) }
}

#[cfg(test)]
fn write_reference(bytes: &mut [u8], offset: usize, value: Option<EntityRef>) {
    let (index, kind) = value.map(|reference| (reference.index as i16, reference.kind)).unwrap_or((-1, 0));
    write_i16(bytes, offset, index);
    bytes[offset + 2] = kind;
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}
#[cfg(test)]
fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parameters(crowding_limit: u8) -> PlantParameterTable {
        let rows = [PlantParameters {
            growth_deltas_i8: [1; 8], spread_period_u8: 4, random_walk_steps_u8: 0,
            crowding_limit_u8: crowding_limit, check_threshold_u8: 1, nutrition_u8: 8,
            retirement_delay_u8: 3,
        }; PARAMETER_COUNT];
        PlantParameterTable { records: rows }
    }

    fn synthetic_world(counter: u8, crowding_limit: u8) -> MissionWorld {
        let mut bytes = vec![0u8; genewars_assets::CONQ_MISSION_BYTES];
        bytes[..4].copy_from_slice(&genewars_assets::mission_source::CONQ_MISSION_FAST_TAG.to_le_bytes());
        const EXTENSION_TAG_OFFSET: usize = 337_054;
        bytes[EXTENSION_TAG_OFFSET..EXTENSION_TAG_OFFSET + 4]
            .copy_from_slice(&0x6BEu32.to_le_bytes());
        write_u16(&mut bytes, PLANT_COUNT_OFFSET, 1);
        write_u16(&mut bytes, SCHEDULE_CURSOR_OFFSET, 0);

        let layout = GroupLayout { offset: 277_838, count: 512, stride: 41 };
        let plant = layout.offset + 8 * layout.stride;
        write_reference(&mut bytes, plant, None);
        write_reference(&mut bytes, plant + 3, None);
        bytes[plant + 6] = PLANT_FAMILY_KIND;
        write_i16(&mut bytes, plant + 7, 0);
        write_i16(&mut bytes, plant + 9, 0);
        bytes[plant + 0x15] = 4;
        bytes[plant + 0x18] = 4;
        bytes[plant + 0x19] = 0;
        bytes[plant + 0x1B] = 0;
        bytes[plant + 0x23] = 0;
        write_i16(&mut bytes, plant + 0x24, 4);
        bytes[plant + 0x26] = 0;
        bytes[plant + 0x27] = counter;
        bytes[plant + 0x28] = 0;

        let cell = MAP_OFFSET;
        write_reference(&mut bytes, cell + MAP_HEAD_OFFSET, Some(EntityRef { index: 8, kind: PLANT_FAMILY_KIND }));
        bytes[cell + 5] = 1;
        bytes[cell + MAP_CELL_COUNT_OFFSET] = 1;
        MissionWorld { bytes, parameters: test_parameters(crowding_limit), tick_number: 0 }
    }

    #[test]
    fn terrain_fallback_excludes_floor_center_after_rounding() {
        let mut world = synthetic_world(0, 2);
        world.bytes[MAP_OFFSET + 5] = 2;
        assert_eq!(world.terrain_class([0, 0]), 2);
        assert_eq!(world.terrain_class([128, 0]), 0);
        world.bytes[MAP_OFFSET + (127 * MAP_SIDE + 127) * MAP_CELL_BYTES + 5] = 4;
        assert_eq!(world.terrain_class([128, 0]), 1);
        world.bytes[MAP_OFFSET + MAP_CELL_BYTES + 5] = 6;
        assert_eq!(world.terrain_class([128, 0]), 6);
    }

    #[test]
    fn parameter_table_rejects_wrong_length_and_bytes() {
        assert!(matches!(PlantParameterTable::parse(&[0; PARAMETER_BYTES - 1]),
            Err(RuntimeError::ParameterTableLength { .. })));
        assert!(matches!(PlantParameterTable::parse(&[0; PARAMETER_BYTES]),
            Err(RuntimeError::ParameterTableHash { .. })));
    }

    #[test]
    fn mission_world_requires_exact_source_and_hash_bound_parameter_provider() {
        let source_bytes = vec![0; genewars_assets::CONQ_MISSION_BYTES];
        let source = ConqMissionSource::parse(&source_bytes);
        assert!(source.is_err());
        assert_eq!(read_reference(&[0xFF, 0xFF, 0], 0), None);
        assert_eq!(read_reference(&[0, 0, 0], 0), None);
    }

    #[test]
    fn first_counter_tick_mutates_plant_and_advances_cursor_from_owned_state() {
        let mut world = synthetic_world(0, 2);
        world.validate_loaded_plant_state().unwrap();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.callbacks, [8]);
        assert_eq!(report.schedule_cursor_before, 0);
        assert_eq!(report.schedule_cursor_after, 16);
        assert_eq!(report.active_plant_count_before, 1);
        assert_eq!(report.active_plant_count_after, 1);
        assert_eq!(report.events, [TickEvent::Grew { record_index: 8, growth_before: 4, growth_after: 5 }]);
        assert_eq!(world.tick_number(), 1);
        assert_eq!(world.render_records().unwrap()[0].growth_i16, 5);
    }

    #[test]
    fn retirement_unlinks_head_and_updates_counter_transactionally() {
        let mut world = synthetic_world(0, 0);
        world.validate_loaded_plant_state().unwrap();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.callbacks, [8]);
        assert_eq!(report.active_plant_count_after, 0);
        assert_eq!(world.plant_count(), 0);
        assert_eq!(world.render_records().unwrap(), []);
        assert_eq!(read_reference(world.source_bytes(), MAP_OFFSET + MAP_HEAD_OFFSET), None);
        assert_eq!(read_i16(world.source_bytes(), MAP_OFFSET + MAP_HEAD_OFFSET), 8);
        assert_eq!(world.source_bytes()[MAP_OFFSET + MAP_HEAD_OFFSET + 2], 0);
        assert_eq!(world.map_cells()[MAP_CELL_COUNT_OFFSET], 3);
        assert!(report.events.contains(&TickEvent::Retired {
            record_index: 8, old_cell_counter: 1, new_cell_counter: 3,
        }));
    }

    #[test]
    fn retirement_relinks_list_but_preserves_retired_records_own_reference_bytes() {
        let mut world = synthetic_world(0, 0);
        let layout = GroupLayout { offset: 277_838, count: 512, stride: 41 };
        let previous = layout.offset + 7 * layout.stride;
        let retired = layout.offset + 8 * layout.stride;
        write_u16(&mut world.bytes, PLANT_COUNT_OFFSET, 2);
        world.bytes[previous + 6] = PLANT_FAMILY_KIND;
        world.bytes[previous + 21] = 4;
        world.bytes[previous + 24] = 4;
        world.bytes[previous + 35] = 0;
        write_i16(&mut world.bytes, previous + 36, 4);
        write_reference(&mut world.bytes, MAP_OFFSET + MAP_HEAD_OFFSET, Some(EntityRef { index: 7, kind: PLANT_FAMILY_KIND }));
        write_reference(&mut world.bytes, previous, None);
        write_reference(&mut world.bytes, previous + 3, Some(EntityRef { index: 8, kind: PLANT_FAMILY_KIND }));
        write_reference(&mut world.bytes, retired, Some(EntityRef { index: 7, kind: PLANT_FAMILY_KIND }));
        world.validate_loaded_plant_state().unwrap();

        let report = world.tick_plants().unwrap();
        assert_eq!(report.callbacks, [8]);
        assert_eq!(world.plant_count(), 1);
        assert_eq!(read_reference(world.source_bytes(), MAP_OFFSET + MAP_HEAD_OFFSET), Some(EntityRef { index: 7, kind: PLANT_FAMILY_KIND }));
        assert_eq!(read_reference(world.source_bytes(), previous + 3), None);
        assert_eq!(&world.source_bytes()[previous + 3..previous + 6], &world.source_bytes()[retired + 3..retired + 6]);
        assert_eq!(read_reference(world.source_bytes(), retired), Some(EntityRef { index: 7, kind: PLANT_FAMILY_KIND }));
        assert_eq!(read_reference(world.source_bytes(), retired + 3), None);
        world.validate_loaded_plant_state().unwrap();
    }

    #[test]
    fn later_counter_subtracts_four_then_retires_without_incrementing_counter() {
        let mut world = synthetic_world(1, 2);
        world.validate_loaded_plant_state().unwrap();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.callbacks, [8]);
        assert_eq!(report.active_plant_count_after, 0);
        assert_eq!(world.plant_count(), 0);
        assert_eq!(world.source_bytes()[277_838 + 8 * 41 + 39], 2);
        assert_eq!(read_i16(world.source_bytes(), 277_838 + 8 * 41 + 36), 0);
    }

    #[test]
    fn later_counter_increment_wraps_and_old_counter_gate_subtracts_four() {
        let mut world = synthetic_world(255, 2);
        let plant = 277_838 + 8 * 41;
        write_i16(&mut world.bytes, plant + 36, 20);
        world.validate_loaded_plant_state().unwrap();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.events, [TickEvent::Grew { record_index: 8, growth_before: 20, growth_after: 16 }]);
        assert_eq!(world.source_bytes()[plant + 39], 0);
        assert_eq!(read_i16(world.source_bytes(), plant + 36), 16);
        assert_eq!(world.source_bytes()[plant + 25], 2);
    }

    #[test]
    fn final_dirty_flag_accounts_for_flags_twoe_bit_one() {
        let mut world = synthetic_world(1, 2);
        let plant = 277_838 + 8 * 41;
        world.bytes[plant + 21] = 0;
        world.bytes[plant + 38] = 0x02;
        write_i16(&mut world.bytes, plant + 36, 5);
        world.parameters.records[0].check_threshold_u8 = 10;
        world.validate_loaded_plant_state().unwrap();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.active_plant_count_after, 1);
        assert_eq!(world.source_bytes()[plant + 21], 0x04);
    }

    #[test]
    fn modulo_reproduction_path_fails_transactionally_instead_of_skipping_rng() {
        let mut world = synthetic_world(1, 2);
        let plant = 277_838 + 8 * 41;
        write_i16(&mut world.bytes, plant + 36, 20);
        world.parameters.records[0].check_threshold_u8 = 10;
        world.parameters.records[0].spread_period_u8 = 1;
        world.validate_loaded_plant_state().unwrap();
        let before = world.source_bytes().to_vec();
        assert!(matches!(world.tick_plants(), Err(RuntimeError::UnsupportedScheduledPlantState { reason: "scheduled reproduction/RNG path is not implemented", .. })));
        assert_eq!(world.source_bytes(), before);
        assert_eq!(world.tick_number(), 0);
        assert_eq!(world.schedule_cursor(), 0);
    }

    #[test]
    fn retirement_with_existing_delay_preserves_growth_and_animation() {
        let mut world = synthetic_world(9, 2);
        let plant = 277_838 + 8 * 41;
        write_i16(&mut world.bytes, plant + 36, -3);
        world.bytes[plant + 38] = 0x80;
        world.bytes[plant + 40] = 8;
        world.bytes[plant + 25] = 7;
        world.validate_loaded_plant_state().unwrap();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.events, [TickEvent::DelayedRetirement { record_index: 8, remaining_delay: 4 }]);
        assert_eq!(read_i16(world.source_bytes(), plant + 36), -3);
        assert_eq!(world.source_bytes()[plant + 25], 7);
        assert_eq!(world.source_bytes()[plant + 38], 0x80);
        assert_eq!(world.source_bytes()[plant + 39], 9);
        assert_eq!(world.source_bytes()[plant + 40], 4);
    }

    #[test]
    fn flags_twoe_bit_ten_callback_is_a_noop_and_not_queued() {
        let mut world = synthetic_world(0, 2);
        let plant = 277_838 + 8 * 41;
        world.bytes[plant + 38] = 0x10;
        world.validate_loaded_plant_state().unwrap();
        let before = world.source_bytes().to_vec();
        let report = world.tick_plants().unwrap();
        assert_eq!(report.events, [TickEvent::SuppressedNoop { record_index: 8, flags_at_15: 4, flags_at_2e: 0x10 }]);
        assert!(report.queued_indices.is_empty());
        let mut expected = before;
        write_u16(&mut expected, SCHEDULE_CURSOR_OFFSET, 16);
        assert_eq!(world.source_bytes(), expected);
    }
}

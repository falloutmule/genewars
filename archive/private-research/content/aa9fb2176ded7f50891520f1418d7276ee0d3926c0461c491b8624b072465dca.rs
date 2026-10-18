//! Atomic terrain-job maintenance for the source path with no height setter.
use super::*;
use crate::construction_maintenance::{
    plan_terrain_job_without_height_changes, TerrainEnvironment, TerrainMaintenanceError,
};
use crate::construction_terrain::TerrainJob;
use crate::terrain_job_pool::{TerrainJobPool, TerrainJobPoolError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainUpdateError {
    World(ActorWorldError),
    Pool(TerrainJobPoolError),
    Maintenance(TerrainMaintenanceError),
    CountMismatch,
    TerrainObjectRefreshRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainUpdateReceipt {
    pub slot: u8,
    pub before: TerrainJob,
    /// Includes source 79220's counter=-1 when the job retires.
    pub after: TerrainJob,
    pub retired: bool,
    pub count_before: u8,
    pub count_after: u8,
    /// The caller must run the post-pass 79273 terrain-object refresh after any retirement.
    pub terrain_objects_refresh_required: bool,
    /// Ordered source viewport requests; presentation remains caller-owned.
    pub viewport_invalidations: Vec<u16>,
}

impl ActorWorld {
    /// Entire source 79220 pass for an empty terrain-object list. Active jobs
    /// run in slot order, seeing prior jobs' map edits. A late unsupported
    /// height setter rolls back every earlier job in this pass. The empty
    /// list explicitly discharges 79273's post-retirement 79D50 work.
    pub fn step_terrain_jobs_without_height_changes(
        &mut self,
        jobs: &mut TerrainJobPool,
        environment: TerrainEnvironment,
        border_table: &[u8; 8192],
    ) -> Result<Vec<TerrainUpdateReceipt>, TerrainUpdateError> {
        if self.backing[0x2ff7] != jobs.count() {
            return Err(TerrainUpdateError::CountMismatch);
        }
        if self.backing[0x34b8] != 0 {
            return Err(TerrainUpdateError::TerrainObjectRefreshRequired);
        }
        let mut staged = self.clone();
        let mut staged_jobs = jobs.clone();
        let mut receipts = Vec::new();
        for slot in 0..25u8 {
            let record = &staged_jobs.records()[usize::from(slot)];
            if i16::from_le_bytes(record[12..14].try_into().unwrap()) < 0 {
                continue;
            }
            let mut receipt = staged.step_terrain_job_without_height_changes(
                &mut staged_jobs, slot, environment, border_table,
            )?;
            receipt.terrain_objects_refresh_required = false;
            receipts.push(receipt);
        }
        *self = staged;
        *jobs = staged_jobs;
        Ok(receipts)
    }

    /// Source 78E30 and the single-record retirement within 79220. This does
    /// not run the full job scheduler or its post-pass terrain-object refresh.
    /// Map, complete
    /// job record and mission job count commit together. Environment globals
    /// are explicit inputs; the admitted path consumes no random numbers.
    /// A height setter or unsupported rectangle rejects the entire update.
    pub fn step_terrain_job_without_height_changes(
        &mut self,
        jobs: &mut TerrainJobPool,
        slot: u8,
        environment: TerrainEnvironment,
        border_table: &[u8; 8192],
    ) -> Result<TerrainUpdateReceipt, TerrainUpdateError> {
        self.validate_links().map_err(TerrainUpdateError::World)?;
        if self.backing[0x2ff7] != jobs.count() {
            return Err(TerrainUpdateError::CountMismatch);
        }
        let before = *jobs.records().get(usize::from(slot))
            .ok_or(TerrainUpdateError::Pool(TerrainJobPoolError::InvalidSlot))?;
        let plan = plan_terrain_job_without_height_changes(
            self.map(), &before, environment, border_table,
        ).map_err(TerrainUpdateError::Maintenance)?;
        let mut staged = self.clone();
        let mut staged_jobs = jobs.clone();
        staged_jobs.apply_step(slot, &before, &plan.after, plan.retirement_required)
            .map_err(TerrainUpdateError::Pool)?;
        staged.backing[MAP_START..MAP_START + MAP_BYTES].copy_from_slice(&plan.map_after);
        staged.backing[0x2ff7] = staged_jobs.count();
        staged.validate_links().map_err(TerrainUpdateError::World)?;
        let receipt = TerrainUpdateReceipt {
            slot, before, after: staged_jobs.records()[usize::from(slot)],
            retired: plan.retirement_required,
            count_before: jobs.count(), count_after: staged_jobs.count(),
            terrain_objects_refresh_required: plan.retirement_required,
            viewport_invalidations: plan.viewport_invalidations,
        };
        *self = staged;
        *jobs = staged_jobs;
        Ok(receipt)
    }
}

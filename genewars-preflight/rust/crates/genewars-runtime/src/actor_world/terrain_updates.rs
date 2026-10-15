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
    Height(crate::terrain_height::TerrainHeightError),
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
    /// Ordered8C550 transaction primitive. Its caller owns the world rollback.
    fn apply_terrain_height(&mut self,cell:u16,height:i16,environment:TerrainEnvironment,
        invalidations:&mut Vec<u16>)->Result<(),TerrainUpdateError>{
        use crate::terrain_height::TerrainHeightStep;
        let plan=crate::terrain_height::plan_height_change(self.map(),cell,height,self.rng)
            .map_err(TerrainUpdateError::Height)?;
        for step in plan.steps {
            match step {
                TerrainHeightStep::SetHeight(change)=>{
                    let p=MAP_START+usize::from(change.cell)*14+2;
                    if word(&self.backing,p)!=change.before{return Err(TerrainUpdateError::World(ActorWorldError::StaleLink));}
                    self.backing[p..p+2].copy_from_slice(&change.after.to_le_bytes());
                },
                TerrainHeightStep::RefreshVertex{position}|TerrainHeightStep::RefreshSlopeViewport{position}=>{
                    let slope=matches!(step,TerrainHeightStep::RefreshSlopeViewport{..});
                    self.refresh_terrain_vertex(position,slope).map_err(TerrainUpdateError::World)?;
                    invalidations.push(floor_cell(position)as u16);
                },
                TerrainHeightStep::RngDraw{seed_before,seed_after,ring_slot,..}=>{
                    if self.rng.seed!=seed_before||self.rng.ring!=ring_slot{return Err(TerrainUpdateError::World(ActorWorldError::StaleLink));}
                    self.rng.seed=seed_after;self.rng.ring=(self.rng.ring+1)&31;
                    self.rng.draw_count=self.rng.draw_count.wrapping_add(1);
                },
                TerrainHeightStep::RefreshSlopeShade{cell,source_index}=>{
                    crate::construction_maintenance::refresh_cell_shade(&mut self.backing[MAP_START..MAP_START+MAP_BYTES],cell,source_index,environment)
                        .map_err(|e|TerrainUpdateError::Maintenance(e.into()))?;
                },
                TerrainHeightStep::RefreshLinkedObjects{cell}=>self.refresh_terrain_object_heights(cell).map_err(TerrainUpdateError::World)?,
            }
        }
        debug_assert_eq!(self.rng,plan.rng_after);Ok(())
    }

    /// Full terrain job path, including ordered height and linked-object updates.
    /// Terrain objects outside the common entity pools remain unsupported.
    pub fn step_terrain_jobs(&mut self,jobs:&mut TerrainJobPool,environment:TerrainEnvironment,
        border_table:&[u8;8192])->Result<Vec<TerrainUpdateReceipt>,TerrainUpdateError>{
        use crate::construction_terrain::*;
        if self.backing[0x2ff7]!=jobs.count(){return Err(TerrainUpdateError::CountMismatch);}
        //79220 reaches zone refresh only from an active job. With no jobs,
        //its25 negative-counter slots leave the zone list unchanged.
        if self.backing[0x34b8]!=0&&jobs.count()!=0{return Err(TerrainUpdateError::TerrainObjectRefreshRequired);}
        self.validate_links().map_err(TerrainUpdateError::World)?;
        let mut staged=self.clone();let mut pool=jobs.clone();let mut receipts=Vec::new();
        let terrain_error=|e:TerrainJobError|TerrainUpdateError::Maintenance(e.into());
        for slot in 0..25u8 {
            let before=pool.records()[usize::from(slot)];
            if word(&before,12)<0{continue;}
            // Validate first, but recompute each vertex from the live map:
            // a preceding8C550 slope repair can alter later job vertices.
            plan_terrain_job_numeric_step(&before,staged.map()).map_err(terrain_error)?;
            let mut invalidations=Vec::new();let step=word(&before,10);
            for row in 0..before[9]{for col in 0..before[8]{
                let cell=floor_cell([word(&before,0).wrapping_add(i16::from(col)*256),word(&before,2).wrapping_add(i16::from(row)*256)])as u16;
                let current=word(staged.map(),usize::from(cell)*14+2);
                let target=word(&before,14+usize::from(row)*18+usize::from(col)*2);
                if current!=target {
                    let delta=i32::from(target)-i32::from(current);
                    let height=if delta.abs()>i32::from(step){current.wrapping_add(if delta>0{step}else{-step})}else{target};
                    staged.apply_terrain_height(cell,height,environment,&mut invalidations)?;
                }
            }}
            let plan=plan_terrain_job_numeric_step(&before,staged.map()).map_err(terrain_error)?;
            for request in plan.maintenance {
                match request {
                    TerrainMaintenanceRequest::ClassifyCell(cell)=>classify_terrain_cell(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],cell,environment.climate_center,environment.raw_water_height,environment.climate).map_err(terrain_error)?,
                    TerrainMaintenanceRequest::BorderCell(cell)=>repair_terrain_material_boundary(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],cell).map_err(terrain_error)?,
                    TerrainMaintenanceRequest::BorderRectangle{min,max}=>invalidations.extend(repair_terrain_rectangle(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],min,max,border_table).map_err(terrain_error)?),
                    TerrainMaintenanceRequest::RefreshRectangle{min,max,completed}=>{
                        if min.iter().any(|v|*v<256)||max.iter().any(|v|*v>32256){return Err(TerrainUpdateError::Maintenance(TerrainMaintenanceError::UnsupportedRectangle));}
                        let x=i32::from(min[0]>>8)-1;let y=i32::from(min[1]>>8)-1;
                        let width=i32::from(max[0]>>8)-i32::from(min[0]>>8)+2;let height=i32::from(max[1]>>8)-i32::from(min[1]>>8)+2;
                        for row in 0..height{for col in 0..width{
                            let cell=((y+row)*128+x+col)as u16;let p=MAP_START+usize::from(cell)*14;
                            if staged.backing[p+4]==0x97{invalidations.extend(crate::construction_setup::prepare_terrain_site_cell(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],cell,border_table).map_err(terrain_error)?);}
                            //8CDE0 calls setters between its four ordered corner tests.
                            if completed&&staged.backing[p+11]&2!=0 {
                                let cx=cell&127;let cy=cell>>7;
                                for(dx,dy)in[(0,0),(1,0),(1,1),(0,1)]{
                                    let corner=(((cy+dy)&127)<<7)|((cx+dx)&127);let at=MAP_START+usize::from(corner)*14;
                                    if staged.backing[at+5]&7==0&&i32::from(environment.raw_water_height)-i32::from(word(&staged.backing,at+2))>=384{
                                        staged.rng.seed=staged.rng.seed.wrapping_mul(0xe62d);staged.rng.ring=(staged.rng.ring+1)&31;staged.rng.draw_count=staged.rng.draw_count.wrapping_add(1);
                                        let target=environment.raw_water_height.wrapping_sub(((staged.rng.seed>>8)%384)as i16);
                                        staged.apply_terrain_height(corner,target,environment,&mut invalidations)?;
                                    }
                                }
                            }
                            crate::construction_maintenance::refresh_cell_shade(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],cell,i32::from(cell),environment).map_err(terrain_error)?;
                        }}
                    },
                }
            }
            let count_before=pool.count();pool.apply_step(slot,&before,&plan.after,plan.retirement_required).map_err(TerrainUpdateError::Pool)?;
            staged.backing[0x2ff7]=pool.count();
            receipts.push(TerrainUpdateReceipt{slot,before,after:pool.records()[usize::from(slot)],retired:plan.retirement_required,count_before,count_after:pool.count(),terrain_objects_refresh_required:false,viewport_invalidations:invalidations});
        }
        staged.validate_links().map_err(TerrainUpdateError::World)?;
        *self=staged;*jobs=pool;Ok(receipts)
    }

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

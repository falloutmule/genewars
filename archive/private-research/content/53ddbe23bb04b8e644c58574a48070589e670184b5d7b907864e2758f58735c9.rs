//! Owned building storage and the source42FA0 factory transaction.
//! Scheduling, construction completion and rendering remain separate.
use super::*;
use crate::building::{self, BuildingAnimationBinding, BuildingFactoryPlan, BuildingRecord, BuildingReference};
use crate::building_pool::BuildingPool;

impl ActorWorld {
    /// Source45010's inactive loader preserves the supplied native pointer gap.
    /// Active serialized buildings require their separately bound resources.
    pub fn admit_buildings(&mut self, prior: &[BuildingRecord; 256],
        bindings: [Option<BuildingAnimationBinding>; 256]) -> Result<(), ActorWorldError> {
        if self.buildings.is_some() { return Err(ActorWorldError::BuildingsAlreadyAdmitted); }
        let group=self.layouts[3];
        let pool=BuildingPool::from_packed_inactive(
            &self.backing[group.file_offset..group.file_offset+group.count*group.packed_stride],prior,bindings)?;
        let mut staged=self.clone();staged.buildings=Some(Box::new(pool));
        staged.validate_building_counts()?;staged.validate_links()?;*self=staged;Ok(())
    }

    /// Source42FA0/43E30/43560 with actual terrain and intrusive-list state.
    /// Each player's 64-slot partition, count and center update atomically.
    pub fn create_building(&mut self, owner:u8, selector:u8, position:[i16;2])
        ->Result<Option<BuildingFactoryPlan>,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let player=*self.player(owner)?.bytes();
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let ground=genewars_core::cd_candidate::interpolate_height(self.map(),position[0],position[1])
            .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        let shade=genewars_core::cd_candidate::interpolate_entity_shade(self.map(),position[0],position[1])
            .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
        let cell=floor_cell(position);let old=reference(self.map(),cell*14+6);
        let head=if old[2]==0 {None}else{
            self.entity(old)?;Some(BuildingReference{index:word(&old,0),kind:old[2]})
        };
        let Some(plan)=building::plan_building_factory(pool.records(),&player,selector,position,ground,shade,head)?
            else{return Ok(None)};
        let mut staged=self.clone();
        staged.buildings.as_mut().unwrap().apply_factory(&plan)?;
        staged.players[usize::from(owner)].replace_bookkeeping_checked(&player,&plan.player_after)?;
        let raw=[plan.slot as u8,(plan.slot>>8)as u8,4];
        if head.is_some(){staged.replace_previous(old,raw)?;}
        staged.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&raw);
        staged.validate_building_counts()?;staged.validate_links()?;*self=staged;Ok(Some(plan))
    }

    pub(super) fn validate_building_counts(&self)->Result<(),ActorWorldError>{
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        for owner in 0..4usize{
            let declared=self.players[owner].bytes()[0x4a2] as i8;
            let active=pool.records()[owner*64..(owner+1)*64].iter().filter(|r|r[6]!=0).count();
            if declared<0||declared as usize!=active {
                return Err(ActorWorldError::BuildingCountMismatch{owner:owner as u8,declared,active});
            }
        }Ok(())
    }

    pub(super) fn owner_building_records(&self,owner:u8)->Result<Vec<u8>,ActorWorldError>{
        self.player(owner)?;
        let mut records=Vec::with_capacity(64*67);
        if let Some(pool)=&self.buildings {
            for r in &pool.records()[usize::from(owner)*64..usize::from(owner+1)*64]{records.extend_from_slice(r);}
        }else{
            let group=self.layouts[3];
            for slot in usize::from(owner)*64..usize::from(owner+1)*64{
                let start=group.file_offset+slot*group.packed_stride;let packed=&self.backing[start..start+59];
                // Existing target queries admit only inactive records. Resource
                // pointers are never synthesized for an active packed building.
                if packed[6]!=0{return Err(ActorWorldError::BuildingsNotAdmitted);}
                records.extend_from_slice(&packed[..31]);records.extend_from_slice(&[0;8]);records.extend_from_slice(&packed[31..]);
            }
        }Ok(records)
    }
}

impl ActorWorld {
    /// Source441C0 selector1 progress, including43F60 completion,43620
    /// player aggregation and6A6D0 live-creature target cleanup. A late failure
    /// leaves every pool unchanged. Audio and selected UI remain requests.
    pub fn progress_selector1_building(&mut self, slot:u16, increment:i16, loading:bool)
        ->Result<crate::building_progress::BuildingProgressPlan,ActorWorldError> {
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        if pool.record(usize::from(slot))?[45]!=1{return Err(ActorWorldError::BuildingProgress(crate::building_progress::BuildingProgressError::InvalidBuilding));}
        self.progress_admitted_grade0_building(slot,increment,loading)
    }

    /// Shared441C0 completion for reviewed selectors1 and6. This does not
    /// admit their per-frame behavior or bypass construction/menu gates.
    pub fn progress_admitted_grade0_building(&mut self,slot:u16,increment:i16,loading:bool)
        ->Result<crate::building_progress::BuildingProgressPlan,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let record=pool.record(usize::from(slot))?;
        let creatures=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;
        let plan=crate::building_progress::plan_admitted_grade0_progress(pool.records(),slot,
            self.player(record[24])?.bytes(),creatures.records(),word(&self.backing,0x3550),increment,loading)
            .map_err(ActorWorldError::BuildingProgress)?;
        let mut staged=self.clone();
        staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&plan.before,&plan.after)?;
        staged.players[usize::from(record[24])].replace_bookkeeping_checked(&plan.player_before,&plan.player_after)?;
        for change in &plan.creature_changes {
            staged.creatures.as_mut().unwrap().replace_record_checked(usize::from(change.slot),&change.before,&change.after)?;
        }
        staged.validate_links()?;staged.validate_building_counts()?;*self=staged;Ok(plan)
    }
}

impl ActorWorld {
    /// Source451F0 incomplete selector1 update using the exact BTURR frame
    /// holds. Stop-audio23 and queue admission remain explicit receipt fields.
    pub fn step_incomplete_selector1_building(&mut self,slot:u16,
        resource:&crate::building_resources::BuildingResourceDatabase)
        ->Result<crate::building_callback::IncompleteBuildingPlan,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let before=*pool.record(usize::from(slot))?;
        if pool.binding(usize::from(slot))?!=Some(BuildingAnimationBinding{selector:resource.selector()}) {
            return Err(crate::building_pool::BuildingPoolError::InvalidBinding(usize::from(slot)).into());
        }
        let plan=crate::building_callback::plan_incomplete_selector1(&before,resource.animation0().holds().as_slice())
            .map_err(ActorWorldError::BuildingCallback)?;
        let mut staged=self.clone();
        staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&plan.before,&plan.after)?;
        staged.validate_links()?;staged.validate_building_counts()?;*self=staged;Ok(plan)
    }
}

impl ActorWorld {
    /// Incomplete first-mission buildings using their exact source resource.
    /// Completed callbacks are deliberately separate and cannot fall through.
    pub fn step_incomplete_source_building(&mut self,slot:u16,
        resource:&crate::source_building_resources::SourceBuildingResourceDatabase)
        ->Result<crate::building_callback::IncompleteBuildingPlan,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let before=*pool.record(usize::from(slot))?;
        if before[45]!=resource.selector()||pool.binding(usize::from(slot))?!=Some(BuildingAnimationBinding{selector:resource.selector()}) {
            return Err(crate::building_pool::BuildingPoolError::InvalidBinding(usize::from(slot)).into());
        }
        let holds=resource.animation(0).map_err(|_|ActorWorldError::BuildingCallback(crate::building_callback::BuildingCallbackError::InvalidAnimation))?.iter().map(|f|f.hold).collect::<Vec<_>>();
        let plan=crate::building_callback::plan_incomplete_admitted(&before,&holds).map_err(ActorWorldError::BuildingCallback)?;
        let mut staged=self.clone();
        staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&plan.before,&plan.after)?;
        staged.validate_links()?;staged.validate_building_counts()?;*self=staged;Ok(plan)
    }

    /// Actual8B000 cell-list reader. It returns the first eligible kind1,
    /// before48540 applies its separate state18/timer and distance filters.
    fn first_turret_cell_target(&self,owner:u8,point:[i16;2])
        ->Result<Option<crate::completed_building::TurretTarget>,ActorWorldError> {
        let player=self.player(owner)?.bytes();
        let mut raw=reference(self.map(),floor_cell(point)*14+6);
        let mut visited=HashSet::new();
        while raw[2]!=0 {
            if !visited.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
            let record=self.entity(raw)?;
            if raw[2]==1 {
                if record.len()!=124||record[24]>4{return Err(ActorWorldError::EnvironmentUnavailable);}
                let allied=(u32::from(player[0x6e5])&(1u32<<record[24]))!=0;
                // Kind1 vtable68=1EC50 returns state>=24, without side effects.
                if record[24]!=owner&&!allied&&record[0x5e]<24 {
                    return Ok(Some(crate::completed_building::TurretTarget{reference:raw,
                        candidate:crate::completed_building::TurretCellCandidate{
                        position:[word(record,7),word(record,9)],state:record[0x5e],timer:word(record,0x6c)}}));
                }
            }
            raw=reference(record,3);
        }
        Ok(None)
    }

    /// Completed selector1/grade0 with actual shared-map queries, restricted
    /// to no accepted attack target. Any failure preserves every owned byte,
    /// typed handle and live RNG draw. Queueing remains a presentation request.
    pub fn step_quiet_completed_selector1_building(&mut self,slot:u16,
        resource:&crate::building_resources::BuildingResourceDatabase)
        ->Result<crate::completed_building::QuietBuildingPlan,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let before=*pool.record(usize::from(slot))?;
        if pool.binding(usize::from(slot))?!=Some(BuildingAnimationBinding{selector:resource.selector()}) {
            return Err(crate::building_pool::BuildingPoolError::InvalidBinding(usize::from(slot)).into());
        }
        // The pure callback's fixed animation contract is independently tied
        // here to exact hash-admitted BTURR bytes, rather than guessed holds.
        let sections=genewars_assets::AnbSections::parse(resource.bytes())
            .map_err(|_|ActorWorldError::EnvironmentUnavailable)?;
        let metadata=sections.ui_metadata().map_err(|_|ActorWorldError::EnvironmentUnavailable)?;
        let animation=metadata.animation(1).map_err(|_|ActorWorldError::EnvironmentUnavailable)?;
        if animation.first_frame_offset!=42||animation.last_frame_offset!=140||animation.trailing_word!=8 {
            return Err(ActorWorldError::EnvironmentUnavailable);
        }
        for frame in 0..8 {
            if metadata.frame_at(42+frame*14).map_err(|_|ActorWorldError::EnvironmentUnavailable)?.byte_10!=0 {
                return Err(ActorWorldError::EnvironmentUnavailable);
            }
        }
        let player=*self.player(before[24])?.bytes();let mut query_error=None;
        let planned=crate::completed_building::plan_quiet_selector1(&before,&player,self.rng,|point| {
            self.first_turret_cell_target(before[24],point).map(|result|result.map(|target|target.candidate)).map_err(|error| {
                query_error=Some(error);crate::completed_building::QuietBuildingError::Environment
            })
        });
        if let Some(error)=query_error{return Err(error);}
        let plan=planned.map_err(ActorWorldError::QuietBuilding)?;
        let mut staged=self.clone();
        staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&plan.after)?;
        staged.players[usize::from(before[24])].replace_bookkeeping_checked(&player,&plan.player_after)?;
        staged.rng=plan.rng_after;staged.validate_links()?;staged.validate_building_counts()?;
        *self=staged;Ok(plan)
    }

    /// Actual-world target reuse/search and rotation, before the separately
    /// unimplemented475D5 projectile branch. A projectile boundary or query
    /// failure leaves all world/player/RNG/handle bytes unchanged.
    pub fn step_tracking_completed_selector1_building(&mut self,slot:u16,
        resource:&crate::building_resources::BuildingResourceDatabase,frame_counter:u32,
        physics:&crate::physics_tables::PhysicsTables)
        ->Result<crate::completed_building::TurretTrackingPlan,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let before=*pool.record(usize::from(slot))?;
        if pool.binding(usize::from(slot))?!=Some(BuildingAnimationBinding{selector:resource.selector()}) {
            return Err(crate::building_pool::BuildingPoolError::InvalidBinding(usize::from(slot)).into());
        }
        //BuildingResourceDatabase owns immutable exact-hash BTURR bytes;
        // the fixed animation1 eight-zero-hold contract is source reviewed.
        let existing=if before[66]==0{None}else{
            let raw=reference(&before,64);
            if raw[2]!=1{return Err(ActorWorldError::QuietBuilding(crate::completed_building::QuietBuildingError::UnsupportedRecord));}
            let target=self.entity(raw)?;
            if target.len()!=124{return Err(ActorWorldError::EnvironmentUnavailable);}
            Some(crate::completed_building::TurretTarget{reference:raw,
                candidate:crate::completed_building::TurretCellCandidate{
                    position:[word(target,7),word(target,9)],state:target[0x5e],timer:word(target,0x6c)}})
        };
        let player=*self.player(before[24])?.bytes();let mut query_error=None;
        let planned=crate::completed_building::plan_tracking_selector1(&before,&player,self.rng,
            frame_counter,physics,existing,|point| {
                self.first_turret_cell_target(before[24],point).map_err(|error| {
                    query_error=Some(error);crate::completed_building::QuietBuildingError::Environment
                })
            });
        if let Some(error)=query_error{return Err(error);}
        let plan=planned.map_err(ActorWorldError::QuietBuilding)?;
        let mut staged=self.clone();
        staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&before,&plan.numeric.after)?;
        staged.players[usize::from(before[24])].replace_bookkeeping_checked(&player,&plan.numeric.player_after)?;
        staged.rng=plan.numeric.rng_after;staged.validate_links()?;staged.validate_building_counts()?;
        *self=staged;Ok(plan)
    }
}

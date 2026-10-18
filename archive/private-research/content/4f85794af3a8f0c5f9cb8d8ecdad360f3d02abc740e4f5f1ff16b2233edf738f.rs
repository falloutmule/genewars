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
        let mut staged=self.clone();staged.buildings=Some(pool);
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
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        let record=pool.record(usize::from(slot))?;
        let creatures=self.creatures.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;
        let plan=crate::building_progress::plan_selector1_progress(pool.records(),slot,
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

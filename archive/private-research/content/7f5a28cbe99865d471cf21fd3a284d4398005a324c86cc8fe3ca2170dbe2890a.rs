//! Final neutral BONES callback:8ACC0 unlink,724C0 references and kind1 count.
use super::*;
use crate::creature_world::CommonPhysicsPlan;
use crate::landing_effect_lifecycle::{self as lifecycle, CleanupBytePatch};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MuleRetirementReceipt {
    pub before:CreatureRecord,
    pub after:CreatureRecord,
    pub cleanup:Vec<CleanupBytePatch>,
    pub count_after:i16,
    /// Source6A1AD writes this display-phase byte; the native host keeps the
    /// request explicit rather than claiming the original presentation loop.
    pub pending_display_phase_1185a8:u8,
}

impl ActorWorld {
    pub(super) fn retire_mule_bones_after_callback(&mut self,before:&CreatureRecord,
        plan:&CommonPhysicsPlan<124>)->Result<MuleRetirementReceipt,ActorWorldError> {
        let links=self.validate_links()?;
        if !links.contains(&(1,0)){return Err(ActorWorldError::InvalidReference([0,0,1]));}
        let pool=self.creatures.as_ref().ok_or(ActorWorldError::UnsupportedCreatureSchedule)?;
        if pool.record(0)?!=before{return Err(CreatureError::StaleRecord(0).into());}
        let declared=word(&self.backing,0x3550);
        let active=pool.records().iter().filter(|record|record[6]!=0).count();
        if declared<=0||declared as usize!=active{return Err(ActorWorldError::CreatureCountMismatch{declared,active});}
        if plan.old_cell!=plan.physics_cell||plan.physics_cell!=plan.new_cell
            ||plan.old_cell!=floor_cell([word(before,7),word(before,9)])
            ||plan.after[..6]!=before[..6]||plan.after[7..11]!=before[7..11] {
            return Err(CreatureError::UnsupportedMotionState.into());
        }
        let mut staged=self.clone();
        staged.creatures.as_mut().unwrap().replace_record_checked(0,before,&plan.after)?;
        staged.unlink_mule(plan.old_cell)?;
        let dead=landing::LandingReference{index:0,kind:1};
        let mut cleanup=Vec::new();
        for(kind,slot,record)in staged.effect_cleanup_records(){
            for patch in lifecycle::plan_reference_cleanup(kind,slot,&record,dead)?{
                staged.apply_cleanup_byte(patch)?;cleanup.push(patch);
            }
        }
        let current=*staged.creatures.as_ref().unwrap().record(0)?;
        staged.creatures.as_mut().unwrap().retire_bones_checked(0,&current)?;
        let count_after=declared-1;staged.backing[0x3550..0x3552].copy_from_slice(&count_after.to_le_bytes());
        let after=*staged.creatures.as_ref().unwrap().record(0)?;
        staged.validate_links()?;
        let active=staged.creatures.as_ref().unwrap().records().iter().filter(|record|record[6]!=0).count();
        if count_after as usize!=active{return Err(ActorWorldError::CreatureCountMismatch{declared:count_after,active});}
        *self=staged;
        Ok(MuleRetirementReceipt{before:*before,after,cleanup,count_after,pending_display_phase_1185a8:11})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->ActorWorld {
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());
        bytes[337_054..337_058].copy_from_slice(&0x6beu32.to_le_bytes());
        let source=ConqMissionSource::parse(&bytes).unwrap();
        let players=core::array::from_fn(|slot|PlayerState::fresh_campaign(slot as u8,[0,0],b"fixture\0").unwrap());
        let mut world=ActorWorld::from_parts(&source,&[[0;200];20],core::array::from_fn(|_|ActorHandleBindings::default()),players,
            BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap();
        let mut packed=vec![0;300*116];packed[6]=1;packed[24]=4;packed[0x7a-8]=4;packed[0x5e-8]=25;
        world.creatures=Some(Box::new(CreaturePool::from_packed(&packed,&[[0;124];300]).unwrap()));
        let before=*world.creatures.as_ref().unwrap().record(0).unwrap();let mut bones=before;
        bones[0x5e]=27;bones[0x68..0x6a].copy_from_slice(&1i16.to_le_bytes());
        world.creatures.as_mut().unwrap().replace_record_checked(0,&before,&bones).unwrap();
        world.backing[0x3550..0x3552].copy_from_slice(&1i16.to_le_bytes());
        world.backing[MAP_START+6..MAP_START+9].copy_from_slice(&[0,0,1]);
        world.admit_effects(&[[0;81];200],[None;200]).unwrap();
        world.create_water_splash([0,0],1000).unwrap().unwrap();
        let pool=world.effects.as_mut().unwrap();let before=*pool.record(0).unwrap();let mut referring=before;
        referring[71..74].copy_from_slice(&[0,0,1]);pool.replace_record_checked(0,&before,&referring).unwrap();
        world
    }
    fn plan(world:&ActorWorld)->(CreatureRecord,CommonPhysicsPlan<124>){
        let before=*world.creatures.as_ref().unwrap().record(0).unwrap();let mut after=before;
        after[0x68..0x6a].fill(0);after[0x6c..0x6e].copy_from_slice(&32766i16.to_le_bytes());
        (before,CommonPhysicsPlan{after,old_cell:0,physics_cell:0,new_cell:0})
    }
    #[test]
    fn retirement_unlinks_from_live_neighbor_and_clears_shared_references(){
        let mut world=fixture();let rng=world.rng;let (before,plan)=plan(&world);
        let receipt=world.retire_mule_bones_after_callback(&before,&plan).unwrap();
        assert_eq!(receipt.count_after,0);assert_eq!(receipt.pending_display_phase_1185a8,11);
        assert_eq!(receipt.after[6],0);assert_eq!(&receipt.after[..6],&before[..6]);
        assert_eq!(world.creatures.as_ref().unwrap().binding(0).unwrap(),None);
        let effect=world.effects.as_ref().unwrap().record(0).unwrap();assert_eq!(effect[5],0);assert_eq!(effect[73],0);
        assert_eq!(word(effect,71),0);assert!(receipt.cleanup.iter().any(|patch|patch.kind==5&&patch.offset==73));
        assert_eq!(world.validate_links().unwrap(),[(5,0)].into_iter().collect());assert_eq!(world.rng,rng);
    }
    #[test]
    fn late_retirement_cleanup_failure_preserves_all_pools_and_links(){
        let mut world=fixture();let group=world.layouts[6];world.backing[group.file_offset+6]=7;
        world.backing[group.file_offset+15..group.file_offset+17].copy_from_slice(&9i16.to_le_bytes());
        let unchanged=world.clone();let (before,plan)=plan(&world);
        assert!(world.retire_mule_bones_after_callback(&before,&plan).is_err());assert_eq!(world,unchanged);
    }
}

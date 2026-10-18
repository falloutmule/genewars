//! Owned completed Extractor transaction with explicit unsupported contact damage.
use super::*;
use crate::building::BuildingAnimationBinding;
use crate::source_building_resources::SourceBuildingResourceDatabase;
use crate::extractor_production::{ExtractorCompletedPlan,ExtractorCallbackError};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ExtractorWorldReceipt {
    pub slot:u16,
    pub plan:ExtractorCompletedPlan,
    pub created_effect:Option<u16>,
    pub factory_draw:Option<landing::LandingDraw>,
    pub terrain_invalidations:Vec<u16>,
    pub rng_before:BlueprintRng,
    pub rng_after:BlueprintRng,
}
impl ActorWorld {
    /// Source451F0 selector11, grade0. Actual linked-cell queries must prove
    /// no eligible enemy or own kind1. A target rejects the entire transaction;
    /// zero grade damage is not treated as permission to skip its callback.
    pub fn step_completed_extractor(&mut self,slot:u16,resource:&SourceBuildingResourceDatabase,
        frame:u32,work_audio_active:bool,context:LandingPhysicsContext,trig:&TrigTables,
        terrain_table:&[u8;8192])->Result<ExtractorWorldReceipt,ActorWorldError> {
        self.validate_links()?;self.validate_building_counts()?;
        let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;
        if resource.selector()!=11||pool.binding(usize::from(slot))?!=Some(BuildingAnimationBinding{selector:11}){
            return Err(crate::building_pool::BuildingPoolError::InvalidBinding(usize::from(slot)).into());
        }
        let before=*pool.record(usize::from(slot))?;let owner=before[24];
        let player=*self.player(owner)?.bytes();
        let plan=crate::extractor_production::plan_completed_with_requests(&before,&player,work_audio_active,
            self.map(),frame,self.rng.seed,self.rng.ring,self.rng.draw_count).map_err(ActorWorldError::ExtractorCallback)?;
        let mut staged=self.clone();let old_rng=self.rng;let mut created_effect=None;let mut factory_draw=None;let mut terrain_invalidations=Vec::new();
        if let Some(contact)=plan.contact{
            staged.rng=BlueprintRng{seed:contact.seed_after,ring:contact.ring_after,draw_count:contact.draw_count_after};
            if let Some(angles)=contact.smoke_angles{
                // Original7B140 height followed by48790 code6, owner0, flags0;
                // unlike landing smoke, this factory does not mark newborn.
                let height=genewars_core::cd_candidate::interpolate_height(staged.map(),contact.position[0],contact.position[1])
                    .map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
                if let Some((slot,draw))=staged.create_unmarked_smoke(contact.position,height,angles.map(|a|a as u16),context,trig)?{
                    created_effect=Some(slot);factory_draw=Some(draw);
                }
            }
            let alliances=player[0x6e5];let identity=player[1];
            let mut raw=reference(staged.map(),usize::from(contact.cell)*14+6);let mut visited=HashSet::new();
            while raw[2]!=0{
                if !visited.insert(raw){return Err(ActorWorldError::DuplicateLink(raw));}
                let entity=staged.entity(raw)?;
                if raw[2]==1{
                    if entity.len()!=124{return Err(ActorWorldError::EnvironmentUnavailable);}
                    let target_owner=entity[24];
                    let allied=(u32::from(alliances)&1u32.wrapping_shl(u32::from(target_owner)&31))!=0;
                    let foreign=target_owner!=identity&&!allied;
                    let own=target_owner==identity;
                    if entity[0x5e]<24&&(foreign||own){
                        return Err(ActorWorldError::ExtractorCallback(ExtractorCallbackError::HarvestContactRequired));
                    }
                }
                raw=reference(entity,3);
            }
        }
        if let Some(harvest)=&plan.harvest{
            if let Some(index)=harvest.depleted_terrain{
                terrain_invalidations=crate::construction_setup::prepare_terrain_site_cell(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],index,terrain_table)
                    .map_err(|e|ActorWorldError::Terrain(format!("{e:?}")))?;
            }
            if let Some(resource)=harvest.resource{
                let at=MAP_START+usize::from(resource.cell)*14+13;
                if staged.backing[at]!=resource.before{return Err(ActorWorldError::EnvironmentUnavailable);}
                staged.backing[at]=resource.after;
            }
        }
        let current=*staged.buildings.as_ref().unwrap().record(usize::from(slot))?;
        let mut after=plan.state.building_after;
        // Inserting smoke may change this building's intrusive-list links.
        after[..6].copy_from_slice(&current[..6]);
        staged.buildings.as_mut().unwrap().replace_record_checked(usize::from(slot),&current,&after)?;
        staged.players[usize::from(owner)].replace_bookkeeping_checked(&player,&plan.state.player_after)?;
        staged.validate_links()?;staged.validate_building_counts()?;
        if staged.effects.is_some(){staged.validate_effect_count()?;}
        let receipt=ExtractorWorldReceipt{slot,plan,created_effect,factory_draw,terrain_invalidations,rng_before:old_rng,rng_after:staged.rng};
        *self=staged;Ok(receipt)
    }
}


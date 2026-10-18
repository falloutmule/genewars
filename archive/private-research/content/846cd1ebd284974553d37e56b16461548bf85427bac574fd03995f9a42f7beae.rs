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

// Appended only to the immutable private source snapshot by its validator.
#[cfg(test)]mod extractor_owned_review {
 use super::*;
 #[test]fn captures(){
  let root=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_ROOT").unwrap());let out=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_OUT").unwrap());
  let resource=SourceBuildingResourceDatabase::load(crate::source_building_resources::SourceBuildingBinding::CdBmineSelector11,std::fs::read(root.join("private/integration/game-runtime/buildings/BMINE.ANB")).unwrap()).unwrap();
  let object=std::fs::read(root.join("private/ghidra/validated/memory-.object3.bin")).unwrap();let table:&[u8;8192]=object[0x12d24..0x14d24].try_into().unwrap();
  let trig=TrigTables::generate_pinned().unwrap();let context=LandingPhysicsContext{gravity:1024,water_height:1632};
  for case in 0..8u8{
   let mut bytes=vec![0;343958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());bytes[337054..337058].copy_from_slice(&0x6beu32.to_le_bytes());
   for i in 0..16384{let at=MAP_START+i*14;bytes[at..at+2].copy_from_slice(&8192i16.to_le_bytes());bytes[at+2..at+4].copy_from_slice(&3136i16.to_le_bytes());bytes[at+5]=7;bytes[at+13]=if case==2{0}else if case%2==0{2}else{9};}
   let players=core::array::from_fn(|id|PlayerState::fresh_campaign(id as u8,[0,0],b"fixture\0").unwrap());
   let mut w=ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:65535,ring:31,draw_count:u32::MAX}).unwrap();
   w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();
   let slot=w.create_building(0,11,[16384,15872]).unwrap().unwrap().slot;
   let before=*w.buildings.as_ref().unwrap().record(0).unwrap();let mut b=before;b[49]=1;b[27]=7+(case%4);b[51]=0;b[61..63].copy_from_slice(&200i16.to_le_bytes());b[63]=0;
   w.buildings.as_mut().unwrap().replace_record_checked(0,&before,&b).unwrap();
   let mut p=*w.players[0].bytes();p[0x4a7..0x4ab].fill(0);p[0x5e6..0x5ea].copy_from_slice(&600u32.to_le_bytes());if case==7{p[0x6e5]=2;}w.players[0]=PlayerState::from_record(&p).unwrap();
   if case>=4{let pos=[16384i16,16128];let cell=floor_cell(pos);let mut c=[0u8;124];c[..2].copy_from_slice(&(cell as i16).to_le_bytes());c[6]=1;c[7..9].copy_from_slice(&pos[0].to_le_bytes());c[9..11].copy_from_slice(&pos[1].to_le_bytes());c[24]=if case==4{0}else{1};c[0x5e]=if case==6{24}else{0};w.backing[243038..243069].copy_from_slice(&c[..31]);w.backing[243069..243154].copy_from_slice(&c[39..]);w.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&[0,0,1]);w.backing[0x3550..0x3552].copy_from_slice(&1i16.to_le_bytes());}
   w.admit_creatures(&[[0;124];300]).unwrap();
   let snapshot=|w:&ActorWorld|{let mut s=w.backing.clone();for r in w.buildings.as_ref().unwrap().records(){s.extend(r);}for p in &w.players{s.extend(p.bytes());}for r in w.effects.as_ref().unwrap().records(){s.extend(r);}for r in w.creatures.as_ref().unwrap().records(){s.extend(r);}s.extend(w.rng.seed.to_le_bytes());s.push(w.rng.ring);s.extend(w.rng.draw_count.to_le_bytes());s};
   std::fs::write(out.join(format!("case{case}-before.bin")),snapshot(&w)).unwrap();let prior=w.clone();
   let result=w.step_completed_extractor(slot,&resource,u32::from(case%2),false,context,&trig,table);
   if matches!(case,2|4|5){assert!(result.is_err(),"case{case}");assert_eq!(w,prior,"case{case}");}else{let q=result.unwrap();assert_eq!(q.created_effect.is_some(),case%2==1);assert_eq!(q.rng_after.draw_count.wrapping_sub(q.rng_before.draw_count),if case%2==1{4}else{0});}
   std::fs::write(out.join(format!("case{case}-after.bin")),snapshot(&w)).unwrap();
  }
 }
}

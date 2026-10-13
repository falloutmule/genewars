//! Source-backed neutral plant creation and explicit reproduction scheduling.
use super::*;
use crate::plant_spread::SpreadDirections;
use crate::world::PlantReproductionContext;

impl ActorWorld {
    /// Neutral6C4B0 factory, without the caller's newborn flag or scheduler.
    pub fn create_neutral_plant(&mut self,selector:u8,requested:[i16;2])->Result<Option<u16>,ActorWorldError> {
        let subsystem=self.plants.as_ref().ok_or(ActorWorldError::PlantsNotAdmitted)?.clone();
        self.validate_links()?;let mut staged=self.clone();
        let mut view=MissionWorld::with_shared_storage(SharedPlantStorage{backing:&mut staged.backing,actors:&mut staged.actors,
            effects:staged.effects.as_deref_mut(),creatures:staged.creatures.as_deref_mut(),family8:staged.family8.as_deref_mut(),buildings:staged.buildings.as_deref_mut()},subsystem.parameters,subsystem.tick_number)?;
        let result=view.create_neutral_plant_inner(selector,requested)?;
        staged.validate_links()?;*self=staged;Ok(result)
    }

    /// Source6D0B0 with neutral reproduction, live shared RNG and a newborn
    /// queue pass. All overlays, RNG and plant tick state commit together.
    pub fn tick_plants_with_reproduction(&mut self,directions:&SpreadDirections)->Result<PlantTickReport,ActorWorldError> {
        let subsystem=self.plants.as_ref().ok_or(ActorWorldError::PlantsNotAdmitted)?.clone();
        self.validate_links()?;let mut staged=self.clone();
        let height_threshold=word(&staged.backing,0x1c63);
        let mut context=PlantReproductionContext{rng:&mut staged.rng,directions,height_threshold};
        let mut view=MissionWorld::with_shared_storage(SharedPlantStorage{backing:&mut staged.backing,actors:&mut staged.actors,
            effects:staged.effects.as_deref_mut(),creatures:staged.creatures.as_deref_mut(),family8:staged.family8.as_deref_mut(),buildings:staged.buildings.as_deref_mut()},subsystem.parameters,subsystem.tick_number)?;
        let report=view.tick_plants_inner_with_reproduction(&mut context)?;
        // 724C0 visits native target references after an actual retirement.
        // UI roots remain pending, but typed actor/creature/other-family references
        // must not continue pointing at an inactive or subsequently reused slot.
        for event in &report.events {
            if let crate::TickEvent::Retired{record_index,..}=event {
                let target=crate::landing::LandingReference{index:*record_index as i16,kind:2};
                for(kind,slot,bytes)in staged.effect_cleanup_records(){
                    for patch in crate::landing_effect_lifecycle::plan_reference_cleanup(kind,slot,&bytes,target)?{staged.apply_cleanup_byte(patch)?;}
                }
            }
        }
        staged.plants.as_mut().unwrap().tick_number=report.tick_number;
        staged.validate_links()?;*self=staged;Ok(report)
    }
}

//! Complete bounded grade0 Distributor/Photon callbacks over the shared world.
use super::*;
use crate::{building_power::{self,PowerPlan},source_building_resources::SourceBuildingResourceDatabase,physics_tables::PhysicsTables};
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PowerBuildingReceipt{pub slot:u16,pub numeric:PowerPlan,pub created_effects:Vec<u16>,pub queue_requested:bool}
impl ActorWorld{
 /// Global11A9BF is supplied by the mission runtime. Each ordered transfer is
 /// allocated before commit; exhausted or unsupported pools roll back all
 /// building/player/RNG/map/effect changes. Grade1/2 resource animations are
 /// not admitted until their exact metadata and callbacks are recovered.
 pub fn step_completed_power_building(&mut self,slot:u16,resource:&SourceBuildingResourceDatabase,frame:u32,solar_generation:i16,physics:&PhysicsTables,trig:&TrigTables)->Result<PowerBuildingReceipt,ActorWorldError>{
  self.validate_links()?;self.validate_building_counts()?;self.validate_effect_count()?;
  let pool=self.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?;let before=*pool.record(slot as usize)?;let owner=before[24];
  if owner>=4||before[46]!=0||resource.selector()!=before[45]||pool.binding(slot as usize)?!=Some(crate::building::BuildingAnimationBinding{selector:resource.selector()}){return Err(ActorWorldError::UnsupportedEffectEvent)}
  let start=usize::from(owner)*64;let partition:[building_power::PowerBuilding;64]=pool.records()[start..start+64].try_into().unwrap();let player=*self.player(owner)?.bytes();
  let mut plan=building_power::plan_power_prefix(&partition,slot as usize-start,&player,self.rng,frame,solar_generation,physics).map_err(|_|ActorWorldError::UnsupportedEffectEvent)?;
  let index=slot as usize-start;plan.after[index]=building_power::finish_power_animation(&plan.after[index],resource).map_err(|_|ActorWorldError::UnsupportedEffectEvent)?;
  let mut staged=self.clone();
  for i in 0..64{staged.buildings.as_mut().unwrap().replace_record_checked(start+i,&plan.before[i],&plan.after[i])?}
  staged.players[owner as usize].replace_bookkeeping_checked(&player,&plan.player_after)?;staged.rng=plan.rng_after;
  let mut created_effects=Vec::new();for request in &plan.transfers{created_effects.push(staged.create_power_transfer33(request,trig)?)}
  staged.validate_links()?;staged.validate_building_counts()?;staged.validate_effect_count()?;
  let receipt=PowerBuildingReceipt{slot,numeric:plan,created_effects,queue_requested:true};*self=staged;Ok(receipt)
 }
}

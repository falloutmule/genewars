//! Shared5A210 factory transaction; restoration never substitutes for creation.
use super::*;
use crate::mule_factory::{self,MuleFactoryInput,MuleFactoryPlan};
#[derive(Debug,Clone,PartialEq,Eq)]pub struct NewMuleReceipt{pub slot:u16,pub numeric:MuleFactoryPlan}
impl ActorWorld{
 pub(super) fn create_mule(&mut self,owner:u8,position:[i16;2],quality:u8,parent:Option<&CreatureRecord>,trig:&TrigTables)->Result<Option<NewMuleReceipt>,ActorWorldError>{
  self.validate_links()?;
  let pool=self.creatures.as_ref().ok_or(ActorWorldError::UnsupportedCreatureSchedule)?;
  let declared=word(&self.backing,0x3550);let active=pool.records().iter().filter(|r|r[6]!=0).count();
  if declared<0||declared as usize!=active{return Err(ActorWorldError::CreatureCountMismatch{declared,active});}
  if declared>=300{return Ok(None);}
  let slot=pool.records().iter().position(|r|r[6]==0).ok_or(CreatureError::UnsupportedInitialState)?;
  let before=*pool.record(slot)?;
  let ground=genewars_core::cd_candidate::interpolate_height(self.map(),position[0],position[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
  let shade=genewars_core::cd_candidate::interpolate_entity_shade(self.map(),position[0],position[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
  let plan=mule_factory::plan_mule_factory(&before,MuleFactoryInput{slot:slot as u16,owner,position,quality,ground,shade},parent,self.rng,trig).map_err(|_|CreatureError::UnsupportedInitialState)?;
  let mut staged=self.clone();staged.creatures.as_mut().unwrap().apply_mule_factory_checked(slot,&before,&plan.record_after)?;
  staged.insert_mule_head(slot,floor_cell(position))?;
  staged.backing[0x3550..0x3552].copy_from_slice(&(declared+1).to_le_bytes());
  if owner!=4{
   let before=*staged.player(owner)?.bytes();let mut after=before;
   let count=i32::from_le_bytes(before[0x6f9..0x6fd].try_into().unwrap()).wrapping_add(1);after[0x6f9..0x6fd].copy_from_slice(&count.to_le_bytes());
   staged.players[usize::from(owner)].replace_bookkeeping_checked(&before,&after)?;
  }
  staged.rng=plan.rng_after;staged.validate_links()?;*self=staged;Ok(Some(NewMuleReceipt{slot:slot as u16,numeric:plan}))
 }
}

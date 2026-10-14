//! Atomic power-effect factories and callbacks over the shared world.
//! Producer scheduling and resource-bound building animation remain upstream.
use super::*;
use crate::building_power::{self,PowerTransfer33Request};
use crate::physics_tables::PhysicsTables;
use crate::landing_effect_lifecycle::RetirementUiState;
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PowerStepReceipt{pub effect:EffectStepReceipt,pub credited_building:Option<u16>,pub payload_before:u8,pub payload_after:u8}
impl ActorWorld{
 /// Source48790 free-pool transfer factory. Full-pool replacement remains an
 /// unsupported transaction; it must never silently drop a required packet.
 pub fn create_power_transfer33(&mut self,request:&PowerTransfer33Request,trig:&TrigTables)->Result<u16,ActorWorldError>{
  self.validate_links()?;self.validate_effect_count()?;self.entity(request.destination)?;
  let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;let count=word(&self.backing,0x3554);
  if count==200{return Err(ActorWorldError::UnsupportedEffectEvent)}
  let cell=floor_cell(request.source_position);let raw=reference(self.map(),cell*14+6);let head=if raw[2]==0{None}else{let resolved=self.entity(raw)?;Some(landing::ResolvedCellHead{identity:landing::LandingReference{index:word(resolved,15),kind:resolved[6]}})};
  let(ground,shade)=self.effect_ground_sample(request.source_position)?;
  let plan=building_power::plan_transfer_factory(pool.records(),count,request,ground,shade,head,trig)?;
  let mut staged=self.clone();staged.effects.as_mut().unwrap().commit_landing_factory(&plan)?;staged.insert_effect_head(plan.slot)?;
  staged.backing[0x3554..0x3556].copy_from_slice(&plan.pool_count_after.to_le_bytes());staged.validate_effect_count()?;staged.validate_links()?;*self=staged;Ok(plan.slot)
 }
 /// Source4B630 physics -> animation -> energy delivery/decay -> retirement.
 /// A late unsupported cleanup leaves every pool/player/RNG/map unchanged.
 pub fn step_power_transfer33(&mut self,slot:u16,context:LandingPhysicsContext,physics:&PhysicsTables,ui:&RetirementUiState)->Result<PowerStepReceipt,ActorWorldError>{
  if !self.validate_links()?.contains(&(5,slot as i16)){return Err(ActorWorldError::MissingEffectLink(slot))}self.validate_effect_count()?;
  let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
  if pool.binding(slot as usize)?!=Some(EffectAnimationBinding::EffAnimation33){return Err(ActorWorldError::EffectBinding(slot))}
  let before=*pool.record(slot as usize)?;let at=building_power::transfer_candidate(&before)?;let(ground,shade)=self.effect_ground_sample(at)?;
  let x=(i32::from(at[0])>>8)&127;let y=(i32::from(at[1])>>8)&127;
  let corners=[(0,0),(1,0),(0,1),(1,1)].map(|(dx,dy)|word(self.map(),((((y+dy)&127)*128+((x+dx)&127))*14+2)as usize));
  let moved=building_power::plan_transfer_motion(&before,ground,shade,context.water_height,corners,physics.bounce(self.terrain_class_at(at)?)?,physics)?;
  let changed_cell=floor_cell([word(&before,7),word(&before,9)])!=floor_cell([word(&moved,7),word(&moved,9)]);
  let mut staged=self.clone();if changed_cell{staged.unlink_effect(slot)?}staged.effects.as_mut().unwrap().replace_record_checked(slot as usize,&before,&moved)?;if changed_cell{staged.insert_effect_head(slot)?}
  let moved=*staged.effects.as_ref().unwrap().record(slot as usize)?;let target=if moved[73]==4{Some(*staged.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?.record(word(&moved,71)as usize)?)}else{None};
  let (ground,_)=staged.effect_ground_sample([word(&moved,7),word(&moved,9)])?;
  let plan=building_power::plan_power_delivery(&moved,target.as_ref(),ground).map_err(|_|ActorWorldError::UnsupportedEffectEvent)?;
  let credited=if plan.retire_requested&&plan.target_after!=plan.target_before{Some(word(&moved,71)as u16)}else{None};
  if let(Some(old),Some(new))=(plan.target_before,plan.target_after){staged.buildings.as_mut().unwrap().replace_record_checked(word(&moved,71)as usize,&old,&new)?}
  staged.effects.as_mut().unwrap().replace_record_checked(slot as usize,&moved,&plan.effect_after)?;
  let mut effect=if plan.retire_requested{staged.retire_effect(slot,&plan.effect_after,ui)?}else{EffectStepReceipt{slot,created_effect:None,changed_cell,retired:false,cleaned_reference_bytes:0,ui_after:ui.clone(),pending_ui_events:Vec::new()}};effect.changed_cell=changed_cell;
  staged.validate_links()?;staged.validate_effect_count()?;*self=staged;Ok(PowerStepReceipt{effect,credited_building:credited,payload_before:before[76],payload_after:plan.effect_after[76]})
 }
}

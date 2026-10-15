//! Null-associated failed-production particle factory transaction only.
//! Lifecycle/scheduler admission is separate; full-pool replacement stays blocked.
use super::*;
impl ActorWorld {
 pub fn create_genepod_failure_effect(&mut self,owner:u8,position:[i16;2],height:i32,
  angles:[u16;3],trig:&TrigTables)->Result<u16,ActorWorldError>{
  self.validate_links()?;self.validate_effect_count()?;
  let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
  if word(&self.backing,0x3554)>=200{return Err(ActorWorldError::UnsupportedEffectEvent);}
  let cell=floor_cell(position);let raw=reference(self.map(),cell*14+6);
  let head=if raw[2]==0{None}else{self.entity(raw)?;Some(landing::ResolvedCellHead{identity:landing::LandingReference{index:word(&raw,0),kind:raw[2]}})};
  let(ground,shade)=self.effect_ground_sample(position)?;
  let plan=crate::effect20_factory::plan_factory(pool.records(),word(&self.backing,0x3554),owner,position,ground,shade,height,angles,head,trig.sine().try_into().unwrap(),trig.cosine().try_into().unwrap())?;
  let mut staged=self.clone();staged.effects.as_mut().unwrap().commit_landing_factory(&plan)?;
  let id=[plan.slot as u8,(plan.slot>>8)as u8,5];if raw[2]!=0{staged.replace_previous(raw,id)?;}
  staged.backing[MAP_START+cell*14+6..MAP_START+cell*14+9].copy_from_slice(&id);
  staged.backing[0x3554..0x3556].copy_from_slice(&plan.pool_count_after.to_le_bytes());
  staged.validate_links()?;staged.validate_effect_count()?;*self=staged;Ok(plan.slot)
 }
}

impl ActorWorld{
 pub fn step_genepod_failure_effect(&mut self,slot:u16,context:LandingPhysicsContext,
 physics:&crate::physics_tables::PhysicsTables,ui:&crate::landing_effect_lifecycle::RetirementUiState)
 ->Result<EffectStepReceipt,ActorWorldError>{
  if !self.validate_links()?.contains(&(5,slot as i16)){return Err(ActorWorldError::MissingEffectLink(slot));}
  self.validate_effect_count()?;let pool=self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
  if pool.binding(slot as usize)?!=Some(EffectAnimationBinding::EffAnimation32){return Err(ActorWorldError::EffectBinding(slot));}
  let before=*pool.record(slot as usize)?;let at=crate::effect20_motion::candidate(&before)?;
  let(ground,shade)=self.effect_ground_sample(at)?;
  //4AF40/4AF74: exact code20 virtual8 table byte0 disables collision impulse.
  //74270 can find a building, but this callback changes no numeric state.
  let plan=crate::effect20_motion::plan_airborne(&before,ground,shade,context.water_height,context.gravity,
   true,physics.angles())?;
  let changed_cell=floor_cell([word(&before,7),word(&before,9)])!=floor_cell([word(&plan.after,7),word(&plan.after,9)]);
  let mut staged=self.clone();if changed_cell{staged.unlink_effect(slot)?;}
  staged.effects.as_mut().unwrap().replace_record_checked(slot as usize,&before,&plan.after)?;
  if changed_cell{staged.insert_effect_head(slot)?;}
  let moved=*staged.effects.as_ref().unwrap().record(slot as usize)?;
  let animation=crate::effect20_factory::plan_animation_after_physics(&moved)?;
  staged.effects.as_mut().unwrap().replace_record_checked(slot as usize,&moved,&animation.after)?;
  let mut receipt=EffectStepReceipt{slot,created_effect:None,changed_cell,retired:false,cleaned_reference_bytes:0,ui_after:ui.clone(),pending_ui_events:Vec::new()};
  if animation.retirement_required{let current=*staged.effects.as_ref().unwrap().record(slot as usize)?;receipt=staged.retire_effect(slot,&current,ui)?;receipt.changed_cell=changed_cell;}
  staged.validate_links()?;staged.validate_effect_count()?;*self=staged;Ok(receipt)
 }
}

#[cfg(test)]mod tests{
 use super::*;
 fn export(world:&ActorWorld,root:&std::path::Path,name:&str){
  let dir=root.join(name);std::fs::create_dir(&dir).unwrap();
  for(name,bytes)in[("backing.bin",world.backing.clone()),("map.bin",world.map().to_vec()),
   ("actors.bin",world.actors.records().iter().flatten().copied().collect()),
   ("effects.bin",world.effects.as_ref().unwrap().records().iter().flatten().copied().collect()),
   ("buildings.bin",world.buildings.as_ref().unwrap().records().iter().flatten().copied().collect()),
   ("creatures.bin",world.creatures.as_ref().unwrap().records().iter().flatten().copied().collect()),
   ("players.bin",world.players.iter().flat_map(|p|p.bytes().iter().copied()).collect())]{std::fs::write(dir.join(name),bytes).unwrap();}
  let mut rng=world.rng.seed.to_le_bytes().to_vec();rng.push(world.rng.ring);rng.extend(world.rng.draw_count.to_le_bytes());std::fs::write(dir.join("rng.bin"),rng).unwrap();
 }
 #[test]fn emitted_failure_particles_move_and_retire_transactionally(){
  let base=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime");
  let bytes=std::fs::read(base.join("terrain/CONQ001.DAT")).unwrap();let mission=genewars_assets::ConqMissionSource::parse(&bytes).unwrap();
  let players=core::array::from_fn(|p|{let fresh=PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap();let mut a=*fresh.bytes();a[0x4a3..0x4a7].copy_from_slice(&2000i32.to_le_bytes());a[0x62a..0x62e].copy_from_slice(&(-2048i32).to_le_bytes());PlayerState::from_record(&a).unwrap()});
  let mut world=ActorWorld::from_mission(&mission,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:0}).unwrap();
  world.admit_effects(&[[0;81];200],[None;200]).unwrap();world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
  let slot=world.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;assert!(world.progress_admitted_grade0_building(slot,280,false).unwrap().completed);
  let pool=world.buildings.as_mut().unwrap();let b=*pool.record(slot as usize).unwrap();let mut a=b;a[51]=4;a[54..58].copy_from_slice(&1i32.to_le_bytes());a[58]=0;pool.replace_record_checked(slot as usize,&b,&a).unwrap();
  let trig=TrigTables::generate_pinned().unwrap();let produced=world.step_genepod_terminal(slot,0,&trig).unwrap();assert_eq!(produced.created_effect_slots.len(),7);
  let physics=crate::physics_tables::PhysicsTables::from_mapped_bytes(&std::fs::read(base.join("physics/angle.bin")).unwrap(),&std::fs::read(base.join("physics/bounce.bin")).unwrap()).unwrap();
  let context=LandingPhysicsContext{water_height:i32::from_le_bytes(world.backing[0x1c67..0x1c6b].try_into().unwrap()),gravity:1024};let ui=crate::landing_effect_lifecycle::RetirementUiState::native_empty();let rng=world.rng;
  let capture=std::env::var_os("GENEWARS_EFFECT20_LIFECYCLE_CAPTURE").map(std::path::PathBuf::from);let mut retired=0;
  for frame in 0..9{for &slot in &produced.created_effect_slots{
   if let Some(root)=&capture{export(&world,root,&format!("frame-{frame}-slot-{slot}-before"));}
   let r=world.step_genepod_failure_effect(slot,context,&physics,&ui).unwrap();retired+=usize::from(r.retired);
   if let Some(root)=&capture{export(&world,root,&format!("frame-{frame}-slot-{slot}-after"));}
   assert_eq!(r.retired,frame==8);assert_eq!(world.rng,rng);
  }}assert_eq!(retired,7);world.validate_links().unwrap();world.validate_effect_count().unwrap();
 }
}

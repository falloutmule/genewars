//! Atomic completed Genepod terminal callback. Failed quality remains an
//! atomic effect-factory transaction; unsupported full-pool paths roll back.
use super::*;
use crate::{genepod_idle,genepod_terminal::{self,TerminalPlan,TerminalRequest}};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum GenepodProductionError {
 World(ActorWorldError), Building, Player,
 Terminal(genepod_terminal::TerminalError),
}
impl From<ActorWorldError> for GenepodProductionError {
 fn from(e:ActorWorldError)->Self{Self::World(e)}
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct GenepodProductionReceipt {
 pub slot:u16,pub newborn_slots:Vec<u16>,pub created_effect_slots:Vec<u16>,pub waiting:bool,
 /// Original sound request, not evidence of audio playback.
 pub sound_requested:Option<u8>,pub randomized_sound_requested:Option<(u8,u16)>,pub queue_requested:bool,
}
impl ActorWorld {
 pub fn step_genepod_terminal(&mut self,slot:u16,frame:u32,trig:&TrigTables)
  ->Result<GenepodProductionReceipt,GenepodProductionError>{
  use GenepodProductionError as E;
  self.validate_links()?;self.validate_building_counts()?;
  let pool=self.buildings.as_ref().ok_or(E::Building)?;
  let before=*pool.record(slot as usize).map_err(ActorWorldError::from)?;
  let flags=u16::from_le_bytes([before[49],before[50]]);
  if before[6]!=4||before[24]>=4||before[45]!=0||before[46]!=0||flags&!4!=1||before[51]!=4
   ||i32::from_le_bytes(before[54..58].try_into().unwrap())!=1
   ||pool.binding(slot as usize).map_err(ActorWorldError::from)?!=Some(crate::building::BuildingAnimationBinding{selector:0}){return Err(E::Building);}
  let owner=before[24]as usize;let original_players=core::array::from_fn(|p|*self.players[p].bytes());
  if original_players[owner][1]!=before[24]||original_players[owner][0x4a1]>=4{return Err(E::Player);}
  let(prefix,player)=genepod_idle::shared_prefix(&before,&original_players[owner]);
  let mut players=original_players;players[owner]=player;
  let plan=genepod_terminal::plan_terminal(&prefix,&players,frame,self.rng).map_err(E::Terminal)?;
  let mut staged=self.clone();let mut newborn_slots=Vec::new();let mut created_effect_slots=Vec::new();let mut sound_requested=None;let mut randomized_sound_requested=None;
  let (building,players_after,waiting)=match plan {
   TerminalPlan::Waiting(work)=>(work.building_after,work.players_after,true),
   TerminalPlan::Continue(c)=>{
    for p in 0..4{staged.players[p].replace_bookkeeping_checked(&original_players[p],&c.players_staged[p]).map_err(ActorWorldError::from)?;}
    staged.rng=c.rng_after;
    let after=match c.request {
     TerminalRequest::FailedQuality=>{
      sound_requested=Some(8);let mut cursor=genepod_terminal::begin_failure(&c).map_err(E::Terminal)?;
      randomized_sound_requested=Some((cursor.randomized_sound,cursor.randomized_parameter));staged.rng=cursor.rng_after;
      while cursor.remaining>0{
       let(next,request)=genepod_terminal::next_failure_effect(&cursor,staged.rng).map_err(E::Terminal)?;
       staged.rng=next.rng_after;
       let effect=staged.create_genepod_failure_effect(0,request.position,request.physical_height,[request.yaw,request.pitch,request.speed],trig)?;
       created_effect_slots.push(effect);cursor=next;
      }
      genepod_terminal::finish_failure(&cursor,staged.rng).map_err(E::Terminal)?
     },
     TerminalRequest::Factory{owner,position,quality,sound,..}=>{
      sound_requested=Some(sound);let newborn=staged.create_mule(owner,position,quality,None,trig)?;
      let factory_players=core::array::from_fn(|p|*staged.players[p].bytes());
      genepod_terminal::plan_factory_return(&c,newborn.as_ref().map(|n|n.slot),&factory_players,staged.rng).map_err(E::Terminal)?
     }
    };
    if let Some(e)=after.ejection {
     let pool=staged.creatures.as_mut().ok_or(E::Building)?;
     let prior=*pool.record(e.slot as usize).map_err(ActorWorldError::from)?;let mut record=prior;
     for(offset,value)in[(0x39,e.yaw),(0x3b,e.pitch),(0x3d,e.speed)]{record[offset..offset+2].copy_from_slice(&value.to_le_bytes());}
     let pitch=usize::from(e.pitch.wrapping_sub(512)&2047);let yaw=usize::from(e.yaw.wrapping_sub(512)&2047);
     let horizontal=(i64::from(trig.cosine()[pitch])*i64::from(e.speed)>>16)as i32;
     let velocity=[(i64::from(trig.cosine()[yaw])*i64::from(horizontal)>>16)as i16,
      (i64::from(trig.sine()[yaw])*i64::from(horizontal)>>16)as i16,
      (-i64::from(trig.sine()[pitch])*i64::from(e.speed)>>16)as i16];
     for(offset,value)in[0x2d,0x2f,0x31].into_iter().zip(velocity){record[offset..offset+2].copy_from_slice(&value.to_le_bytes());}
     let height=i32::from_le_bytes(record[0x27..0x2b].try_into().unwrap()).wrapping_add(e.height_delta);
     record[0x27..0x2b].copy_from_slice(&height.to_le_bytes());
     pool.replace_record_checked(e.slot as usize,&prior,&record).map_err(ActorWorldError::from)?;
     newborn_slots.push(e.slot);
    }
    staged.rng=after.rng_after;
    (after.building_staged,after.players_staged,false)
   }
  };
  let mut final_building=genepod_idle::shared_tail(&before,building);
  // Factory insertion can update this building's intrusive previous link.
  let linked_before=*staged.buildings.as_ref().unwrap().record(slot as usize).map_err(ActorWorldError::from)?;
  final_building[..6].copy_from_slice(&linked_before[..6]);
  staged.buildings.as_mut().unwrap().replace_record_checked(slot as usize,&linked_before,&final_building).map_err(ActorWorldError::from)?;
  for p in 0..4{let prior=*staged.players[p].bytes();staged.players[p].replace_bookkeeping_checked(&prior,&players_after[p]).map_err(ActorWorldError::from)?;}
  staged.validate_links()?;staged.validate_building_counts()?;*self=staged;
  Ok(GenepodProductionReceipt{slot,newborn_slots,created_effect_slots,waiting,sound_requested,randomized_sound_requested,queue_requested:true})
 }
}

#[cfg(test)]mod tests{
 use super::*;

 fn export(world:&ActorWorld,name:&str){
  if let Some(root)=std::env::var_os("GENEWARS_GENEPOD_TERMINAL_CAPTURE"){
   let dir=std::path::PathBuf::from(root).join(name);std::fs::create_dir(&dir).unwrap();
   std::fs::write(dir.join("backing.bin"),&world.backing).unwrap();
   std::fs::write(dir.join("map.bin"),world.map()).unwrap();
   let buildings:Vec<u8>=world.buildings.as_ref().unwrap().records().iter().flatten().copied().collect();std::fs::write(dir.join("buildings.bin"),buildings).unwrap();
   let effects:Vec<u8>=world.effects.as_ref().unwrap().records().iter().flatten().copied().collect();std::fs::write(dir.join("effects.bin"),effects).unwrap();
   let creatures:Vec<u8>=world.creatures.as_ref().unwrap().records().iter().flatten().copied().collect();std::fs::write(dir.join("creatures.bin"),creatures).unwrap();
   let players:Vec<u8>=world.players.iter().flat_map(|p|p.bytes().iter().copied()).collect();std::fs::write(dir.join("players.bin"),players).unwrap();
   let mut rng=world.rng.seed.to_le_bytes().to_vec();rng.push(world.rng.ring);rng.extend(world.rng.draw_count.to_le_bytes());std::fs::write(dir.join("rng.bin"),rng).unwrap();
  }
 }
 fn fixture(quality:i32)->(ActorWorld,u16){
  let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime/terrain/CONQ001.DAT");
  let bytes=std::fs::read(path).unwrap();let mission=genewars_assets::ConqMissionSource::parse(&bytes).unwrap();
  let players=core::array::from_fn(|p|{let fresh=PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap();let mut a=*fresh.bytes();a[0x4a3..0x4a7].copy_from_slice(&2000i32.to_le_bytes());a[0x62a..0x62e].copy_from_slice(&(quality<<11).to_le_bytes());PlayerState::from_record(&a).unwrap()});
  let mut world=ActorWorld::from_mission(&mission,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:31,draw_count:0}).unwrap();
  world.admit_effects(&[[0;81];200],[None;200]).unwrap();world.admit_creatures(&[[0;124];300]).unwrap();world.admit_buildings(&[[0;67];256],[None;256]).unwrap();
  let slot=world.create_building(0,0,[19839,12927]).unwrap().unwrap().slot;
  assert!(world.progress_admitted_grade0_building(slot,280,false).unwrap().completed);
  let pool=world.buildings.as_mut().unwrap();let before=*pool.record(slot as usize).unwrap();let mut a=before;
  a[51]=4;a[54..58].copy_from_slice(&1i32.to_le_bytes());a[58]=0;
  pool.replace_record_checked(slot as usize,&before,&a).unwrap();(world,slot)
 }
 #[test]fn terminal_success_owns_factory_and_ejection(){
  let(mut world,slot)=fixture(50);let before=world.clone();let trig=TrigTables::generate_pinned().unwrap();
  export(&world,"success-before");let r=world.step_genepod_terminal(slot,0,&trig).unwrap();export(&world,"success-after");assert!(!r.waiting);let expected_slot=before.creatures().unwrap().records().iter().position(|r|r[6]==0).unwrap();assert_eq!(r.newborn_slots,vec![expected_slot as u16]);assert_eq!(r.sound_requested,Some(0x1a));
  let c=world.creatures().unwrap().record(expected_slot).unwrap();assert_eq!(c[24],0);assert_eq!(c[0x7a],4);
  assert_eq!(i32::from_le_bytes(c[0x27..0x2b].try_into().unwrap()),i32::from_le_bytes(c[0x11..0x15].try_into().unwrap())+3840);
  assert_eq!(i32::from_le_bytes(world.player(0).unwrap().bytes()[0x4a3..0x4a7].try_into().unwrap()),1999);
  assert_eq!(world.rng.draw_count-before.rng.draw_count,7);world.validate_links().unwrap();
 }
 #[test]fn terminal_failure_and_factory_rejection_roll_back_everything(){
  let(mut world,slot)=fixture(-1);let before=world.clone();let trig=TrigTables::generate_pinned().unwrap();
  export(&world,"failure-before");let receipt=world.step_genepod_terminal(slot,0,&trig).unwrap();export(&world,"failure-after");assert!(receipt.newborn_slots.is_empty());assert!((3..=8).contains(&receipt.created_effect_slots.len()));assert_eq!(receipt.sound_requested,Some(8));assert_eq!(world.creatures(),before.creatures());
  let(mut world,slot)=fixture(50);let declared=i16::from_le_bytes(world.backing[0x3550..0x3552].try_into().unwrap());world.backing[0x3550..0x3552].copy_from_slice(&(declared+1).to_le_bytes());let before=world.clone();
  assert!(world.step_genepod_terminal(slot,0,&trig).is_err());assert_eq!(world,before);
 }
}

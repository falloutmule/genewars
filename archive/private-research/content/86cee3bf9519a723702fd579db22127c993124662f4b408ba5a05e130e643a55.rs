//! Bounded72AD0 startup visibility and72960 global entity-shade refresh.
use super::*;
use genewars_core::cd_candidate::interpolate_entity_shade;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BootstrapWorldError { World(ActorWorldError), TerrainObjectInitializationRequired, SpecialEffectCreationRequired, UnsupportedShadeFamily(u8) }
impl From<ActorWorldError> for BootstrapWorldError {fn from(e:ActorWorldError)->Self{Self::World(e)}}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BootstrapVisibilityReceipt {pub reveal:RevealReceipt,pub shade_writes:Vec<[u8;3]>,pub reset_cells:usize}
impl ActorWorld {
 /// Exact ordinary72AD0 branch with absent terrain jobs andspecial effect.
 /// The caller invokes this after80830 landing creation, beforeMAINPANL.
 pub fn initialize_mission_visibility(&mut self,selected_player:u8)->Result<BootstrapVisibilityReceipt,BootstrapWorldError>{
  if self.backing[0x34b8]!=0{return Err(BootstrapWorldError::TerrainObjectInitializationRequired)}
  if self.backing[0x1ec2..0x1ec6]!=[0;4]{return Err(BootstrapWorldError::SpecialEffectCreationRequired)}
  let target=self.player(selected_player)?.target();let mut staged=self.clone();
  for cell in 0..16384{staged.backing[MAP_START+cell*14+12]=0;}
  let reveal=staged.apply_reveal_4096(target)?;let mut shade_writes=Vec::new();
  for kind in [1u8,2,3,4,5,7,8]{
   let layout=staged.layouts[usize::from(kind-1)];
   for slot in 0..layout.count{
    let active=match kind{
     1=>staged.creatures.as_ref().ok_or(ActorWorldError::EnvironmentUnavailable)?.record(slot).map_err(ActorWorldError::from)?.get(6).copied().unwrap(),
     3=>staged.actors.record(slot).map_err(ActorWorldError::from)?[6],
     4=>staged.buildings.as_ref().ok_or(ActorWorldError::BuildingsNotAdmitted)?.record(slot).map_err(ActorWorldError::from)?[6],
     5=>staged.effects.as_ref().ok_or(ActorWorldError::EnvironmentUnavailable)?.record(slot).map_err(ActorWorldError::from)?[6],
     8=>staged.family8.as_ref().ok_or(ActorWorldError::EnvironmentUnavailable)?.record(slot).map_err(ActorWorldError::from)?[6],
     _=>staged.backing[layout.file_offset+slot*layout.packed_stride+6],
    };
    if active==0{continue}
    if kind==7{return Err(BootstrapWorldError::UnsupportedShadeFamily(kind))}
    let raw=[slot as u8,(slot>>8)as u8,kind];let before=staged.entity(raw)?.to_vec();
    let shade=interpolate_entity_shade(staged.map(),word(&before,7),word(&before,9)).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
    let mut after=before.clone();after[0x17]=shade;staged.replace_terrain_entity(raw,&before,&after)?;shade_writes.push(raw);
   }
  }
  staged.validate_links()?;*self=staged;
  Ok(BootstrapVisibilityReceipt{reveal,shade_writes,reset_cells:16384})
 }
}
#[cfg(test)]mod bootstrap_visibility_review{
 use super::*;
 #[test]fn captures(){
  let root=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_ROOT").unwrap());let out=std::path::PathBuf::from(std::env::var("GENEWARS_REVIEW_OUT").unwrap());
  let mission=std::fs::read(root.join("private/ghidra/campaign-resources/152.dat")).unwrap();
  let text=std::fs::read(root.join("private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT")).unwrap();let strings=genewars_assets::TextStringTable::parse(&text).unwrap();let names=BlueprintNames::new(&strings.strings()[..128]).unwrap();
  for case in 0..4{
   let players=core::array::from_fn(|p|PlayerState::fresh_campaign(p as u8,[20736,13568],b"fixture\0").unwrap());
   let mut w=ActorWorld::from_parts(&ConqMissionSource::parse(&mission).unwrap(),&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:5428,ring:0,draw_count:16855}).unwrap();
   w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_family8(&[[0;102];16],[None;16]).unwrap();
   if case&1!=0{
    w.regenerate_player_blueprints(0,[0;36],&names,true).unwrap();let origin=ActorOriginSample{position:[0,0],height:genewars_core::cd_candidate::interpolate_height(w.map(),0,0).unwrap(),shade:interpolate_entity_shade(w.map(),0,0).unwrap()};w.prepare_fresh_player(0,5,origin,[20736,13568]).unwrap();w.create_landing_core(0,0).unwrap();
   }
   if case&2!=0{for cell in 0..16384{w.backing[MAP_START+cell*14+12]=63;}}
   let snap=|w:&ActorWorld|{let mut b=w.backing.clone();for r in w.actors.records(){b.extend(r);}for r in w.creatures.as_ref().unwrap().records(){b.extend(r);}for r in w.buildings.as_ref().unwrap().records(){b.extend(r);}for r in w.effects.as_ref().unwrap().records(){b.extend(r);}for r in w.family8.as_ref().unwrap().records(){b.extend(r);}for p in &w.players{b.extend(p.bytes());}b.extend(w.rng.seed.to_le_bytes());b.push(w.rng.ring);b.extend(w.rng.draw_count.to_le_bytes());b};
   std::fs::write(out.join(format!("case{case}-before.bin")),snap(&w)).unwrap();let receipt=w.initialize_mission_visibility(0).unwrap();assert_eq!(receipt.reset_cells,16384);std::fs::write(out.join(format!("case{case}-after.bin")),snap(&w)).unwrap();let mut events=Vec::new();for p in receipt.reveal.pending_presentation_vertices{for v in p{events.extend(v.to_le_bytes());}}std::fs::write(out.join(format!("case{case}-events.bin")),events).unwrap();
  }
 }
}

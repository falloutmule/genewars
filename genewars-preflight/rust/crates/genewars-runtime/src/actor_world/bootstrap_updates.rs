//! Bounded72AD0 startup visibility and72960 global entity-shade refresh.
use super::*;
use genewars_core::cd_candidate::interpolate_entity_shade;
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BootstrapWorldError { World(ActorWorldError), TerrainObjectInitializationRequired, SpecialEffectCreationRequired, UnsupportedShadeFamily(u8) }
impl From<ActorWorldError> for BootstrapWorldError {fn from(e:ActorWorldError)->Self{Self::World(e)}}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BootstrapVisibilityReceipt {pub reveal:RevealReceipt,pub shade_writes:Vec<[u8;3]>,pub reset_cells:usize,pub zones:ZoneBootstrapReceipt}
impl ActorWorld {
 /// Exact ordinary72AD0 branch with absent terrain jobs andspecial effect.
 /// The caller invokes this after80830 landing creation, beforeMAINPANL.
 pub fn initialize_mission_visibility(&mut self,selected_player:u8)->Result<BootstrapVisibilityReceipt,BootstrapWorldError>{
  if self.backing[0x1ec2..0x1ec6]!=[0;4]{return Err(BootstrapWorldError::SpecialEffectCreationRequired)}
  let target=self.player(selected_player)?.target();let mut staged=self.clone();
  for cell in 0..16384{staged.backing[MAP_START+cell*14+12]=0;}
  let zones=staged.initialize_terrain_zones()?;
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
  Ok(BootstrapVisibilityReceipt{reveal,shade_writes,reset_cells:16384,zones})
 }
}

#[derive(Debug,Clone,PartialEq,Eq,Default)]
pub struct ZoneBootstrapReceipt{pub cell_visits:usize,pub marker_slots:Vec<u16>,pub viewport_invalidations:Vec<[i16;2]>}
impl ActorWorld{
 ///79D50 and ordinary72AD0 zone markers. Each zone sees the staged live map.
 ///This bounded domain admits four zone indices and radius512..4096 only.
 pub fn initialize_terrain_zones(&mut self)->Result<ZoneBootstrapReceipt,BootstrapWorldError>{
  self.update_terrain_zones(true)
 }
 /// Source79273 post-retirement79D50 refresh. Never creates markers.
 pub fn refresh_terrain_zones(&mut self)->Result<ZoneBootstrapReceipt,BootstrapWorldError>{
  self.update_terrain_zones(false)
 }
 fn update_terrain_zones(&mut self,create_markers:bool)->Result<ZoneBootstrapReceipt,BootstrapWorldError>{
  let count=self.backing[0x34b8]as usize;
  if count>4{return Err(BootstrapWorldError::TerrainObjectInitializationRequired);}
  self.validate_links()?;let mut staged=self.clone();let mut receipt=ZoneBootstrapReceipt::default();
  #[cfg(debug_assertions)]if create_markers{staged.emit_zone_fixture("before");}
  let water=word(&staged.backing,0x1c63);
  let environment=crate::construction_maintenance::TerrainEnvironment{climate_center:[word(&staged.backing,0x1ea2),word(&staged.backing,0x1ea4)],raw_water_height:water,climate:word(&staged.backing,0x1c61),light:[181,0,-181]};
  for index in 0..count{
   let offset=0x2ff8+index*19;let zone=&staged.backing[offset..offset+19];
   if i32::from_le_bytes(zone[14..18].try_into().unwrap())<=0{continue;}
   let center=[word(zone,0),word(zone,2)];let radius=i32::from_le_bytes(zone[6..10].try_into().unwrap());
   if !(512..=4096).contains(&radius){return Err(BootstrapWorldError::TerrainObjectInitializationRequired);}
   let radius_word=radius as i16;
   let start=center.map(|v|(v.wrapping_sub(radius_word)as u16>>8)as u8);
   let end=center.map(|v|(v.wrapping_add(radius_word)as u16>>8)as u8);
   let center_height=genewars_core::cd_candidate::interpolate_height(staged.map(),center[0],center[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
   for dy in 0..=end[1].wrapping_sub(start[1]){for dx in 0..=end[0].wrapping_sub(start[0]){
    let point=[((u16::from(start[0].wrapping_add(dx)))<<8)as i16,((u16::from(start[1].wrapping_add(dy)))<<8)as i16];
    let cell=floor_cell(point)as u16;let p=MAP_START+usize::from(cell)*14;
    let height=genewars_core::cd_candidate::interpolate_height(staged.map(),point[0],point[1]).map_err(|e|ActorWorldError::Terrain(e.to_string()))?;
    let z=height.wrapping_sub(center_height)>>3;
    let distance=(genewars_core::cd_candidate::distance_squared_wrapping_15(point,center)as i32).wrapping_add(z.wrapping_mul(z));
    crate::construction_maintenance::refresh_cell_shade(&mut staged.backing[MAP_START..MAP_START+MAP_BYTES],cell,-1,environment).map_err(|e|ActorWorldError::Terrain(format!("zone shade: {e:?}")))?;
    let inner=(radius-512)*(radius-512);
    if distance<inner{
     let shade=word(&staged.backing,p).wrapping_add(0x1100).min(0x3000);staged.backing[p..p+2].copy_from_slice(&shade.to_le_bytes());staged.backing[p+12]=0x3f;
    }else if distance<radius*radius{
     let d=radius-genewars_core::cd_candidate::integer_sqrt(distance as u32)as i32-1;
     let shade=word(&staged.backing,p).wrapping_add((d*0x1100/512)as i16).min(0x3000);staged.backing[p..p+2].copy_from_slice(&shade.to_le_bytes());
     staged.backing[p+12]=staged.backing[p+12].max((d*64/512)as u8);
    }
    staged.refresh_terrain_vertex(point,true)?;receipt.viewport_invalidations.push(point);receipt.cell_visits+=1;
   }}
   if create_markers{if let Some(marker)=staged.create_family8(2,index as u8,center,center_height.wrapping_add(32000))?{receipt.marker_slots.push(marker.slot);}}
  }
  staged.validate_links()?;
  #[cfg(debug_assertions)]if create_markers{staged.emit_zone_fixture("after");}
  *self=staged;Ok(receipt)
 }
 #[cfg(debug_assertions)]fn emit_zone_fixture(&self,label:&str){
  let Some(root)=std::env::var_os("GENEWARS_ZONE_FIXTURE")else{return;};
  let d=std::path::PathBuf::from(root).join(label);std::fs::create_dir_all(&d).unwrap();
  for (name,bytes)in [("backing",self.backing.clone()),("players",self.players.iter().flat_map(|p|p.bytes().iter().copied()).collect()),("actors",self.actors.records().concat()),("creatures",self.creatures.as_ref().unwrap().records().concat()),("buildings",self.buildings.as_ref().unwrap().records().concat()),("effects",self.effects.as_ref().unwrap().records().concat()),("family8",self.family8.as_ref().unwrap().records().concat())]{std::fs::write(d.join(format!("{name}.bin")),bytes).unwrap();}
  let mut rng=self.rng.seed.to_le_bytes().to_vec();rng.push(self.rng.ring);rng.extend(self.rng.draw_count.to_le_bytes());std::fs::write(d.join("rng.bin"),rng).unwrap();
 }
}

#[cfg(test)]mod zone_tests{
 use super::*;
 #[test]fn late_unsupported_zone_rolls_back_map_entities_marker_and_rng(){
  let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime/terrain/CONQ002.DAT");
  let bytes=std::fs::read(&path).unwrap();let source=ConqMissionSource::parse(&bytes).unwrap();
  let players=core::array::from_fn(|i|PlayerState::from_record(&bytes[20+i*1805..20+(i+1)*1805]).unwrap());
  assert_eq!(crate::identity::sha256_hex(&bytes),"4042c6df3b161132c12a424dace7b9fcbf253a197261f1cf76e0db4af41ea27a");
  let mut w=ActorWorld::from_parts(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:47117,ring:17,draw_count:33620}).unwrap();
  w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_family8(&[[0;102];16],[None;16]).unwrap();
  let zone=w.backing[0x2ff8..0x2ff8+19].to_vec();w.backing[0x2ff8+19..0x2ff8+38].copy_from_slice(&zone);w.backing[0x34b8]=2;
  w.backing[0x2ff8+19+6..0x2ff8+19+10].copy_from_slice(&1i32.to_le_bytes());
  let before=w.clone();assert_eq!(w.initialize_terrain_zones().unwrap_err(),BootstrapWorldError::TerrainObjectInitializationRequired);assert_eq!(w,before);
 }

 #[test]fn occupied_marker_zone_refresh_source_fixture(){
  let path=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../private/integration/game-runtime/terrain/CONQ002.DAT");
  let bytes=std::fs::read(&path).unwrap();let source=ConqMissionSource::parse(&bytes).unwrap();
  let players=core::array::from_fn(|i|PlayerState::from_record(&bytes[20+i*1805..20+(i+1)*1805]).unwrap());
  assert_eq!(crate::identity::sha256_hex(&bytes),"4042c6df3b161132c12a424dace7b9fcbf253a197261f1cf76e0db4af41ea27a");
  let mut w=ActorWorld::from_parts(&source,&[[0;200];20],[ActorHandleBindings::empty();20],players,BlueprintRng{seed:47117,ring:17,draw_count:33620}).unwrap();
  w.admit_creatures(&[[0;124];300]).unwrap();w.admit_buildings(&[[0;67];256],[None;256]).unwrap();w.admit_effects(&[[0;81];200],[None;200]).unwrap();w.admit_family8(&[[0;102];16],[None;16]).unwrap();

  w.initialize_terrain_zones().unwrap();
  let before=*w.family8.as_ref().unwrap().record(0).unwrap();let mut marker=before;marker[0x56]=12;
  w.family8.as_mut().unwrap().replace_record_checked(0,&before,&marker).unwrap();
  w.emit_zone_fixture("before");let saved=w.clone();let receipt=w.refresh_terrain_zones().unwrap();w.emit_zone_fixture("after");
  assert!(receipt.marker_slots.is_empty());assert!(receipt.cell_visits>0);assert_eq!(w.rng,saved.rng);
  assert_eq!(w.family8.as_ref().unwrap().record(0).unwrap()[0x56],12);
  let mut records=[[0u8;176];25];for record in &mut records{record[12..14].copy_from_slice(&(-1i16).to_le_bytes());}
  let job=&mut records[0];for(at,value)in[(0,19712i16),(2,12800),(4,19968),(6,13056),(10,1),(12,1)]{job[at..at+2].copy_from_slice(&value.to_le_bytes());}job[8]=2;job[9]=2;
  for row in 0..2{for col in 0..2{let cell=(50+row)*128+77+col;job[14+row*18+col*2..16+row*18+col*2].copy_from_slice(&w.map()[cell*14+2..cell*14+4]);}}
  let mut jobs=crate::terrain_job_pool::TerrainJobPool::admit(records,1).unwrap();w.backing[0x2ff7]=1;
  let environment=crate::construction_maintenance::TerrainEnvironment{climate_center:[word(&w.backing,0x1ea2),word(&w.backing,0x1ea4)],raw_water_height:word(&w.backing,0x1c63),climate:word(&w.backing,0x1c61),light:[181,0,-181]};
  let patterns=std::fs::read(path.parent().unwrap().parent().unwrap().join("construction/terrain-patterns.bin")).unwrap();let table=crate::terrain_border_table::build(&patterns).unwrap();
  let valid=w.clone();let valid_jobs=jobs.clone();
  let zone=w.backing[0x2ff8..0x2ff8+19].to_vec();w.backing[0x2ff8+19..0x2ff8+38].copy_from_slice(&zone);w.backing[0x34b8]=2;
  w.backing[0x2ff8+19+6..0x2ff8+19+10].copy_from_slice(&1i32.to_le_bytes());
  let before=w.clone();assert_eq!(w.refresh_terrain_zones().unwrap_err(),BootstrapWorldError::TerrainObjectInitializationRequired);assert_eq!(w,before);
  let before_jobs=jobs.clone();assert!(w.step_terrain_jobs(&mut jobs,environment,&table).is_err());assert_eq!(w,before);assert_eq!(jobs,before_jobs);
  w=valid;jobs=valid_jobs;w.emit_zone_fixture("job-before");
  if let Some(root)=std::env::var_os("GENEWARS_ZONE_FIXTURE"){std::fs::write(std::path::PathBuf::from(root).join("job-before/jobs.bin"),jobs.records().concat()).unwrap();}
  let result=w.step_terrain_jobs(&mut jobs,environment,&table).unwrap();assert!(result[0].retired);assert_eq!(jobs.count(),0);assert_eq!(w.family8.as_ref().unwrap().record(0).unwrap()[0x56],12);
  w.emit_zone_fixture("job-after");if let Some(root)=std::env::var_os("GENEWARS_ZONE_FIXTURE"){std::fs::write(std::path::PathBuf::from(root).join("job-after/jobs.bin"),jobs.records().concat()).unwrap();}

 }
}

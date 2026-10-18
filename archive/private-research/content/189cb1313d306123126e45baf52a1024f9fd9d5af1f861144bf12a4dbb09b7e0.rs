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

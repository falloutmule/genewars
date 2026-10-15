//! Source action5 single-Mule producer followed atomically by opcode11.
//! Caller supplies admitted action5, resolved selection, projection and native
//! manager priors. Hardware hit testing/mouse mapping remain outside this API.
use super::*;
use crate::creature_world::CreatureAnimationBinding;
use crate::{mule_human_producer as producer,mule_owned_command as command};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum OwnedMuleInputError { World(ActorWorldError), Producer(producer::ProducerError), Selection }
impl From<ActorWorldError> for OwnedMuleInputError {fn from(e:ActorWorldError)->Self{Self::World(e)}}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct OwnedMuleInputReceipt {pub producer:producer::ProducerPlan,pub command:Option<command::CommandPlan>}
impl ActorWorld {
 pub fn dispatch_owned_mule_action5(&mut self,selected:Option<u16>,packet:producer::CommandPacket,manager:&[u8;0x59d],input:producer::ProducerInput,selected_panel_refresh:bool)->Result<OwnedMuleInputReceipt,OwnedMuleInputError> {
  self.validate_links()?;
  if self.player(input.selected_player_index)?.bytes()[1]!=input.selected_player_owner{return Err(OwnedMuleInputError::Selection);}
  let record=if let Some(slot)=selected {
   let pool=self.creatures.as_ref().ok_or(OwnedMuleInputError::Selection)?;
   let r=*pool.record(usize::from(slot)).map_err(ActorWorldError::from)?;
   if word(&r,15)!=slot as i16||!matches!(pool.binding(usize::from(slot)).map_err(ActorWorldError::from)?,Some(CreatureAnimationBinding::MuleSelector4|CreatureAnimationBinding::BonesSelector4)){return Err(OwnedMuleInputError::Selection);}
   Some(r)
  }else{None};
  let plan=producer::plan_action5(record.as_ref(),packet,manager,input).map_err(OwnedMuleInputError::Producer)?;
  if plan.outcome!=producer::ProducerOutcome::Issued{return Ok(OwnedMuleInputReceipt{producer:plan,command:None});}
  let slot=selected.ok_or(OwnedMuleInputError::Selection)?;let r=record.unwrap();
  if plan.packet_after.opcode!=0x11||plan.packet_after.words[0]!=slot as i16{return Err(OwnedMuleInputError::Selection);}
  let mut staged=self.clone();
  let command=staged.dispatch_owned_mule_move(slot,[plan.packet_after.words[1],plan.packet_after.words[2]],command::CommandContext{caller_owner:input.selected_player_owner,virtual_dead:r[94]>=24,caller_is_selected:true,selected_panel_refresh})?;
  staged.validate_links()?;*self=staged;Ok(OwnedMuleInputReceipt{producer:plan,command:Some(command)})
 }
}

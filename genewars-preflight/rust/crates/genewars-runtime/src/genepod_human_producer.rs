//!21E20 action8 human producer, bounded to original Mule species4.
//! Caller supplies original captured opaque hit (GENEPANL sprite1,29x29),
//! manager-relative coordinates and actual19FF70 event flags; this is not a
//! rectangle shortcut or input injection. No production work is mutated here.
use crate::genepod_command::GenepodRequest;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ProducerInput{
 pub selected_player_index:u8,pub panel_state:u8,pub species_list_index:i8,
 pub mouse_flags:u16,pub captured_opaque_hit:bool,
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum ProducerError{Building,Player,ListIndex,Species}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum IgnoreReason{NoCapturedHit,Disabled,PanelNotReady,SuppressedInput,NoActivationEdge,BuildingFlags}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub enum ProducerOutcome{
 Ignored(IgnoreReason),Denied{sound:u8},Issued{request:GenepodRequest,sound:Option<u8>},
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct ProducerPlan{pub panel_state_after:u8,pub outcome:ProducerOutcome}
pub fn plan_mule_producer(building:&[u8;67],player:&[u8;1805],input:ProducerInput)->Result<ProducerPlan,ProducerError>{
 let word=|at|i16::from_le_bytes([building[at],building[at+1]]);
 let d=|a:&[u8],at|i32::from_le_bytes(a[at..at+4].try_into().unwrap());
 if building[6]!=4||building[24]>=4||building[45]!=0||building[46]>2||!(0..256).contains(&word(15)){return Err(ProducerError::Building);}
 if input.selected_player_index>=4||player[1]>=4||player[2]!=0{return Err(ProducerError::Player);}
 let plan=|state,outcome|Ok(ProducerPlan{panel_state_after:state,outcome});
 let ignore=|state,reason|plan(state,ProducerOutcome::Ignored(reason));
 if !input.captured_opaque_hit{return ignore(input.panel_state,IgnoreReason::NoCapturedHit);}
 let flags=word(49)as u16;
 //21E96..21F0A panel disabled flag: allied admission alone does not enable it.
 if flags&1==0||flags&0x40!=0||building[24]!=player[1]{return ignore(input.panel_state,IgnoreReason::Disabled);}
 let start=building[27]==0&&d(building,54)<=0;
 let admitted=flags&0x141==1;
 if start{
  if input.panel_state<2{return ignore(input.panel_state,IgnoreReason::PanelNotReady);}
  if input.mouse_flags&0x8001!=0{return ignore(3,IgnoreReason::SuppressedInput);}
  if input.mouse_flags&8==0{return ignore(2,IgnoreReason::NoActivationEdge);}
  if !admitted{return ignore(input.panel_state,IgnoreReason::BuildingFlags);}
  let index=usize::try_from(input.species_list_index).map_err(|_|ProducerError::ListIndex)?;
  if index>=usize::from(player[0x6e0])||index>=50{return Err(ProducerError::ListIndex);}
  let species=i16::from(player[0x6ae+index]as i8);if species!=4{return Err(ProducerError::Species);}
  //Mule passes grade0 species<5; no capacity predicate exists in this producer.
  if d(player,0x4a3).wrapping_add(d(player,0x4a7))<75||d(player,0x61a+4*4)==0{return plan(0,ProducerOutcome::Denied{sound:0x26});}
  plan(0,ProducerOutcome::Issued{request:GenepodRequest{species,building_slot:word(15),start:true},sound:Some(0x2c)})
 }else{
  if !admitted||d(building,54)<=0{return plan(input.panel_state,ProducerOutcome::Denied{sound:0x1b});}
  if input.mouse_flags&0x8001!=0{return ignore(0,IgnoreReason::SuppressedInput);}
  if input.mouse_flags&8==0{return ignore(1,IgnoreReason::NoActivationEdge);}
  if input.selected_player_index!=building[58]{return plan(0,ProducerOutcome::Denied{sound:0x26});}
  if building[51]!=4{return Err(ProducerError::Species);}
  plan(0,ProducerOutcome::Issued{request:GenepodRequest{species:4,building_slot:word(15),start:false},sound:None})
 }
}

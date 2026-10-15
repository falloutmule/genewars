//! Bounded source49D30/724C0 cleanup and4E950 scheduling protocol.
//! UI references are typed explicit input. Map unlink remains a host transaction.
use crate::landing::{LandingRecord,LandingReference,FAMILY4_CAPACITY};
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum LifecycleError{InvalidDeadRecord,InvalidPoolCount,InvalidPoolShape,RecordIdentity,UnknownSelector(u8),UnaccountedNewborns{remaining:u32},UnsupportedRetirementCode(u8)}
#[derive(Debug,Clone,Copy)]
pub struct CleanupRecord<'a>{pub kind:u8,pub slot:u16,pub bytes:&'a[u8]}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct CleanupBytePatch{pub kind:u8,pub slot:u16,pub offset:u16,pub before:u8,pub after:u8}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum RetirementPanelReference{Panel193(Option<LandingReference>),Panel1C4(Option<LandingReference>),Other}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct RetirementUiState{
 pub selection_active:bool,pub selected:Option<LandingReference>,
 pub pointer_slots:[Option<LandingReference>;9],pub panel:RetirementPanelReference,
}
impl RetirementUiState{
 /// Explicit native host policy: none of the source GUI pointer slots exist.
 pub fn native_empty()->Self{Self{selection_active:false,selected:None,pointer_slots:[None;9],panel:RetirementPanelReference::Other}}
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum RetirementUiEvent{ClearSelection25150,ClearPointerSlot409F0(u8),ClearPanelReference2C200(u16)}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct EffectRetirementPlan{
 pub dead:LandingReference,pub before:LandingRecord,pub after:LandingRecord,
 pub pool_count_after:i16,pub cleanup:Vec<CleanupBytePatch>,
 pub ui_after:RetirementUiState,pub ui_events:Vec<RetirementUiEvent>,
 /// Must unlink the old map reference before applying the byte patches/type0.
 pub map_unlink_required:bool,
}
/// Source724C0 scans these native pools in order. Supply every record,
/// including inactive ones, so an omitted pool cannot hide a reference.
const SHAPES:[(u8,usize,usize);5]=[(1,300,124),(3,20,200),(5,200,81),(7,32,97),(8,16,102)];
fn reference(b:&[u8],o:usize)->LandingReference{LandingReference{index:i16::from_le_bytes([b[o],b[o+1]]),kind:b[o+2]}}
/// Native offsets inspected by724C0; packed generic offsets>=39 map to-8.
pub fn cleanup_reference_offsets(kind:u8)->&'static[usize]{match kind{1=>&[0x4d,0x50,0x55,0x58,0x5b],3=>&[0x4d,0x50,0x7f,0x7b],5=>&[0x47],7=>&[0x4d,0x50,0x57,0x5a],8=>&[0x59],_=>&[]}}
/// Plan one native record's reference cleanup, preserving every index word.
/// The host must call this for all five source pools, including inactive slots.
pub fn plan_reference_cleanup(kind:u8,slot:u16,bytes:&[u8],dead:LandingReference)->Result<Vec<CleanupBytePatch>,LifecycleError>{
 let(_,count,width)=SHAPES.iter().copied().find(|s|s.0==kind).ok_or(LifecycleError::InvalidPoolShape)?;
 if usize::from(slot)>=count||bytes.len()!=width{return Err(LifecycleError::InvalidPoolShape);}
 if bytes[6]==0{return Ok(Vec::new());}
 if bytes[6]!=kind||i16::from_le_bytes([bytes[15],bytes[16]])!=slot as i16{return Err(LifecycleError::RecordIdentity);}
 let mut patches=Vec::new();for &o in cleanup_reference_offsets(kind){if reference(bytes,o)==dead{patches.push(CleanupBytePatch{kind,slot,offset:(o+2) as u16,before:bytes[o+2],after:0});if kind==1&&o==0x5b&&bytes[0x60]&2!=0{patches.push(CleanupBytePatch{kind,slot,offset:0x60,before:bytes[0x60],after:bytes[0x60]&!2});}}}Ok(patches)
}
/// Retire boundedcode6/code2F/fresh4C0code31. Child selectorsFF prove no
/// factory/RNG at retirement. Code31playerreference/counter effects must
/// already be committed from RetireLanding before this common cleanup.
/// Index words and unmentioned bytes remain unchanged, including stale links.
pub fn plan_effect6_retirement(
 dead:&LandingRecord,pool_count:i16,records:&[CleanupRecord<'_>],ui:&RetirementUiState,
)->Result<EffectRetirementPlan,LifecycleError>{
 if dead[6]!=5{return Err(LifecycleError::InvalidDeadRecord);}
 // Code15's child request must already be handled by the caller, before unlink.
 if !matches!(dead[69],6|0x15|0x16|0x1a|0x20|0x21|0x2c|0x2f|0x30|0x31)||dead[69]==0x31&&dead[76]!=0{return Err(LifecycleError::UnsupportedRetirementCode(dead[69]));}
 let identity=LandingReference{index:i16::from_le_bytes([dead[15],dead[16]]),kind:5};
 if identity.index<0||identity.index>=200{return Err(LifecycleError::RecordIdentity);}
 if !(1..=200).contains(&pool_count){return Err(LifecycleError::InvalidPoolCount);}
 if records.len()!=568{return Err(LifecycleError::InvalidPoolShape);}
 let mut cleanup=Vec::new();let mut cursor=0;
 for(kind,count,width)in SHAPES{for slot in 0..count{
  let r=records[cursor];cursor+=1;
  if r.kind!=kind||usize::from(r.slot)!=slot||r.bytes.len()!=width{return Err(LifecycleError::InvalidPoolShape);}
  if r.bytes[6]==0{continue;}
  if reference(r.bytes,15).index!=slot as i16||r.bytes[6]!=kind{return Err(LifecycleError::RecordIdentity);}
  let offsets:&[usize]=match kind{1=>&[0x4d,0x50,0x55,0x58,0x5b],3=>&[0x4d,0x50,0x7f,0x7b],5=>&[0x47],7=>&[0x4d,0x50,0x57,0x5a],8=>&[0x59],_=>unreachable!()};
  for &o in offsets{if reference(r.bytes,o)==identity{
   cleanup.push(CleanupBytePatch{kind,slot:slot as u16,offset:(o+2) as u16,before:r.bytes[o+2],after:0});
   if kind==1&&o==0x5b&&r.bytes[0x60]&2!=0{cleanup.push(CleanupBytePatch{kind,slot:slot as u16,offset:0x60,before:r.bytes[0x60],after:r.bytes[0x60]&!2});}
  }}
 }}
 let mut ui_after=ui.clone();let mut ui_events=Vec::new();
 if ui.selection_active&&ui.selected==Some(identity){ui_events.push(RetirementUiEvent::ClearSelection25150);}
 for(i,r)in ui_after.pointer_slots.iter_mut().enumerate(){if *r==Some(identity){*r=None;ui_events.push(RetirementUiEvent::ClearPointerSlot409F0(i as u8));}}
 match &mut ui_after.panel{
  RetirementPanelReference::Panel193(r)if *r==Some(identity)=>{*r=None;ui_events.push(RetirementUiEvent::ClearPanelReference2C200(0x19f));},
  RetirementPanelReference::Panel1C4(r)if *r==Some(identity)=>{*r=None;ui_events.push(RetirementUiEvent::ClearPanelReference2C200(0x1cc));},_=>{}
 }
 let mut after=*dead;after[6]=0;
 // A kind5 self-child reference is compared by724C0 before clearing type.
 for p in &cleanup{if p.kind==5&&p.slot==identity.index as u16{after[p.offset as usize]=p.after;}}
 Ok(EffectRetirementPlan{dead:identity,before:*dead,after,pool_count_after:pool_count-1,cleanup,ui_after,ui_events,map_unlink_required:true})
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family4PrimaryPlan{pub slots:Vec<u16>}
/// Initial scan is bounded200, ascending, active and newborn bit clear.
/// Host rechecks each slot immediately before its callback: prior callbacks
/// may retire/create records, so this snapshot is an admission protocol only.
pub fn family4_primary_plan(records:&[LandingRecord;FAMILY4_CAPACITY])->Family4PrimaryPlan{
 Family4PrimaryPlan{slots:records.iter().enumerate().filter(|(_,r)|r[6]!=0&&r[21]&1==0).map(|(i,_)|i as u16).collect()}
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family4NewbornProtocol{pub next_slot:u16,pub remaining:u32}
impl Family4NewbornProtocol{
 pub fn new(primary_created_count:u32)->Self{Self{next_slot:0,remaining:primary_created_count}}
 /// Call against the LIVE pool after each callback. Callback return values
 /// are ignored in this pass, matching4E9C6. Returned record clears bit1
 /// before callback; host must commit it transactionally with that callback.
 pub fn next(&mut self,records:&[LandingRecord;FAMILY4_CAPACITY])->Result<Option<(u16,LandingRecord)>,LifecycleError>{
  if self.remaining==0{return Ok(None);}
  while usize::from(self.next_slot)<FAMILY4_CAPACITY{let slot=self.next_slot;self.next_slot+=1;let r=records[usize::from(slot)];if r[6]!=0&&r[21]&1!=0{let mut after=r;after[21]&=!1;self.remaining-=1;return Ok(Some((slot,after)));}}
  Err(LifecycleError::UnaccountedNewborns{remaining:self.remaining})
 }
}
/// Post-update queue order: ascending active slots, excluding selector46.
/// Actual83140 projection/dirty flag admission belongs to the render adapter.
pub fn family4_queue_slots(records:&[LandingRecord;FAMILY4_CAPACITY],pool_count:i16)->Result<Vec<u16>,LifecycleError>{
 if pool_count<0||pool_count>200||records.iter().filter(|r|r[6]!=0).count()!=pool_count as usize{return Err(LifecycleError::InvalidPoolCount);}
 Ok(records.iter().enumerate().filter(|(_,r)|r[6]!=0&&r[69]!=0x46).map(|(i,_)|i as u16).collect())
}
#[cfg(test)]mod tests{
 use super::*;
 fn records()->Vec<(u8,u16,Vec<u8>)>{let mut result=Vec::new();for(kind,count,width)in SHAPES{for slot in 0..count{result.push((kind,slot as u16,vec![0;width]));}}result}
 #[test]fn cleanup_clears_matching_kinds_preserves_ids_and_handles_flags(){
  let mut records=records();for(kind,slot,b)in &mut records{b[6]=*kind;b[15..17].copy_from_slice(&slot.to_le_bytes());}
  let mut dead=[0;81];dead[6]=5;dead[69]=6;dead[15..17].copy_from_slice(&7i16.to_le_bytes());
  for o in[0x4d,0x50,0x55,0x58,0x5b]{records[0].2[o..o+3].copy_from_slice(&[7,0,5]);}records[0].2[0x60]=0xa7;
  // Same index, other kind must survive; inactive record must remain opaque.
  records[1].2[0x4d..0x50].copy_from_slice(&[7,0,3]);records[2].2[6]=0;records[2].2[0x4d..0x50].copy_from_slice(&[7,0,5]);
  let refs=records.iter().map(|(kind,slot,b)|CleanupRecord{kind:*kind,slot:*slot,bytes:b}).collect::<Vec<_>>();
  let p=plan_effect6_retirement(&dead,2,&refs,&RetirementUiState::native_empty()).unwrap();
  assert_eq!(p.cleanup.len(),6);assert_eq!(p.pool_count_after,1);assert_eq!(p.after[6],0);assert_eq!(&p.after[15..17],&[7,0]);assert!(p.ui_events.is_empty());
  assert!(p.cleanup.iter().all(|p|p.slot==0&&p.kind==1));assert!(p.cleanup.iter().any(|p|p.offset==0x60&&p.after==0xa5));
 }
 #[test]fn newborn_pass_rechecks_live_pool_and_ignores_secondary_creation_count(){
  let mut pool=[[0;81];200];pool[0][6]=5;pool[0][21]=0x20;pool[3][6]=5;pool[3][21]=0x21;
  assert_eq!(family4_primary_plan(&pool).slots,vec![0]);let mut pass=Family4NewbornProtocol::new(2);
  let(slot,after)=pass.next(&pool).unwrap().unwrap();assert_eq!(slot,3);assert_eq!(after[21],0x20);pool[3]=after;
  pool[8][6]=5;pool[8][21]=1;assert_eq!(pass.next(&pool).unwrap().unwrap().0,8);assert_eq!(pass.next(&pool).unwrap(),None);
  let mut invalid=Family4NewbornProtocol::new(2);assert!(invalid.next(&[[0;81];200]).is_err());
 }
 #[test]fn ui_reference_cleanup_is_typed_and_selection_is_an_explicit_request(){
  let mut r=records();let refs=r.iter_mut().map(|(kind,slot,b)|CleanupRecord{kind:*kind,slot:*slot,bytes:b}).collect::<Vec<_>>();let mut dead=[0;81];dead[6]=5;dead[69]=6;
  let identity=LandingReference{index:0,kind:5};let mut ui=RetirementUiState::native_empty();ui.selection_active=true;ui.selected=Some(identity);ui.pointer_slots[8]=Some(identity);ui.panel=RetirementPanelReference::Panel193(Some(identity));
  let p=plan_effect6_retirement(&dead,1,&refs,&ui).unwrap();assert_eq!(p.ui_after.selected,Some(identity));assert_eq!(p.ui_after.pointer_slots[8],None);assert_eq!(p.ui_events.len(),3);
 }
}

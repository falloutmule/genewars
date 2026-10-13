#[path = "../../rust/crates/genewars-runtime/src/identity.rs"] mod identity;
#[path = "../../rust/crates/genewars-runtime/src/actor_activation.rs"] mod activation;
use activation::*;
use std::{fs, io::{BufWriter,Write},path::Path};
fn word(out:&mut impl Write,x:u16){out.write_all(&x.to_le_bytes()).unwrap();}
fn main(){
 let dir=std::env::args().nth(1).unwrap();let p=Path::new(&dir);
 let tables=TrigTables::from_mapped_bytes(&fs::read(p.join("sin.bin")).unwrap(),&fs::read(p.join("cos.bin")).unwrap()).unwrap();
 assert_eq!(TrigTables::generate_pinned().unwrap(),tables);
 let mut out=BufWriter::new(fs::File::create(p.join("actual.bin")).unwrap());
 for seed in 0..=u16::MAX {for center in [[0,0],[32767,32767],[20736,13568],[16383,16384]] {
  let r=sample_placement_candidate(seed,center,&tables).unwrap();for x in [r.seed_before,r.seed_after,r.angle_index,r.position_u15[0],r.position_u15[1]] {word(&mut out,x);}
 }}
 let mut map=vec![0;128*128*14];
 for line in fs::read_to_string(p.join("cases.txt")).unwrap().lines(){
  let v:Vec<i64>=line.split_whitespace().map(|s|s.parse().unwrap()).collect();
  let id=v[0]as i16;let slot=v[1]as usize;let n=v[2]as usize;let seed=v[3]as u16;let center=[v[4]as u16,v[5]as u16];let flags=v[6]as u16;
  let mut actor:Vec<u8>=(0..n).map(|i|((i*37)as i64+v[7])as u8).collect();
  for (i,b) in [(6,v[8]as u8),(0x53,v[9]as u8),(0x74,v[10]as u8)]{if i<n {actor[i]=b;}}
  if n>=0x72{actor[0x70..0x72].copy_from_slice(&flags.to_le_bytes());}if n>=0x11{actor[15..17].copy_from_slice(&(v[11]as i16).to_le_bytes());}
  let mut bp:Vec<u8>=(0..v[12]as usize).map(|i|((i*53)as i64+v[7]+11)as u8).collect();
  let sel=(v[10]as u8)as usize;
  if sel<32 && bp.len()>sel*36+32{let a=sel*36;bp[a+0x16]=v[13]as u8;bp[a+0x18]=v[14]as u8;bp[a+0x1c..a+0x20].copy_from_slice(&(v[15]as u32).to_le_bytes());}
  let mut envs:Vec<CandidateEnvironment>=match v[18]{0=>vec![],1=>vec![CandidateEnvironment{terrain_class:v[19]as u8,blocker_weight:v[20]as i32,resolved_cell_head:None}],2=>vec![CandidateEnvironment{terrain_class:v[19]as u8,blocker_weight:10,resolved_cell_head:None},CandidateEnvironment{terrain_class:v[19]as u8,blocker_weight:v[20]as i32,resolved_cell_head:None}],3=>vec![CandidateEnvironment{terrain_class:0,blocker_weight:10,resolved_cell_head:None};4097],4=>vec![CandidateEnvironment{terrain_class:0,blocker_weight:10,resolved_cell_head:None};4096],_=>vec![CandidateEnvironment{terrain_class:0,blocker_weight:10,resolved_cell_head:None},CandidateEnvironment{terrain_class:1,blocker_weight:9,resolved_cell_head:None},CandidateEnvironment{terrain_class:255,blocker_weight:-1,resolved_cell_head:None}]};
  for e in &mut envs { e.resolved_cell_head = if v[23]!=1 && (v[17]!=0 || v[23]==4) {Some(ResolvedCellHead{record_id:(v[16]as i16).wrapping_add(if v[23]==2 {1}else{0}),record_type:(v[17]as u8).wrapping_add(if v[23]==3 {1}else{0})})}else{None}; }
  let before_actor=actor.clone();let before_bp=bp.clone();
  map.fill(0);let mut target_cell=None;
  if center.iter().all(|&x|x<32768) && !envs.is_empty(){let mut next=seed;for e in &envs{let c=sample_placement_candidate(next,center,&tables).unwrap();next=c.seed_after;if e.terrain_class<8 && e.blocker_weight>=0 && v[9]as u8&(1<<e.terrain_class)==0 && e.blocker_weight<10 {target_cell=Some((c.position_u15[1]as usize/256*128+c.position_u15[0]as usize/256)*14);break;}}}
  if let Some(cell)=target_cell {map[cell+6..cell+8].copy_from_slice(&(v[16]as i16).to_le_bytes());map[cell+8]=v[17]as u8;}
  let map_slice=&map[..v[21]as usize];
  let result=plan_actor_activation(id,slot,&actor,&bp,map_slice,center,seed,&tables,&envs,v[22]!=0);
  assert_eq!(actor,before_actor);assert_eq!(bp,before_bp);
  match result {
   Ok(ActivationPlanningOutcome::Ineligible{actor_id,slot_index})=>{out.write_all(&[1]).unwrap();assert_eq!(actor_id,id);assert_eq!(slot_index,slot);}
   Ok(ActivationPlanningOutcome::Planned(r))=>{
    out.write_all(&[0]).unwrap();word(&mut out,r.seed_after);word(&mut out,r.candidate_attempts.len()as u16);word(&mut out,r.blueprint_index as u16);word(&mut out,r.link.cell_index);
    out.write_all(&r.actor_before).unwrap();out.write_all(&r.actor_after).unwrap();out.write_all(&r.blueprint_before).unwrap();out.write_all(&r.blueprint_after).unwrap();
    for x in [r.link.expected_old_head,r.link.actor_previous_after,r.link.actor_next_after,r.link.new_cell_head]{out.write_all(&x).unwrap();}
    out.write_all(&[r.link.old_head_previous_after.is_some()as u8]).unwrap();out.write_all(&r.link.old_head_previous_after.unwrap_or([0;3])).unwrap();
    for a in r.candidate_attempts{word(&mut out,a.candidate.seed_before);word(&mut out,a.candidate.seed_after);word(&mut out,a.candidate.angle_index);for x in a.candidate.position_u15{word(&mut out,x);}out.write_all(&[a.accepted as u8]).unwrap();}
   }
   Err(e)=>{let code=match e {ActorActivationError::InvalidActorId(_)=>2,ActorActivationError::InvalidSlotIndex(_)=>3,ActorActivationError::ActorRecordLength{..}=>4,ActorActivationError::UnsupportedActorType(_)=>5,ActorActivationError::ActorSlotMismatch{..}=>6,ActorActivationError::ActorIdMismatch{..}=>7,ActorActivationError::AssociatedType5RouteUnsupported=>8,ActorActivationError::BlueprintTableLength{..}=>9,ActorActivationError::MapLength{..}=>10,ActorActivationError::InvalidCenter(_)=>11,ActorActivationError::InvalidBlueprintIndex(_)=>12,ActorActivationError::InvalidTerrainClass(_)=>13,ActorActivationError::NegativeBlockerWeight(_)=>14,ActorActivationError::CandidateListExhausted{..}=>15,ActorActivationError::InvalidReference{..}=>16,ActorActivationError::UnexpectedResolvedCellHead=>17,ActorActivationError::MissingResolvedCellHead{..}=>18,ActorActivationError::ResolvedCellHeadMismatch{..}=>19,_=>panic!("unexpected {e:?}")};out.write_all(&[code]).unwrap();}
  }
 }
 // Table and coordinate admission are checked independently of planner gates.
 assert!(matches!(sample_placement_candidate(0,[32768,0],&tables),Err(ActorActivationError::InvalidCenter(_))));
 let mut sine=fs::read(p.join("sin.bin")).unwrap();let cosine=fs::read(p.join("cos.bin")).unwrap();
 assert!(matches!(TrigTables::from_mapped_bytes(&sine[..8191],&cosine),Err(ActorActivationError::TrigTableLength{..})));
 sine[4096]^=1;assert!(matches!(TrigTables::from_mapped_bytes(&sine,&cosine),Err(ActorActivationError::TrigTableHash{..})));
 out.flush().unwrap();
}

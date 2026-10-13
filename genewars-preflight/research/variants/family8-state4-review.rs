use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){
 let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();
 let input=fs::read(p.join("input.bin")).unwrap();assert_eq!(input.len()%113,0);
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();
 let trig=TrigTables::generate_pinned().unwrap();let anb=fs::read(p.join("ethereal.anb")).unwrap();let animation=Family8Animation::from_ethereal_anb(&anb).unwrap();
 let mut actual=Vec::new();let mut checks=0;
 for r in input.chunks_exact(113){let before:[u8;102]=r[..102].try_into().unwrap();let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());
  let rng=BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())};
  let got=plan_selector0_state4(&before,&map,frame,1632,&physics,&trig,&animation,rng).unwrap();actual.extend_from_slice(&got.after);
  actual.extend_from_slice(&got.rng_after.seed.to_le_bytes());actual.push(got.rng_after.ring);actual.extend_from_slice(&got.rng_after.draw_count.to_le_bytes());
  actual.extend_from_slice(&[got.child.is_some()as u8,got.pending_ui_message as u8,matches!(got.pending_audio,Family8AudioRequest::Moving{..})as u8,matches!(got.pending_audio,Family8AudioRequest::StopIfPlaying{..})as u8]);
  assert!(got.queue_requested);assert_eq!(got.old_cell,got.new_cell);
  if let Some(c)=got.child {assert_eq!(c.selector,1);assert_eq!(c.owner,before[0x18]);assert_eq!(c.position,[20704,13536]);assert_eq!(c.physical_height,i32::from_le_bytes(got.after[0x27..0x2b].try_into().unwrap())-320);}
  if let Family8AudioRequest::Moving{adjusted_parameter,..}=got.pending_audio {let speed=i16::from_le_bytes(got.after[0x3d..0x3f].try_into().unwrap());assert_eq!(adjusted_parameter,(100+20*i32::from(speed)/1920)as i16);}
  for offset in [6,0x15,0x16,0x4f,0x52,0x55,0x56]{let mut bad=before;bad[offset]^=0xff;let copy=bad;
   assert!(plan_selector0_state4(&bad,&map,frame,1632,&physics,&trig,&animation,rng).is_err(),"admission{offset}");assert_eq!(bad,copy);checks+=1;}
  assert!(plan_selector0_state4(&before,&map[..map.len()-1],frame,1632,&physics,&trig,&animation,rng).is_err());checks+=1;
  let mut bad_rng=rng;bad_rng.ring=32;assert!(plan_selector0_state4(&before,&map,frame,1632,&physics,&trig,&animation,bad_rng).is_err());checks+=1;
  //Pointer and opaque link bytes are preserved by this numeric planner.
  let mut opaque=before;for i in 0..6{opaque[i]=0xa5;}opaque[31..39].fill(0x5a);
  let q=plan_selector0_state4(&opaque,&map,frame,1632,&physics,&trig,&animation,rng).unwrap();assert_eq!(&q.after[..6],&opaque[..6]);assert_eq!(&q.after[31..39],&opaque[31..39]);checks+=1;
 }
 assert!(Family8Animation::from_ethereal_anb(&anb[..anb.len()-1]).is_err());let mut bad=anb;bad[99]^=1;assert!(Family8Animation::from_ethereal_anb(&bad).is_err());checks+=2;
 fs::write(p.join("actual.bin"),actual).unwrap();println!("{} literal callback afterimages; {checks} atomic/admission checks",input.len()/113);
}

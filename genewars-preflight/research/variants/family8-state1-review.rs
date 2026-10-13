use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();let input=fs::read(p.join("input.bin")).unwrap();assert_eq!(input.len()%113,0);
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();
 let animation=Family8DirectionalAnimations::from_ethereal_anb(&fs::read(p.join("ethereal.anb")).unwrap()).unwrap();let mut actual=Vec::new();let mut particles=Vec::new();let mut checks=0;
 for r in input.chunks_exact(113){let before:[u8;102]=r[..102].try_into().unwrap();let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());let rng=BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())};
  let got=plan_selector1_state1_travel(&before,&map,frame,1632,&physics,&trig,&animation,rng).unwrap_or_else(|e|panic!("frame{frame}:{e:?}"));
  actual.extend_from_slice(&got.motion.after);actual.extend_from_slice(&got.motion.rng_after.seed.to_le_bytes());actual.push(got.motion.rng_after.ring);actual.extend_from_slice(&got.motion.rng_after.draw_count.to_le_bytes());
  if let Some(q)=got.particle {particles.extend_from_slice(&[1,q.code]);for v in q.position{particles.extend_from_slice(&v.to_le_bytes());}particles.extend_from_slice(&q.physical_height.to_le_bytes());for v in q.angles{particles.extend_from_slice(&v.to_le_bytes());}}else{particles.extend_from_slice(&[0;16]);}
  assert!(!got.target_selection_required);assert_eq!(&got.motion.after[..6],&before[..6]);assert_eq!(&got.motion.after[31..39],&before[31..39]);checks+=3;
  for offset in [6,0x15,0x16,0x4f,0x52,0x55,0x56,0x57,0x64]{let mut bad=before;bad[offset]^=0xff;assert!(plan_selector1_state1_travel(&bad,&map,frame,1632,&physics,&trig,&animation,rng).is_err());checks+=1;}
 }fs::write(p.join("actual.bin"),actual).unwrap();fs::write(p.join("actual-particles.bin"),particles).unwrap();println!("{} travel afterimages;{checks} admission/preservation checks",input.len()/113);}

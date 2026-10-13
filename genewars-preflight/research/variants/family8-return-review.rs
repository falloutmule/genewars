use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();let input=fs::read(p.join("input.bin")).unwrap();let parents=fs::read(p.join("parents.bin")).unwrap();assert_eq!(input.len()%113,0);assert_eq!(parents.len(),input.len()/113*1632);
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();let animation=Family8DirectionalAnimations::from_ethereal_anb(&fs::read(p.join("ethereal.anb")).unwrap()).unwrap();
 let mut actual=Vec::new();let mut particles=Vec::new();let mut checks=0;
 for (r,raw_parents) in input.chunks_exact(113).zip(parents.chunks_exact(1632)){let before:[u8;102]=r[..102].try_into().unwrap();let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());let rng=BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())};
  let pool:[ [u8;102];16]=std::array::from_fn(|i|raw_parents[i*102..(i+1)*102].try_into().unwrap());
  let got=plan_selector1_state3_return(&before,&map,frame,1632,&physics,&trig,&animation,rng,&pool,true).unwrap_or_else(|e|panic!("frame{frame}:{e:?}"));let motion=&got.onward.motion;
  actual.extend_from_slice(&motion.after);actual.extend_from_slice(&motion.rng_after.seed.to_le_bytes());actual.push(motion.rng_after.ring);actual.extend_from_slice(&motion.rng_after.draw_count.to_le_bytes());actual.extend_from_slice(&[got.retirement_parent.is_some()as u8,got.retirement_parent.map_or(255,|v|v as u8),motion.queue_requested as u8]);
  if let Some(q)=got.onward.particle{particles.extend_from_slice(&[1,q.code]);for v in q.position{particles.extend_from_slice(&v.to_le_bytes());}particles.extend_from_slice(&q.physical_height.to_le_bytes());for v in q.angles{particles.extend_from_slice(&v.to_le_bytes());}}else{particles.extend_from_slice(&[0;16]);}
  assert_eq!(&motion.after[..6],&before[..6]);assert_eq!(&motion.after[31..39],&before[31..39]);assert_eq!(motion.after[6],8);checks+=3;
  let mut invalid=before;invalid[0x57]=3;assert!(plan_selector1_state3_return(&invalid,&map,frame,1632,&physics,&trig,&animation,rng,&pool,true).is_err());checks+=1;
  let mut bad_pool=pool;let parent=if before[0x57]==0{pool.iter().position(|r|r[6]!=0&&r[24]==before[24]&&r[85]==0).unwrap()}else{i16::from_le_bytes(before[89..91].try_into().unwrap())as usize};bad_pool[parent][15]^=1;
  assert!(plan_selector1_state3_return(&before,&map,frame,1632,&physics,&trig,&animation,rng,&bad_pool,true).is_err());checks+=1;
  if frame.wrapping_add(u32::from(before[15]))&31==0&&before[0x49]&0x10==0{assert_eq!(plan_selector1_state3_return(&before,&map,frame,1632,&physics,&trig,&animation,rng,&pool,false),Ok(got));checks+=1;}
 }fs::write(p.join("actual.bin"),actual).unwrap();fs::write(p.join("actual-particles.bin"),particles).unwrap();println!("{} return afterimages; {checks} admission/preservation checks",input.len()/113);}

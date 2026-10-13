use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables};
use std::{fs,path::PathBuf};
fn main(){let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();let buildings=fs::read(p.join("buildings.bin")).unwrap();
 let input=fs::read(p.join("input.bin")).unwrap();assert_eq!(input.len()%113,0);let trig=TrigTables::generate_pinned().unwrap();let mut actual=Vec::new();let mut checks=0;
 for r in input.chunks_exact(113){let before:[u8;102]=r[..102].try_into().unwrap();let target=[i16::from_le_bytes(r[102..104].try_into().unwrap()),i16::from_le_bytes(r[104..106].try_into().unwrap())];
  let rng=BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())};
  let got=plan_selector1_initial_empty_building_target(&before,&map,target,&buildings,&trig,rng).unwrap();actual.extend_from_slice(&got.after);actual.extend_from_slice(&got.rng_after.seed.to_le_bytes());actual.push(got.rng_after.ring);actual.extend_from_slice(&got.rng_after.draw_count.to_le_bytes());
  for q in got.target{actual.extend_from_slice(&q.to_le_bytes());}actual.extend_from_slice(&got.target_height.to_le_bytes());
  for offset in [0x55,0x56,0x57,0x5b]{let mut bad=before;bad[offset]^=0xff;assert!(plan_selector1_initial_empty_building_target(&bad,&map,target,&buildings,&trig,rng).is_err());checks+=1;}
  for slot in [0,31,63]{let mut active=buildings.clone();active[slot*67+6]=4;assert!(plan_selector1_initial_empty_building_target(&before,&map,target,&active,&trig,rng).is_err());checks+=1;}
  assert_eq!(&got.after[31..39],&before[31..39]);assert_eq!(&got.after[..6],&before[..6]);checks+=2;
 }fs::write(p.join("actual.bin"),actual).unwrap();println!("{} target afterimages;{checks} rejection/preservation checks",input.len()/113);}

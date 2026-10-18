extern crate genewars_runtime;extern crate genewars_core;pub use genewars_runtime::*;#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/family8_motion.rs"] mod family8_motion;#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/family8_zone_target.rs"] mod family8_zone_target;mod actor_motion{pub(crate) fn angle(x:i32,y:i32,t:&[u16;257])->u16 {
    if x==0 && y==0 { return 0; }
    let ax=x.unsigned_abs(); let ay=y.unsigned_abs();
    let a=if ax<ay { t[(u64::from(ax)*256/u64::from(ay)) as usize] }
          else { t[(u64::from(ay)*256/u64::from(ax)) as usize] };
    match (x>=0,y>=0,ax<ay) {
        (true,false,true)=>a,(true,false,false)=>512-a,
        (true,true,true)=>1024-a,(true,true,false)=>512+a,
        (false,false,true)=>2048-a,(false,false,false)=>1536+a,
        (false,true,true)=>1024+a,(false,true,false)=>1536-a,
    }
}}use family8_motion::*;use genewars_runtime::{blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();let input=fs::read(p.join("input.bin")).unwrap();assert_eq!(input.len()%113,0);
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();
 let animation=Family8DirectionalAnimations::from_ethereal_anb(&fs::read(p.join("ethereal.anb")).unwrap()).unwrap();let pth=p.clone();let mut actual=Vec::new();let mut particles=Vec::new();let mut checks=0;
 for r in input.chunks_exact(113){let before:[u8;102]=r[..102].try_into().unwrap();let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());let rng=BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())};
  let got=plan_selector1_state2_hover_with_particle(&before,&map,frame,2000,&physics,&trig,&animation,rng,&fs::read(p.join("zones.bin")).unwrap(),true,|_,_,rng|Ok(rng)).map(|p|{let initial=fs::read(pth.join("zones.bin")).unwrap();let mut expected=initial.clone();assert_eq!(p.zones_after,expected);p.onward}).unwrap_or_else(|e|panic!("frame{frame}:{e:?}"));
  actual.extend_from_slice(&got.motion.after);actual.extend_from_slice(&got.motion.rng_after.seed.to_le_bytes());actual.push(got.motion.rng_after.ring);actual.extend_from_slice(&got.motion.rng_after.draw_count.to_le_bytes());
  if let Some(q)=got.particle {particles.extend_from_slice(&[1,q.code]);for v in q.position{particles.extend_from_slice(&v.to_le_bytes());}particles.extend_from_slice(&q.physical_height.to_le_bytes());for v in q.angles{particles.extend_from_slice(&v.to_le_bytes());}}else{particles.extend_from_slice(&[0;16]);}
  assert!(!got.target_selection_required);assert_eq!(&got.motion.after[..6],&before[..6]);assert_eq!(&got.motion.after[31..39],&before[31..39]);checks+=3;
 }fs::write(p.join("actual.bin"),actual).unwrap();fs::write(p.join("actual-particles.bin"),particles).unwrap();println!("{} travel afterimages;{checks} admission/preservation checks",input.len()/113);}

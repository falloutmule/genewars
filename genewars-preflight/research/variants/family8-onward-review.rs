use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn main(){
 let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();
 let trig=TrigTables::generate_pinned().unwrap();let anb=fs::read(p.join("ethereal.anb")).unwrap();let animation=Family8Animation::from_ethereal_anb(&anb).unwrap();let directional=Family8DirectionalAnimations::from_ethereal_anb(&anb).unwrap();
 let mut checks=0;
 for name in ["parent","child"]{
  let input=fs::read(p.join(format!("{name}-input.bin"))).unwrap();assert_eq!(input.len()%113,0);let mut actual=Vec::new();let mut particles=Vec::new();
  for r in input.chunks_exact(113){let before:[u8;102]=r[..102].try_into().unwrap();let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());
   let rng=BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())};
   let (got,particle,target)=if name=="parent" {
    let got=plan_selector0_state5_wait(&before,&map,frame,1632,&physics,&trig,&animation,rng,true).unwrap_or_else(|e|panic!("parent{frame}: {e:?}"));
    if frame&31==0 {assert_eq!(plan_selector0_state5_wait(&before,&map,frame,1632,&physics,&trig,&animation,rng,false),Err(Family8MotionError::IdleSearchRequired));checks+=1;}
    (got,None,false)
   }else {let got=plan_selector1_state0(&before,&map,frame,1632,&physics,&trig,&directional,rng).unwrap_or_else(|e|panic!("child{frame}: {e:?}"));(got.motion,got.particle,got.target_selection_required)};
   actual.extend_from_slice(&got.after);actual.extend_from_slice(&got.rng_after.seed.to_le_bytes());actual.push(got.rng_after.ring);actual.extend_from_slice(&got.rng_after.draw_count.to_le_bytes());
   actual.extend_from_slice(&[got.child.is_some()as u8,got.pending_ui_message as u8,matches!(got.pending_audio,Family8AudioRequest::Moving{..})as u8,matches!(got.pending_audio,Family8AudioRequest::StopIfPlaying{..})as u8,particle.is_some()as u8,target as u8]);
   if let Some(q)=particle{particles.extend_from_slice(&[1,q.code]);for v in q.position{particles.extend_from_slice(&v.to_le_bytes());}particles.extend_from_slice(&q.physical_height.to_le_bytes());for v in q.angles{particles.extend_from_slice(&v.to_le_bytes());}}
   else{particles.extend_from_slice(&[0;16]);}
   assert_eq!(&got.after[31..39],&before[31..39]);assert_eq!(&got.after[..6],&before[..6]);checks+=2;
  }
  fs::write(p.join(format!("{name}-actual.bin")),actual).unwrap();fs::write(p.join(format!("{name}-actual-particles.bin")),particles).unwrap();
  println!("{name}:{} full callback afterimages",input.len()/113);
 }
 println!("{checks} pointer/link/query boundary checks");
}

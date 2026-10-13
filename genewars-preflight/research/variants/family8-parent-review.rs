use genewars_runtime::{family8_motion::*,blueprint::BlueprintRng,actor_activation::TrigTables,physics_tables::PhysicsTables};
use std::{fs,path::PathBuf};
fn rng(r:&[u8])->BlueprintRng{BlueprintRng{seed:u16::from_le_bytes(r[106..108].try_into().unwrap()),ring:r[108],draw_count:u32::from_le_bytes(r[109..113].try_into().unwrap())}}
fn pack(v:&mut Vec<u8>,a:&[u8;102],r:BlueprintRng){v.extend_from_slice(a);v.extend_from_slice(&r.seed.to_le_bytes());v.push(r.ring);v.extend_from_slice(&r.draw_count.to_le_bytes());}
fn main(){let p=PathBuf::from(std::env::args().nth(1).unwrap());let map=fs::read(p.join("map.bin")).unwrap();let input=fs::read(p.join("input.bin")).unwrap();assert_eq!(input.len()%119,0);
 let physics=PhysicsTables::from_mapped_bytes(&fs::read(p.join("angles.bin")).unwrap(),&fs::read(p.join("bounce.bin")).unwrap()).unwrap();let trig=TrigTables::generate_pinned().unwrap();let animation=Family8Animation::from_ethereal_anb(&fs::read(p.join("ethereal.anb")).unwrap()).unwrap();let mut actual=Vec::new();let mut checks=0;
 for r in input.chunks_exact(119){let before:[u8;102]=r[..102].try_into().unwrap();let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());let counter=i32::from_le_bytes(r[113..117].try_into().unwrap());let threshold=u16::from_le_bytes(r[117..119].try_into().unwrap());
  let got=plan_selector0_state5_notified(&before,&map,frame,1632,&physics,&trig,&animation,rng(r),true,counter,threshold).unwrap();pack(&mut actual,&got.motion.after,got.motion.rng_after);actual.extend_from_slice(&got.player_counter_after.to_le_bytes());actual.push(got.state7_message_variant.unwrap_or(255));
  assert_eq!(&got.motion.after[..6],&before[..6]);assert_eq!(&got.motion.after[31..39],&before[31..39]);assert!(got.motion.queue_requested);checks+=3;
  let mut invalid=before;invalid[0x57]=0;assert!(plan_selector0_state5_notified(&invalid,&map,frame,1632,&physics,&trig,&animation,rng(r),true,counter,threshold).is_err());checks+=1;
 }fs::write(p.join("actual.bin"),actual).unwrap();
 let input8=fs::read(p.join("state8-input.bin")).unwrap();let pools=fs::read(p.join("state8-pools.bin")).unwrap();let mut actual8=Vec::new();assert_eq!(input8.len()%113,0);assert_eq!(pools.len(),input8.len()/113*1632);
 for (r,raw) in input8.chunks_exact(113).zip(pools.chunks_exact(1632)){let before:[u8;102]=r[..102].try_into().unwrap();let pool:[[u8;102];16]=std::array::from_fn(|i|raw[i*102..(i+1)*102].try_into().unwrap());let frame=u32::from_le_bytes(r[102..106].try_into().unwrap());let got=plan_selector0_state8_group_departure(&before,&map,frame,1632,&physics,&trig,&animation,rng(r),true,&pool).unwrap();pack(&mut actual8,&got.after,got.rng_after);
  assert_eq!(&got.after[7..11],&before[7..11]);assert_eq!(&got.after[0x27..0x2b],&before[0x27..0x2b]);assert!([8,9].contains(&got.after[0x56]));checks+=3;
  let mut invalid=before;invalid[0x56]=9;assert!(plan_selector0_state8_group_departure(&invalid,&map,frame,1632,&physics,&trig,&animation,rng(r),true,&pool).is_err());checks+=1;
 }fs::write(p.join("state8-actual.bin"),actual8).unwrap();println!("{} notification decisions + {} state8 callbacks; {checks} checks",input.len()/119,input8.len()/113);}

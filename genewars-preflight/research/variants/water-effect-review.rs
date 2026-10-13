use genewars_runtime::{water_effect,actor_activation::TrigTables};
use std::{fs,path::PathBuf};
fn main(){let p=PathBuf::from(std::env::args().nth(1).unwrap());let trig=TrigTables::generate_pinned().unwrap();let sin:&[i32;2048]=trig.sine().try_into().unwrap();let cos:&[i32;2048]=trig.cosine().try_into().unwrap();
 let input=fs::read(p.join("factory-input.bin")).unwrap();assert_eq!(input.len()%103,0);let mut actual=Vec::new();
 for r in input.chunks_exact(103){let prior:[u8;81]=r[..81].try_into().unwrap();let code=r[81];let slot=u16::from_le_bytes(r[82..84].try_into().unwrap());let point=[i16::from_le_bytes(r[84..86].try_into().unwrap()),i16::from_le_bytes(r[86..88].try_into().unwrap())];let ground=i32::from_le_bytes(r[88..92].try_into().unwrap());let height=i32::from_le_bytes(r[92..96].try_into().unwrap());let owner=r[96];let angles=std::array::from_fn(|i|u16::from_le_bytes(r[97+i*2..99+i*2].try_into().unwrap()));let mut pool=[[0;81];200];for r in &mut pool[..slot as usize]{r[6]=5}pool[slot as usize]=prior;
  let got=water_effect::plan_factory(code,&pool,slot as i16,owner,point,ground,0,height,angles,None,sin,cos).unwrap();assert_eq!(got.animation_index,u16::from(code));actual.extend_from_slice(&got.after_nonpointer);
 }
 fs::write(p.join("factory-actual.bin"),actual).unwrap();let input=fs::read(p.join("update-input.bin")).unwrap();assert_eq!(input.len()%83,0);let mut actual=Vec::new();
 for (i,r) in input.chunks_exact(83).enumerate(){let before:[u8;81]=r[..81].try_into().unwrap();let raw_water=i16::from_le_bytes(r[81..83].try_into().unwrap());let got=water_effect::plan_animation_after_physics(&before,raw_water).unwrap_or_else(|e|panic!("case{i}:{e:?}"));actual.extend_from_slice(&got.after);actual.extend_from_slice(&[got.retirement_required as u8,got.create_child as u8]);}
 fs::write(p.join("update-actual.bin"),actual).unwrap();println!("64 factory and{} postphysics afterimages",input.len()/83);
}

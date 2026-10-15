//! Source114B0 state2 terrain-zone target selection, with explicit zone afterimages.
use crate::{actor_activation::TrigTables,blueprint::BlueprintRng,family8_creation::Family8Record,family8_motion::Family8MotionError};
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8ZoneTargetPlan {pub after:Family8Record,pub zones_after:Vec<u8>,pub rng_after:BlueprintRng,pub selected:Option<u8>}
pub fn plan_zone_target(before:&Family8Record,map:&[u8],zones:&[u8],trig:&TrigTables,rng:BlueprintRng)->Result<Family8ZoneTargetPlan,Family8MotionError>{
 if map.len()!=229376{return Err(Family8MotionError::MapLength);}
 if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
 if before[6]!=8||before[0x55]!=1||before[0x56]!=2||before[24]>=4||zones.len()%19!=0||zones.len()>64*19||before[0x57]>64||usize::from(before[0x57])>zones.len()/19{return Err(Family8MotionError::UnsupportedRecord);}
 let mut a=*before;let mut z=zones.to_vec();let mut r=rng;let start=usize::from(a[0x57]);
 if start>0 {let o=(start-1)*19+18;z[o]=z[o].wrapping_sub(1);}
 let selected=(start..z.len()/19).find(|&i|i32::from_le_bytes(z[i*19+14..i*19+18].try_into().unwrap())>0);
 if let Some(i)=selected{
  a[0x5c..0x60].copy_from_slice(&z[i*19..i*19+4]);a[0x57]=i as u8;a[0x59..0x5b].copy_from_slice(&(i as i16).to_le_bytes());
  r.seed=r.seed.wrapping_mul(58925);r.ring=r.ring.wrapping_add(1)&31;r.draw_count=r.draw_count.wrapping_add(1);
  let index=(usize::from(r.seed>>8)*8+1536)&2047;
  let target:[i16;2]=std::array::from_fn(|axis|{let center=i16::from_le_bytes([a[0x5c+axis*2],a[0x5d+axis*2]]);let term=if axis==0{trig.cosine()[index]}else{trig.sine()[index]};let offset=(2048i32.wrapping_mul(term)>>16)as i16;(center.wrapping_mul(2).wrapping_add(offset)>>1)&0x7fff});
  let height=genewars_core::cd_candidate::interpolate_height(map,target[0],target[1]).map_err(|_|Family8MotionError::MapLength)?.wrapping_add(2560);
  for axis in 0..2{a[11+axis*2..13+axis*2].copy_from_slice(&target[axis].to_le_bytes());}
  a[0x54]=20;let flags=(u16::from_le_bytes([a[0x49],a[0x4a]])|0x14)&0xeffc;a[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());a[0x16]&=0x3f;a[0x4f]=0;a[0x52]=0;a[0x60..0x64].copy_from_slice(&height.to_le_bytes());a[0x4a]&=0x3f;a[0x3f..0x41].copy_from_slice(&3840i16.to_le_bytes());a[0x64..0x66].fill(0);a[0x57]=a[0x57].wrapping_add(1);
 }
 Ok(Family8ZoneTargetPlan{after:a,zones_after:z,rng_after:r,selected:selected.map(|i|i as u8)})
}

use genewars_core::cd_candidate::{camera_cells_for_target,interpolate_height};
use std::{env,fs};
fn main(){let a:Vec<String>=env::args().collect();let b=fs::read(&a[1]).unwrap();let map=&b[0x355e..0x3b55e];let mut out=Vec::new();
 for target in b[0x1c7e..0x1c8e].chunks_exact(4){let x=i16::from_le_bytes(target[..2].try_into().unwrap());let y=i16::from_le_bytes(target[2..].try_into().unwrap());out.extend(interpolate_height(map,x,y).unwrap().to_le_bytes());for c in camera_cells_for_target(map,x,y).unwrap(){out.extend(c.to_le_bytes());}}
 fs::write(&a[2],out).unwrap();}

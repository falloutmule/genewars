#[path="C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/rust/crates/genewars-runtime/src/building_obstacle.rs"]mod obstacle;
use std::io::{Read,Write};
fn main(){let mut data=Vec::new();std::io::stdin().read_to_end(&mut data).unwrap();let mut out=Vec::new();
for c in data.chunks_exact(608){let point=[i16::from_le_bytes(c[0..2].try_into().unwrap()),i16::from_le_bytes(c[2..4].try_into().unwrap())];
let cell=|p:[i16;2]|((((i32::from(p[1])>>8)&127)*128)+((i32::from(p[0])>>8)&127))as usize;
let offsets:[[i16;2];9]=[[0,0],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[256,-256],[7680,0]];
let indices:Vec<_>=offsets.iter().map(|v|cell([point[0].wrapping_add(v[0]),point[1].wrapping_add(v[1])])).collect();
let mut map=vec![0;229376];map[cell(point)*14+11]=c[4];let mut queries=Vec::new();
let result=obstacle::move_obstacle(&map,point,|at|{let i=indices.iter().position(|v|*v==at).unwrap();queries.push(i as u8);let record:[u8;67]=c[5+i*67..5+(i+1)*67].try_into().unwrap();Ok::<_,()>(if record[6]==0{None}else{Some(record)})}).unwrap();
out.push(result as u8);out.push(queries.len()as u8);queries.resize(9,255);out.extend(queries);}
std::io::stdout().write_all(&out).unwrap();}

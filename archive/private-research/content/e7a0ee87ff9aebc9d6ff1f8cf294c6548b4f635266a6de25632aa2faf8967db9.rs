mod obstacle;
use std::{fs,io::Write,path::Path};
fn main(){let path=std::env::args().nth(1).unwrap();let path=Path::new(&path);let data=fs::read(path.join("input.bin")).unwrap();let mut out=fs::File::create(path.join("actual.bin")).unwrap();
for c in data.chunks_exact(610){let point=[i16::from_le_bytes(c[..2].try_into().unwrap()),i16::from_le_bytes(c[2..4].try_into().unwrap())];
let nominated=i16::from_le_bytes(c[5..7].try_into().unwrap());let nominated=if nominated<0{None}else{Some(nominated as u16)};
let cell=|p:[i16;2]|((((i32::from(p[1])>>8)&127)*128)+((i32::from(p[0])>>8)&127))as usize;
let offsets:[[i16;2];9]=[[0,0],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[256,-256],[7680,0]];
let indices:Vec<_>=offsets.iter().map(|v|cell([point[0].wrapping_add(v[0]),point[1].wrapping_add(v[1])])).collect();
let mut map=vec![0;229376];map[cell(point)*14+11]=c[4];let mut queries=Vec::new();
let blocked=obstacle::move_obstacle_excluding(&map,point,nominated,|at|{let i=indices.iter().position(|v|*v==at).unwrap();queries.push(i as u8);let record:[u8;67]=c[7+i*67..7+(i+1)*67].try_into().unwrap();Ok::<_,()>(if record[6]==0{None}else{Some(record)})}).unwrap();
out.write_all(&[!blocked as u8,queries.len()as u8]).unwrap();queries.resize(9,255);out.write_all(&queries).unwrap();}}

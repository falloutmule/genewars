use genewars_core::cd_candidate::interpolate_entity_shade;
use std::{env,fs,path::Path};
fn main(){
 let a:Vec<String>=env::args().collect();let p=Path::new(&a[1]);let data=fs::read(p.join("map-0.bin")).unwrap();let coords=fs::read(p.join("coords.bin")).unwrap();let mut actual=Vec::new();
 for c in coords.chunks_exact(4){actual.push(interpolate_entity_shade(&data,i16::from_le_bytes(c[..2].try_into().unwrap()),i16::from_le_bytes(c[2..].try_into().unwrap())).unwrap());}
 fs::write(p.join("actual.bin"),actual).unwrap();let mut out=Vec::new();
 for i in 1..=a[2].parse::<usize>().unwrap(){let map=fs::read(p.join(format!("map-{i}.bin"))).unwrap();for y in 0..256i16{for x in 0..256i16{out.push(interpolate_entity_shade(&map,x,y).unwrap())}}}
 fs::write(p.join("fractions.bin"),out).unwrap();let mut out=Vec::with_capacity(16777216);let mut map=vec![0;229376];
 for light in i16::MIN..=i16::MAX{map[..2].copy_from_slice(&light.to_le_bytes());for factor in 0..=255u8{map[12]=factor;out.push(interpolate_entity_shade(&map,0,0).unwrap());}}
 fs::write(p.join("vertex.bin"),out).unwrap();assert!(interpolate_entity_shade(&data[..data.len()-1],0,0).is_err());let mut data=data;data.push(0);assert!(interpolate_entity_shade(&data,0,0).is_err());
}

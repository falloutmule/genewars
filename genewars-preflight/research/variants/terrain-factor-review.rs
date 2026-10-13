use genewars_core::cd_candidate::plan_terrain_factor_raise_4096;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn main(){
 let a:Vec<String>=env::args().collect();let out=Path::new(&a[1]);let count:usize=a[2].parse().unwrap();
 let maps:Vec<Vec<u8>>=(0..count).map(|i|fs::read(out.join(format!("map-{i}.bin"))).unwrap()).collect();
 let cases=fs::read(out.join("cases.bin")).unwrap();let mut results=BufWriter::new(fs::File::create(out.join("results.bin")).unwrap());
 for row in cases.chunks_exact(8){let i=u32::from_le_bytes(row[..4].try_into().unwrap())as usize;let target=[i16::from_le_bytes([row[4],row[5]]),i16::from_le_bytes([row[6],row[7]])];
  let changes=plan_terrain_factor_raise_4096(&maps[i],target).unwrap();results.write_all(&(changes.len()as u32).to_le_bytes()).unwrap();
  for c in changes{results.write_all(&c.grid_position[0].to_le_bytes()).unwrap();results.write_all(&c.grid_position[1].to_le_bytes()).unwrap();results.write_all(&[c.source_cell[0],c.source_cell[1],c.previous,c.next]).unwrap();}
 }results.flush().unwrap();
 assert!(plan_terrain_factor_raise_4096(&maps[0][..maps[0].len()-1],[0,0]).is_err());let mut extra=maps[0].clone();extra.push(0);assert!(plan_terrain_factor_raise_4096(&extra,[0,0]).is_err());
}

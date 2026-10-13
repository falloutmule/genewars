#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn main(){
 let a:Vec<String>=env::args().collect();let out=Path::new(&a[1]);
 let mut shades=BufWriter::new(fs::File::create(out.join("shades.bin")).unwrap());
 for light in i16::MIN..=i16::MAX{for factor in 0..=u8::MAX{shades.write_all(&terrain::terrain_vertex_shade_16_16(light,factor).to_le_bytes()).unwrap();}}
 shades.flush().unwrap();let fade=fs::read(&a[2]).unwrap();let table=terrain::TerrainFadeTable::parse(&fade).unwrap();let mut mapped=Vec::new();
 for shade in 0..64{for index in 0..=u8::MAX{mapped.push(table.map_palette_index(shade,index).unwrap());}}
 fs::write(out.join("mapped-fade.bin"),mapped).unwrap();
 for shade in 64..=u8::MAX{for index in [0,255]{assert!(table.map_palette_index(shade,index).is_err());}}
 for n in [0,1,16383]{assert!(terrain::TerrainFadeTable::parse(&fade[..n]).is_err());}
 let mut extra=fade;extra.push(0);assert!(terrain::TerrainFadeTable::parse(&extra).is_err());
}

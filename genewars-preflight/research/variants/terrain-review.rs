use genewars_assets::{ConqMapCells,B32Atlas};
use genewars_core::cd_candidate::interpolate_height;
use std::{env,fs,path::Path};

fn main(){
 let a:Vec<String>=env::args().collect();let out=Path::new(&a[1]);let map=fs::read(&a[2]).unwrap();let atlas=fs::read(&a[3]).unwrap();
 let cells=ConqMapCells::parse(&map).unwrap();assert_eq!(cells.bytes(),map);let mut records=Vec::new();
 for y in 0..128{for x in 0..128{let cell=cells.cell(x,y).unwrap();records.extend(cell.height().to_le_bytes());records.push(cell.texture_selector());records.extend(cell.raw());}}
 fs::write(out.join("cells.bin"),records).unwrap();
 for (x,y) in [(128,0),(0,128),(usize::MAX,0),(0,usize::MAX),(usize::MAX,usize::MAX)]{assert!(cells.cell(x,y).is_err());}
 assert!(ConqMapCells::parse(&map[..map.len()-1]).is_err());let mut oversized=map.clone();oversized.push(0);assert!(ConqMapCells::parse(&oversized).is_err());
 assert!(interpolate_height(&map[..map.len()-1],0,0).is_err());assert!(interpolate_height(&oversized,0,0).is_err());
 let parsed=B32Atlas::parse(&atlas).unwrap();let mut tiles=Vec::new();for i in 0..160{tiles.extend(parsed.tile(i).unwrap());}fs::write(out.join("tiles.bin"),tiles).unwrap();
 assert!(parsed.tile(160).is_err());assert!(parsed.tile(usize::MAX).is_err());assert!(B32Atlas::parse(&atlas[..atlas.len()-1]).is_err());let mut extra=atlas.clone();extra.push(0);assert!(B32Atlas::parse(&extra).is_err());
 let coordinates=fs::read(out.join("coordinates.bin")).unwrap();let mut samples=Vec::new();
 for c in coordinates.chunks_exact(4){let x=i16::from_le_bytes([c[0],c[1]]);let y=i16::from_le_bytes([c[2],c[3]]);samples.extend(interpolate_height(&map,x,y).unwrap().to_le_bytes());}
 fs::write(out.join("heights.bin"),samples).unwrap();
 let sets:[[i16;4];8]=[[0,0,0,0],[7,7,7,7],[0,256,512,768],[-32768,32767,-32768,32767],[32767,-32768,32767,-32768],[-5,8,-13,21],[-32768,32767,32767,-32768],[32767,-32768,-32768,32767]];
 let mut synthetic=vec![0u8;128*128*14];let mut samples=Vec::new();
 for h in sets{
  for (at,value) in [(2,h[0]),(16,h[1]),(128*14+2,h[2]),(129*14+2,h[3])]{synthetic[at..at+2].copy_from_slice(&value.to_le_bytes());}
  for fx in 0..256{for fy in 0..256{samples.extend(interpolate_height(&synthetic,fx as i16,fy as i16).unwrap().to_le_bytes());}}
 }fs::write(out.join("synthetic-heights.bin"),samples).unwrap();
}

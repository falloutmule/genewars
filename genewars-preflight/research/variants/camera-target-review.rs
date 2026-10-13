use genewars_core::cd_candidate as terrain;
use std::{env,fs,path::Path};
fn main(){
 let a:Vec<String>=env::args().collect();let out=Path::new(&a[1]);let map=fs::read(&a[2]).unwrap();let coords=fs::read(out.join("coords.bin")).unwrap();let mut result=Vec::new();
 for row in coords.chunks_exact(4){let x=i16::from_le_bytes([row[0],row[1]]);let y=i16::from_le_bytes([row[2],row[3]]);for cell in terrain::camera_cells_for_target(&map,x,y).unwrap(){result.extend(cell.to_le_bytes());}}
 fs::write(out.join("actual-cameras.bin"),result).unwrap();
 let levels:[i16;14]=[-32768,-1025,-1024,-1023,-993,-992,-991,-1,0,991,992,1023,1024,32767];let mut synthetic=vec![0u8;128*128*14];let mut result=Vec::new();
 for level in levels{
  for cell in synthetic.chunks_exact_mut(14){cell[2..4].copy_from_slice(&level.to_le_bytes());}
  for x in 0..256{for y in 0..256{for camera in terrain::camera_cells_for_target(&synthetic,((x<<8)|0x55)as i16,((y<<8)|0xaa)as i16).unwrap(){result.extend(camera.to_le_bytes());}}}
 }fs::write(out.join("synthetic-cameras.bin"),result).unwrap();
 assert!(terrain::camera_cells_for_target(&map[..map.len()-1],0,0).is_err());let mut extra=map;extra.push(0);assert!(terrain::camera_cells_for_target(&extra,0,0).is_err());
}

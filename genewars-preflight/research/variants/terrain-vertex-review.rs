#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
use std::{env,fs,path::Path};
fn main(){
 let a:Vec<String>=env::args().collect();let out=Path::new(&a[1]);let mut heights=Vec::new();
 for h in i16::MIN..=i16::MAX{heights.extend(terrain::terrain_vertex_height_displacement(h).to_le_bytes());}
 fs::write(out.join("displacements.bin"),heights).unwrap();let mut rows=Vec::new();
 for flag in 0..=u8::MAX{for (u,v) in terrain::b32_uv_corners_for_cell_flags(flag){rows.extend(u.to_le_bytes());rows.extend(v.to_le_bytes());}}
 fs::write(out.join("uv-flags.bin"),rows).unwrap();
 for row in 0..=u8::MAX{assert_eq!(terrain::b32_uv_corners(row).is_ok(),row<8);}
 assert_eq!(terrain::CELL_TRIANGLES,[[0,2,3],[0,1,2]]);
 let cases=fs::read(out.join("cases.bin")).unwrap();let mut output=Vec::new();
 for row in cases.chunks_exact(32){let mut v=[0i32;8];for i in 0..8{v[i]=i32::from_le_bytes(row[i*4..i*4+4].try_into().unwrap());}
  match terrain::terrain_vertex_buffer_position([v[0],v[1]],[v[2],v[3]],v[4] as i16,v[5],[v[6],v[7]]){
   Ok([x,y])=>{output.push(1);output.extend(x.to_le_bytes());output.extend(y.to_le_bytes());}
   Err(_)=>{output.push(0);output.extend([0u8;8]);}
  }
 }fs::write(out.join("positions.bin"),output).unwrap();
}

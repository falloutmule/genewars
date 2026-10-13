#[path="../../rust/crates/genewars-runtime/src/target_projection.rs"]mod projection;
use projection::MinimapContext;
use std::{fs,io::{BufWriter,Write},path::Path};
fn main(){let dir=std::env::args().nth(1).unwrap();let mut out=BufWriter::new(fs::File::create(Path::new(&dir).join("minimap-output.bin")).unwrap());
 for base in [[0,0],[127,127],[128,255],[255,128],[255,255]]{for origin in [[0i16,0i16],[-32768,-32768],[32767,32767],[32700,-32700],[10,20]]{for y in -1..=89 {for x in -1..=88 {
  let c=MinimapContext{screen_origin:origin,map_base:base};let screen=[origin[0].wrapping_add(x),origin[1].wrapping_add(y)];let mut b=[0;7];if let Some(r)=c.project(screen){b[0]=1;b[1..3].copy_from_slice(&r.point[0].to_le_bytes());b[3..5].copy_from_slice(&r.point[1].to_le_bytes());b[5..7].copy_from_slice(&r.cell_index.to_le_bytes());}out.write_all(&b).unwrap();
 }}}}
 for raw in 0..=u16::MAX {for delta in [-1i16,0,43,87,88] {let origin=[raw as i16,raw as i16];let c=MinimapContext{screen_origin:origin,map_base:[raw as u8,(raw>>8)as u8]};let screen=[origin[0].wrapping_add(delta),origin[1]];let mut b=[0;7];if let Some(r)=c.project(screen){b[0]=1;b[1..3].copy_from_slice(&r.point[0].to_le_bytes());b[3..5].copy_from_slice(&r.point[1].to_le_bytes());b[5..7].copy_from_slice(&r.cell_index.to_le_bytes());}out.write_all(&b).unwrap();}}
 out.flush().unwrap();}

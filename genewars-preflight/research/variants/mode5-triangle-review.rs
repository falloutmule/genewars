#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
use terrain::*;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn n(b:&[u8],i:usize)->i32{i32::from_le_bytes(b[i*4..i*4+4].try_into().unwrap())}
fn put(w:&mut impl Write,n:i32){w.write_all(&n.to_le_bytes()).unwrap();}
fn error(e:TerrainPrimitiveError)->i32{match e {
 TerrainPrimitiveError::TriangleZeroGeneralWidth=>2,
 TerrainPrimitiveError::TriangleCoordinateOutOfRange{..}=>3,
 TerrainPrimitiveError::TriangleClipHeightOutOfRange{..}=>4,
 TerrainPrimitiveError::FadeShadeIndexOutOfRange{..}=>5,
 TerrainPrimitiveError::TextureSampleOutOfBounds{..}=>6,
 TerrainPrimitiveError::TriangleSpanInvariant=>7,
 TerrainPrimitiveError::TriangleClipWidthOutOfRange{..}=>8,
 TerrainPrimitiveError::TriangleFramebufferPitchTooSmall{..}=>9,
 TerrainPrimitiveError::TriangleFramebufferTooSmall{..}=>10,
 TerrainPrimitiveError::TriangleFramebufferExtentOverflow=>11,
 e=>panic!("unexpected {e:?}")}}
fn main(){
 let a:Vec<String>=env::args().collect();let dir=Path::new(&a[1]);
 let source:Vec<u8>=(0..65536).map(|i|((i%256*3+i/256*5)&255)as u8).collect();
 let fb:Vec<u8>=(0..16384).map(|i|((i%256+7*(i/256))&255)as u8).collect();
 let fade=TerrainFadeTable::parse(&fb).unwrap();
 let data=fs::read(dir.join("cases.bin")).unwrap();let mut out=BufWriter::new(fs::File::create(dir.join("results.bin")).unwrap());
 std::panic::set_hook(Box::new(|_|{}));
 for b in data.chunks_exact(80){
  let h=n(b,0);let width=n(b,1);let pitch=n(b,2)as usize;let len=n(b,3)as usize;let mutation=n(b,4);
  let vertices=core::array::from_fn(|j|Mode5TriangleVertex{x:n(b,5+j*5),y:n(b,6+j*5),u:n(b,7+j*5),v:n(b,8+j*5),shade:n(b,9+j*5)});
  let prep=prepare_mode5_triangle(vertices,h);
  let mut triangle=match prep {Ok(Some(t))=>{put(&mut out,0);t},Ok(None)=>{put(&mut out,1);continue},Err(e)=>{put(&mut out,error(e));continue}};
  for v in triangle.gradients_du_dv_dshade{put(&mut out,v)}put(&mut out,triangle.rows.len()as i32);
  for r in &triangle.rows{for v in [r.y,r.left_16_16,r.right_16_16,r.u_16_16,r.v_16_16,r.shade_16_16]{put(&mut out,v)}}
  if let Some(last)=triangle.rows.last_mut(){match mutation{
   1=>{last.left_16_16=0;last.right_16_16=1<<16;last.shade_16_16=64<<16},
   2=>{last.left_16_16=0;last.right_16_16=1<<16;last.u_16_16=255<<16;last.v_16_16=255<<16;last.shade_16_16=0},
   3=>last.y=-1,4=>last.y=h,
   5=>{last.y=h-1;last.left_16_16=i32::MAX;last.right_16_16=i32::MAX},
   6=>{last.left_16_16=i32::MIN;last.right_16_16=-1},_=>{}}}
  let frame_len=if mutation==10 {1}else{pitch.saturating_mul(h as usize)};
  if mutation==7{triangle.clip_height=0;}if mutation==8{triangle.clip_height=481;}
  let mut frame=vec![165;frame_len+8];
  let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||draw_mode5_triangle(&source[..len],&fade,&triangle,width,&mut frame[4..4+frame_len],pitch)));
  let (status,count)=match result{Ok(Ok(r))=>(0,r.pixels_written as i32),Ok(Err(e))=>(error(e),0),Err(_)=>(99,0)};
  put(&mut out,status);put(&mut out,count);put(&mut out,frame.len()as i32);out.write_all(&frame).unwrap();
 }
 out.flush().unwrap();
}

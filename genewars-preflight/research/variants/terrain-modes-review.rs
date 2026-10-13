#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
#[path="../../rust/crates/genewars-render/src/terrain_modes.rs"] mod terrain_modes;
use terrain::*;use terrain_modes::*;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn put(w:&mut impl Write,n:i32){w.write_all(&n.to_le_bytes()).unwrap();}
fn int(b:&[u8],i:usize)->i32{i32::from_le_bytes(b[i*4..i*4+4].try_into().unwrap())}
fn prim(e:TerrainPrimitiveError)->i32{match e{
 TerrainPrimitiveError::FadeShadeIndexOutOfRange{..}=>5,TerrainPrimitiveError::TextureSampleOutOfBounds{..}=>6,TerrainPrimitiveError::TextureAddressOverflow=>7,
 TerrainPrimitiveError::TriangleClipHeightOutOfRange{..}=>8,TerrainPrimitiveError::TriangleCoordinateOutOfRange{..}=>9,TerrainPrimitiveError::TriangleSpanInvariant=>10,
 TerrainPrimitiveError::TriangleFramebufferPitchTooSmall{..}=>11,TerrainPrimitiveError::TriangleFramebufferTooSmall{..}=>12,TerrainPrimitiveError::TriangleFramebufferExtentOverflow=>13,TerrainPrimitiveError::TriangleZeroGeneralWidth=>14,
 TerrainPrimitiveError::Mode6ClipWidthOutOfRange{..}=>1,TerrainPrimitiveError::Mode6EdgeOutOfRange{..}=>2,TerrainPrimitiveError::Mode6InvertedSpan{..}=>3,TerrainPrimitiveError::Mode6OutputTooSmall{..}=>4,
 e=>panic!("unexpected {e:?}")}}
fn error(e:TerrainModeError)->i32{match e{TerrainModeError::Primitive(e)=>prim(e),TerrainModeError::ClipWidthOutOfRange{..}=>1,TerrainModeError::EdgeOutOfRange{..}=>2,TerrainModeError::InvertedSpan{..}=>3,TerrainModeError::OutputTooSmall{..}=>4,e=>panic!("unexpected {e:?}")}}
fn main(){
 let a:Vec<String>=env::args().collect();let p=Path::new(&a[1]);
 let sources=[(0..65536).map(|i|((i%256*3+i/256*5)&255)as u8).collect::<Vec<u8>>(),fs::read(&a[2]).unwrap(),vec![0;65536],vec![1;65536]];
 let synthetic_fade:Vec<u8>=(0..16384).map(|i|((i%256+7*(i/256))&255)as u8).collect();let actual_fade=fs::read(&a[3]).unwrap();let gla_bytes=fs::read(&a[4]).unwrap();let gla=MainGlaTable::parse(&gla_bytes).unwrap();
 let mut out=BufWriter::new(fs::File::create(p.join("spans-out.bin")).unwrap());
 for b in fs::read(p.join("spans.bin")).unwrap().chunks_exact(56){
  let mode=int(b,0)as usize;let len=int(b,1)as usize;let base=u64::from_le_bytes(b[8..16].try_into().unwrap())as usize;let cap=int(b,4)as usize;let v:Vec<i32>=(5..14).map(|i|int(b,i)).collect();
  let fade=TerrainFadeTable::parse(if mode==1{&actual_fade}else{&synthetic_fade}).unwrap();
  let input=Mode6SpanInput{clip_width:v[0],left_16_16:v[1],right_16_16:v[2],u_16_16:v[3],v_16_16:v[4],shade_16_16:v[5],du_16_16:v[6],dv_16_16:v[7],dshade_16_16:v[8],texture_base:base};
  let mut pixels=vec![Some(165);cap+8];let result=render_mode6_span(&sources[mode][..len],&fade,input,&mut pixels[4..4+cap]);
  let (status,start,count,opaque)=match result{Ok(r)=>(0,r.start_x,r.pixel_count as i32,r.opaque_count as i32),Err(e)=>(error(e),0,0,0)};
  for n in [status,start,count,opaque]{put(&mut out,n)}for b in pixels{out.write_all(&[u8::from(b.is_some()),b.unwrap_or(0)]).unwrap();}
 }out.flush().unwrap();
 let mut tableout=vec![];for d in 0..=255u8{for c in 0..=255u8{tableout.push(gla.map_destination(d,c));}}fs::write(p.join("gla-out.bin"),tableout).unwrap();
 for size in [0,1,65535,65537]{assert!(MainGlaTable::parse(&vec![0;size]).is_err());}
 let mut out=BufWriter::new(fs::File::create(p.join("geometry-out.bin")).unwrap());let fade=TerrainFadeTable::parse(&synthetic_fade).unwrap();
 std::panic::set_hook(Box::new(|_|{}));
 for b in fs::read(p.join("geometry.bin")).unwrap().chunks_exact(80){
  let h=int(b,0);let width=int(b,1);let pitch=int(b,2)as usize;let color=int(b,3)as u8;let mutation=int(b,4);
  let vertices:[Mode5TriangleVertex;3]=core::array::from_fn(|j|Mode5TriangleVertex{x:int(b,5+j*5),y:int(b,6+j*5),u:int(b,7+j*5),v:int(b,8+j*5),shade:int(b,9+j*5)});
  let mut g=match prepare_mode0_15_triangle(vertices.map(|v|TerrainGeometryVertex{x:v.x,y:v.y}),h){Ok(Some(t))=>{put(&mut out,0);t},Ok(None)=>{put(&mut out,15);continue},Err(e)=>{put(&mut out,prim(e));continue}};
  put(&mut out,g.rows.len()as i32);for r in &g.rows{for x in [r.y,r.left_16_16,r.right_16_16]{put(&mut out,x)}}
  let rows_original=g.rows.clone();
  if let Some(r)=g.rows.last_mut(){match mutation{1=>r.y=h,3=>r.left_16_16=i32::MAX,4=>{r.left_16_16=2<<16;r.right_16_16=1<<16},5=>{r.left_16_16=4096<<16;r.right_16_16=4096<<16},6=>{r.left_16_16=-4096<<16;r.right_16_16=-1},_=>{}}}
  if mutation==7{g.clip_height=0;}
  let length=if mutation==9{1}else{pitch*h as usize};let original:Vec<u8>=(0..length+8).map(|i|((i*17+3)&255)as u8).collect();
  for mode in [0,15]{let mut frame=original.clone();let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||if mode==0{draw_mode0_triangle(&g,width,&mut frame[4..4+length],pitch,color)}else{draw_mode15_triangle(&g,&gla,width,&mut frame[4..4+length],pitch,color)}));let(status,count)=match result{Ok(Ok(r))=>(0,r.pixels_written as i32),Ok(Err(e))=>(error(e),0),Err(_)=>(99,0)};put(&mut out,status);put(&mut out,count);put(&mut out,frame.len()as i32);out.write_all(&frame).unwrap();}
  let mut t=match prepare_mode5_triangle(vertices,h){Ok(Some(t))=>{put(&mut out,0);t},Ok(None)=>{put(&mut out,15);continue},Err(e)=>{put(&mut out,prim(e));continue}};
  assert_eq!(t.rows.len(),rows_original.len());
  for (row,geo) in t.rows.iter_mut().zip(&g.rows){row.y=geo.y;row.left_16_16=geo.left_16_16;row.right_16_16=geo.right_16_16;}
  if mutation==2{let r=t.rows.last_mut().unwrap();r.left_16_16=0;r.right_16_16=1<<16;r.u_16_16=1<<16;r.v_16_16=0;r.shade_16_16=64<<16;}
  if mutation==7{t.clip_height=0;}
  let mut frame=original.clone();let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||draw_mode6_triangle(&sources[0],&fade,&t,width,&mut frame[4..4+length],pitch)));let(status,count)=match result{Ok(Ok(r))=>(0,r.pixels_written as i32),Ok(Err(e))=>(prim(e),0),Err(_)=>(99,0)};put(&mut out,status);put(&mut out,count);put(&mut out,frame.len()as i32);out.write_all(&frame).unwrap();
 }out.flush().unwrap();
}

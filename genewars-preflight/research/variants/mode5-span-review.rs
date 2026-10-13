#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
use terrain::{Mode5SpanInput,render_mode5_span,TerrainFadeTable,TerrainPrimitiveError};
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn main(){
 let a:Vec<String>=env::args().collect();let out=Path::new(&a[1]);
 let texture:Vec<u8>=(0..65536).map(|i|((i%256*3+i/256*5)&255)as u8).collect();
 let fade:Vec<u8>=(0..16384).map(|i|((i%256+7*(i/256))&255)as u8).collect();
 let actual_texture=fs::read(&a[2]).unwrap();let actual_fade=fs::read(&a[3]).unwrap();
 let cases=fs::read(out.join("cases.bin")).unwrap();let mut result=BufWriter::new(fs::File::create(out.join("outputs.bin")).unwrap());
 for record in cases.chunks_exact(56){
  let mode=u32::from_le_bytes(record[..4].try_into().unwrap());let len=u32::from_le_bytes(record[4..8].try_into().unwrap())as usize;
  let base=u64::from_le_bytes(record[8..16].try_into().unwrap())as usize;let capacity=u32::from_le_bytes(record[16..20].try_into().unwrap())as usize;
  let mut v=[0i32;9];for i in 0..9{v[i]=i32::from_le_bytes(record[20+i*4..24+i*4].try_into().unwrap());}
  let (src,fd)=if mode==0{(&texture,&fade)}else{(&actual_texture,&actual_fade)};
  let table=TerrainFadeTable::parse(fd).unwrap();let input=Mode5SpanInput{clip_width:v[0],left_16_16:v[1],right_16_16:v[2],u_16_16:v[3],v_16_16:v[4],shade_16_16:v[5],du_16_16:v[6],dv_16_16:v[7],dshade_16_16:v[8],texture_base:base};
  let mut output=vec![0xa5u8;capacity+8];let rendered=render_mode5_span(&src[..len],&table,input,&mut output[4..4+capacity]);
  let (status,start,count)=match rendered{
   Ok(r)=>(0,r.start_x,r.pixel_count as u32),
   Err(TerrainPrimitiveError::NegativeClipWidth{..})=>(1,0,0),
   Err(TerrainPrimitiveError::SpanOutputTooSmall{..})=>(2,0,0),
   Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange{..})=>(3,0,0),
   Err(TerrainPrimitiveError::TextureAddressOverflow)=>(4,0,0),
   Err(TerrainPrimitiveError::TextureSampleOutOfBounds{..})=>(5,0,0),
   Err(e)=>panic!("unexpected error {e:?}"),
  };
  result.write_all(&[status]).unwrap();result.write_all(&start.to_le_bytes()).unwrap();result.write_all(&count.to_le_bytes()).unwrap();result.write_all(&output).unwrap();
 }result.flush().unwrap();
}

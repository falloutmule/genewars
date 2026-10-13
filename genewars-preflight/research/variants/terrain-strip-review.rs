#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
#[path="../../rust/crates/genewars-render/src/terrain_strip.rs"] mod terrain_strip;
use terrain_strip::*;
use genewars_assets::ConqMapCells;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn put(w:&mut impl Write,n:i32){w.write_all(&n.to_le_bytes()).unwrap();}
fn main(){
 let args:Vec<String>=env::args().collect();let dir=Path::new(&args[1]);let maps:Vec<Vec<u8>>=(0..args[2].parse::<usize>().unwrap()).map(|i|fs::read(dir.join(format!("map-{i}.bin"))).unwrap()).collect();
 let cases=fs::read(dir.join("cases.bin")).unwrap();let mut out=BufWriter::new(fs::File::create(dir.join("results.bin")).unwrap());
 for b in cases.chunks_exact(28){
  let v:Vec<i32>=b.chunks_exact(4).map(|x|i32::from_le_bytes(x.try_into().unwrap())).collect();
  let map=ConqMapCells::parse(&maps[v[0]as usize]).unwrap();
  let plan=plan_terrain_strip(&map,[v[1],v[2]],v[3],v[4],v[5],v[6]as u8);
  let plan=match plan{Ok(p)=>{put(&mut out,0);p},Err(e)=>{put(&mut out,match e{TerrainStripError::CameraOutOfRange{..}=>1,TerrainStripError::ColumnOutOfRange{..}=>2,TerrainStripError::VerticalRangeInvalid{..}=>3,TerrainStripError::StopBoundExceeded=>4,_=>panic!("unexpected {e:?}")});continue}};
  put(&mut out,plan.pairs as i32);put(&mut out,plan.events.len()as i32);
  for event in plan.events{
   let mut r=vec![];
   match event{
    TerrainStripEvent::DirtyAnd{world,map_index,mask}=>r.extend([0,world[0],world[1],map_index as i32,mask as i32]),
    TerrainStripEvent::Triangle(t)=>{
     r.extend([1,t.pair_index as i32,t.half as i32,t.mode as i32,t.owner_world[0],t.owner_world[1],t.owner_map_index as i32,t.texture_selector as i32,match t.texture_source{TerrainTextureSource::B32Tile(x)=>{assert_eq!(x,t.texture_selector);0},TerrainTextureSource::OutOfRangeSelector(x)=>{assert_eq!(x,t.texture_selector);1}},t.orientation as i32]);
     r.extend(t.corner_indices.map(i32::from));for p in t.vertex_world{r.extend(p)}for p in t.vertices{r.extend([p.x,p.y,p.u,p.v,p.shade])}
     r.push(t.overlay_table_index.map(|i|i as i32).unwrap_or(-1));
    }}
   assert!(r.len()<=35);r.resize(35,0);for x in r{put(&mut out,x)}
  }
 }
 for (i,map) in maps.iter().enumerate(){assert_eq!(*map,fs::read(dir.join(format!("map-{i}.bin"))).unwrap());}
 out.flush().unwrap();
}

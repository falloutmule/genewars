#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
#[path="../../rust/crates/genewars-render/src/terrain_modes.rs"] mod terrain_modes;
#[path="../../rust/crates/genewars-render/src/terrain_strip.rs"] mod terrain_strip;
#[path="../../rust/crates/genewars-render/src/terrain_view.rs"] mod terrain_view;
use genewars_assets::ConqMapCells;
use std::{env,fs,io::{BufWriter,Write},path::Path};
fn put(w:&mut impl Write,n:usize){w.write_all(&(n as u32).to_le_bytes()).unwrap();}
fn main(){
 let a:Vec<String>=env::args().collect();let p=Path::new(&a[1]);let maps:Vec<Vec<u8>>=(0..a[2].parse::<usize>().unwrap()).map(|i|fs::read(p.join(format!("map-{i}.bin"))).unwrap()).collect();
 let atlases=[fs::read(&a[3]).unwrap(),vec![7;163840]];let fades=[fs::read(&a[4]).unwrap(),(0..16384).map(|i|i as u8).collect()];let glas=[fs::read(&a[5]).unwrap(),(0..65536).map(|i|((i/256+3)&255)as u8).collect()];
 let mut out=BufWriter::new(fs::File::create(p.join("results.bin")).unwrap());
 for b in fs::read(p.join("cases.bin")).unwrap().chunks_exact(28){
  let v:Vec<i32>=b.chunks_exact(4).map(|x|i32::from_le_bytes(x.try_into().unwrap())).collect();let mode=v[6]as usize;
  let map=ConqMapCells::parse(&maps[v[0]as usize]).unwrap();let fade=terrain::TerrainFadeTable::parse(&fades[mode]).unwrap();let gla=terrain_modes::MainGlaTable::parse(&glas[mode]).unwrap();
  match terrain_view::render_full_terrain_view(&map,[v[1],v[2]],&atlases[mode][..v[5]as usize],&fade,if v[4]!=0{Some(&gla)}else{None},v[3]as u8){
   Ok(r)=>{put(&mut out,0);for n in [r.pairs,r.prepared_triangles,r.pixels_written]{put(&mut out,n)}for n in r.requested_modes{put(&mut out,n)}put(&mut out,r.dirty_requests.len());for(i,m)in r.dirty_requests{put(&mut out,i);out.write_all(&[m]).unwrap()}out.write_all(&r.indices).unwrap();},
   Err(e)=>{put(&mut out,1);put(&mut out,e.len());out.write_all(e.as_bytes()).unwrap();}}
 }
 for(i,map)in maps.iter().enumerate(){assert_eq!(*map,fs::read(p.join(format!("map-{i}.bin"))).unwrap());}out.flush().unwrap();
}

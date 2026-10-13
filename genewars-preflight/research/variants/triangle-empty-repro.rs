#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
use terrain::*;
fn main() {
    let v=|x,y|Mode5TriangleVertex{x,y,u:0,v:0,shade:0};
    let t=prepare_mode5_triangle([v(1000,0),v(1010,0),v(1005,3)],3).unwrap().unwrap();
    let b=vec![0;16384]; let f=TerrainFadeTable::parse(&b).unwrap();
    let mut out=vec![165;1920];
    let r=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||draw_mode5_triangle(&[0],&f,&t,640,&mut out,640)));
    println!("panic={} unchanged={}",r.is_err(),out.iter().all(|&v|v==165));
}

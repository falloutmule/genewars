#[path = "../../rust/crates/genewars-assets/src/lib.rs"] mod assets;
use std::{fs,path::Path};
fn main() {
    let args:Vec<String>=std::env::args().collect();
    let raw=fs::read(&args[1]).unwrap();
    let decoded=assets::decode(&raw,163840).unwrap();
    let output=Path::new(&args[2]);fs::write(output.join("decoded.bin"),&decoded).unwrap();
    let atlas=assets::PlanetTextureAtlas::parse(&decoded).unwrap();
    let mut pixels=0;
    for kind in 0..14 {
        let tile=atlas.tile(kind).unwrap();
        assert_eq!(tile.tile_type,kind);
        let side=if kind<8 {128}else{64};
        assert_eq!((tile.width,tile.height,tile.indices.len()),(side,side,side*side));
        let mut mask=Vec::with_capacity(side*side);
        for y in 0..side { for x in 0..side {
            mask.push(u8::from(tile.is_opaque(x,y).unwrap()));
        }}
        assert_eq!(tile.is_opaque(side,0),None);assert_eq!(tile.is_opaque(0,side),None);
        assert_eq!(tile.is_opaque(usize::MAX,usize::MAX),None);
        fs::write(output.join(format!("tile-{kind:02}.indices")),&tile.indices).unwrap();
        fs::write(output.join(format!("tile-{kind:02}.mask")),&mask).unwrap();
        pixels+=tile.indices.len();
    }
    for kind in 14..=255 { assert!(atlas.tile(kind).is_err()); }
    for length in [0,1,163839,163841,327680] {
        assert!(assets::PlanetTextureAtlas::parse(&vec![0;length]).is_err());
    }
    let mut synthetic=vec![0;163840];
    for (i,pixel) in synthetic.iter_mut().enumerate() {*pixel=(i%256) as u8;}
    let tile=assets::PlanetTextureAtlas::parse(&synthetic).unwrap().tile(0).unwrap();
    for x in 0..128 { assert_eq!(tile.is_opaque(x,0),Some(x!=0)); }
    // Public fields may be externally modified; opacity remains panic-free.
    let malformed=assets::PlanetTextureTile {tile_type:0,width:usize::MAX,height:usize::MAX,indices:vec![1]};
    assert_eq!(malformed.is_opaque(1,1),None);
    assert_eq!(malformed.is_opaque(0,usize::MAX-1),None);
    println!("{{\"tiles\":14,\"indices\":{pixels},\"rejected_types\":242,\"malformed_lengths\":5}}");
}

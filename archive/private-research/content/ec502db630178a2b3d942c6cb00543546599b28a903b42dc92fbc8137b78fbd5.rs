#[path="../../rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"]mod family8_scene;
use family8_scene::{EtherealAnimationDatabase,draw_family8_selector2_state10};
use genewars_render::{terrain::TerrainFadeTable,terrain_modes::MainGlaTable};
use std::{fs,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=std::env::args().collect();let resource=EtherealAnimationDatabase::from_bytes(&fs::read(&a[1])?)?;
 let fb=fs::read(&a[2])?;let gb=fs::read(&a[3])?;let fade=TerrainFadeTable::parse(&fb)?;let gla=MainGlaTable::parse(&gb)?;
 let bytes=fs::read(&a[4])?;let original:[u8;102]=bytes[..102].try_into()?;let out=Path::new(&a[5]);fs::create_dir_all(out)?;
 for frame in 0..11u8{for (variant,origin) in [[280,430],[120,700],[1,1],[637,475]].into_iter().enumerate(){
  let mut r=original;r[0x1b]=frame;let mut pixels=vec![0x35;(480-1)*648+640];
  let receipt=draw_family8_selector2_state10(&resource,&r,origin,&fade,&gla,&mut pixels,[640,480],648)?;
  fs::write(out.join(format!("{frame}-{variant}.raw")),pixels)?;
  println!("{frame}-{variant} shadow={} color={} shadow_clipped={} color_clipped={} global_animation={} virtual_height={} effective_shade={} color_y={}",receipt.shadow_pixels,receipt.color_pixels,receipt.shadow_clipped,receipt.color_clipped,receipt.global_animation,receipt.virtual_height,receipt.effective_shade,receipt.color_origin[1]);
 }}
 for (offset,value) in [(6,3),(0x55,0),(0x56,11),(0x56,12),(0x56,13),(0x15,0x30),(0x16,0),(0x19,1),(0x1b,11)]{
  let mut r=original;r[offset]=value;let mut pixels=[0x5a;64];assert!(draw_family8_selector2_state10(&resource,&r,[0,0],&fade,&gla,&mut pixels,[8,8],8).is_err());assert_eq!(pixels,[0x5a;64]);
 }
 println!("atomic_rejections=9");Ok(())
}

#[path="../../rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"]mod family8_scene;
use family8_scene::*;
use genewars_render::{terrain::TerrainFadeTable,terrain_modes::MainGlaTable};
use std::{fs,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>>{
 let a:Vec<_>=std::env::args().collect();let resource=EtherealAnimationDatabase::from_bytes(&fs::read(&a[1])?)?;
 let db=ZoneDecorationDatabase::from_bytes(&fs::read(&a[2])?)?;
 let fb=fs::read(&a[3])?;let gb=fs::read(&a[4])?;let fade=TerrainFadeTable::parse(&fb)?;let gla=MainGlaTable::parse(&gb)?;
 let cases=fs::read(&a[5])?;assert_eq!(cases.len()%145,0);let out=Path::new(&a[6]);fs::create_dir_all(out)?;
 for (index,case) in cases.chunks_exact(145).enumerate(){
  let record:[u8;102]=case[..102].try_into()?;let zone:[u8;19]=case[102..121].try_into()?;
  let context=ZoneDecorationContext{zone_index:record[0x58],zone:&zone,ui_frames:case[121..129].try_into()?,
   source_frame:u32::from_le_bytes(case[133..137].try_into()?),white_index:case[137],type_colors:case[138..145].try_into()?};
  let origin=[i16::from_le_bytes(case[129..131].try_into()?),i16::from_le_bytes(case[131..133].try_into()?)];
  let mut pixels=vec![0x35;(480-1)*648+640];
  draw_family8_selector2_arrival(&resource,&db,&context,&record,origin,&fade,&gla,&mut pixels,[640,480],648)?;
  fs::write(out.join(format!("{index}.raw")),pixels)?;
  if index==0 {
   for variant in 0..9 {
    let mut r=record;let mut z=zone;if variant==5{z[4]=6;}let mut c=ZoneDecorationContext{zone_index:context.zone_index,zone:&z,ui_frames:context.ui_frames,
     source_frame:context.source_frame,white_index:context.white_index,type_colors:context.type_colors};
    match variant{0=>r[0x56]=13,1=>r[0x19]=1,2=>r[0x1b]=11,3=>r[0x55]=1,4=>c.zone_index^=1,
     5=>{},6=>c.ui_frames[1]=8,7=>c.ui_frames[0]=4,_=>{r[0x56]=11;r[0x64]=21;}}
    let mut pixels=vec![0x5a;311032];assert!(draw_family8_selector2_arrival(&resource,&db,&c,&r,origin,&fade,&gla,&mut pixels,[640,480],648).is_err());
    assert!(pixels.iter().all(|p|*p==0x5a));
   }
  }
 }
 println!("cases={} atomic_rejections=9",cases.len()/145);Ok(())
}

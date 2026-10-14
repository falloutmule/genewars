use genewars_runtime::source_building_resources::{SourceBuildingResourceDatabase as Db,SourceBuildingBinding as Binding};
fn main(){
 let args=std::env::args().skip(1).collect::<Vec<_>>();let mut out=Vec::new();
 for (name,binding) in [("BSOLA",Binding::CdBsolaSelector6),("BMINE",Binding::CdBmineSelector11),("BPOWE",Binding::CdBpoweSelector3)]{
  let bytes=std::fs::read(std::path::Path::new(&args[0]).join(format!("{name}.ANB"))).unwrap();
  let db=Db::load(binding,bytes).unwrap();
  for a in 0..2 {for f in db.animation(a).unwrap(){
   out.extend([db.selector(),a as u8,f.index,f.hold,f.metadata.component_count]);
   out.extend(f.metadata.width_raw.to_le_bytes());out.extend(f.metadata.height_raw.to_le_bytes());
   for c in &f.components {
    out.extend(c.record.sprite_offset.to_le_bytes());out.extend(c.record.x_raw.to_le_bytes());out.extend(c.record.y_raw.to_le_bytes());out.extend([c.record.flags,c.record.extra]);
    out.extend([c.sprite.width,c.sprite.height]);out.extend(&c.sprite.indices);out.extend(c.sprite.opaque.iter().map(|b|u8::from(*b)));
   }
  }}
 }
 std::fs::write(&args[1],out).unwrap();
}

use genewars_assets::{FontResource,Palette256,PaletteChannelMode,TextStringTable};
use genewars_render::{RgbImage,measure_text_line,blit_text_line};
use std::{fs,path::Path,io::Write};
fn main(){
 let a:Vec<String>=std::env::args().collect();let dat=fs::read(&a[1]).unwrap();let tab=fs::read(&a[2]).unwrap();
 let output=Path::new(&a[5]);fs::create_dir_all(output).unwrap();
 let font=FontResource::parse(&dat,&tab).unwrap();let mut out=fs::File::create(output.join("glyphs.bin")).unwrap();
 let mut pixels=0;for r in 0..font.record_count(){
  let rec=font.record(r).unwrap();let g=font.decode_glyph(r).unwrap();
  for n in [r as u32,rec.data_offset as u32,g.width as u32,g.height as u32,g.consumed_bytes as u32,g.indices.len() as u32]{out.write_all(&n.to_le_bytes()).unwrap();}
  out.write_all(&g.indices).unwrap();out.write_all(&g.opaque.iter().map(|v|u8::from(*v)).collect::<Vec<_>>()).unwrap();pixels+=g.indices.len();
 }
 assert!(font.record(usize::MAX).is_err());
 for code in 0..=255u8 {let mapped=font.record_for_byte(code);if code<32 || usize::from(code-32)>=font.record_count(){assert!(mapped.is_err());}else{assert_eq!(mapped.unwrap().record,usize::from(code-32));}}
 let bytes=fs::read(&a[3]).unwrap();let strings=TextStringTable::parse(&bytes).unwrap();
 let palette=Palette256::parse(&fs::read(&a[4]).unwrap(),PaletteChannelMode::Rgb).unwrap();
 let mut lines=fs::File::create(output.join("lines.tsv")).unwrap();
 for (index,label) in strings.strings().iter().enumerate(){if let Ok(width)=measure_text_line(&font,label){writeln!(lines,"{index}\t{width}").unwrap();}}
 // Native line surfaces isolate glyph mapping/advance/clipping from the menu.
 for (i,(index,x,y)) in [(255,17,11),(258,-9,37),(259,610,456)].iter().enumerate(){
  if let Ok(width)=measure_text_line(&font,strings.strings()[*index]){
   let mut image=RgbImage::from_rgb24(640,480,vec![91;640*480*3]).unwrap();
   let end=blit_text_line(&mut image,&font,strings.strings()[*index],&palette,*x,*y).unwrap();assert_eq!(end,*x+width);
   fs::write(output.join(format!("line-{i}.rgb")),image.rgb_bytes()).unwrap();
  }
 }
 let table=[0,0,0,0,0,0,0,0,0,0,3,2];
 // First row overlong literals, then controls beyond the width; next row
 // must still start after EOL. Literal index zero remains opaque.
 let data=[5,0,2,3,4,5,255,1,99,0,255,2,7,0,0];
 let f=FontResource::parse(&data,&table).unwrap();let g=f.decode_glyph(0).unwrap();
 assert_eq!(g.indices,[0,2,3,0,7,0]);assert_eq!(g.opaque,[true,true,true,false,true,true]);assert_eq!(g.consumed_bytes,15);
 assert!(FontResource::parse(&data[..14],&table).unwrap().decode_glyph(0).is_err());
 let exact=[3,0,2,3];assert!(FontResource::parse(&exact,&table[..6].iter().copied().chain([0,0,0,0,3,1]).collect::<Vec<_>>()).unwrap().decode_glyph(0).is_err());
 assert!(measure_text_line(&f,b"\n").is_err());
 assert_eq!(strings.strings().len(),604);
 println!("{{\"glyphs\":{},\"pixels\":{pixels},\"text_consumed\":{},\"text_tail\":{},\"synthetic_clipping_passed\":true}}",font.record_count(),strings.consumed_len(),strings.trailing().len());
}

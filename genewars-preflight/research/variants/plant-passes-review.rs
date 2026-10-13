// Native adapter only: expected parsing, placement and raster bytes are Python-owned.
#[path="../../rust/crates/genewars-render/src/terrain.rs"] mod terrain;
#[path="../../rust/crates/genewars-render/src/terrain_modes.rs"] mod terrain_modes;
#[path="../../rust/crates/genewars-render/src/plant.rs"] mod plant;
#[path="../../rust/crates/genewars-render/src/entity_shadow.rs"] mod entity_shadow;
#[path="../../rust/crates/genewars-render/src/plant_draw.rs"] mod plant_draw;
use std::{env,fs,io::{BufWriter,Write},path::Path};
use plant::{PlantFrame,PlantColorInputs,PlantColorRoute};
fn u32out(out:&mut impl Write,n:usize){out.write_all(&(n as u32).to_le_bytes()).unwrap();}
fn main(){
 let a:Vec<String>=env::args().collect();let p=Path::new(&a[1]);
 let paths=fs::read_to_string(p.join("resources.txt")).unwrap();let resources:Vec<Vec<u8>>=paths.lines().map(|s|fs::read(s).unwrap()).collect();
 let frames:Vec<PlantFrame>=fs::read(p.join("frames.bin")).unwrap().chunks_exact(3).map(|s|PlantFrame::parse_indexed8(&resources[s[0]as usize],s[1],s[2]).unwrap()).collect();
 let fades=[fs::read(p.join("fade.bin")).unwrap(),(0..16384).map(|i|((i/256)*13+i*7+3)as u8).collect()];
 let glas=[fs::read(p.join("gla.bin")).unwrap(),(0..65536).map(|i|((i/256)*19+i*7+5)as u8).collect()];
 let mut out=BufWriter::new(fs::File::create(p.join("results.bin")).unwrap());
 for f in &frames{
  u32out(&mut out,f.frame_offset()as usize);let(w,h)=f.display_dimensions();out.write_all(&w.to_le_bytes()).unwrap();out.write_all(&h.to_le_bytes()).unwrap();u32out(&mut out,f.component_count());
  for c in f.components(){out.write_all(&c.local_x.to_le_bytes()).unwrap();out.write_all(&c.local_y.to_le_bytes()).unwrap();out.write_all(&[c.sprite.width,c.sprite.height]).unwrap();u32out(&mut out,c.sprite.consumed_bytes);out.write_all(&c.sprite.indices).unwrap();out.write_all(&c.sprite.opaque.iter().map(|&b|u8::from(b)).collect::<Vec<_>>()).unwrap();}
 }
 let cases=fs::read(p.join("cases.bin")).unwrap();
 for b in cases.chunks_exact(76){
  let v:Vec<i32>=b.chunks_exact(4).map(|x|i32::from_le_bytes(x.try_into().unwrap())).collect();
  let [w,h,pitch,offset]=[v[7]as usize,v[8]as usize,v[9]as usize,v[10]as usize];
  let minimum=if w==0||h==0{0}else{(h-1)*pitch+w};let extent=(minimum as i64+i64::from(v[11])).max(0)as usize;
  let mut dest:Vec<u8>=(0..offset+extent+3).map(|i|(i*37+v[17]as usize)as u8).collect();let before=dest.clone();
  let fade=terrain::TerrainFadeTable::parse(&fades[v[18]as usize]).unwrap();let gla=terrain_modes::MainGlaTable::parse(&glas[v[18]as usize]).unwrap();
  let mut header=[0i64;6];let status;
  if v[0]==0||v[0]==2{
   let inputs=PlantColorInputs{context_flags_at_16e0c4:v[12]as u8,entity_flags_at_15:v[13]as u8,native_flags_at_2e:v[14]as u8,shade_at_17:v[15]as u8};
   if v[0]==2{match plant_draw::draw_normal_plant(&frames[v[1]as usize],&mut dest[offset..offset+extent],[w,h],pitch,[v[2],v[3]],inputs,if v[16]!=0{Some(&fade)}else{None},&gla){
    Ok(r)=>{status=0;header[0]=match r.foreground_route{PlantColorRoute::Direct=>-1,PlantColorRoute::Fade{shade}=>shade as i64};header[1]=r.shadow_pixels_written as i64;header[2]=r.shadow_pixels_clipped as i64;header[3]=(r.foreground_pixels_written+r.foreground_pixels_clipped)as i64;header[4]=r.foreground_pixels_written as i64;header[5]=r.foreground_pixels_clipped as i64;assert_eq!(r.component_shadows.len(),frames[v[1]as usize].component_count());assert_eq!(r.component_shadows.iter().map(|r|r.pixels_written).sum::<usize>(),r.shadow_pixels_written);assert_eq!(r.component_shadows.iter().map(|r|r.clipped_opaque_samples).sum::<usize>(),r.shadow_pixels_clipped);},Err(_)=>{status=1;assert_eq!(dest,before);}}
   }else{match frames[v[1]as usize].draw_color_pass(&mut dest[offset..offset+extent],w,h,pitch,[v[2],v[3]],inputs,if v[16]!=0{Some(&fade)}else{None}){
    Ok(r)=>{status=0;header[0]=match r.route{PlantColorRoute::Direct=>-1,PlantColorRoute::Fade{shade}=>shade as i64};header[3]=(r.opaque_pixels_written+r.opaque_pixels_clipped)as i64;header[4]=r.opaque_pixels_written as i64;header[5]=r.opaque_pixels_clipped as i64;assert_eq!(r.shadow,plant::PlantShadowStatus::RequiredButNotRendered);},Err(_)=>{status=1;assert_eq!(dest,before);}}
   }
  }else{
   let c=frames[v[1]as usize].components().next().unwrap();
   match entity_shadow::draw_normal_component_shadow(c.sprite,&gla,[v[2]as i16,v[3]as i16],[v[4]as i16,v[5]as i16],v[6]as u8,[w,h],pitch,&mut dest[offset..offset+extent]){
    Ok(r)=>{status=0;header=[r.component_position[0]as i64,r.component_position[1]as i64,r.shadow_height as i64,r.opaque_samples as i64,r.pixels_written as i64,r.clipped_opaque_samples as i64];},Err(_)=>{status=1;assert_eq!(dest,before);}}
  }
  u32out(&mut out,status);for n in header{out.write_all(&n.to_le_bytes()).unwrap();}u32out(&mut out,dest.len());out.write_all(&dest).unwrap();
 }
 // Malformed public sprite planes and impossible destination arithmetic are atomic.
 let gla=terrain_modes::MainGlaTable::parse(&glas[1]).unwrap();let original=frames.last().unwrap().components().next().unwrap().sprite.clone();let mut errors=0;
 for plane in 0..4{let mut sprite=original.clone();match plane{0=>{sprite.indices.pop();},1=>{sprite.opaque.pop();},2=>sprite.indices.push(0),_=>sprite.opaque.push(false)}let mut d=[99;4];assert!(entity_shadow::draw_normal_component_shadow(&sprite,&gla,[0,0],[0,0],0,[1,1],1,&mut d).is_err());assert_eq!(d,[99;4]);errors+=1;}
 for(clip,pitch)in[([1,2],usize::MAX),([2,1],1),([641,1],641),([1,481],1),([0,1],1),([1,0],1)]{let mut d=[99;4];assert!(entity_shadow::draw_normal_component_shadow(&original,&gla,[0,0],[0,0],0,clip,pitch,&mut d).is_err());assert_eq!(d,[99;4]);errors+=1;}
 let inputs=PlantColorInputs{context_flags_at_16e0c4:0,entity_flags_at_15:2,native_flags_at_2e:0,shade_at_17:0};
 for(w,h,pitch)in[(1,3,usize::MAX),(2,1,1),(usize::MAX,1,usize::MAX),(1,usize::MAX,1)]{let mut d=[99;4];assert!(frames[0].draw_color_pass(&mut d,w,h,pitch,[0,0],inputs,None).is_err());assert_eq!(d,[99;4]);errors+=1;}
 u32out(&mut out,errors);
 let mut rejected=0;
 for resource in resources.iter().take(12){for size in 0..resource.len(){assert!(PlantFrame::parse_indexed8(&resource[..size],0,0).is_err());rejected+=1;}let mut extra=resource.clone();extra.push(0);assert!(PlantFrame::parse_indexed8(&extra,0,0).is_err());rejected+=1;for i in 9..=255{assert!(PlantFrame::parse_indexed8(resource,i,0).is_err());rejected+=1;}for i in 1..=255{assert!(PlantFrame::parse_indexed8(resource,0,i).is_err());rejected+=1;}}
 for file in fs::read_to_string(p.join("invalid.txt")).unwrap().lines(){assert!(PlantFrame::parse_indexed8(&fs::read(file).unwrap(),0,0).is_err(),"{file}");rejected+=1;}
 u32out(&mut out,rejected);
 for path in fs::read_to_string(p.join("missions.txt")).unwrap().lines(){let bytes=fs::read(path).unwrap();let source=genewars_assets::ConqMissionSource::parse(&bytes).unwrap();for i in 0..512{let f=source.family1_sprite_fields(i).unwrap();assert_eq!(f.record_index,i);for n in f.signed_position_words{out.write_all(&n.to_le_bytes()).unwrap();}out.write_all(&[f.flags_at_15,f.shade_at_17,f.animation_selector,f.frame_selector,f.resource_slot,f.signed_resource_selector as u8,f.native_flags_at_2e]).unwrap();}for i in [512,usize::MAX]{assert!(source.family1_sprite_fields(i).is_err());}assert_eq!(bytes,fs::read(path).unwrap());}
 out.flush().unwrap();
}

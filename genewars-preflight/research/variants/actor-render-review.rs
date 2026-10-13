use genewars_assets::{AnbFrameRecord,AnbComponentRecord,IndexedSprite};
use genewars_runtime::{actor_render::*,actor_resources::ActorResourceCatalog,actor_initialization::{BlueprintRecordHandle,ActorResourceTableKey},player_state::PlayerState};
use std::{fs,io::{BufWriter,Write},path::Path};
fn record(frame:AnbFrameRecord,components:Vec<AnbComponentRecord>)->ActorRenderRecord{
 ActorRenderRecord{actor_slot:0,actor_id:0,owner:0,blueprint:BlueprintRecordHandle{player_id:0,blueprint_index:0},resource_key:ActorResourceTableKey{style:0,category:0,flat_index:0},world_position:[0,0],virtual_height:0,shade:30,actor_flags_15:0,animation_index:0,frame_index:0,draw_flags_55:0,draw_flag_58:0,draw_flag_59:0,draw_flags_70:0,draw_flag_7e:0,draw_flag_82:0,draw_mode_73:0,frame_record:frame,components}
}
fn frame()->AnbFrameRecord{AnbFrameRecord{frame_offset:0,component_offset:0,component_count:1,width_raw:0,height_raw:0,byte_10:0,trailing_bytes:[0;2]}}
fn draw(out:&mut impl Write,r:&ActorRenderRecord,d:&[ActorDecodedComponent],ctx:ActorDrawContext,w:usize,h:usize,p:usize,fade:Option<&[u8]>,blend:Option<&[u8]>){
 let size=if h==0||w==0{0}else{(h-1)*p+w};let mut image:Vec<u8>=(0..size+9).map(|i|(i*37+19)as u8).collect();let old=image.clone();
 let result=draw_actor_components(r,d,ctx,fade,blend,&mut image,w,h,p);
 match result{Ok(v)=>{out.write_all(&[1]).unwrap();for n in [v.pixels_written,v.pixels_clipped,v.opaque_pixels]{out.write_all(&(n as u32).to_le_bytes()).unwrap();}},Err(_)=>{assert_eq!(image,old);out.write_all(&[0]).unwrap();out.write_all(&[0;12]).unwrap();}}
 out.write_all(&image).unwrap();
}
fn main(){
 let args:Vec<String>=std::env::args().collect();let inputs=Path::new(&args[1]);let work=Path::new(&args[2]);let cat=ActorResourceCatalog;
 let fade:Vec<u8>=(0..256).map(|i|(i*73+17)as u8).collect();let blend:Vec<u8>=(0..65536).map(|i|((i/256)*3+(i%256)*7+11)as u8).collect();
 let mut sprites=BufWriter::new(fs::File::create(work.join("sprites.bin")).unwrap());let mut images=BufWriter::new(fs::File::create(work.join("frames.bin")).unwrap());
 for spec in cat.specs(){let db=cat.load(spec.key,fs::read(inputs.join(format!("key-{:02}.anb",spec.key.flat_index))).unwrap()).unwrap();
  for i in 0..db.sprite_record_count(){let s=db.decode_sprite_indexed8(i).unwrap();sprites.write_all(&[s.width,s.height]).unwrap();sprites.write_all(&(s.consumed_bytes as u32).to_le_bytes()).unwrap();sprites.write_all(&s.indices).unwrap();sprites.write_all(&s.opaque.iter().map(|&b|b as u8).collect::<Vec<_>>()).unwrap();}
  for a in 0..db.animation_count(){for f in 0..db.frame_count(a).unwrap(){let fr=db.frame(a,f).unwrap();let cs=(0..fr.component_count as usize).map(|i|db.component(a,f,i).unwrap()).collect();let mut r=record(fr,cs);r.resource_key=spec.key;r.animation_index=a as u8;r.frame_index=f as u8;r.owner=spec.key.flat_index%4;let d=decode_actor_components_indexed8(&r,&db).unwrap();
   for n in 0..4 {r.shade=if n%2==0{29}else{30};draw(&mut images,&r,&d,ActorDrawContext{origin:[[40,32],[-1,0],[79,63],[i32::MAX,i32::MIN]][n],virtual_height:128,global_flags:[0,3,4,12][n]},80,64,84,Some(&fade),Some(&blend));}
  }}
 }
 sprites.flush().unwrap();images.flush().unwrap();
 let mut synthetic=BufWriter::new(fs::File::create(work.join("synthetic.bin")).unwrap());
 let c=AnbComponentRecord{sprite_offset:0,x_raw:-3,y_raw:3,flags:0,extra:0};
 let s=IndexedSprite{width:5,height:3,indices:vec![0,192,207,208,255,193,1,194,195,196,2,197,198,199,3],opaque:vec![true,true,true,true,false,true,false,true,true,true,true,true,false,true,true],consumed_bytes:0};
 for global in 0..16u8{for component in 0..16u8{for owner in 0..4u8{for shade in [0,29,30,255]{for origin in [[3,2],[-2,-1],[8,6],[i32::MIN,i32::MAX],[i32::MAX,i32::MIN]]{
  let c=AnbComponentRecord{flags:component,..c};let d=vec![ActorDecodedComponent{component:c,sprite:s.clone()},ActorDecodedComponent{component:AnbComponentRecord{x_raw:1,y_raw:-1,..c},sprite:s.clone()}];let mut r=record(frame(),d.iter().map(|d|d.component).collect());r.owner=owner;r.shade=shade;
  draw(&mut synthetic,&r,&d,ActorDrawContext{origin,virtual_height:128,global_flags:global},9,7,13,Some(&fade),Some(&blend));
 }}}}}
 // Missing dependencies and late malformed components must preserve every byte.
 for n in 0..8{let mut r=record(frame(),vec![c,c]);r.shade=if n<2{0}else{30};let mut d=vec![ActorDecodedComponent{component:c,sprite:s.clone()};2];let mut ctx=ActorDrawContext{origin:[2,2],virtual_height:0,global_flags:0};
  match n{0=>{},1=>ctx.virtual_height=-128,2=>ctx.global_flags=4,3=>{d[1].component.flags=0x80;r.components[1].flags=0x80},4=>{d[1].sprite.opaque.pop();},5=>{d[1].component.x_raw=2;},6=>ctx.global_flags=0x10,7=>{d[1].sprite.indices.pop();},_=>unreachable!()};
  draw(&mut synthetic,&r,&d,ctx,9,7,13,if n==0{None}else{Some(&fade)},if n==2{None}else{Some(&blend)});
 }
 synthetic.flush().unwrap();
 let mut policies=BufWriter::new(fs::File::create(work.join("policies.bin")).unwrap());
 for n in 0..65536u32{let mut r=record(frame(),vec![]);r.actor_id=n as u16;r.owner=((n>>14)&3)as u8;r.blueprint.player_id=r.owner;r.shade=n as u8;r.actor_flags_15=((n>>7)&0x12)as u8;r.draw_flags_55=((n>>4)&7)as u8;r.draw_flag_58=(n>>5)as u8;r.draw_flag_59=(n>>8)as i8;r.draw_flags_70=n.wrapping_mul(257)as u16;r.draw_flag_7e=((n>>6)&7)as u8;r.draw_flag_82=((n>>10)&1)as u8;r.draw_mode_73=(n>>8)as u8;
  let p=PlayerState::fresh_campaign(r.owner,[0,0],b"review\0").unwrap();let mut b=*p.bytes();b[0x16+0x15]=(n&3)as u8;b[0x16+0x16]=((n>>2)&3)as u8;let p=PlayerState::from_record(&b).unwrap();let height=((n*97)%1024)as i32-512;
  match plan_actor_draw(&r,&p,((n>>12)&3)as u8,(n>>13)as u8,[-32768,32767],height,if n&1==0{ActorWrapperKind::World}else{ActorWrapperKind::PortraitDirect}){Ok(v)=>{policies.write_all(&[1,v.visible as u8,v.context.global_flags,v.shadow_required as u8]).unwrap();for x in [v.context.origin[0],v.context.origin[1],v.effective_shade,v.fade_row.unwrap_or(i32::MIN)]{policies.write_all(&x.to_le_bytes()).unwrap();}},Err(_)=>policies.write_all(&[0;20]).unwrap()};
 }policies.flush().unwrap();
}

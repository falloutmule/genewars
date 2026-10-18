//! Bounded source-bound Genepod action8 control. Host owns world selection,
//! enabled-control traversal and manager origin; this is not a complete panel.
use genewars_assets::{IndexedSprite,SpriteTable};
use crate::capture as genepod_capture;use crate::capture::{CaptureState,CapturePlan,ReadyPlan};
const ANB_HASH:&str="1df1e8fb436cc7030c40b94438c5e530a1b8a21716fad4354e0c82c94ba27f52";
#[derive(Clone,Debug)]pub struct GenepodControl{sprite:IndexedSprite}
#[derive(Clone,Debug,Default)]pub struct PanelInput{pub capture:CaptureState}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct ControlFeedback{pub captured:bool,pub pressed:bool,pub event:u16,pub action:u16}
impl GenepodControl{
 pub fn parse(anb:&[u8])->Result<Self,String>{
  if genewars_runtime::identity::sha256_hex(anb)!=ANB_HASH{return Err("GENEPANL.ANB reference hash mismatch".into());}
  let sprite=SpriteTable::from_anb(anb).map_err(|e|e.to_string())?.decode(1,anb).map_err(|e|e.to_string())?;
  if sprite.width!=29||sprite.height!=29||sprite.opaque.iter().filter(|&&b|b).count()!=665{return Err("GENEPANL action8 descriptor mismatch".into());}
  Ok(Self{sprite})
 }
 /// Caller supplies the original manager origin, not a guessed absolute point.
 pub fn hit(&self,parent:[i32;2],pointer:Option<[i32;2]>)->bool{
  let Some(p)=pointer else{return false;};let x=i64::from(p[0])-i64::from(parent[0])-26;let y=i64::from(p[1])-i64::from(parent[1])-344;
  (0..29).contains(&x)&&(0..29).contains(&y)&&self.sprite.opaque[y as usize*29+x as usize]
 }
 /// Draws only the verified sprite1 control face. Original22FD0 animation,
 /// disabled tint and pressed variants remain unimplemented, not synthesized.
 pub fn draw_face(&self,parent:[i32;2],size:[usize;2],pitch:usize,pixels:&mut[u8])->Result<usize,String>{
  if pitch<size[0]||size[1].checked_mul(pitch).is_none_or(|n|n>pixels.len()){return Err("invalid indexed control target".into());}
  let mut written=0;for y in 0..29usize{for x in 0..29usize{let i=y*29+x;if !self.sprite.opaque[i]{continue;}
   let dx=i64::from(parent[0])+26+x as i64;let dy=i64::from(parent[1])+344+y as i64;
   if dx>=0&&dy>=0&&(dx as usize)<size[0]&&(dy as usize)<size[1]{pixels[dy as usize*pitch+dx as usize]=self.sprite.indices[i];written+=1;}
  }}Ok(written)
 }
}
impl PanelInput{
 /// Enabled isolated control only. Use after host hit-order/disabled admission.
 /// Returns low16 action; host invokes the producer only for action8 and uses
 /// its returned event flags unchanged. No production mutation occurs here.
 pub fn poll(&mut self,control:&GenepodControl,parent:[i32;2],pointer:Option<[i32;2]>,packet8:u8)->Result<ControlFeedback,String>{
  let CapturePlan{after,action}=genepod_capture::plan_capture(self.capture,packet8,control.hit(parent,pointer)).map_err(|e|format!("{e:?}"))?;
  self.capture=after;Ok(ControlFeedback{captured:after.captured,pressed:after.draw!=0,event:after.event,action})
 }
 pub fn ready(panel_state:u8,building:&[u8;67])->ReadyPlan{genepod_capture::plan_ready(panel_state,u16::from_le_bytes([building[49],building[50]]),building[27],i32::from_le_bytes(building[54..58].try_into().unwrap()))}
}

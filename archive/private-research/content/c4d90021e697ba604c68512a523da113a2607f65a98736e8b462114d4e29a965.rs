//! Source451F0 selector3/6 numeric prefix through47F2C, before queue/animation.
//! Effect0x21 (decimal33) requests must be fulfilled atomically by the owner;
//! no recipient energy is credited here. Unsupported states never partially commit.
use crate::blueprint::BlueprintRng;
pub type PowerBuilding=[u8;67];
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum PowerError{UnsupportedRecord,InvalidPartition,InvalidPlayer,InvalidRng}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum PowerAudio{
 /// Virtual44: code0x3D, mode2, parameter0, volume127, range100.
 Distributor61,
 /// Virtual48: code0x23.
 StopPhoton35,
 /// Virtual44: code0x23, mode2, parameter255, volume90.
 Photon35{range:u16}
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PowerTransfer33Request{pub destination:[u8;3],pub source_position:[i16;2],pub destination_position:[i16;2],pub payload:u8,pub yaw:u16,pub speed:u16,pub pitch:u16,pub height_above_ground:i32}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PowerPlan{pub before:[PowerBuilding;64],pub after:[PowerBuilding;64],pub player_before:[u8;1805],pub player_after:[u8;1805],pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,pub transfers:Vec<PowerTransfer33Request>,pub audio:Vec<PowerAudio>,pub queue_and_animation_required:bool}
fn word(b:&[u8],o:usize)->i16{i16::from_le_bytes(b[o..o+2].try_into().unwrap())}
fn put(b:&mut[u8],o:usize,v:i16){b[o..o+2].copy_from_slice(&v.to_le_bytes())}
fn point(b:&[u8])->[i16;2]{[word(b,7),word(b,9)]}
fn distance(a:[i16;2],b:[i16;2])->i32{let x=i32::from(a[0].wrapping_sub(b[0]).wrapping_mul(2)>>1);let y=i32::from(a[1].wrapping_sub(b[1]).wrapping_mul(2)>>1);x*x+y*y}
fn draw(r:&mut BlueprintRng)->u16{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=(r.ring+1)&31;r.draw_count=r.draw_count.wrapping_add(1);r.seed>>8}
/// `partition` is the owner's exact contiguous64 native slots in source order.
/// `solar_generation` is mission global11A9BF, not an invented default.
/// Only running completed grades0..2, without disabling/destruction bits,
/// are admitted. Stale temporary target marks reject rather than invent origin.
pub fn plan_power_prefix(partition:&[PowerBuilding;64],slot:usize,player:&[u8;1805],rng:BlueprintRng,frame:u32,solar_generation:i16,physics:&crate::physics_tables::PhysicsTables)->Result<PowerPlan,PowerError>{
 let before=partition.get(slot).ok_or(PowerError::InvalidPartition)?;let owner=before[24];let selector=before[45];
 if before[6]!=4||owner>=4||!matches!(selector,3|6)||before[46]>2||u16::from_le_bytes(before[49..51].try_into().unwrap())&!4!=1{return Err(PowerError::UnsupportedRecord)}
 if player[1]!=owner||player[0x4a1]!=owner{return Err(PowerError::InvalidPlayer)}
 if rng.ring>=32{return Err(PowerError::InvalidRng)}
 for(i,b)in partition.iter().enumerate(){if b[49]&16!=0||b[6]!=0&&(b[6]!=4||b[24]!=owner||word(b,15)!=i16::from(owner)*64+i as i16){return Err(PowerError::InvalidPartition)}}
 let mut after=*partition;let mut p=*player;let mut r=rng;let mut a=*before;let mut audio=Vec::new();let mut transfers=Vec::new();
 // Both selected source upkeep-table entries104D10 are zero.
 let mut energy=word(&a,61).wrapping_sub(i16::from(a[63]));
 a[49]|=4;a[28]=u8::from(a[27]!=0);
 let generated=if selector==3{i16::from(a[46])*4}else{solar_generation.wrapping_add(i16::from(a[46]))};
 let generated=if selector==3{generated.max(2)}else{generated};
 energy=energy.wrapping_add(generated);let total=word(&p,0x4ad).wrapping_add(generated);put(&mut p,0x4ad,total);
 let pulse=if selector==3{if energy>=2000{energy=2000};(frame&63)==u32::from(a[46])*16||energy==2000}else{a[29]=3;if a[27]==0{audio.push(PowerAudio::StopPhoton35)};energy>199};
 if pulse{
  let n=draw(&mut r);if selector==3{if n%7==0{a[27]=1;audio.push(PowerAudio::Distributor61)}}else if n%50==0&&a[27]==0{a[27]=1;audio.push(PowerAudio::Photon35{range:400-u16::from(a[46])*40})}
  let origin=point(&a);let mut selected=Vec::new();
  for(i,b)in after.iter().enumerate(){let flags=u16::from_le_bytes(b[49..51].try_into().unwrap());let eligible=if selector==3{b[6]!=0&&flags&1!=0&&flags&0x100==0&&!matches!(b[45],3|6|8|9)}else{b[6]!=0&&flags&1!=0&&b[45]==3};let radius=1536+768*i32::from(if selector==3{a[46]}else{b[46]});if eligible&&distance(origin,point(b))<=radius*radius{selected.push(i)}}
  if selector==3{a[54..58].copy_from_slice(&(selected.len()as u32).to_le_bytes())}
  if !selected.is_empty(){let share=if selector==3{(i32::from(energy)/selected.len()as i32)as u16}else{200/selected.len()as u16};
   for i in selected{if selector==6||share!=0{let b=&after[i];transfers.push(PowerTransfer33Request{destination:[b[15],b[16],b[6]],source_position:origin,destination_position:point(b),payload:share.min(255)as u8,yaw:crate::actor_motion::angle(i32::from(point(b)[0].wrapping_sub(origin[0]).wrapping_mul(2)),i32::from(point(b)[1].wrapping_sub(origin[1]).wrapping_mul(2)),physics.angles())as u16,speed:2048,pitch:512,height_above_ground:512})}}
   energy=0;
  }else if selector==6{energy=0}
 }
 put(&mut a,61,energy);if a[63]!=0{a[63]-=1}
 if a[28]!=0||(before[49]^a[49])&4!=0||a[48]!=0{a[21]|=4}
 if a[48]!=0{a[48]-=1};a[25]=a[46]+1;after[slot]=a;
 Ok(PowerPlan{before:*partition,after,player_before:*player,player_after:p,rng_before:rng,rng_after:r,transfers,audio,queue_and_animation_required:true})
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PowerDeliveryPlan{pub effect_before:[u8;81],pub effect_after:[u8;81],pub target_before:Option<PowerBuilding>,pub target_after:Option<PowerBuilding>,pub retire_requested:bool}
/// Effect0x21 dispatch4C8E6 AFTER ordinary physics and animation. Ground is
/// sampled at the postphysics position. Retirement/list cleanup is a request.
/// Selector12's two extra effect requests remain explicitly unsupported.
pub fn plan_power_delivery(effect:&[u8;81],target:Option<&PowerBuilding>,ground:i32)->Result<PowerDeliveryPlan,PowerError>{
 if effect[6]!=5||effect[69]!=33||!matches!(effect[73],0|4){return Err(PowerError::UnsupportedRecord)}
 if (effect[73]==4)!=target.is_some(){return Err(PowerError::InvalidPartition)}
 if let Some(t)=target{if t[6]!=4||t[15..17]!=effect[71..73]{return Err(PowerError::InvalidPartition)}}
 let mut a=*effect;let mut after=target.copied();let mut retire=false;a[39..43].copy_from_slice(&ground.wrapping_add(512).to_le_bytes());
 if let Some(t)=after.as_mut().filter(|t|distance(point(effect),point(*t))<=16384){
  if t[45]==12&&t[60]&4!=0{return Err(PowerError::UnsupportedRecord)}
  let value=word(t,61).wrapping_add(i16::from(effect[76]));let cap=if t[45]==3{2000}else{200};put(t,61,value.min(cap));t[49]|=4;retire=true;
 }else{a[76]=a[76].wrapping_sub(1);if a[76]==0{retire=true}}
 Ok(PowerDeliveryPlan{effect_before:*effect,effect_after:a,target_before:target.copied(),target_after:after,retire_requested:retire})
}

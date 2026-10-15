//!5A210 species4 factory numeric initialization before8AC60 link insertion.
//! Caller selects first inactive slot and validates count/capacity. Pointer bytes
//! remain logical handles. Counts, map insertion and presentation are world-owned.
use crate::{blueprint::BlueprintRng,creature_world::CreatureRecord,actor_activation::TrigTables};
#[derive(Debug,Clone,PartialEq,Eq)]pub enum MuleFactoryError{Record,Input}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct MuleFactoryPlan{pub record_after:CreatureRecord,pub rng_after:BlueprintRng}
#[derive(Debug,Clone,Copy)]pub struct MuleFactoryInput{pub slot:u16,pub owner:u8,pub position:[i16;2],pub quality:u8,pub ground:i32,pub shade:u8}
pub fn plan_mule_factory(prior:&CreatureRecord,input:MuleFactoryInput,parent:Option<&CreatureRecord>,rng_before:BlueprintRng,trig:&TrigTables)->Result<MuleFactoryPlan,MuleFactoryError>{
 fn put(a:&mut CreatureRecord,o:usize,v:i16){a[o..o+2].copy_from_slice(&v.to_le_bytes());}
 fn put32(a:&mut CreatureRecord,o:usize,v:i32){a[o..o+4].copy_from_slice(&v.to_le_bytes());}
 fn draw(r:&mut BlueprintRng)->i32{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=r.ring.wrapping_add(1)&31;r.draw_count=r.draw_count.wrapping_add(1);i32::from(r.seed>>8)}
 if prior[6]!=0{return Err(MuleFactoryError::Record);}
 if input.slot>=300||input.owner>4||input.quality>100||input.position.iter().any(|x|*x<0)||rng_before.ring>=32{return Err(MuleFactoryError::Input);}
 if parent.is_some_and(|p|p[6]!=1||p[0x7a]!=4){return Err(MuleFactoryError::Record);}
 let mut r=rng_before;let speed=(draw(&mut r)%20)<<6;let pitch=draw(&mut r)%256;let yaw=(draw(&mut r)%256)<<3;
 let mut a=*prior;a[0x7a]=4;a[2]=0;a[5]=0;a[6]=1;put(&mut a,15,input.slot as i16);a[24]=input.owner;
 put(&mut a,7,input.position[0]);put(&mut a,9,input.position[1]);put32(&mut a,0x11,input.ground);a[0x17]=input.shade;
 put(&mut a,11,0);put(&mut a,13,0);a[0x19]=(((((yaw>>7)+1)>>1)+1)&7)as u8;
 a[0x1a]=0;a[0x1b]=0;a[0x1c]=1;a[0x1d]=0;
 put(&mut a,0x39,yaw as i16);put(&mut a,0x3b,pitch as i16);put(&mut a,0x3d,speed as i16);
 let pitch_i=((pitch+1536)&2047)as usize;let yaw_i=((yaw+1536)&2047)as usize;
 let horizontal=(i64::from(trig.cosine()[pitch_i])*i64::from(speed)>>16)as i32;
 put(&mut a,0x2d,(i64::from(trig.cosine()[yaw_i])*i64::from(horizontal)>>16)as i16);
 put(&mut a,0x2f,(i64::from(trig.sine()[yaw_i])*i64::from(horizontal)>>16)as i16);
 put(&mut a,0x31,(-i64::from(trig.sine()[pitch_i])*i64::from(speed)>>16)as i16);
 a[0x33..0x39].fill(0);let height=parent.map_or(input.ground,|p|i32::from_le_bytes(p[0x27..0x2b].try_into().unwrap()));put32(&mut a,0x27,height);
 put(&mut a,0x15,0x280);put(&mut a,0x60,0);put(&mut a,0x4b,yaw as i16);put(&mut a,0x2b,80);a[0x53]=1;a[0x54]=0;a[0x4a]&=0x3b;
 a[0x76]=input.quality;let skill=|minimum:i32|minimum+i32::from(input.quality)*(100-minimum)/100;
 let life=19000*skill(30)/100;let energy=if parent.is_some(){life}else{life-(life>>4)};put(&mut a,0x66,energy as i16);
 let reserve=life>>3;let threshold=life-reserve;a[0x79]=0;
 if energy<=threshold{a[0x77]=255;let stage=(energy-reserve)/(threshold/3)+1;a[0x79]=stage as u8;}
 else{a[0x77]=((life-energy)*255/reserve).max(16)as u8;let hunger=i32::from(a[0x77])+1;put(&mut a,0x62,(((160*skill(40)/100)*hunger>>8)+32)as i16);put(&mut a,0x64,(((80*skill(30)/100)*hunger>>8)+16)as i16);}
 if a[0x77]==255{ return Err(MuleFactoryError::Input); } // Fresh factory domain must not fabricate retained maxima.
 a[0x5e]=8;a[0x5f]=0;put(&mut a,0x6c,0);let maxima=a; a[0x6a..0x6c].copy_from_slice(&maxima[0x64..0x66]);a[0x68..0x6a].copy_from_slice(&maxima[0x62..0x64]);
 a[0x7b]=255;a[0x61]&=0x7f;a[0x78]=parent.map_or(0,|p|p[0x78]);
 put(&mut a,0x3f,0);let flags=u16::from_le_bytes([a[0x49],a[0x4a]])&0xefe8;put(&mut a,0x49,flags as i16);a[0x54]=0;a[0x16]&=0x3f;
 Ok(MuleFactoryPlan{record_after:a,rng_after:r})
}

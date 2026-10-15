//! Owned Mule common fallback64CD7..5EF46, after the owned decision.
//! Physics, prior scan draws, lifecycle and the timer-decrement tail are external.
use crate::blueprint::BlueprintRng;
use crate::actor_activation::TrigTables;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ChoiceError { Identity, UnsupportedState, InvalidRng, TargetSearchRequired, QueryUnavailable }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ChoiceContext {
 pub common_choice_eligible:bool,
 /// Source virtual+B0 and virtual+8C results, evaluated on postdecision state.
 pub virtual_b0:bool,pub current_cell_requires_escape:bool,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ChoicePlan {
 pub before:[u8;124],pub after:[u8;124],pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,
 pub draw_values:Vec<u8>,pub target:Option<[i16;2]>,
}
fn word(r:&[u8;124],o:usize)->i16{i16::from_le_bytes([r[o],r[o+1]])}
fn put(r:&mut[u8;124],o:usize,v:i16){r[o..o+2].copy_from_slice(&v.to_le_bytes());}
fn draw(r:&mut BlueprintRng,values:&mut Vec<u8>)->u8{r.seed=r.seed.wrapping_mul(0xe62d);r.ring=(r.ring+1)&31;r.draw_count=r.draw_count.wrapping_add(1);let v=(r.seed>>8)as u8;values.push(v);v}
/// Admits the first accepted point only; a rejected point requires the bounded
/// original retry/search path, never a fabricated target. Query is74CC0.
pub fn plan_choice<F>(before:&[u8;124],rng:BlueprintRng,tables:&TrigTables,c:ChoiceContext,mut accept:F)->Result<ChoicePlan,ChoiceError>
where F:FnMut([i16;2],u8)->Result<bool,ChoiceError>{
 if before[6]!=1||before[0x18]>=4||before[0x7a]!=4{return Err(ChoiceError::Identity);}
 if !c.common_choice_eligible||before[0x5e]!=8||before[0x61]&0x80!=0||before[0x60]&2!=0||before[0x4f]==4||word(before,0x68)<=0{return Err(ChoiceError::UnsupportedState);}
 if rng.ring>=32{return Err(ChoiceError::InvalidRng);}
 let mut after=*before;let mut next=rng;let mut values=Vec::new();
 if after[0x60]&8==0{after[0x72..0x76].copy_from_slice(&before[7..11]);after[0x60]|=8;}
 let delay=!c.virtual_b0&&!c.current_cell_requires_escape&&draw(&mut next,&mut values)%10<4;
 let target=if delay{put(&mut after,0x6c,20);None}else{
  let angle=usize::from(draw(&mut next,&mut values))*8;
  // Owned Mule with positive reserve: species4 table radius512 at1046B0.
  let radius=(512*i32::from(draw(&mut next,&mut values)))>>8;let index=angle.wrapping_sub(512)&2047;
  let delta=[(tables.cosine()[index]*(radius*2))>>16,(tables.sine()[index]*(radius*2))>>16];
  let point=std::array::from_fn(|i|((word(&after,0x72+i*2).wrapping_mul(2).wrapping_add(delta[i]as i16)>>1)as u16&0x7fff)as i16);
  if !accept(point,after[0x53])?{return Err(ChoiceError::TargetSearchRequired);}
  put(&mut after,11,point[0]);put(&mut after,13,point[1]);after[0x16]&=0x3f;after[0x54]=20;
  let flags=(word(&after,0x49)as u16|0x14)&0xeffc;put(&mut after,0x49,flags as i16);after[0x4f]=0;after[0x52]=0;
  let percent=75+25*i32::from(after[0x76].min(100))/100;let speed=((704*percent/100)*(i32::from(after[0x7b])+1)>>8)as i16;
  put(&mut after,0x3f,speed);put(&mut after,0x6c,32767);after[0x60]|=0x80;Some(point)
 };
 Ok(ChoicePlan{before:*before,after,rng_before:rng,rng_after:next,draw_values:values,target})
}

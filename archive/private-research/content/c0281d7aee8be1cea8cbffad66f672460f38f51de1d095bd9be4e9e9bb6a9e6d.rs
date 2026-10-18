//! Source7AC00 packed five-byte UI animation state. No resource pointers or host services.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct AnimationDescriptor { pub frame_holds: Vec<u8>, pub flags: u8 }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum AnimationError { InvalidFrameCount, InvalidPingPong, InvalidAssetLayout }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct AnimationStep { pub after:[u8;5], pub frame:u8 }
///10010 writes only direction and frame delay. Other prior bytes survive.
pub fn initialize_source_state(mut prior:[u8;5])->[u8;5] {prior[2]=1;prior[3]=1;prior}
/// Initial mapped bytes184D04 and184D0C+5*i are zero, followed by10010.
pub const FRESH_SOURCE_STATE:[u8;5]=[0,0,1,1,0];
pub const COUNTDOWN_DESCRIPTOR_OFFSETS:[u16;8]=[0x15e,0x118,0x122,0x12c,0x136,0x140,0x14a,0x154];
pub const COUNTDOWN_STATE_ADDRESSES:[u32;8]=[0x184d04,0x184d0c,0x184d11,0x184d16,0x184d1b,0x184d20,0x184d25,0x184d2a];
impl AnimationDescriptor {
 /// Descriptor offset is relative to the ANB animation table, not the file.
 /// Host remains responsible for reference-build resource identity.
 pub fn from_anb(bytes:&[u8],descriptor_offset:u16)->Result<Self,AnimationError>{
  let err=AnimationError::InvalidAssetLayout;
  let word=|at:usize|bytes.get(at..at+4).map(|x|u32::from_le_bytes(x.try_into().unwrap())as usize).ok_or(err);
  let count=u16::from_le_bytes(bytes.get(..2).ok_or(err)?.try_into().unwrap())as usize;
  let frames=word(10)?;let table=word(14)?;let offset=usize::from(descriptor_offset);
  if offset%10!=0||offset/10>=count||frames>table||table.checked_add(count*10).is_none_or(|end|end>bytes.len()){return Err(err);}
  let at=table.checked_add(offset).ok_or(err)?;let d=bytes.get(at..at.checked_add(10).ok_or(err)?).ok_or(err)?;
  let start=u32::from_le_bytes(d[..4].try_into().unwrap())as usize;let end=u32::from_le_bytes(d[4..8].try_into().unwrap())as usize;let n=d[8]as usize;
  if n==0||start%14!=0||start.checked_add((n-1)*14)!=Some(end)||frames.checked_add(end).and_then(|v|v.checked_add(14)).is_none_or(|end|end>table){return Err(err);}
  let mut holds=Vec::with_capacity(n);
  for i in 0..n{let p=frames.checked_add(start).and_then(|v|v.checked_add(i*14+10)).ok_or(err)?;holds.push(*bytes.get(p).ok_or(err)?);}
  let result=Self{frame_holds:holds,flags:d[9]};result.validate()?;Ok(result)
 }
 fn validate(&self)->Result<(),AnimationError>{if self.frame_holds.is_empty()||self.frame_holds.len()>255{return Err(AnimationError::InvalidFrameCount);}if self.frame_holds.len()==1&&self.flags&1!=0{return Err(AnimationError::InvalidPingPong);}Ok(())}
}
/// Pure transaction: invalid metadata returns without changing the caller's state.
/// State bytes: hold countdown, frame index, signed direction, signed delay, signed delay countdown.
pub fn step(before:[u8;5],descriptor:&AnimationDescriptor)->Result<AnimationStep,AnimationError>{
 descriptor.validate()?;let mut s=before;let last=descriptor.frame_holds.len()-1;let mut frame=usize::from(s[1]).min(last);s[1]=frame as u8;
 if (s[4]as i8)>(s[3]as i8){s[4]=s[3];}
 let delay=s[4]as i8;s[4]=s[4].wrapping_sub(1);
 if delay<=0 {s[4]=s[3];s[0]=s[0].min(descriptor.frame_holds[frame]);let hold=s[0];s[0]=s[0].wrapping_sub(1);
  if hold==0 {let next=frame as i32+i32::from(s[2]as i8);
   frame=if next>last as i32{if descriptor.flags&1!=0{s[2]=255;last-1}else{0}}
   else if next<0{if descriptor.flags&1!=0{s[2]=1;1}else{last}}
   else{next as usize};s[1]=frame as u8;s[0]=descriptor.frame_holds[frame];
  }
 }
 Ok(AnimationStep{after:s,frame:frame as u8})
}

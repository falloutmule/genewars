#[derive(Debug,Clone,PartialEq,Eq)]
pub enum MissionTailError<E>{InvalidContext{active:u8,selected:u8},DeferredLandingRequired{selected:u8},Capability(E),TerminalOutcomeRequired{winner:u8},TerminalLossRequired{selected:u8}}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TailContext {pub flags:u8,pub mode:u8,pub active:u8,pub selected:u8,pub mission_counter:u32,pub frame:u32,pub target_count:u16,pub mission_limit:u16,pub mission_timer:u16,pub terrain_count:u8,pub terrain_limit:i32,pub field_34b9:u8}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TailPlan {pub players:[[u8;1805];4],pub winner:Option<u8>,pub survivors:u8,pub last_survivor:Option<u8>,pub capability_queries:Vec<u8>,pub already_terminal:bool}
fn i32at(b:&[u8],p:usize)->i32{i32::from_le_bytes(b[p..p+4].try_into().unwrap())}
fn u16at(b:&[u8],p:usize)->u16{u16::from_le_bytes(b[p..p+2].try_into().unwrap())}
/// Private draft: exact numeric72ED0 prefix through731B6; terminal ports excluded.
pub fn plan<F,E>(before:&[[u8;1805];4],c:&TailContext,mut exhausted:F)->Result<TailPlan,MissionTailError<E>> where F:FnMut(u8)->Result<bool,E> {
 if c.active>4||c.selected>=4{return Err(MissionTailError::InvalidContext{active:c.active,selected:c.selected})}
 let mut q=TailPlan{players:*before,winner:None,survivors:0,last_survivor:None,capability_queries:vec![],already_terminal:c.flags&0x40!=0};
 if q.already_terminal{return Ok(q)}
 if c.mission_counter!=0&&c.frame==c.mission_counter{return Err(MissionTailError::DeferredLandingRequired{selected:c.selected})}
 for id in 0..c.active {
  if q.winner.is_some(){break}
  let p=&mut q.players[id as usize];
  if i32at(p,0x49d)!=0 {
   let empty=if p[2]!=0{u16at(p,0x60e)>0}else if c.mission_counter!=0&&p[0x498]!=0{false}else{p[3] as i8==0};
   if p[0x4a2] as i8==0&&empty {if p[0x6e3]==0{p[0x6e3]=1}}
   else {
    if c.flags&0x20!=0 {
     if id==c.selected {
      let special=p[0x6e7];
      if special==0 {if p[0x606]>=3{q.winner=Some(id)}}
      else if (special==14||special==20)&&c.mode&1==0&&i32::from(c.mission_limit)<=i32at(p,0x11)&&c.mission_timer==0 {q.winner=Some(id)}
     }
     if id==c.selected&&q.winner.is_none()&&c.mission_counter==0 {q.capability_queries.push(id);if exhausted(id).map_err(MissionTailError::Capability)?{p[0x6e3]=1}}
    }
    if q.winner.is_none() {
     if p[2]==0&&c.target_count>0&&i32::from(c.target_count)<=i32at(p,0x4a7){q.winner=Some(id)}
     if c.terrain_count!=0&&c.terrain_limit<=i32at(p,0xd){q.winner=Some(id)}
    }
   }
  }
  if p[0x6e3]==0 {q.survivors=q.survivors.wrapping_add(1);q.last_survivor=Some(id)}
 }
 if q.winner.is_none()&&c.terrain_count!=0&&c.terrain_count==c.field_34b9 {
  let mut best=0u16;
  for id in 0..c.active {let p=&q.players[id as usize];if p[0x6e3]==0&&i32::from(best)<i32at(p,0xd){q.winner=Some(id);best=i32at(p,0xd) as u16}}
 }
 if c.target_count==0&&(c.mode&1!=0||c.terrain_count==0)&&c.active>1&&q.winner.is_none()&&q.survivors==1 {q.winner=q.last_survivor}
 Ok(q)
}

/// Full nonterminal72ED0 return. No prefix mutation is committed on a port boundary.
pub fn plan_full<F,E>(before:&[[u8;1805];4],c:&TailContext,exhausted:F)->Result<TailPlan,MissionTailError<E>>where F:FnMut(u8)->Result<bool,E>{let q=plan(before,c,exhausted)?;if q.already_terminal{return Ok(q)}if q.winner.is_some(){return Err(MissionTailError::TerminalOutcomeRequired{winner:q.winner.unwrap()})}if c.flags&0x80==0&&q.players[c.selected as usize][0x6e3]!=0{return Err(MissionTailError::TerminalLossRequired{selected:c.selected})}Ok(q)}

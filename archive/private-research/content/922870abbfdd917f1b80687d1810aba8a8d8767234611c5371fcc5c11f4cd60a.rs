//! Verified CD72ED0 mission predicates and ordinary mode0 result requests.
//! No state is committed here. Capability failures and unsupported external
//! branches are errors, never gameplay losses. Campaign advancement is separate.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum MissionOutcome{Running,Won{player:u8},Lost{player:u8,winner:Option<u8>}}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum OutcomeBoundary{AlreadyTerminal,AlternatePresentation}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum MissionResultRequest{EnterVictoryModule185DFC}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MissionOutcomePlan{pub scan:ObjectiveScan,pub flags_before:u8,pub flags_after:u8,pub outcome:MissionOutcome,pub result_request:Option<MissionResultRequest>}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum MissionOutcomeError<E>{InvalidContext{active:u8,selected:u8},DeferredLandingRequired{selected:u8},Capability(E),Unsupported(OutcomeBoundary)}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MissionOutcomeContext {pub flags:u8,pub mode:u8,pub active:u8,pub selected:u8,pub mission_counter:u32,pub frame:u32,pub target_count:u16,pub mission_limit:u16,pub mission_timer:u16,pub terrain_count:u8,pub terrain_limit:i32,pub field_34b9:u8}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ObjectiveScan {pub players:[[u8;1805];4],pub winner:Option<u8>,pub survivors:u8,pub last_survivor:Option<u8>,pub capability_queries:Vec<u8>,pub already_terminal:bool}
fn i32at(b:&[u8],p:usize)->i32{i32::from_le_bytes(b[p..p+4].try_into().unwrap())}
fn u16at(b:&[u8],p:usize)->u16{u16::from_le_bytes(b[p..p+2].try_into().unwrap())}
/// Original72ED0 numeric scan through731B6; terminal requests are handled below.
fn scan<F,E>(before:&[[u8;1805];4],c:&MissionOutcomeContext,mut exhausted:F)->Result<ObjectiveScan,MissionOutcomeError<E>> where F:FnMut(u8)->Result<bool,E> {
 if c.active>4||c.selected>=4{return Err(MissionOutcomeError::InvalidContext{active:c.active,selected:c.selected})}
 let mut q=ObjectiveScan{players:*before,winner:None,survivors:0,last_survivor:None,capability_queries:vec![],already_terminal:c.flags&0x40!=0};
 if q.already_terminal{return Ok(q)}
 if c.mission_counter!=0&&c.frame==c.mission_counter{return Err(MissionOutcomeError::DeferredLandingRequired{selected:c.selected})}
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
     if id==c.selected&&q.winner.is_none()&&c.mission_counter==0 {q.capability_queries.push(id);if exhausted(id).map_err(MissionOutcomeError::Capability)?{p[0x6e3]=1}}
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


/// Evaluate the exact source scan and final flags. Runtime flags40 already
/// latch a terminal result; callers must retain that result instead of
/// reconstructing its identity from this no-op callback. Deferred7F3B0 and
/// mode1 presentation effects remain explicit boundaries.
pub fn evaluate<F,E>(before:&[[u8;1805];4],context:&MissionOutcomeContext,exhausted:F)->Result<MissionOutcomePlan,MissionOutcomeError<E>>where F:FnMut(u8)->Result<bool,E>{
 let q=scan(before,context,exhausted)?;
 if q.already_terminal{return Err(MissionOutcomeError::Unsupported(OutcomeBoundary::AlreadyTerminal))}
 let mut flags=context.flags;let mut request=None;
 let outcome=if let Some(winner)=q.winner{
  if context.mode&1!=0{return Err(MissionOutcomeError::Unsupported(OutcomeBoundary::AlternatePresentation))}
  if winner==context.selected{flags|=0x40;request=Some(MissionResultRequest::EnterVictoryModule185DFC);MissionOutcome::Won{player:winner}}
  else{flags|=0x80;MissionOutcome::Lost{player:context.selected,winner:Some(winner)}}
 }else if flags&0x80!=0{MissionOutcome::Lost{player:context.selected,winner:None}}
 else if q.players[context.selected as usize][0x6e3]!=0{
  if context.mode&1!=0{return Err(MissionOutcomeError::Unsupported(OutcomeBoundary::AlternatePresentation))}
  flags|=0x80;MissionOutcome::Lost{player:context.selected,winner:None}
 }else{MissionOutcome::Running};
 Ok(MissionOutcomePlan{scan:q,flags_before:context.flags,flags_after:flags,outcome,result_request:request})
}

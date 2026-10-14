//! Bounded72E50 mode0 teardown projection for a new campaign world.
//! The old building pool is intentionally not a valid active world afterwards:
//! source42B50 clears bookkeeping, not building objects. Do not resume it.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum CarryError{SelectedPlayer,Mode,PlayerIdentity,ActorList,BlueprintBinding,ActorLink,MapLength}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CampaignCarryPlan{pub players:[[u8;1805];4],pub actors:[[u8;200];20],pub map:Vec<u8>,pub records_184d7c:[u8;600],pub flags_after:u8,pub frame_after:u32,pub retired_actors:Vec<u8>,pub selected_actor_reset_required:bool}
/// Logical actorC0 pointer mapping into the four live player blueprint tables.
pub fn plan_campaign_carry(players:&[[u8;1805];4],actors:&[[u8;200];20],bindings:&[Option<(u8,u8)>;20],map:&[u8],records:&[u8;600],flags:u8,mode:u8,selected_player:u8,selected_actor:Option<u8>)->Result<CampaignCarryPlan,CarryError>{
 if selected_player>=4{return Err(CarryError::SelectedPlayer)}
 if mode&1!=0{return Err(CarryError::Mode)}if map.len()!=229376{return Err(CarryError::MapLength)}
 let mut q=CampaignCarryPlan{players:*players,actors:*actors,map:map.to_vec(),records_184d7c:*records,flags_after:flags&0xf9,frame_after:0,retired_actors:Vec::new(),selected_actor_reset_required:false};
 for owner in 0..4usize{
  if q.players[owner][1]!=owner as u8||q.players[owner][0x4a1]!=owner as u8{return Err(CarryError::PlayerIdentity)}
  let count=q.players[owner][3]as i8;if !(0..=5).contains(&count){return Err(CarryError::ActorList)}
  while q.players[owner][3]!=0{
   let id=q.players[owner][4]as i8;if !(0..20).contains(&id){return Err(CarryError::ActorList)}let id=id as usize;
   let (bpowner,bp)=bindings[id].ok_or(CarryError::BlueprintBinding)?;if bpowner as usize!=owner||bp>=32||id/5!=owner{return Err(CarryError::BlueprintBinding)}
   let first=(owner*5..owner*5+5).find(|i|bindings[*i]==Some((bpowner,bp))).ok_or(CarryError::BlueprintBinding)?;
   if first!=id{return Err(CarryError::BlueprintBinding)}
   let a=&mut q.actors[id];if a[6]!=3||a[24]as usize!=owner||i16::from_le_bytes(a[15..17].try_into().unwrap())!=id as i16{return Err(CarryError::ActorList)}
   if a[112]&1!=0{
    let x=i16::from_le_bytes(a[7..9].try_into().unwrap());let y=i16::from_le_bytes(a[9..11].try_into().unwrap());let cell=((i32::from(y)>>8)&127)*128+((i32::from(x)>>8)&127);let cell=cell as usize;
    if a[2]!=0||a[5]!=0||u16::from_le_bytes(a[..2].try_into().unwrap())as usize!=cell||q.map[cell*14+6..cell*14+9]!=[id as u8,0,3]{return Err(CarryError::ActorLink)}
    a[112]&=!1;q.map[cell*14+8]=0;
   }
   let p=&mut q.players[owner];let offset=0x16+bp as usize*36;p[offset+0x20]&=!3;
   p[3]-=1;for i in 0..p[3]as usize{p[4+i]=p[5+i]}
   a[6]=0;a[196..200].fill(0);q.retired_actors.push(id as u8);
   if owner==selected_player as usize&&selected_actor==Some(id as u8){q.selected_actor_reset_required=true}
  }
  let p=&mut q.players[owner];for b in p[0x16..0x496].chunks_exact_mut(36){if b[0]!=0{b[0x1c..0x20].fill(0)}}
  let identity=p[0x4a1];p[0x4a1..0x61a].fill(0);p[0x4a1]=identity;p[0x498..0x4a1].fill(0);
  p[0xd..0x15].fill(0);p[0x6f9..0x70d].fill(0);p[0x6e5]=0;p[0x6e3]=0;p[1]=owner as u8;
 }
 for r in q.records_184d7c.chunks_exact_mut(2){r[0]&=7;r[1]=255}
 Ok(q)
}

//! Native67-byte kind4 building allocation from42FA0/43E30/43560.
//! Resource animation pointers are represented by selector bindings.
pub type BuildingRecord=[u8;67];
pub const BUILDING_CAPACITY:usize=256;
pub const BUILDING_PLAYER_BYTES:usize=1805;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct BuildingReference{pub index:i16,pub kind:u8}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct BuildingAnimationBinding{pub selector:u8}
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BuildingError{InvalidOwner,InvalidSelector(u8),CountMismatch,InvalidHead}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingFactoryPlan{pub slot:u16,pub before:BuildingRecord,pub after:BuildingRecord,pub player_before:[u8;BUILDING_PLAYER_BYTES],pub player_after:[u8;BUILDING_PLAYER_BYTES],pub binding:BuildingAnimationBinding,pub cell:u16,pub old_head:Option<BuildingReference>}
fn word(b:&[u8],p:usize)->i16{i16::from_le_bytes(b[p..p+2].try_into().unwrap())}
fn put(b:&mut[u8],p:usize,v:i16){b[p..p+2].copy_from_slice(&v.to_le_bytes());}
///Caller supplies actual7B140/7B240 samples and a resolved intrusive cell head.
///Full coherent player partitions returnNULL without mutation or RNG.
pub fn plan_building_factory(pool:&[BuildingRecord;256],player:&[u8;1805],selector:u8,position:[i16;2],ground:i32,shade:u8,head:Option<BuildingReference>)->Result<Option<BuildingFactoryPlan>,BuildingError>{
 const B:usize=0x4a1;let owner=player[B];if owner>=4||player[1]!=owner{return Err(BuildingError::InvalidOwner);}
 if selector>=14{return Err(BuildingError::InvalidSelector(selector));}
 if head.is_some_and(|r|!(1..=8).contains(&r.kind)||r.index<0){return Err(BuildingError::InvalidHead);}
 let count=player[B+1]as i8;let base=usize::from(owner)*64;
 let active=pool[base..base+64].iter().filter(|r|r[6]!=0).count();if count<0||count as usize!=active||pool[base..base+64].iter().any(|r|r[6]!=0&&(r[6]!=4||r[24]!=owner)){return Err(BuildingError::CountMismatch);}
 if count>=64{return Ok(None);}
 let slot=base+pool[base..base+64].iter().position(|r|r[6]==0).ok_or(BuildingError::CountMismatch)?;let before=pool[slot];let mut a=before;
 a[6]=4;a[2]=0;a[5]=0;put(&mut a,15,slot as i16);a[24]=owner;put(&mut a,7,position[0]);put(&mut a,9,position[1]);a[17..21].copy_from_slice(&ground.to_le_bytes());put(&mut a,21,4);a[23]=shade;a[11..15].fill(0);a[25..30].fill(0);put(&mut a,43,0);a[46]=0;a[47]=0;a[60]=0;put(&mut a,49,0);a[54..58].fill(0);a[45]=selector;a[66]=0;
 match selector{0=>{put(&mut a,58,-1);a[60]=0;},5=>{a[52]=0;a[54]=0;},12=>{a[60]=4;put(&mut a,52,0);},_=>{}}
 a[39..43].fill(0);a[31..35].fill(0);
 let cell=((((i32::from(position[1])>>8)&127)*128)+((i32::from(position[0])>>8)&127))as u16;put(&mut a,0,cell as i16);
 if let Some(r)=head{put(&mut a,3,r.index);a[5]=r.kind;}
 let mut after=*player;after[B+1]=count as u8+1;let mut sum=[0i32;2];
 for (i,r)in pool[base..base+64].iter().enumerate(){let r=if base+i==slot{&a}else{r};if r[6]!=0{for axis in 0..2{sum[axis]+=(i32::from(word(r,7+axis*2))>>8)&255;}}}
 for axis in 0..2{put(&mut after,B+0x15b+axis*2,((sum[axis]/i32::from(count+1))<<8)as i16);}
 Ok(Some(BuildingFactoryPlan{slot:slot as u16,before,after:a,player_before:*player,player_after:after,binding:BuildingAnimationBinding{selector},cell,old_head:head}))
}

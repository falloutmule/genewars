//! Source48790 free-pool code2C numeric record construction.
//! Building allocation, ordered audio/map events and cleanup are host transactions.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BuildingEffectError{Occupied,InvalidSlot,InvalidOwner,InvalidSelector,InvalidBuildingReference,InvalidGround}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct BuildingEffectAudioRequest{pub channel:u8,pub mode:u8,pub parameter:u8,pub stack_byte:u8,pub stack_dword:i32}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingEffectPlan{pub before:[u8;81],pub after:[u8;81],pub animation_index:u16,pub map_position:[i16;2],pub pending_audio:BuildingEffectAudioRequest}
///Free-slot constructor after the caller's42FA0 building transaction. The
///resolved map head must be that new building, or NULL on allocation failure.
pub fn plan_building_effect_numeric(before:&[u8;81],slot:u16,at:[i16;2],ground:i32,shade:u8,height:i32,selector:u8,owner:u8,building:Option<u16>)->Result<BuildingEffectPlan,BuildingEffectError>{
 if before[6]!=0{return Err(BuildingEffectError::Occupied);}if slot>=200{return Err(BuildingEffectError::InvalidSlot);}if owner>=4{return Err(BuildingEffectError::InvalidOwner);}if selector>=14{return Err(BuildingEffectError::InvalidSelector);}if building.is_some_and(|v|v>=256){return Err(BuildingEffectError::InvalidBuildingReference);}if !(-65536..=65534).contains(&ground){return Err(BuildingEffectError::InvalidGround);}
 let after=building_effect_numeric(before,slot,at,ground,shade,height,selector,owner,building);Ok(BuildingEffectPlan{before:*before,after,animation_index:44,map_position:at.map(|v|v.wrapping_add(64)),pending_audio:BuildingEffectAudioRequest{channel:0x3f,mode:2,parameter:0,stack_byte:127,stack_dword:100}})
}
fn building_effect_numeric(before:&[u8;81],slot:u16,at:[i16;2],ground:i32,shade:u8,height:i32,selector:u8,owner:u8,building:Option<u16>)->[u8;81]{let mut a=*before;a[2]=0;a[5]=0;a[6]=5;for axis in 0..2{a[7+axis*2..9+axis*2].copy_from_slice(&at[axis].to_le_bytes());}a[11..15].fill(0);a[15..17].copy_from_slice(&slot.to_le_bytes());a[17..21].copy_from_slice(&ground.to_le_bytes());a[21..23].copy_from_slice(&32u16.to_le_bytes());a[23]=shade;a[24]=owner;a[25..28].fill(0);a[28]=1;a[29]=0;a[31..35].fill(0);a[39..43].copy_from_slice(&height.to_le_bytes());a[43..45].copy_from_slice(&16u16.to_le_bytes());a[45..63].fill(0);a[63..65].copy_from_slice(&(-1i16).to_le_bytes());a[69]=44;a[70]=0;a[74..76].copy_from_slice(&(if selector==0{-1i16}else{i16::from(selector)}).to_le_bytes());if let Some(id)=building{a[71..73].copy_from_slice(&id.to_le_bytes());a[73]=4;a[3..5].copy_from_slice(&id.to_le_bytes());a[5]=4;}let x=at[0].wrapping_add(64);let y=at[1].wrapping_add(64);a[0..2].copy_from_slice(&((((i32::from(y)>>8)&127)*128+((i32::from(x)>>8)&127))as u16).to_le_bytes());a}

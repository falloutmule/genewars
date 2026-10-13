//! Source74E80 construction-site classification; entity query remains explicit.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ConstructionSiteError{MapLength,InvalidSelector,InvalidFootprint,QueryFailed}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
#[repr(u8)]
pub enum ConstructionSiteClass{Valid=0,Blocked=1,Water=2,Occupied=3,MissingInteriorValue=4}
const FOOTPRINTS:[u8;14]=[3,1,2,3,2,2,1,2,1,2,3,3,2,1];
///`selector5_offset` is the exact packed1124F0 position delta. The callback
///must perform source8B340 with EDX0, returning whether a nonnull entity exists.
pub fn classify_construction_site(map:&[u8],position:[i16;2],selector:u8,selector5_offset:[i16;2],mut occupied:impl FnMut(u16)->Result<bool,ConstructionSiteError>)->Result<ConstructionSiteClass,ConstructionSiteError>{
 let footprint=*FOOTPRINTS.get(usize::from(selector)).ok_or(ConstructionSiteError::InvalidSelector)?;
 classify_construction_site_with_footprint(map,position,footprint,selector,selector5_offset,&mut occupied)
}
///Direct74E80 ABI: DL is footprint; BL is an independent discriminator.
///507D0's initial new-site check supplies BL100, suppressing5/11 special tails.
pub fn classify_construction_site_with_footprint(map:&[u8],position:[i16;2],footprint:u8,discriminator:u8,selector5_offset:[i16;2],mut occupied:impl FnMut(u16)->Result<bool,ConstructionSiteError>)->Result<ConstructionSiteClass,ConstructionSiteError>{
 if map.len()!=229376{return Err(ConstructionSiteError::MapLength);}if footprint>3{return Err(ConstructionSiteError::InvalidFootprint);}let selector=discriminator;
 fn check(map:&[u8],position:[i16;2],footprint:u8,selector:u8,occupied:&mut impl FnMut(u16)->Result<bool,ConstructionSiteError>)->Result<ConstructionSiteClass,ConstructionSiteError>{
  let offset=if footprint==0{0}else if footprint==1{1}else{2};let x=(i32::from(position[0])>>8)&255;let y=(i32::from(position[1])>>8)&255;let size=i32::from(footprint)+2;let mut interior_values=0;
  for row in 0..size{for col in 0..size{let cell=((((y-offset+row)&127)*128)+((x-offset+col)&127))as u16;let base=usize::from(cell)*14;
   if col!=0&&row!=0{
    if col<size-1&&row<size-1{if map[base+13]>1{interior_values+=1;}if map[base+11]&1!=0||(0x20..=0x22).contains(&map[base+4]){return Ok(ConstructionSiteClass::Blocked);}if occupied(cell)?{return Ok(ConstructionSiteClass::Occupied);}}
    if map[base+5]&7<=1{return Ok(ConstructionSiteClass::Water);}
   }
  }}if selector==11&&interior_values==0{Ok(ConstructionSiteClass::MissingInteriorValue)}else{Ok(ConstructionSiteClass::Valid)}
 }
 let first=check(map,position,footprint,selector,&mut occupied)?;if first!=ConstructionSiteClass::Valid{return Ok(first);}
 if selector==5{let shifted=std::array::from_fn(|axis|(position[axis]&!255).wrapping_add(selector5_offset[axis]));return check(map,shifted,3,100,&mut occupied);}Ok(first)
}

//! Source53A95ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€šÃ‚Â ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬ÃƒÂ¢Ã¢â‚¬Å¾Ã‚Â¢51E80 insufficient-funds cancellation for selector1, stage3.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionCancellationError{UnsupportedActor,InvalidMap,InvalidSite,NotInsufficient}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionCancellationPlan{pub before:[u8;200],pub after:[u8;200],pub map_after:Vec<u8>,pub panel_refresh_pending:bool}
pub fn plan_insufficient_selector1_cancellation(map:&[u8],before:&[u8;200],blueprint:&[u8;36],primary:i32,secondary:i32,owner_selected:bool,actor_selected:bool)
    ->Result<ConstructionCancellationPlan,ConstructionCancellationError>{
    if before[6]!=3||blueprint[0x16]!=0||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||before[0x83]!=1
        ||i16::from_le_bytes(before[0x88..0x8a].try_into().unwrap())!=3||before[0x49]&32==0{return Err(ConstructionCancellationError::UnsupportedActor);}
    if map.len()!=229376{return Err(ConstructionCancellationError::InvalidMap);}
    let cost=750i32-2*i32::from(blueprint[0x17]as i8);
    if primary.wrapping_add(secondary)>=cost{return Err(ConstructionCancellationError::NotInsufficient);}
    let mut after=*before;let mut map_after=map.to_vec();after[0x7a]=8;
    if before[0x58]&1!=0{
        after[0x4a]&=!8;
        let x=(i32::from(i16::from_le_bytes(before[0x8c..0x8e].try_into().unwrap()))>>8)&127;
        let y=(i32::from(i16::from_le_bytes(before[0x8e..0x90].try_into().unwrap()))>>8)&127;
        map_after[((y*128+x)as usize)*14+11]&=!1;
        // 8D5F0 traverses each row linearly; it does not wrap X independently.
        for row in 0..2{let start=(((y+row)&127)*128+x)as usize;
            if start+2>16384{return Err(ConstructionCancellationError::InvalidSite);}
            for column in 0..2{map_after[(start+column)*14+11]&=!16;}
        }
        after[0x58]&=!1;
    }
    after[0x82]=0;after[0x7e]=255;
    let flags=u16::from_le_bytes(after[0x70..0x72].try_into().unwrap())&0xee69;
    after[0x70..0x72].copy_from_slice(&flags.to_le_bytes());
    // Virtual84=1EE50, followed by51FA5's additional flag masks.
    after[0x3f..0x41].fill(0);after[0x54]=0;after[0x16]&=0x3f;
    let flags=u16::from_le_bytes(after[0x49..0x4b].try_into().unwrap())&0xefe8;
    after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
    Ok(ConstructionCancellationPlan{before:*before,after,map_after,panel_refresh_pending:owner_selected&&actor_selected})
}

///53CB3 completed selector1 task calls51E80 without the insufficiency reason8.
pub fn plan_finished_cancellation(map:&[u8],before:&[u8;200],blueprint:&[u8;36],building:&[u8;67],owner_selected:bool,actor_selected:bool)->Result<ConstructionCancellationPlan,ConstructionCancellationError>{
    let maximum=match before[0x83]{1=>120i16,3=>240,6=>80,11=>280,_=>return Err(ConstructionCancellationError::UnsupportedActor)};
    let word=|b:&[u8],p:usize|i16::from_le_bytes(b[p..p+2].try_into().unwrap());
    if before[6]!=3||blueprint[0x16]!=0||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||word(before,0x88)!=5||before[0x49]&32==0||before[0x81]!=4||word(before,0x7f)!=word(building,15)||building[6]!=4||building[24]!=before[24]||building[45]!=before[0x83]||word(building,43)!=maximum||word(building,49)&1==0{return Err(ConstructionCancellationError::UnsupportedActor);}
    if map.len()!=229376{return Err(ConstructionCancellationError::InvalidMap);}
    let mut after=*before;let map_after=map.to_vec();
    if before[0x58]&1!=0{
        after[0x4a]&=!8;
        after[0x58]&=!1;
    }
    after[0x82]=0;after[0x7e]=255;
    let flags=u16::from_le_bytes(after[0x70..0x72].try_into().unwrap())&0xee69;
    after[0x70..0x72].copy_from_slice(&flags.to_le_bytes());
    // Virtual84=1EE50, followed by51FA5's additional flag masks.
    after[0x3f..0x41].fill(0);after[0x54]=0;after[0x16]&=0x3f;
    let flags=u16::from_le_bytes(after[0x49..0x4b].try_into().unwrap())&0xefe8;
    after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
    Ok(ConstructionCancellationPlan{before:*before,after,map_after,panel_refresh_pending:owner_selected&&actor_selected})
}
/// Retained selector1 boundary for older callers.
pub fn plan_finished_selector1_cancellation(map:&[u8],before:&[u8;200],blueprint:&[u8;36],building:&[u8;67],owner_selected:bool,actor_selected:bool)->Result<ConstructionCancellationPlan,ConstructionCancellationError>{
 if before[0x83]!=1{return Err(ConstructionCancellationError::UnsupportedActor);}
 plan_finished_cancellation(map,before,blueprint,building,owner_selected,actor_selected)
}

///507D0 category0 prefix sets55=9/81=0 before class1 rejection.
///51E80 preserves that state and consumes no RNG; stage0 releases the task footprint.
pub fn plan_blocked_site_cancellation(map:&[u8],before:&[u8;200],blueprint:&[u8;36],owner_selected:bool,actor_selected:bool)
 ->Result<ConstructionCancellationPlan,ConstructionCancellationError>{
 if before[6]!=3||blueprint[0x16]!=0||before[0x55]!=1||before[0x7e]!=0||before[0x82]!=1||!matches!(before[0x83],1|3|6|11){return Err(ConstructionCancellationError::UnsupportedActor);}
 if map.len()!=229376{return Err(ConstructionCancellationError::InvalidMap);}
 let mut after=*before;let mut map_after=map.to_vec();after[0x70]&=!4;after[0x81]=0;after[0x55]=9;after[0x88..0x8a].fill(0);after[0x7a]=6;
 if before[0x58]&1!=0{
  after[0x4a]&=!8;
  let footprint=if matches!(before[0x83],1|6){1}else{3};
  let point=[i16::from_le_bytes(before[0x8c..0x8e].try_into().unwrap()),i16::from_le_bytes(before[0x8e..0x90].try_into().unwrap())];
  let x=(i32::from(point[0])>>8)&127;let y=(i32::from(point[1])>>8)&127;
  let range=if footprint==1{0..=0}else{-1..=1};
  for dy in range.clone(){for dx in range.clone(){map_after[((((y+dy)&127)*128+((x+dx)&127))as usize)*14+11]&=!1;}}
  let low=if footprint==1{[x,y]}else{[x-1,y-1]};let size=footprint+1;
  for row in 0..size{let start=(((low[1]+row)&127)*128+(low[0]&127))as usize;if start+size as usize>16384{return Err(ConstructionCancellationError::InvalidSite);}for col in 0..size as usize{map_after[(start+col)*14+11]&=!16;}}
  after[0x58]&=!1;
 }
 after[0x82]=0;after[0x7e]=255;
 let flags=u16::from_le_bytes(after[0x70..0x72].try_into().unwrap())&0xee69;after[0x70..0x72].copy_from_slice(&flags.to_le_bytes());
 after[0x3f..0x41].fill(0);after[0x54]=0;after[0x16]&=0x3f;
 let flags=u16::from_le_bytes(after[0x49..0x4b].try_into().unwrap())&0xefe8;after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
 Ok(ConstructionCancellationPlan{before:*before,after,map_after,panel_refresh_pending:owner_selected&&actor_selected})
}

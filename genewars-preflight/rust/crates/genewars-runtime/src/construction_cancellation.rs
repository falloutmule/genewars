//! Source53A95Ã¢â€ â€™51E80 insufficient-funds cancellation for selector1, stage3.
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
pub fn plan_finished_selector1_cancellation(map:&[u8],before:&[u8;200],blueprint:&[u8;36],building:&[u8;67],owner_selected:bool,actor_selected:bool)->Result<ConstructionCancellationPlan,ConstructionCancellationError>{
    let word=|b:&[u8],p:usize|i16::from_le_bytes(b[p..p+2].try_into().unwrap());
    if before[6]!=3||blueprint[0x16]!=0||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||before[0x83]!=1||word(before,0x88)!=5||before[0x49]&32==0||before[0x81]!=4||word(before,0x7f)!=word(building,15)||building[6]!=4||building[24]!=before[24]||building[45]!=1||word(building,43)!=120||word(building,49)!=1{return Err(ConstructionCancellationError::UnsupportedActor);}
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
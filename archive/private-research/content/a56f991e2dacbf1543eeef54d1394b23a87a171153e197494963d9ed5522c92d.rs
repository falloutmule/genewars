//!534D0 stage5 incomplete selector1 work start and nonterminal timer prefix.
//! A zero post-decrement timer requests the separately composed progress tail.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionWorkError{UnsupportedActor,InvalidBuilding,InvalidBlueprint,InvalidYaw,MovementRequired}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionWorkPlan{pub after:[u8;200],pub seed_after:u16,pub draws:u8,pub audio_4a:bool,pub progress_required:bool}
pub fn plan_incomplete_selector1_work(before:&[u8;200],blueprint:&[u8;36],building:&[u8;67],frame:u32,seed:u16,yaw:u16)->Result<ConstructionWorkPlan,ConstructionWorkError>{
    let word=|b:&[u8],p:usize|i16::from_le_bytes(b[p..p+2].try_into().unwrap());
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||before[0x83]!=1||word(before,0x88)!=5{return Err(ConstructionWorkError::UnsupportedActor);}
    if blueprint[0x16]!=0{return Err(ConstructionWorkError::InvalidBlueprint);}
    if building[6]!=4||building[24]!=before[24]||building[45]!=1||building[46]!=0||building[47]!=0||word(building,49)!=0||!(0..120).contains(&word(building,43))||before[0x81]!=4||word(before,0x7f)!=word(building,15){return Err(ConstructionWorkError::InvalidBuilding);}
    let mut after=*before;let mut seed_after=seed;let mut draws=0;let mut audio_4a=false;let mut progress_required=false;
    if before[0x49]&32!=0{
        if yaw>=2048{return Err(ConstructionWorkError::InvalidYaw);}
        let delay=110i16.wrapping_sub(i16::from(blueprint[0x17]as i8));
        after[0x84..0x86].copy_from_slice(&delay.to_le_bytes());after[0x86..0x88].copy_from_slice(&delay.to_le_bytes());
        after[0x70]=(after[0x70]&0x6f)|0x80;
        if (0x300..=0x700).contains(&yaw){after[0x70]|=4;}else{after[0x70]&=!4;}
        audio_4a=true;
    }else{
        if before[0x70]&0x80==0{return Err(ConstructionWorkError::MovementRequired);}
        let remaining=word(before,0x86).wrapping_sub(1);after[0x86..0x88].copy_from_slice(&remaining.to_le_bytes());
        if frame.wrapping_add(word(before,15)as i32 as u32)&1!=0{
            seed_after=seed.wrapping_mul(58925);draws=1;audio_4a=(seed_after>>8)%10<1;
        }
        progress_required=remaining==0;
    }
    Ok(ConstructionWorkPlan{after,seed_after,draws,audio_4a,progress_required})
}

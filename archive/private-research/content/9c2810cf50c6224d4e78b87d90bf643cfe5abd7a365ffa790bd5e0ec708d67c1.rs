//!534D0 stage5 admitted construction work and nonterminal timer prefix.
//! A zero post-decrement timer requests the separately composed progress tail.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionWorkError{UnsupportedActor,InvalidBuilding,InvalidBlueprint,InvalidYaw,MovementRequired}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionWorkPlan{pub after:[u8;200],pub seed_after:u16,pub draws:u8,pub audio_4a:bool,pub progress_required:bool}
pub fn plan_incomplete_work(before:&[u8;200],blueprint:&[u8;36],building:&[u8;67],frame:u32,seed:u16,yaw:u16)->Result<ConstructionWorkPlan,ConstructionWorkError>{
    let word=|b:&[u8],p:usize|i16::from_le_bytes(b[p..p+2].try_into().unwrap());
    let maximum=match before[0x83]{1=>120i16,3=>240,6=>80,11=>280,_=>return Err(ConstructionWorkError::UnsupportedActor)};
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||word(before,0x88)!=5{return Err(ConstructionWorkError::UnsupportedActor);}
    if blueprint[0x16]!=0{return Err(ConstructionWorkError::InvalidBlueprint);}
    if building[6]!=4||building[24]!=before[24]||building[45]!=before[0x83]||building[46]!=0||building[47]!=0||word(building,49)!=0||!(0..maximum).contains(&word(building,43))||before[0x81]!=4||word(before,0x7f)!=word(building,15){return Err(ConstructionWorkError::InvalidBuilding);}
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

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionWorkPoint{pub point:[i16;2],pub seed_after:u16,pub draws:u8}
///73C20 radius256, two shared LCG draws and fixed-point trig.
pub fn sample_construction_work_point(center:[i16;2],seed:u16,cosine:&[i32;2048],sine:&[i32;2048])->ConstructionWorkPoint{
    let first=seed.wrapping_mul(58925);let second=first.wrapping_mul(58925);
    let angle=usize::from(first>>8)*8;let radius=i32::from(second>>8);let index=angle.wrapping_sub(512)&2047;
    let delta=[cosine[index].wrapping_mul(radius*2)>>16,sine[index].wrapping_mul(radius*2)>>16];
    ConstructionWorkPoint{point:std::array::from_fn(|i|((center[i].wrapping_mul(2).wrapping_add(delta[i]as i16)>>1)as u16&0x7fff)as i16),seed_after:second,draws:2}
}
///The caller commits441C0 first; incomplete work moves around the building,
///completed work installs the source departure order and building reference.
pub fn finish_work_progress(plan:&ConstructionWorkPlan,building_after:&[u8;67],completed:bool,random_point:[i16;2],speed:i16,yaw:u16)->Result<[u8;200],ConstructionWorkError>{
    let word=|b:&[u8],p:usize|i16::from_le_bytes(b[p..p+2].try_into().unwrap());
    if !plan.progress_required{return Err(ConstructionWorkError::UnsupportedActor);}
    let before=&plan.after;
    let maximum=match before[0x83]{1=>120i16,3=>240,6=>80,11=>280,_=>return Err(ConstructionWorkError::UnsupportedActor)};
    if building_after[6]!=4||building_after[24]!=before[24]||building_after[45]!=before[0x83]||word(building_after,15)!=word(before,0x7f)||word(building_after,49)!=i16::from(completed)||(if completed{word(building_after,43)!=maximum}else{!(1..maximum).contains(&word(building_after,43))}){return Err(ConstructionWorkError::InvalidBuilding);}
    let mut after=*before;after[0x70]&=0x7b;after[0x5a]=1;after[0x3f..0x41].copy_from_slice(&speed.to_le_bytes());after[0x70]|=16;
    if completed{
        if yaw>=2048{return Err(ConstructionWorkError::InvalidYaw);}
        let point=[word(building_after,7).wrapping_add(384),word(building_after,9).wrapping_add(384)];
        for axis in 0..2{after[0x60+axis*2..0x62+axis*2].copy_from_slice(&point[axis].to_le_bytes());after[11+axis*2..13+axis*2].copy_from_slice(&point[axis].to_le_bytes());}
        after[0x16]&=0x3f;after[0x54]=20;
        let flags=(u16::from_le_bytes(after[0x49..0x4b].try_into().unwrap())|0x14)&0xeffc;after[0x49..0x4b].copy_from_slice(&flags.to_le_bytes());
        after[0x52]=0;after[0x4d..0x4f].copy_from_slice(&building_after[15..17]);after[0x4f]=4;after[0x49]|=0x80;after[0x39..0x3b].copy_from_slice(&yaw.to_le_bytes());
    }else{
        for axis in 0..2{after[0x60+axis*2..0x62+axis*2].copy_from_slice(&random_point[axis].to_le_bytes());}
        after[0x86..0x88].copy_from_slice(&before[0x84..0x86]);
    }
    Ok(after)
}

/// Retained selector1 API rejects other construction kinds.
pub fn plan_incomplete_selector1_work(before:&[u8;200],blueprint:&[u8;36],building:&[u8;67],frame:u32,seed:u16,yaw:u16)->Result<ConstructionWorkPlan,ConstructionWorkError>{
 if before[0x83]!=1{return Err(ConstructionWorkError::UnsupportedActor);}
 plan_incomplete_work(before,blueprint,building,frame,seed,yaw)
}
pub fn finish_selector1_work_progress(plan:&ConstructionWorkPlan,building:&[u8;67],completed:bool,point:[i16;2],speed:i16,yaw:u16)->Result<[u8;200],ConstructionWorkError>{
 if plan.after[0x83]!=1{return Err(ConstructionWorkError::UnsupportedActor);}
 finish_work_progress(plan,building,completed,point,speed,yaw)
}

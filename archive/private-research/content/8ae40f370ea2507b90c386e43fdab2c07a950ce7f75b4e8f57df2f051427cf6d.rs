//! Source48790 code1A particles emitted by descending family8 children.
//! EFF26 has five frames, flags0, and hold1 on every frame. The common
//! physics branch matches code2F: virtual70 is false and there is no tail.
use crate::landing::*;

pub fn plan_factory(pool:&[LandingRecord;200], count:i16, at:[i16;2],
    ground:i32, shade:u8, height:i32, angles:[u16;3], head:Option<ResolvedCellHead>,
    gravity:i16, rng:LandingRng, sine:&[i32;2048], cosine:&[i32;2048]
) -> Result<LandingEffect6FactoryPlan,LandingError> {
    let mut p=plan_landing_effect2f_factory(pool,count,0,at,ground,shade,height,angles,head,gravity,rng,sine,cosine)?;
    p.factory.after_nonpointer[69]=0x1a;
    p.factory.animation_index=26;
    // The factory leaves hold0; the first animation advance loads hold1.
    Ok(p)
}

fn as_departure(before:&LandingRecord)->Result<LandingRecord,LandingError>{
    if before[6]!=5||before[69]!=0x1a{return Err(LandingError::InvalidRecord);}
    let mut r=*before;r[69]=0x2f;Ok(r)
}
pub fn physics_candidate(before:&LandingRecord)->Result<[i16;2],LandingError>{
    effect2f_physics_candidate(&as_departure(before)?)
}
pub fn plan_flight(before:&LandingRecord,ground:i32,shade:u8,water:i32,
    corners:[i16;4],bounce:i16,angles:&[u16;257])->Result<LandingAirbornePlan,LandingError>{
    let mut p=plan_effect2f_flight(&as_departure(before)?,ground,shade,water,corners,bounce,angles)?;
    p.before=*before;p.after[69]=0x1a;Ok(p)
}
pub fn grounded_candidate(before:&LandingRecord,ground:i32,angles:&[u16;257],
    sine:&[i32;2048],cosine:&[i32;2048])->Result<LandingRecord,LandingError>{
    let mut r=effect2f_grounded_candidate(&as_departure(before)?,ground,angles,sine,cosine)?;r[69]=0x1a;Ok(r)
}
pub fn plan_grounded_after_second_sample(before:&LandingRecord,candidate:&LandingRecord,
    ground:i32,shade:u8,water:i32)->Result<LandingAirbornePlan,LandingError>{
    let mut p=plan_effect2f_grounded_after_second_sample(&as_departure(before)?,&as_departure(candidate)?,ground,shade,water)?;
    p.before=*before;p.after[69]=0x1a;Ok(p)
}
pub fn plan_animation_after_physics(before:&LandingRecord)->Result<Effect6UpdatePlan,LandingError>{
    as_departure(before)?;
    if before[26]>1||before[27]>4||before[28]!=1||!(1..=3).contains(&before[29]){
        return Err(LandingError::UnsupportedAnimationState);
    }
    let mut after=*before;
    let timer=i16::from_le_bytes(after[74..76].try_into().unwrap());
    if timer>=0{after[74..76].copy_from_slice(&timer.wrapping_sub(1).to_le_bytes());}
    // The source compares both counter and delay as signed bytes.
    let counter=(after[30] as i8).min(after[29] as i8);
    after[30]=counter.wrapping_sub(1) as u8;
    if counter<=0{
        after[30]=after[29];
        let hold=after[26];after[26]=hold.wrapping_sub(1);
        if hold==0{after[27]=(after[27]+1)%5;after[26]=1;}
    }
    let retirement_required=i16::from_le_bytes(after[74..76].try_into().unwrap())<0
        &&after[26]==0&&after[27]==4&&after[30]==0;
    Ok(Effect6UpdatePlan{after,retirement_required})
}

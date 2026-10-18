/// State9 returns a retirement request before the common audio/animation tail.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8ParentDeparturePlan {
    pub after:Family8Record,
    pub rng_after:BlueprintRng,
    pub old_cell:u16,
    pub new_cell:u16,
    pub pending_audio:Option<Family8AudioRequest>,
    pub queue_requested:bool,
    pub retirement_requested:bool,
}
pub fn plan_selector0_state9_departure(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animation:&Family8Animation,rng:BlueprintRng,current_point_accepted:bool)
    ->Result<Family8ParentDeparturePlan,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=0||before[0x56]!=9||before[0x57]!=0
        ||before[0x19]!=0||before[0x15]!=0x20||before[0x16]!=0||w(before,0x49)as u16&!0x20!=0x200
        ||before[0x4f]!=0||before[0x52]!=0||w(before,0x3f)!=-1||w(before,0x33)!=0||w(before,0x35)!=0||w(before,0x37)!=640{
        return Err(Family8MotionError::UnsupportedRecord);
    }
    if frame.wrapping_add(i32::from(w(before,15))as u32)&31==0&&!current_point_accepted{return Err(Family8MotionError::IdleSearchRequired);}
    let mut a=*before;a[0x49]&=0x9f;
    let changed=w(&a,0x39)!=w(&a,0x4b)||i32::from(w(&a,0x3b))!=d(&a,0x45);
    let yaw=w(&a,0x4b);let pitch=d(&a,0x45)as i16;pw(&mut a,0x39,yaw);pw(&mut a,0x3b,pitch);if changed{velocity(&mut a,trig)?;}
    airborne_physics(&mut a,map,frame,water,physics,trig,d(before,0x11))?;
    if a[0x49]&0x20!=0{pw(&mut a,0x3f,0);pw(&mut a,0x41,0);}
    let retirement_requested=d(&a,0x27)>=51200;
    let pending_audio=if retirement_requested{None}else{
        animate(&mut a,animation)?;let speed=w(&a,0x3d);
        Some(if speed>0{Family8AudioRequest::Moving{index:0x38,mode:2,priority:255,volume:127,initial_parameter:100,adjusted_parameter:(100+20*i32::from(speed)/1920)as i16}}
            else{Family8AudioRequest::StopIfPlaying{index:0x38,mode:4}})
    };
    Ok(Family8ParentDeparturePlan{old_cell:cell(point(before)),new_cell:cell(point(&a)),after:a,rng_after:rng,pending_audio,queue_requested:!retirement_requested,retirement_requested})
}

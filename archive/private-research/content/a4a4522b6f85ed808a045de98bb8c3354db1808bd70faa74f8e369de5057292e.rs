/// Source11AEB player counter decision after a returned child signals57.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8ParentNotificationPlan {
    pub motion:Family8MotionPlan,
    pub player_counter_after:i32,
    /// Source UI variants10CDD5/10CDE7, respectively; device/module work remains a port.
    pub state7_message_variant:Option<u8>,
}
pub fn plan_selector0_state5_notified(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animation:&Family8Animation,rng:BlueprintRng,
    current_point_accepted:bool,player_counter:i32,threshold:u16)->Result<Family8ParentNotificationPlan,Family8MotionError>{
    if before[0x57]==0{return Err(Family8MotionError::UnsupportedRecord);}
    let mut waiting=*before;waiting[0x57]=0;
    let mut motion=plan_selector0_state5_wait(&waiting,map,frame,water,physics,trig,animation,rng,current_point_accepted)?;
    let mut player_counter_after=player_counter;let mut variant=None;
    if player_counter>=i32::from(threshold){player_counter_after=0;motion.after[0x56]=6;}
    else if player_counter<0{motion.after[0x56]=7;let draw=rng_draw(&mut motion.rng_after);variant=Some(if draw%10<5{0}else{1});}
    else{motion.after[0x56]=8;}
    Ok(Family8ParentNotificationPlan{motion,player_counter_after,state7_message_variant:variant})
}
/// Source12C64 waits for all active selector0 records to be in state8.
pub fn plan_selector0_state8_group_departure(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animation:&Family8Animation,rng:BlueprintRng,
    current_point_accepted:bool,pool:&[Family8Record;16])->Result<Family8MotionPlan,Family8MotionError>{
    if before[0x56]!=8||before[0x57]!=0{return Err(Family8MotionError::UnsupportedRecord);}
    let mut waiting=*before;waiting[0x56]=5;
    let mut motion=plan_selector0_state5_wait(&waiting,map,frame,water,physics,trig,animation,rng,current_point_accepted)?;
    motion.after[0x56]=8;
    if !pool.iter().any(|r|r[6]!=0&&r[0x55]==0&&r[0x56]!=8){
        motion.after[0x56]=9;motion.after[0x33..0x37].fill(0);pw(&mut motion.after,0x37,640);pw(&mut motion.after,0x3f,-1);
        let yaw=i16::from(rng_draw(&mut motion.rng_after)%128)*16;pw(&mut motion.after,0x4b,yaw);
        let pitch=i32::from(rng_draw(&mut motion.rng_after)%128);pd(&mut motion.after,0x45,pitch);
    }
    Ok(motion)
}

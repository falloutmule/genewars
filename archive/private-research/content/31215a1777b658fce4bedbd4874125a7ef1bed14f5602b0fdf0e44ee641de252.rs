// Candidate appended inside family8_motion for private comparison. This file
// is not a standalone crate or a claim of production admission.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8ReturnPlan {
    pub onward:Family8OnwardPlan,
    ///Source103F0 must retire the child before parent57 is written1.
    pub retirement_parent:Option<u16>,
}
fn return_prefix(a:&mut Family8Record,map:&[u8],frame:u32,water:i32,physics:&PhysicsTables,trig:&TrigTables)->Result<(),Family8MotionError>{
    let old_ground=d(a,0x11);let parity=frame.wrapping_add(i32::from(w(a,15))as u32);a[0x49]&=0x9f;
    if w(a,0x3f)>0||a[0x49]&0x10!=0 {
        if a[0x16]&0x40!=0 {
            let delta=d(a,0xb).wrapping_sub(d(a,0x27));
            if delta.wrapping_abs()<=128 {
                a[0x54]=0;let flags=((w(a,0x49)as u16|0x220)&0x2fef)as i16;pw(a,0x49,flags);a[0x16]&=0x3f;
                for off in [0x2d,0x2f,0x31,0x3d,0x3f]{pw(a,off,0);}
            }else if d(a,0xb)>d(a,0x27){a[0x4a]=(a[0x4a]|0x40)&0x7f;pd(a,0x45,0);}
            else{a[0x4a]=(a[0x4a]|0x80)&0xbf;pd(a,0x45,1024);}
        }else if a[0x49]&0x10!=0 {
            a[0x49]&=0xf3;
            if parity&1!=0{let dx=delta15(w(a,7),w(a,0xb));let dy=delta15(w(a,9),w(a,0xd));pw(a,0x4b,crate::actor_motion::angle(dx*2,dy*2,physics.angles())as i16);}
            let distance=target_distance(a);let relative_height=d(a,0x27).wrapping_sub(d(a,0x11));
            if distance<=0x24000 {
                a[0x4a]&=0xbf;
                if distance<=400 {
                    let flags=(w(a,0x49)as u16|0x220)as i16;pw(a,0x49,flags);a[0x4a]&=0x2f;
                    for axis in 0..2{let v=w(a,0xb+axis*2)&0x7fff;pw(a,7+axis*2,v);a[0x43+axis]=0;}
                    let at=point(a);let ground=genewars_core::cd_candidate::interpolate_height(map,at[0],at[1]).map_err(|_|Family8MotionError::MapLength)?;
                    let shade=genewars_core::cd_candidate::interpolate_entity_shade(map,at[0],at[1]).map_err(|_|Family8MotionError::MapLength)?;
                    pd(a,0x11,ground);a[0x17]=shade;pw(a,0x41,0);a[0x16]&=0x3f;a[0x54]=0;a[0x49]&=0xef;
                    for off in [0x2d,0x2f,0x31,0x3d,0x3f]{pw(a,off,0);}
                }else if parity&3==3{set_target_angles(a,physics)?;}
            }else if relative_height<2880{let pitch=d(a,0x45);if pitch>=32{pd(a,0x45,pitch.wrapping_sub(16));}a[0x4a]|=0x40;}
            else if a[0x4a]&0x40!=0||parity&1!=0{a[0x4a]&=0xbf;set_target_angles(a,physics)?;}
        }else{
            let relative_height=d(a,0x27).wrapping_sub(d(a,0x11));
            if relative_height<2880{let pitch=d(a,0x45);if pitch>=32{pd(a,0x45,pitch.wrapping_sub(16));}a[0x4a]|=0x40;}
            else if a[0x4a]&0x40!=0&&relative_height>=2112{a[0x4a]&=0xbf;}
        }
    }
    steer_selector1(a,trig)?;airborne_physics(a,map,frame,water,physics,trig,old_ground)?;
    if a[0x49]&0x20!=0{pw(a,0x3f,0);pw(a,0x41,0);}Ok(())
}
pub fn plan_selector1_state3_return(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,
    parents:&[Family8Record;16],current_point_accepted:bool)->Result<Family8ReturnPlan,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=1||before[0x56]!=3||before[0x57]>2
        ||before[0x15]!=0x20||before[0x16]&!0x40!=0||w(before,0x49)as u16&0x1c0b!=0
        ||before[0x4f]!=0||before[0x52]!=0||w(before,0x3f)<0||w(before,0x41)<0{return Err(Family8MotionError::UnsupportedRecord);}
    let parent_index=if before[0x57]==0 {
        parents.iter().position(|r|r[6]!=0&&r[0x18]==before[0x18]&&r[0x55]==0).ok_or(Family8MotionError::UnsupportedRecord)?
    }else{
        if before[0x5b]!=8||!(0..16).contains(&w(before,0x59)){return Err(Family8MotionError::UnsupportedRecord);}w(before,0x59)as usize
    };
    let parent=&parents[parent_index];
    if parent[6]!=8||w(parent,15)as usize!=parent_index||parent[0x18]!=before[0x18]||parent[0x55]!=0||parent[0x56]!=5||parent[0x57]!=0{
        return Err(Family8MotionError::UnsupportedRecord);
    }
    if before[0x49]&0x10==0&&frame.wrapping_add(i32::from(w(before,15))as u32)&31==0&&!current_point_accepted{return Err(Family8MotionError::IdleSearchRequired);}
    let mut a=*before;return_prefix(&mut a,map,frame,water,physics,trig)?;
    let speed=w(&a,0x41);let audio=Family8AudioRequest::Moving{index:0x53,mode:2,priority:255,volume:127,initial_parameter:100,
        adjusted_parameter:(100+100*i32::from(speed)/(2*3840))as i16};let mut rng_after=rng;let mut particle=None;
    if speed>256{
        let yaw=(i32::from(w(&a,0x39))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        let pitch=(i32::from(w(&a,0x3b))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        particle=Some(Family8ParticleRequest{code:0x1a,position:point(&a),physical_height:d(&a,0x27),angles:[yaw as u16,pitch as u16,(w(&a,0x3d)>>2)as u16]});
    }
    a[0x19]=(((((i32::from(w(&a,0x39))>>7)+1)>>1)+1)&7)as u8;let mut retirement_parent=None;
    match a[0x57] {
        0=>{
            pw(&mut a,0x59,parent_index as i16);a[0x5b]=8;let target=point(parent).map(|v|v.wrapping_sub(32));
            let height=genewars_core::cd_candidate::interpolate_height(map,target[0],target[1]).map_err(|_|Family8MotionError::MapLength)?.wrapping_add(2048);
            a[0x54]=20;for axis in 0..2{pw(&mut a,0xb+2*axis,target[axis]);}a[0x16]&=0xbf;a[0x49]|=0x14;
            let flags=(w(&a,0x49)as u16&0xeffc)as i16;pw(&mut a,0x49,flags);a[0x16]&=0x7f;
            a[0x4f]=0;a[0x52]=0;pd(&mut a,0x60,height);a[0x4a]&=0x3f;pw(&mut a,0x3f,3840);a[0x57]=1;
        },
        1=>if a[0x49]&0x20!=0{
            pd(&mut a,0xb,d(parent,0x27));a[0x16]|=0x40;a[0x49]|=0x10;a[0x4f]=0;a[0x52]=0;pw(&mut a,0x3f,3840);a[0x2d..0x33].fill(0);a[0x57]=2;
        },
        2=>if a[0x49]&0x20!=0{retirement_parent=Some(parent_index as u16);},
        _=>return Err(Family8MotionError::UnsupportedRecord),
    }
    if retirement_parent.is_none(){let direction=usize::from(a[0x19]);animate(&mut a,&animations.directions[direction])?;}
    Ok(Family8ReturnPlan{onward:Family8OnwardPlan{motion:Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after,old_cell:cell(point(before)),child:None,
        pending_audio:audio,pending_ui_message:false,queue_requested:retirement_parent.is_none()},particle,target_selection_required:false},retirement_parent})
}

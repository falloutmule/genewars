//! Bounded CD family8 parent/child callback plans from118E0.
//!
//! The altitude wrapper and airborne6DF30 numeric branch are included.
//! Allocation, map-link commits, optional UI and audio engines remain typed
//! requests. Unsupported callbacks reject; this is not the whole scheduler.

use crate::{actor_activation::TrigTables, blueprint::BlueprintRng,
    family8_creation::Family8Record, physics_tables::PhysicsTables};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Family8MotionError { UnsupportedRecord, MapLength, InvalidRng, AnimationResource, GroundContact, WaterBoundary, ArithmeticBoundary, IdleSearchRequired }

/// Source wrapper invocations, not claims that an audio device played sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family8AudioRequest {
    Moving { index: u8, mode: u8, priority: u8, volume: u8, initial_parameter: u8, adjusted_parameter: i16 },
    /// 97830 query, then virtual50(index38,mode4) only if that query succeeds.
    StopIfPlaying { index: u8, mode: u8 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Family8ChildRequest { pub selector: u8, pub position: [i16;2], pub physical_height: i32, pub owner: u8 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8MotionPlan {
    pub after: Family8Record,
    pub rng_after: BlueprintRng,
    pub old_cell: u16,
    pub new_cell: u16,
    pub child: Option<Family8ChildRequest>,
    pub pending_audio: Family8AudioRequest,
    /// The one-in-ten source UI formatting/module request at11A18.
    pub pending_ui_message: bool,
    pub queue_requested: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Family8ParticleRequest {pub code:u8,pub position:[i16;2],pub physical_height:i32,pub angles:[u16;3]}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8OnwardPlan {
    pub motion:Family8MotionPlan,
    pub particle:Option<Family8ParticleRequest>,
    ///114B0 must run in the world transaction before this afterimage commits.
    pub target_selection_required:bool,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8DirectionalAnimations {directions:[Family8Animation;8]}
impl Family8DirectionalAnimations {
    #[cfg(test)]
    pub(crate) fn fixture()->Self {Self{directions:std::array::from_fn(|_|Family8Animation{holds:vec![0],flags:0})}}
    pub fn from_ethereal_anb(bytes:&[u8])->Result<Self,Family8MotionError>{
        Family8Animation::from_ethereal_anb(bytes)?;
        let frames=u32::from_le_bytes(bytes[10..14].try_into().unwrap())as usize;
        let animations=u32::from_le_bytes(bytes[14..18].try_into().unwrap())as usize;
        Ok(Self{directions:std::array::from_fn(|i|{let d=animations+10*i;let base=u32::from_le_bytes(bytes[d..d+4].try_into().unwrap())as usize;
            Family8Animation{holds:(0..usize::from(bytes[d+8])).map(|j|bytes[frames+base+14*j+10]).collect(),flags:bytes[d+9]}})})
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family8Animation { holds: Vec<u8>, flags: u8 }
impl Family8Animation {
    #[cfg(test)]
    pub(crate) fn fixture()->Self {Self {holds:vec![0;11],flags:1}}
    /// Local animation0 for selector0 is global ETHEREAL animation8.
    pub fn from_ethereal_anb(bytes: &[u8])->Result<Self,Family8MotionError> {
        if bytes.len()!=91123 || crate::identity::sha256_hex(bytes)!="318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4" {
            return Err(Family8MotionError::AnimationResource);
        }
        let frames=u32::from_le_bytes(bytes[10..14].try_into().unwrap())as usize;
        let animations=u32::from_le_bytes(bytes[14..18].try_into().unwrap())as usize;
        let d=animations+80;let base=u32::from_le_bytes(bytes[d..d+4].try_into().unwrap())as usize;
        let holds=(0..usize::from(bytes[d+8])).map(|i|bytes[frames+base+14*i+10]).collect();
        Ok(Self { holds, flags:bytes[d+9] })
    }
}
fn w(r:&Family8Record,p:usize)->i16 { i16::from_le_bytes([r[p],r[p+1]]) }
fn d(r:&Family8Record,p:usize)->i32 { i32::from_le_bytes(r[p..p+4].try_into().unwrap()) }
fn pw(r:&mut Family8Record,p:usize,v:i16){r[p..p+2].copy_from_slice(&v.to_le_bytes());}
fn pd(r:&mut Family8Record,p:usize,v:i32){r[p..p+4].copy_from_slice(&v.to_le_bytes());}
fn point(r:&Family8Record)->[i16;2]{[w(r,7),w(r,9)]}
fn cell(at:[i16;2])->u16 {((at[1]as u16>>8)&127)*128+((at[0]as u16>>8)&127)}
fn velocity(r:&mut Family8Record,trig:&TrigTables)->Result<(),Family8MotionError>{
    let yaw=w(r,0x39);let pitch=w(r,0x3b);let speed=w(r,0x3d);
    if !(0..=2048).contains(&yaw)||!(0..=2048).contains(&pitch)||speed<0{return Err(Family8MotionError::ArithmeticBoundary);}
    let yi=(i32::from(yaw)-512).rem_euclid(2048)as usize;let pi=(i32::from(pitch)-512).rem_euclid(2048)as usize;
    let horizontal=((i64::from(trig.cosine()[pi])*i64::from(speed))>>16)as i32;
    pw(r,0x2d,((i64::from(trig.cosine()[yi])*i64::from(horizontal))>>16)as i16);
    pw(r,0x2f,((i64::from(trig.sine()[yi])*i64::from(horizontal))>>16)as i16);
    pw(r,0x31,((-i64::from(trig.sine()[pi])*i64::from(speed))>>16)as i16);Ok(())
}
fn angles(r:&mut Family8Record,physics:&PhysicsTables,trig:&TrigTables)->Result<(),Family8MotionError>{
    let x=i32::from(w(r,0x2d));let y=i32::from(w(r,0x2f));let z=i32::from(w(r,0x31));
    let xy=genewars_core::cd_candidate::integer_sqrt((x*x).wrapping_add(y*y)as u32)as i32;
    if xy!=0 {pw(r,0x39,crate::actor_motion::angle(x,y,physics.angles())as i16);}
    let speed=genewars_core::cd_candidate::integer_sqrt(xy.wrapping_mul(xy).wrapping_add(z*z)as u32)as i16;
    if speed<0{return Err(Family8MotionError::ArithmeticBoundary);}
    pw(r,0x3d,speed);pw(r,0x3b,if speed==0{512}else{crate::actor_motion::angle(xy,-z,physics.angles())as i16});
    if speed>0x4b00 {pw(r,0x3d,0x3200);velocity(r,trig)?;}Ok(())
}
fn animate(r:&mut Family8Record,a:&Family8Animation)->Result<(),Family8MotionError>{
    if a.holds.is_empty()||a.holds.len()>255||a.flags&!1!=0||(a.holds.len()==1&&a.flags&1!=0){return Err(Family8MotionError::AnimationResource);}
    let last=a.holds.len()-1;let frame=usize::from(r[0x1b]).min(last);r[0x1b]=frame as u8;
    if (r[0x1e]as i8)>(r[0x1d]as i8){r[0x1e]=r[0x1d];}
    let counter=r[0x1e]as i8;r[0x1e]=r[0x1e].wrapping_sub(1);
    if counter<=0 {r[0x1e]=r[0x1d];r[0x1a]=r[0x1a].min(a.holds[frame]);let hold=r[0x1a];r[0x1a]=hold.wrapping_sub(1);
        if hold==0 {let next=frame as i32+i32::from(r[0x1c]as i8);
            let next=if next>last as i32 {if a.flags&1!=0{r[0x1c]=255;last-1}else{0}}
                else if next<0 {if a.flags&1!=0{r[0x1c]=1;1}else{last}}else{next as usize};
            r[0x1b]=next as u8;r[0x1a]=a.holds[next];}
    }Ok(())
}

fn airborne_physics(a:&mut Family8Record,map:&[u8],frame:u32,water:i32,physics:&PhysicsTables,trig:&TrigTables,previous_ground:i32)->Result<(),Family8MotionError>{
    let original=*a;
    //6DF30 with15mask20: acceleration without gravity.
    for axis in 0..3{let v=w(a,0x2d+2*axis).wrapping_add(w(a,0x33+2*axis));pw(a,0x2d+2*axis,v);}
    let height=d(a,0x27).wrapping_add(i32::from(w(a,0x31))>>3);
    let fixed=std::array::from_fn::<_,2,_>(|axis|(i32::from(w(a,7+2*axis))<<16)
        .wrapping_add(i32::from(a[0x43+axis])<<8).wrapping_add(i32::from(w(a,0x2d+2*axis))<<10));
    let at=fixed.map(|v|(v>>16)as i16);
    let ground=genewars_core::cd_candidate::interpolate_height(map,at[0],at[1]).map_err(|_|Family8MotionError::MapLength)?;
    let shade=genewars_core::cd_candidate::interpolate_entity_shade(map,at[0],at[1]).map_err(|_|Family8MotionError::MapLength)?;
    pd(a,0x11,ground);a[0x17]=shade;
    if height<=ground {
        //Restricted contact branch of6DF30: no gravity under15mask20.
        //The shared ground/bounce numeric path is identical withgravity0;
        //virtualA4 is explicitly the family8 free-flight speed policy.
        let mut compatibility=original;compatibility[0x15]&=!0x20;
        let plan=crate::creature_world::plan_common_nonflight_physics(&compatibility,map,frame,0,water,physics,trig,
            |r,_,old_ground|crate::creature_world::plan_common_contact_speed(crate::creature_world::CommonSpeedInput{
                speed:w(r,0x41),desired_speed:w(r,0x3f),width:w(r,0x2b)as u16,flags_4a:r[0x4a],free_flight:true,
                ground_height:d(r,0x11),previous_height_low:old_ground,surface_term:0,distance_squared:target_distance(r)}),
            |_|Ok(false)).map_err(|_|Family8MotionError::GroundContact)?;
        *a=plan.after;a[0x15]=original[0x15];return Ok(());
    }
    for axis in 0..2{pw(a,7+2*axis,at[axis]);a[0x43+axis]=(fixed[axis]>>8)as u8;}
    pd(a,0x27,height.max(0));
    if w(a,0x3f)>0 {
        let distance=target_distance(a);
        let speed=crate::creature_world::plan_common_contact_speed(crate::creature_world::CommonSpeedInput{
            speed:w(a,0x41),desired_speed:w(a,0x3f),width:w(a,0x2b)as u16,flags_4a:a[0x4a],free_flight:true,
            ground_height:ground,previous_height_low:previous_ground as i16,surface_term:0,distance_squared:distance
        }).map_err(|_|Family8MotionError::ArithmeticBoundary)?;
        pw(a,0x41,speed);pw(a,0x3d,speed);velocity(a,trig)?;
    }
    angles(a,physics,trig)?;
    if frame.wrapping_add(i32::from(w(a,15))as u32)&3==3&&(water as u32).wrapping_add(320)>=d(a,0x27)as u32 {
        return Err(Family8MotionError::WaterBoundary);
    }
    for off in [7,9]{let v=w(a,off)&0x7fff;pw(a,off,v);}
    Ok(())
}
fn delta15(from:i16,to:i16)->i32 {i32::from(to.wrapping_mul(2).wrapping_sub(from.wrapping_mul(2))>>1)}
fn target_distance(a:&Family8Record)->i32 {
    let vertical=if a[0x16]&0x40!=0 {d(a,0xb)}else{d(a,0x60)};
    let dz=vertical.wrapping_sub(d(a,0x27))>>3;
    let mut result=dz.wrapping_mul(dz);
    if a[0x16]&0x40==0 {for axis in 0..2{let dx=delta15(w(a,7+2*axis),w(a,0xb+2*axis));result=result.wrapping_add(dx*dx);}}
    result
}

/// Pure afterimage plan. All inputs remain unchanged on every rejection.
/// The admitted virtual70 returns false, so source74270 is not called here.
/// Dirty virtual2C, non-null target refs, contact and water callbacks reject.
pub fn plan_selector0_state4(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animation:&Family8Animation,rng:BlueprintRng)
    ->Result<Family8MotionPlan,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=0||before[0x56]!=4
        ||before[0x19]!=0||before[0x15]&0x20==0||before[0x16]!=0x40
        ||w(before,0x49)as u16&0x1413!=0x10||before[0x4f]!=0||before[0x52]!=0
        ||w(before,0x3f)<0||w(before,0x41)<0 {
        return Err(Family8MotionError::UnsupportedRecord);
    }
    let mut a=*before;let old_cell=cell(point(before));let mut rng_after=rng;
    //6FB60 clears transient bits before the altitude wrapper.
    a[0x49]&=0x9f;
    let delta=d(&a,0xb).wrapping_sub(d(&a,0x27));
    if delta.wrapping_abs()<=128 {
        a[0x54]=0;let flags=((w(&a,0x49)as u16|0x220)&0x2fef)as i16;pw(&mut a,0x49,flags);a[0x16]&=0x3f;
        for off in [0x2d,0x2f,0x31,0x3d,0x3f]{pw(&mut a,off,0);}
    }else if d(&a,0xb)>d(&a,0x27) {a[0x4a]=(a[0x4a]|0x40)&0x7f;pd(&mut a,0x45,0);}
    else {a[0x4a]=(a[0x4a]|0x80)&0xbf;pd(&mut a,0x45,1024);}
    let changed=w(&a,0x39)!=w(&a,0x4b)||i32::from(w(&a,0x3b))!=d(&a,0x45);
    let yaw=w(&a,0x4b);let pitch=d(&a,0x45)as i16;pw(&mut a,0x39,yaw);pw(&mut a,0x3b,pitch);
    if changed{velocity(&mut a,trig)?;}
    airborne_physics(&mut a,map,frame,water,physics,trig,d(before,0x11))?;
    let mut child=None;let mut pending_ui_message=false;
    if a[0x49]&0x20!=0 {
        pw(&mut a,0x3f,0);pw(&mut a,0x41,0);
        child=Some(Family8ChildRequest{selector:1,position:point(&a).map(|v|v.wrapping_sub(32)),physical_height:d(&a,0x27).wrapping_sub(320),owner:a[0x18]});
        rng_after.seed=rng_after.seed.wrapping_mul(0xe62d);rng_after.ring=rng_after.ring.wrapping_add(1)&31;
        rng_after.draw_count=rng_after.draw_count.wrapping_add(1);pending_ui_message=(rng_after.seed>>8)%10==5;
        a[0x56]=5;a[0x57]=0;
    }
    let speed=w(&a,0x3d);
    let pending_audio=if speed>0 {Family8AudioRequest::Moving{index:0x38,mode:2,priority:255,volume:127,initial_parameter:100,
        adjusted_parameter:(100+20*i32::from(speed)/1920)as i16}}
        else {Family8AudioRequest::StopIfPlaying{index:0x38,mode:4}};
    animate(&mut a,animation)?;
    Ok(Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after,old_cell,child,pending_audio,pending_ui_message,queue_requested:true})
}

/// State5's waiting branch, with an explicit74D90 current-point result.
/// The source periodic query result remains explicit. If blocked,305F0
/// only reads the world and its family8 A8 callback is RET. That search's
/// scratch-only work is omitted by this numeric port, for either outcome.
pub fn plan_selector0_state5_wait(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animation:&Family8Animation,rng:BlueprintRng,current_point_accepted:bool)
    ->Result<Family8MotionPlan,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=0||before[0x56]!=5||before[0x57]!=0
        ||before[0x19]!=0||before[0x15]!=0x20||before[0x16]!=0||w(before,0x49)as u16&!0x20!=0x200
        ||before[0x4f]!=0||before[0x52]!=0||[0x2d,0x2f,0x31,0x33,0x35,0x37,0x3d,0x3f,0x41].iter().any(|&p|w(before,p)!=0){
        return Err(Family8MotionError::UnsupportedRecord);
    }
    // Family8 A8 is RET; the read-only neighborhood search has no numeric effect.
    let _ = current_point_accepted;
    let mut a=*before;a[0x49]&=0x9f;
    let changed=w(&a,0x39)!=w(&a,0x4b)||i32::from(w(&a,0x3b))!=d(&a,0x45);
    let yaw=w(&a,0x4b);let pitch=d(&a,0x45)as i16;pw(&mut a,0x39,yaw);pw(&mut a,0x3b,pitch);if changed{velocity(&mut a,trig)?;}
    airborne_physics(&mut a,map,frame,water,physics,trig,d(before,0x11))?;
    animate(&mut a,animation)?;
    Ok(Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after:rng,old_cell:cell(point(before)),child:None,
        pending_audio:Family8AudioRequest::StopIfPlaying{index:0x38,mode:4},pending_ui_message:false,queue_requested:true})
}

fn angular_delta(current:i16,target:i16)->i32 {
    let delta=i32::from(target)-i32::from(current);
    let other=if delta<0 {2048+delta}else{-2048+delta};if delta.abs()<=other.abs(){delta}else{other}
}
fn steer_selector1(a:&mut Family8Record,trig:&TrigTables)->Result<(),Family8MotionError>{
    let mut changed=false;let width=2*i32::from(w(a,0x2b)as u16);
    let yaw=w(a,0x39);let target=w(a,0x4b);
    if yaw!=target {let delta=angular_delta(yaw,target);if delta.abs()>=256{a[0x4a]|=0x20;}else{a[0x4a]&=!0x20;}
        pw(a,0x39,if delta.abs()<=width{target}else{((i32::from(yaw)+if delta<0{-width}else{width})&2047)as i16});changed=true;
    }else{a[0x4a]&=!0x20;}
    let pitch=w(a,0x3b);let target32=d(a,0x45);
    if i32::from(pitch)!=target32 {
        if !(0..=2048).contains(&target32){return Err(Family8MotionError::ArithmeticBoundary);}
        let target=target32 as i16;let delta=angular_delta(pitch,target);
        pw(a,0x3b,if delta.abs()<=width{target}else{((i32::from(pitch)+if delta<0{-width}else{width})&2047)as i16});changed=true;
    }
    if changed{velocity(a,trig)?;}Ok(())
}
fn rng_draw(rng:&mut BlueprintRng)->u8 {
    rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);(rng.seed>>8)as u8
}
/// Selector1/state0 altitude descent. Particle creation and114B0 target
/// selection must be applied by the owning world in the returned order.
/// All parent numeric writes and the two pre-particle RNG draws are included;
/// any RNG consumed inside the particle factory belongs to that factory.
pub fn plan_selector1_state0(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng)
    ->Result<Family8OnwardPlan,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=1||before[0x56]!=0
        ||before[0x15]!=0x20||before[0x16]!=0x40||w(before,0x49)as u16&0x1413!=0x10
        ||before[0x4f]!=0||before[0x52]!=0||w(before,0x3f)<0||w(before,0x41)<0{
        return Err(Family8MotionError::UnsupportedRecord);
    }
    let mut a=*before;a[0x49]&=0x9f;let delta=d(&a,0xb).wrapping_sub(d(&a,0x27));
    if delta.wrapping_abs()<=128 {
        a[0x54]=0;let flags=((w(&a,0x49)as u16|0x220)&0x2fef)as i16;pw(&mut a,0x49,flags);a[0x16]&=0x3f;
        for off in [0x2d,0x2f,0x31,0x3d,0x3f]{pw(&mut a,off,0);}
    }else if d(&a,0xb)>d(&a,0x27){a[0x4a]=(a[0x4a]|0x40)&0x7f;pd(&mut a,0x45,0);}
    else {a[0x4a]=(a[0x4a]|0x80)&0xbf;pd(&mut a,0x45,1024);}
    steer_selector1(&mut a,trig)?;
    airborne_physics(&mut a,map,frame,water,physics,trig,d(before,0x11))?;
    if a[0x49]&0x20!=0 {pw(&mut a,0x3f,0);pw(&mut a,0x41,0);}
    let speed=w(&a,0x41);let audio=Family8AudioRequest::Moving{index:0x53,mode:2,priority:255,volume:127,initial_parameter:100,
        adjusted_parameter:(100+100*i32::from(speed)/(2*3840))as i16};
    let mut rng_after=rng;let mut particle=None;
    if speed>256 {
        let yaw=(i32::from(w(&a,0x39))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        let pitch=(i32::from(w(&a,0x3b))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        particle=Some(Family8ParticleRequest{code:0x1a,position:point(&a),physical_height:d(&a,0x27),angles:[yaw as u16,pitch as u16,(w(&a,0x3d)>>2)as u16]});
    }
    a[0x19]=(((((i32::from(w(&a,0x39))>>7)+1)>>1)+1)&7)as u8;
    let target_selection_required=a[0x49]&0x20!=0;
    if target_selection_required {a[0x56]=1;a[0x57]=0;}
    let direction=usize::from(a[0x19]);animate(&mut a,&animations.directions[direction])?;
    Ok(Family8OnwardPlan{motion:Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after,old_cell:cell(point(before)),child:None,
        pending_audio:audio,pending_ui_message:false,queue_requested:true},particle,target_selection_required})
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8TargetPlan {pub after:Family8Record,pub rng_after:BlueprintRng,pub target:[i16;2],pub target_height:i32}
/// Exact114B0 initial state1 scan of the64 owner buildings, followed by
/// the source player-position fallback when no eligible completed target exists.
/// The compatibility name is retained for existing fixture callers.
pub fn plan_selector1_initial_empty_building_target(before:&Family8Record,map:&[u8],player_target:[i16;2],
    owner_buildings:&[u8],trig:&TrigTables,rng:BlueprintRng)->Result<Family8TargetPlan,Family8MotionError>{
    if before[0x57]!=0||before[0x5b]!=0{return Err(Family8MotionError::UnsupportedRecord);}
    plan_selector1_building_target(before,map,player_target,owner_buildings,trig,rng)?
        .ok_or(Family8MotionError::UnsupportedRecord)
}

pub fn plan_selector1_building_target(before:&Family8Record,map:&[u8],player_target:[i16;2],
    owner_buildings:&[u8],trig:&TrigTables,rng:BlueprintRng)->Result<Option<Family8TargetPlan>,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||before[0x55]!=1||before[0x56]!=1||before[0x18]>=4||!matches!(before[0x5b],0|4)
        ||owner_buildings.len()!=64*67{return Err(Family8MotionError::UnsupportedRecord);}
    let mut a=*before;let mut rng_after=rng;
    //115B3..1165E: inactive/incomplete and selector2/3/5/6/8/9/10
    // records are skipped. Flag0100 restricts candidates to selector11.
    let first=if before[0x5b]==4 {
        let next=i32::from(w(before,0x59))+1-i32::from(before[0x18])*64;
        if next<0{return Err(Family8MotionError::UnsupportedRecord);}next as usize
    }else{0};
    let candidate=owner_buildings.chunks_exact(67).skip(first).find(|r| {
        let flags=u16::from_le_bytes([r[0x31],r[0x32]]);
        r[6]!=0 && flags&1!=0 && matches!(r[0x2d],0|1|4|7|11)
            && (flags&0x100==0 || r[0x2d]==11)
    });
    let center=if let Some(r)=candidate {
        a[0x59..0x5b].copy_from_slice(&r[0xf..0x11]);a[0x5b]=r[6];
        [i16::from_le_bytes([r[7],r[8]]),i16::from_le_bytes([r[9],r[10]])]
    }else if before[0x57]==0 {player_target}else{return Ok(None)};
    let high=rng_draw(&mut rng_after);let idx=(usize::from(high)*8+1536)&2047;
    //73E00: double signed lowword coordinates, add the lowword scaled
    // radius vector, arithmetic halve, then mask to15 bits.
    let target=std::array::from_fn(|axis|{
        let term=if axis==0{trig.cosine()[idx]}else{trig.sine()[idx]};
        let offset=(2048i32.wrapping_mul(term)>>16)as i16;
        (center[axis].wrapping_mul(2).wrapping_add(offset)>>1)&0x7fff
    });
    for axis in 0..2{pw(&mut a,0x5c+axis*2,center[axis]);pw(&mut a,0xb+axis*2,target[axis]);}
    let target_height=genewars_core::cd_candidate::interpolate_height(map,target[0],target[1]).map_err(|_|Family8MotionError::MapLength)?.wrapping_add(2560);
    a[0x54]=20;a[0x49]|=0x14;a[0x16]&=0x3f;let flags=(w(&a,0x49)as u16&0xeffc)as i16;pw(&mut a,0x49,flags);
    a[0x16]&=0x7f;a[0x4f]=0;a[0x52]=0;pd(&mut a,0x60,target_height);a[0x4a]&=0x3f;
    pw(&mut a,0x64,0);pw(&mut a,0x3f,3840);a[0x57]=a[0x57].wrapping_add(1);
    Ok(Some(Family8TargetPlan{after:a,rng_after,target,target_height}))
}

/// Selector1/state1's initial travel and first arrival (timer64==0).
/// The subsequent positive-timer hover/retarget helper11720 is separate.
pub fn plan_selector1_state1_travel(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng)
    ->Result<Family8OnwardPlan,Family8MotionError>{
    if before[0x56]!=1||before[0x57]!=1{return Err(Family8MotionError::UnsupportedRecord);}
    plan_selector1_travel(before,map,frame,water,physics,trig,animations,rng,|_,_,rng|Ok(rng))
}
/// Source state2 travel shares common motion; arrival increments selected zone18.
pub fn plan_selector1_state2_travel(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,zones:&[u8])
    ->Result<Family8ZoneOnwardPlan,Family8MotionError>{
    plan_selector1_state2_travel_with_particle(before,map,frame,water,physics,trig,animations,rng,zones,|_,_,rng|Ok(rng))
}
pub fn plan_selector1_state2_travel_with_particle<F>(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,zones:&[u8],particle_factory:F)
    ->Result<Family8ZoneOnwardPlan,Family8MotionError>
where F:FnMut(&Family8Record,Family8ParticleRequest,BlueprintRng)->Result<BlueprintRng,Family8MotionError>{
    if before[0x56]!=2||zones.len()%19!=0||zones.len()>64*19||before[0x57]==0||usize::from(before[0x57])>zones.len()/19{return Err(Family8MotionError::UnsupportedRecord);}
    let onward=plan_selector1_travel(before,map,frame,water,physics,trig,animations,rng,particle_factory)?;
    let mut zones_after=zones.to_vec();
    if w(&onward.motion.after,0x64)>0{let index=(usize::from(before[0x57])-1)*19+18;zones_after[index]=zones_after[index].wrapping_add(1);}
    Ok(Family8ZoneOnwardPlan{onward,zones_after})
}
fn plan_selector1_travel<F>(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,mut particle_factory:F)
    ->Result<Family8OnwardPlan,Family8MotionError>
where F:FnMut(&Family8Record,Family8ParticleRequest,BlueprintRng)->Result<BlueprintRng,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=1||!matches!(before[0x56],1|2)||before[0x57]==0||before[0x57]>64
        ||before[0x15]!=0x20||before[0x16]!=0||w(before,0x49)as u16&!0x6024!=0x210
        ||before[0x4f]!=0||before[0x52]!=0||w(before,0x3f)<0||w(before,0x41)<0||w(before,0x64)!=0{
        return Err(Family8MotionError::UnsupportedRecord);
    }
    let mut a=*before;let parity=frame.wrapping_add(i32::from(w(&a,15))as u32);a[0x49]&=0x9f;a[0x49]&=0xf3;
    let dx=delta15(w(&a,7),w(&a,0xb));let dy=delta15(w(&a,9),w(&a,0xd));
    if parity&1!=0 {pw(&mut a,0x4b,crate::actor_motion::angle(dx*2,dy*2,physics.angles())as i16);}
    let distance=target_distance(&a);let relative_height=d(&a,0x27).wrapping_sub(d(&a,0x11));
    if distance<=0x24000 {
        a[0x4a]&=0xbf;
        if distance<=400 {
            let flags=(w(&a,0x49)as u16|0x220)as i16;pw(&mut a,0x49,flags);a[0x4a]&=0x2f;
            //6E9E0(BL1) snaps XY, preserves physical height and clears
            // fractions/current speed; its map relocation is an adapter port.
            for axis in 0..2{let v=w(&a,0xb+axis*2)&0x7fff;pw(&mut a,7+axis*2,v);a[0x43+axis]=0;}
            let at=point(&a);let ground=genewars_core::cd_candidate::interpolate_height(map,at[0],at[1]).map_err(|_|Family8MotionError::MapLength)?;
            let shade=genewars_core::cd_candidate::interpolate_entity_shade(map,at[0],at[1]).map_err(|_|Family8MotionError::MapLength)?;
            pd(&mut a,0x11,ground);a[0x17]=shade;pw(&mut a,0x41,0);
            a[0x16]&=0x3f;a[0x54]=0;a[0x49]&=0xef;for off in [0x2d,0x2f,0x31,0x3d,0x3f]{pw(&mut a,off,0);}
        }else if parity&3==3 {set_target_angles(&mut a,physics)?;}
    }else if relative_height<2880 {
        let pitch=d(&a,0x45);if pitch>=32{pd(&mut a,0x45,pitch.wrapping_sub(16));}a[0x4a]|=0x40;
    }else if a[0x4a]&0x40!=0||parity&1!=0 {a[0x4a]&=0xbf;set_target_angles(&mut a,physics)?;}
    steer_selector1(&mut a,trig)?;
    airborne_physics(&mut a,map,frame,water,physics,trig,d(before,0x11))?;
    if a[0x49]&0x20!=0{pw(&mut a,0x3f,0);pw(&mut a,0x41,0);}
    let speed=w(&a,0x41);let audio=Family8AudioRequest::Moving{index:0x53,mode:2,priority:255,volume:127,initial_parameter:100,
        adjusted_parameter:(100+100*i32::from(speed)/(2*3840))as i16};let mut rng_after=rng;let mut particle=None;
    if speed>256 {
        let yaw=(i32::from(w(&a,0x39))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        let pitch=(i32::from(w(&a,0x3b))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        particle=Some(Family8ParticleRequest{code:0x1a,position:point(&a),physical_height:d(&a,0x27),angles:[yaw as u16,pitch as u16,(w(&a,0x3d)>>2)as u16]});
    }
    if let Some(request)=particle{rng_after=particle_factory(&a,request,rng_after)?;if rng_after.ring>=32{return Err(Family8MotionError::InvalidRng);}}
    a[0x19]=(((((i32::from(w(&a,0x39))>>7)+1)>>1)+1)&7)as u8;
    if a[0x49]&0x20!=0 {
        pw(&mut a,0x3f,7680);
        let first=rng_draw(&mut rng_after);pw(&mut a,0x64,300+4*i16::from(first%100));
        let second=rng_draw(&mut rng_after);let timer=if before[0x56]==2 {500+4*i16::from(second%200)}else{50+4*i16::from(second%50)};pw(&mut a,0x64,timer);
    }
    let direction=usize::from(a[0x19]);animate(&mut a,&animations.directions[direction])?;
    Ok(Family8OnwardPlan{motion:Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after,old_cell:cell(point(before)),child:None,
        pending_audio:audio,pending_ui_message:false,queue_requested:true},particle,target_selection_required:false})
}
fn set_target_angles(a:&mut Family8Record,physics:&PhysicsTables)->Result<(),Family8MotionError>{
    let dx=delta15(w(a,7),w(a,0xb));let dy=delta15(w(a,9),w(a,0xd));
    let horizontal=genewars_core::cd_candidate::integer_sqrt((dx*dx+dy*dy)as u32)as i32;
    let dz=d(a,0x27).wrapping_sub(d(a,0x60))>>3;
    let yaw=crate::actor_motion::angle(dx*2,dy*2,physics.angles())as i16;
    let pitch=crate::actor_motion::angle(horizontal,dz,physics.angles())as i16;
    pw(a,0x4b,yaw);pd(a,0x45,i32::from(pitch));Ok(())
}

/// State1 positive-timer hover and its empty-search transition to state3.
/// The two list inputs close the actual first-mission empty-building/site
/// searches at timer expiry; active targets are an unsupported boundary.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct Family8ZoneOnwardPlan {pub onward:Family8OnwardPlan,pub zones_after:Vec<u8>}
pub fn plan_selector1_state1_hover(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,
    owner_buildings:&[u8],site_count:u8,current_point_accepted:bool)->Result<Family8OnwardPlan,Family8MotionError>{
    let plan=plan_selector1_state1_hover_with_zones(before,map,frame,water,physics,trig,animations,rng,owner_buildings,&[],current_point_accepted)?;
    if site_count!=0&&plan.onward.motion.after[0x56]==3{return Err(Family8MotionError::UnsupportedRecord);}
    Ok(plan.onward)
}
pub fn plan_selector1_state1_hover_with_zones(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,
    owner_buildings:&[u8],zones:&[u8],current_point_accepted:bool)->Result<Family8ZoneOnwardPlan,Family8MotionError>{
    plan_selector1_state1_hover_with_zones_and_particle(before,map,frame,water,physics,trig,animations,rng,owner_buildings,zones,current_point_accepted,|_,_,rng|Ok(rng))
}
/// The factory hook runs at source48790, before timer and114B0 target RNG.
/// Caller must stage external mutations and commit only after this plan succeeds.
pub fn plan_selector1_state1_hover_with_zones_and_particle<F>(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,
    owner_buildings:&[u8],zones:&[u8],current_point_accepted:bool,mut particle_factory:F)->Result<Family8ZoneOnwardPlan,Family8MotionError>
where F:FnMut(&Family8Record,Family8ParticleRequest,BlueprintRng)->Result<BlueprintRng,Family8MotionError>{
    if before[0x56]!=1||before[0x57]!=1{return Err(Family8MotionError::UnsupportedRecord);}
    plan_selector1_hover_with_zones_and_particle(before,map,frame,water,physics,trig,animations,rng,owner_buildings,zones,current_point_accepted,particle_factory)
}
pub fn plan_selector1_state2_hover_with_particle<F>(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,
    zones:&[u8],current_point_accepted:bool,particle_factory:F)->Result<Family8ZoneOnwardPlan,Family8MotionError>
where F:FnMut(&Family8Record,Family8ParticleRequest,BlueprintRng)->Result<BlueprintRng,Family8MotionError>{
    let index=w(before,0x59);
    if before[0x56]!=2||index<0||usize::from(index as u16)>=zones.len()/19||before[0x57]!=index as u8+1{return Err(Family8MotionError::UnsupportedRecord);}
    plan_selector1_hover_with_zones_and_particle(before,map,frame,water,physics,trig,animations,rng,&[],zones,current_point_accepted,particle_factory)
}
fn plan_selector1_hover_with_zones_and_particle<F>(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animations:&Family8DirectionalAnimations,rng:BlueprintRng,
    owner_buildings:&[u8],zones:&[u8],current_point_accepted:bool,mut particle_factory:F)->Result<Family8ZoneOnwardPlan,Family8MotionError>
where F:FnMut(&Family8Record,Family8ParticleRequest,BlueprintRng)->Result<BlueprintRng,Family8MotionError>{
    if zones.len()%19!=0||zones.len()>64*19{return Err(Family8MotionError::UnsupportedRecord);}
    let mut zones_after=zones.to_vec();
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]>=4||before[0x55]!=1||!matches!(before[0x56],1|2)||before[0x57]==0||before[0x57]>64
        ||before[0x15]!=0x20||before[0x16]!=0||w(before,0x49)as u16&!0x6020!=0x200
        ||before[0x4f]!=0||before[0x52]!=0||!matches!(before[0x5b],0|4)||w(before,0x3f)<0||w(before,0x41)<0||w(before,0x64)<=0
        ||(before[0x56]==1&&owner_buildings.len()!=64*67){
        return Err(Family8MotionError::UnsupportedRecord);
    }
    // Family8 A8 is RET; the read-only neighborhood search has no numeric effect.
    let _ = current_point_accepted;
    let mut a=*before;a[0x49]&=0x9f;
    let relative_height=d(&a,0x27).wrapping_sub(d(&a,0x11));
    if relative_height<2880{let pitch=d(&a,0x45);if pitch>=32{pd(&mut a,0x45,pitch.wrapping_sub(16));}a[0x4a]|=0x40;}
    else if a[0x4a]&0x40!=0&&relative_height>=2112{a[0x4a]&=0xbf;}
    steer_selector1(&mut a,trig)?;airborne_physics(&mut a,map,frame,water,physics,trig,d(before,0x11))?;
    if a[0x49]&0x20!=0{pw(&mut a,0x3f,0);pw(&mut a,0x41,0);}
    let speed=w(&a,0x41);let audio=Family8AudioRequest::Moving{index:0x53,mode:2,priority:255,volume:127,initial_parameter:100,
        adjusted_parameter:(100+100*i32::from(speed)/(2*3840))as i16};let mut rng_after=rng;let mut particle=None;
    if speed>256 {
        let yaw=(i32::from(w(&a,0x39))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        let pitch=(i32::from(w(&a,0x3b))+1920+i32::from(rng_draw(&mut rng_after)))&2047;
        particle=Some(Family8ParticleRequest{code:0x1a,position:point(&a),physical_height:d(&a,0x27),angles:[yaw as u16,pitch as u16,(w(&a,0x3d)>>2)as u16]});
    }
    if let Some(request)=particle {rng_after=particle_factory(&a,request,rng_after)?;if rng_after.ring>=32{return Err(Family8MotionError::InvalidRng);}}
    a[0x19]=(((((i32::from(w(&a,0x39))>>7)+1)>>1)+1)&7)as u8;
    let remaining=w(&a,0x64).wrapping_sub(1);pw(&mut a,0x64,remaining);
    let zone_live=before[0x56]!=2||i32::from_le_bytes(zones[w(before,0x59)as usize*19+10..w(before,0x59)as usize*19+14].try_into().unwrap())!=0;
    if remaining>0&&zone_live {
        let center=[w(&a,0x5c),w(&a,0x5e)];let height=genewars_core::cd_candidate::interpolate_height(map,center[0],center[1]).map_err(|_|Family8MotionError::MapLength)?.wrapping_add(4800);
        hover_acceleration(&mut a,center,height,physics,trig)?;
    }else{
        //114B0 continues after the current building. With no next target,
        //site_count0 closes the state2 search and source130CE entersstate3.
        let building=if before[0x56]==1{plan_selector1_building_target(&a,map,[0,0],owner_buildings,trig,rng_after)?}else{None};
        if let Some(next)=building {a=next.after;rng_after=next.rng_after;}
        else{
            if before[0x56]==1{a[0x57]=0;a[0x56]=2;}
            let next=crate::family8_zone_target::plan_zone_target(&a,map,zones,trig,rng_after)?;
            a=next.after;rng_after=next.rng_after;zones_after=next.zones_after;
            if next.selected.is_none(){a[0x57]=0;a[0x56]=3;a[0x5b]=0;a[0x33..0x39].fill(0);}
        }
    }
    let direction=usize::from(a[0x19]);animate(&mut a,&animations.directions[direction])?;
    Ok(Family8ZoneOnwardPlan{onward:Family8OnwardPlan{motion:Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after,old_cell:cell(point(before)),child:None,
        pending_audio:audio,pending_ui_message:false,queue_requested:true},particle,target_selection_required:false},zones_after})
}
fn hover_acceleration(a:&mut Family8Record,target:[i16;2],height:i32,physics:&PhysicsTables,trig:&TrigTables)->Result<(),Family8MotionError>{
    let dx=delta15(w(a,7),target[0]);let dy=delta15(w(a,9),target[1]);let dz=d(a,0x27).wrapping_sub(height)>>3;
    let horizontal=genewars_core::cd_candidate::integer_sqrt((dx*dx+dy*dy)as u32)as i32;
    let yaw=crate::actor_motion::angle(dx*2,dy*2,physics.angles())as i16;let pitch=crate::actor_motion::angle(horizontal,dz,physics.angles())as i16;
    if angular_delta(w(a,0x39),yaw).abs()<256 {
        let mut vector=*a;pw(&mut vector,0x39,yaw);pw(&mut vector,0x3b,pitch);pw(&mut vector,0x3d,192);velocity(&mut vector,trig)?;
        a[0x33..0x39].copy_from_slice(&vector[0x2d..0x33]);pw(a,0x4b,yaw);pd(a,0x45,i32::from(pitch));
    }else{
        a[0x33..0x39].fill(0);let distance=(dx*dx+dy*dy).wrapping_add(dz.wrapping_mul(dz));
        let next_yaw=if distance as u32>0x40000{yaw}else{w(a,0x39)};pw(a,0x4b,next_yaw);
    }Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]fn invalid_inputs_are_rejected_without_mutation(){
        let before=[0u8;102];let copy=before;let rng=BlueprintRng{seed:1,ring:0,draw_count:0};
        let result=plan_selector0_state4(&before,&[],0,0,&PhysicsTables::fixture(),&TrigTables::generate_pinned().unwrap(),&Family8Animation{holds:vec![0;11],flags:1},rng);
        assert_eq!(result,Err(Family8MotionError::MapLength));assert_eq!(before,copy);
    }
}

/// Source132FA: bounded selector1 homeward phases with explicit retirement.
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
    // Family8 A8 is RET; the read-only neighborhood search has no numeric effect.
    let _ = current_point_accepted;
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
    // Family8 A8 is RET; the read-only neighborhood search has no numeric effect.
    let _ = current_point_accepted;
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

/// Selector2 zone-marker descent, exact20-tick activation, and external-event wait.
/// Later departure/retirement remains a separate supported-state boundary.
pub fn plan_zone_marker_wait(before:&Family8Record,map:&[u8],frame:u32,water:i32,
    physics:&PhysicsTables,trig:&TrigTables,animation:&Family8Animation,rng:BlueprintRng)
    ->Result<Family8MotionPlan,Family8MotionError>{
    if map.len()!=229376{return Err(Family8MotionError::MapLength);}
    if rng.ring>=32{return Err(Family8MotionError::InvalidRng);}
    if before[6]!=8||!(0..16).contains(&w(before,15))||before[0x18]!=4||before[0x55]!=2||!matches!(before[0x56],10|11|12)
        ||before[0x19]!=0||before[0x15]!=0x20||before[0x16]&!0x40!=0
        ||w(before,0x49)as u16&0x1413!=before[0x49]as u16&0x10
        ||before[0x4f]!=0||before[0x52]!=0||w(before,0x3f)<0||w(before,0x41)<0 {
        return Err(Family8MotionError::UnsupportedRecord);
    }
    let mut a=*before;return_prefix(&mut a,map,frame,water,physics,trig)?;
    match before[0x56]{
        10=>if a[0x49]&0x20!=0{a[0x56]=11;pw(&mut a,0x64,0);},
        11=>{let timer=w(&a,0x64).wrapping_add(1);pw(&mut a,0x64,timer);if timer==20{a[0x56]=12;a[0x57]=0;}},
        12=>if a[0x57]!=0{a[0x56]=13;pw(&mut a,0x64,0);},
        _=>unreachable!(),
    }
    let speed=w(&a,0x3d);
    let pending_audio=if speed>0{Family8AudioRequest::Moving{index:0x38,mode:2,priority:255,volume:127,initial_parameter:100,adjusted_parameter:(100+20*i32::from(speed)/1920)as i16}}else{Family8AudioRequest::StopIfPlaying{index:0x38,mode:4}};
    animate(&mut a,animation)?;
    Ok(Family8MotionPlan{new_cell:cell(point(&a)),after:a,rng_after:rng,old_cell:cell(point(before)),child:None,pending_audio,pending_ui_message:false,queue_requested:true})
}

//! Native family0/kind1 storage for the exact admitted first-mission source.
//!
//! This is the 5A850 copy/resource-binding boundary, not a complete AI tick.
//! It owns no map. Native pointers are logical handles and are never invoked.

use genewars_assets::ConqMissionSource;

pub const CREATURE_CAPACITY: usize = 300;
pub const NATIVE_CREATURE_BYTES: usize = 124;
pub const PACKED_CREATURE_BYTES: usize = 116;
pub type CreatureRecord = [u8; NATIVE_CREATURE_BYTES];
const MISSION_SHA256: &str = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureAnimationBinding {
    /// 5A1C0 selector4 -> creature loader slot4; exact CD MULE.ANB.
    MuleSelector4,
    /// Source state>=26 routes animation lookups to the admitted BONES.ANB.
    BonesSelector4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreatureError {
    MissionSource(genewars_assets::MissionSourceError),
    MissionHash,
    PackedLength(usize),
    UnsupportedActiveRecord(usize),
    InvalidIdentity(usize),
    InvalidSlot(usize),
    StaleRecord(usize),
    IdentityChanged(usize),
    PointerBytesChanged(usize),
    UnsupportedInitialState,
    InvalidRngRing(u8),
    InitialEnvironment,
    UnsupportedNeighborhood,
    TargetSearchRequired,
    ZeroContactWidth,
    UnsupportedMotionState,
    MotionEnvironment,
    MotionRayBlocked,
    UnsupportedCellEntry,
}

/// Explicit virtual/environment inputs to the shared 6F8F0 -> 6DB00 speed
/// calculation. The caller supplies the virtual B8 distance and virtual10
/// result; this primitive performs no world query or record mutation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommonSpeedInput {
    pub speed: i16,
    pub desired_speed: i16,
    pub width: u16,
    pub flags_4a: u8,
    pub free_flight: bool,
    pub ground_height: i32,
    pub previous_height_low: i16,
    pub surface_term: i32,
    pub distance_squared: i32,
}

/// Exact signed comparisons and low-word stores of the shared contact speed
/// path. `surface_term` is the signed dword [caller EDX+8], not a map byte.
/// A zero width rejects only if the source division is reached. The threshold
/// square and surface multiplication retain the source's i32 wrapping.
pub fn plan_common_contact_speed(input: CommonSpeedInput)->Result<i16,CreatureError> {
    let brake=||if i32::from(input.speed)>i32::from(input.width) {
        input.speed.wrapping_sub(input.width as i16)
    } else { 0 };
    if input.flags_4a&0x20!=0 { return Ok(brake()); }
    if input.flags_4a&0x10==0 {
        if input.width==0 { return Err(CreatureError::ZeroContactWidth); }
        let quotient=(i32::from(input.speed)*i32::from(input.speed))/(i32::from(input.width)<<7);
        if input.distance_squared<=quotient.wrapping_mul(quotient) { return Ok(brake()); }
    }
    let desired=if input.free_flight { i32::from(input.desired_speed) } else {
        let offset=if i32::from(input.previous_height_low)<input.ground_height {0i32} else {128};
        input.surface_term.wrapping_add(offset).wrapping_sub(128)
            .wrapping_mul(i32::from(input.desired_speed))>>7
    };
    let current=i32::from(input.speed);
    let difference=current.wrapping_sub(desired).wrapping_abs();
    Ok(if difference<=i32::from(input.width) {desired as i16}
       else if desired<current {brake()}
       else {input.speed.wrapping_add(input.width as i16)})
}

fn motion_word(r:&[u8],p:usize)->i16 {i16::from_le_bytes([r[p],r[p+1]])}
fn motion_dword(r:&[u8],p:usize)->i32 {i32::from_le_bytes(r[p..p+4].try_into().unwrap())}
fn motion_put(r:&mut[u8],p:usize,v:i16){r[p..p+2].copy_from_slice(&v.to_le_bytes());}
fn motion_put32(r:&mut[u8],p:usize,v:i32){r[p..p+4].copy_from_slice(&v.to_le_bytes());}
fn motion_point(r:&[u8])->[i16;2] {[motion_word(r,7),motion_word(r,9)]}
fn motion_cell(at:[i16;2])->usize {((at[1] as u16>>8)&127)as usize*128+((at[0]as u16>>8)&127)as usize}
fn motion_angles(r:&mut[u8],physics:&crate::physics_tables::PhysicsTables) {
    let (x,y,z)=(i32::from(motion_word(r,0x2d)),i32::from(motion_word(r,0x2f)),i32::from(motion_word(r,0x31)));
    let xy=genewars_core::cd_candidate::integer_sqrt((x*x).wrapping_add(y*y)as u32) as i32;
    if xy!=0 {motion_put(r,0x39,crate::actor_motion::angle(x,y,physics.angles())as i16);}
    let speed=genewars_core::cd_candidate::integer_sqrt(xy.wrapping_mul(xy).wrapping_add(z*z)as u32)as i16;
    motion_put(r,0x3d,speed);
    motion_put(r,0x3b,if speed==0 {512}else{crate::actor_motion::angle(xy,-z,physics.angles())as i16});
}
fn motion_velocity(r:&mut[u8],trig:&crate::actor_activation::TrigTables)->Result<(),CreatureError> {
    let yaw=motion_word(r,0x39);let pitch=motion_word(r,0x3b);let speed=motion_word(r,0x3d);
    if !(0..=2048).contains(&yaw)||!(0..=2048).contains(&pitch)||speed<0{return Err(CreatureError::UnsupportedMotionState);}
    let yi=(i32::from(yaw)-512).rem_euclid(2048)as usize;let pi=(i32::from(pitch)-512).rem_euclid(2048)as usize;
    let horizontal=((i64::from(trig.cosine()[pi])*i64::from(speed))>>16)as i32;
    motion_put(r,0x2d,((i64::from(trig.cosine()[yi])*i64::from(horizontal))>>16)as i16);
    motion_put(r,0x2f,((i64::from(trig.sine()[yi])*i64::from(horizontal))>>16)as i16);
    motion_put(r,0x31,((-i64::from(trig.sine()[pi])*i64::from(speed))>>16)as i16);Ok(())
}
fn motion_fixed(r:&[u8],axis:usize)->i32 {
    (i32::from(motion_word(r,7+2*axis))<<16).wrapping_add(i32::from(r[0x43+axis])<<8)
        .wrapping_add(i32::from(motion_word(r,0x2d+2*axis))<<10)
}
fn motion_sample(map:&[u8],at:[i16;2])->Result<(i32,u8,[i16;4]),CreatureError> {
    let h=genewars_core::cd_candidate::interpolate_height(map,at[0],at[1]).map_err(|_|CreatureError::MotionEnvironment)?;
    let shade=genewars_core::cd_candidate::interpolate_entity_shade(map,at[0],at[1]).map_err(|_|CreatureError::MotionEnvironment)?;
    let (x,y)=((at[0]as u16>>8)&127,(at[1]as u16>>8)&127);
    let corners=std::array::from_fn(|i|{let p=((((y+(i/2)as u16)&127)as usize*128+((x+(i%2)as u16)&127)as usize)*14)+2;motion_word(map,p)});
    Ok((h,shade,corners))
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CommonPhysicsPlan<const N:usize> {pub after:[u8;N],pub old_cell:usize,pub new_cell:usize,pub physics_cell:usize}
/// Shared 6DF30 non-flight numeric path. Water/damage/obstacle callbacks and
/// virtual88 cell-entry effects are explicit rejection/request boundaries.
/// `contact` implements the class-specific A4 speed policy, at the first
/// candidate's sampled normal. No map or entity links are changed here.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct CommonWaterEffect {pub code:u8,pub position:[i16;2],pub physical_height:i32,pub angles:[u16;3]}
pub fn plan_common_nonflight_physics<const N:usize,F,O>(before:&[u8;N],map:&[u8],frame:u32,gravity:i16,water:i32,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    contact:F,obstacle:O)->Result<CommonPhysicsPlan<N>,CreatureError>
where F:FnMut(&[u8;N],[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError> {
    Ok(common_nonflight_physics_water(before,map,frame,gravity,water,physics,trig,contact,obstacle,false,true,|record,_,_|Ok(*record))?.0)
}
///Shared numeric physics with explicit source virtual90 and typed water factory request.
///Zero-height class0 damage remains an unsupported callback boundary.
pub fn plan_common_nonflight_physics_with_water_tail<const N:usize,F,O>(before:&[u8;N],map:&[u8],frame:u32,gravity:i16,water:i32,
 physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,contact:F,obstacle:O,virtual90:bool)->Result<(CommonPhysicsPlan<N>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&[u8;N],[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>{
 common_nonflight_physics_water(before,map,frame,gravity,water,physics,trig,contact,obstacle,true,virtual90,|record,_,_|Ok(*record))
}
/// Source-ordered virtual88 hook after crossing a map cell, before airborne
/// obstacle and water callbacks. Caller owns unlink/prepend and atomicity.
pub fn plan_common_nonflight_physics_with_cell_entry<const N:usize,F,O,C>(before:&[u8;N],map:&[u8],frame:u32,gravity:i16,water:i32,
 physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,contact:F,obstacle:O,admit_water:bool,cell_entry:C)->Result<(CommonPhysicsPlan<N>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&[u8;N],[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,C:FnMut(&[u8;N],usize,usize)->Result<[u8;N],CreatureError>{
 common_nonflight_physics_water(before,map,frame,gravity,water,physics,trig,contact,obstacle,admit_water,true,cell_entry)
}
fn common_nonflight_physics_water<const N:usize,F,O,C>(before:&[u8;N],map:&[u8],frame:u32,gravity:i16,water:i32,
 physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,mut contact:F,mut obstacle:O,admit_water:bool,virtual90:bool,mut cell_entry:C)->Result<(CommonPhysicsPlan<N>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&[u8;N],[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,C:FnMut(&[u8;N],usize,usize)->Result<[u8;N],CreatureError>{
    if N<0x55||map.len()!=229376||before[0x16]&9!=0||before[0x15]&0x20!=0||motion_word(before,0x49)as u16&0x400!=0
        ||(motion_word(before,0x3f)<0&&!(motion_word(before,0x3f)==-1&&N>=124&&before[6]==1&&before[0x5e]>=24&&before[0x15]&0x80!=0
            &&motion_dword(before,0x27)<=motion_dword(before,0x11)&&(0..3).all(|i|motion_word(before,0x33+2*i)==0))) {return Err(CreatureError::UnsupportedMotionState);}
    let mut a=*before;let old_cell=motion_cell(motion_point(before));let old_ground=motion_dword(before,0x11);
    let was_grounded=motion_dword(before,0x27)<=old_ground;
    for axis in 0..3 {let mut v=motion_word(before,0x2d+axis*2).wrapping_add(motion_word(before,0x33+axis*2));if axis==2{v=v.wrapping_sub(gravity);}motion_put(&mut a,0x2d+axis*2,v);}
    let candidate_height=motion_dword(before,0x27).wrapping_add(i32::from(motion_word(&a,0x31))>>3);
    let fixed=[motion_fixed(&a,0),motion_fixed(&a,1)];let at=fixed.map(|v|(v>>16)as i16);
    let (ground,shade,corners)=motion_sample(map,at)?;motion_put32(&mut a,0x11,ground);a[0x17]=shade;
    if candidate_height<=ground {
        motion_put32(&mut a,0x27,ground);
        if was_grounded||before[0x15]&0x40!=0 {
            let normal=crate::landing::landing_terrain_normal(motion_point(before),motion_sample(map,motion_point(before))?.2);
            let corpse=motion_word(before,0x3f)==-1;
            if corpse{
                let sum=(0..3).fold(0u32,|sum,i|{let v=i32::from(motion_word(&a,0x2d+2*i));sum.wrapping_add(v.wrapping_mul(v)as u32)});
                let speed=genewars_core::cd_candidate::integer_sqrt(sum)as i32;
                if speed!=0{
                    let class=crate::world::map_terrain_class(map,motion_point(before)).map_err(|_|CreatureError::MotionEnvironment)?;
                    let friction=i32::from(physics.friction(class).map_err(|_|CreatureError::MotionEnvironment)?);
                    if speed<=friction {a[0x2d..0x33].fill(0);}
                    else{let factor=((speed-friction)<<6)/speed;for axis in 0..3{let v=((i64::from(motion_word(&a,0x2d+2*axis))*i64::from(factor))>>6)as i16;motion_put(&mut a,0x2d+2*axis,v);}}
                }
            }
            motion_put(&mut a,0x31,(ground as i16).wrapping_sub(old_ground as i16).wrapping_mul(8));
            motion_angles(&mut a,physics);
            if !corpse{let speed=contact(&a,normal,old_ground as i16)?;motion_put(&mut a,0x41,speed);motion_put(&mut a,0x3d,speed);}
            motion_velocity(&mut a,trig)?;
            for axis in 0..2 {let v=motion_fixed(&a,axis);motion_put(&mut a,7+2*axis,(v>>16)as i16);a[0x43+axis]=(v>>8)as u8;}
            let (g,s,_)=motion_sample(map,motion_point(&a))?;motion_put32(&mut a,0x11,g);motion_put32(&mut a,0x27,g);a[0x17]=s;
        }else{
            for axis in 0..2 {motion_put(&mut a,7+2*axis,at[axis]);a[0x43+axis]=(fixed[axis]>>8)as u8;}
            let vz=motion_word(&a,0x31);if (i32::from(vz)>>6)/16>0{return Err(CreatureError::UnsupportedMotionState);}
            let normal=crate::landing::landing_terrain_normal(at,corners);
            let class=crate::world::map_terrain_class(map,at).map_err(|_|CreatureError::MotionEnvironment)?;
            let coefficient=physics.bounce(class).map_err(|_|CreatureError::MotionEnvironment)?;
            let bounced=((i32::from(vz).abs()*i32::from(coefficient))>>8)as i16;motion_put(&mut a,0x31,bounced);
            for axis in 0..2 {let v=motion_word(&a,0x2d+2*axis).wrapping_add((i32::from(bounced)*normal[axis]/256)as i16);motion_put(&mut a,0x2d+2*axis,v);}
            motion_angles(&mut a,physics);
        }
    }else{
        if motion_word(before,0x3f)==-1{return Err(CreatureError::UnsupportedMotionState);}
        for axis in 0..2 {motion_put(&mut a,7+2*axis,at[axis]);a[0x43+axis]=(fixed[axis]>>8)as u8;}
        motion_put32(&mut a,0x27,candidate_height.max(0));motion_angles(&mut a,physics);
        if !(0..=0x4b00).contains(&motion_word(&a,0x3d)){return Err(CreatureError::UnsupportedMotionState);}
    }
    let entered_cell=motion_cell(motion_point(&a));
    if entered_cell!=old_cell&&a[0x16]&0x20==0 {a=cell_entry(&a,old_cell,entered_cell)?;}
    // Source only invokes74270 on the odd airborne path, and the water
    // callback every fourth frame. Query ordering remains observable.
    let parity=frame.wrapping_add(motion_word(&a,0x0f)as i32 as u32);
    if parity&1!=0&&motion_dword(&a,0x27)>motion_dword(&a,0x11)&&obstacle(motion_point(&a))? {return Err(CreatureError::MotionEnvironment);}
    let mut water_effect=None;
    if parity&3==3&&(water as u32).wrapping_add(320)>=motion_dword(&a,0x27)as u32 {
        let class=crate::world::map_terrain_class(map,motion_point(&a)).map_err(|_|CreatureError::MotionEnvironment)?;
        if !admit_water {if class<=1&&!(before[6]==1&&motion_word(&a,0x3d)<128&&motion_dword(&a,0x27)!=0){return Err(CreatureError::MotionEnvironment);}}
        else {water_effect=plan_common_water_tail(before,&a,frame,water,class,virtual90)?;}

    }
    for o in [7,9] {let p=motion_word(&a,o)&0x7fff;motion_put(&mut a,o,p);}
    Ok((CommonPhysicsPlan{new_cell:motion_cell(motion_point(&a)),physics_cell:motion_cell(motion_point(&a)),old_cell,after:a},water_effect))
}

///6E810..6E967 water-tail classification after the actual terrain query.
pub fn plan_common_water_tail<const N:usize>(before:&[u8;N],after:&[u8;N],frame:u32,water:i32,class:u8,virtual90:bool)->Result<Option<CommonWaterEffect>,CreatureError>{
 if N<0x54||class>7{return Err(CreatureError::MotionEnvironment);}
 let height=motion_dword(after,0x27);let parity=frame.wrapping_add(motion_word(after,15)as i32 as u32);
 if parity&3!=3||(water as u32).wrapping_add(320)<height as u32||class>1{return Ok(None);}
 let mut effect=None;
 if virtual90&&motion_word(after,0x3d)>=128{
  let above=(motion_dword(before,0x27)as u32)>water as u32;let deep=!above&&(water as u32).wrapping_sub(512)>height as u32;
  let ground=motion_dword(after,0x11);effect=Some(CommonWaterEffect{code:if deep{0x15}else{0x30},position:motion_point(after),physical_height:if deep{height}else{if ground as u32>water as u32{ground}else{water}},angles:if deep{[0,0,128]}else{[0;3]}});
 }
 if height==0&&class==0&&matches!(before[6],1|3)&&after[0x53]&1!=0{return Err(CreatureError::MotionEnvironment);}Ok(effect)
}

/// 6F980 steering for a non-flight class whose virtual74 forbids airborne
/// yaw changes. Recomputed velocity uses the source's signed64 products.
pub fn plan_common_ground_steering<const N:usize>(before:&[u8;N],trig:&crate::actor_activation::TrigTables)->Result<[u8;N],CreatureError> {
    if N<0x4d||motion_word(before,0x49)as u16&0x400!=0{return Err(CreatureError::UnsupportedMotionState);}
    let mut a=*before;let yaw=motion_word(&a,0x39);let target=motion_word(&a,0x4b);let mut changed=false;
    if yaw!=target&&motion_dword(&a,0x27)<=motion_dword(&a,0x11) {
        let delta=i32::from(target)-i32::from(yaw);let alternative=if delta<0 {2048+delta}else{-2048+delta};
        let delta=if delta.abs()<=alternative.abs(){delta}else{alternative};
        if delta.abs()>=256{a[0x4a]|=0x20;}else{a[0x4a]&=!0x20;}
        let step=2*i32::from(motion_word(&a,0x2b)as u16);
        let next=if delta.abs()<=step {target}else{((i32::from(yaw)+if delta<0{-step}else{step})&2047)as i16};
        motion_put(&mut a,0x39,next);changed=true;
    }else if yaw==target {a[0x4a]&=!0x20;}
    if changed {motion_velocity(&mut a,trig)?;}Ok(a)
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct MuleStepContext {pub frame_counter:u32,pub gravity:i16,pub water_height:i32,pub climate:i16}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MuleAnimations {holds:[Vec<u8>;24],flags:[u8;24],bones:Option<BonesAnimations>}
#[derive(Debug,Clone,PartialEq,Eq)]
struct BonesAnimations {holds:[Vec<u8>;5],flags:[u8;5]}
impl MuleAnimations {
    /// Exact admitted CD MULE.ANB; all24 source descriptors are exposed for bounded walking and feeding.
    /// This is metadata access, not another sprite decoder.
    pub fn from_anb(bytes:&[u8])->Result<Self,CreatureError> {
        if bytes.len()!=57038||crate::identity::sha256_hex(bytes)!="f6c62221b52a62088f91ae8d00becfef513282cbf98a5990c1a0859cd8b2d346" {return Err(CreatureError::UnsupportedMotionState);}
        let frames=u32::from_le_bytes(bytes[10..14].try_into().unwrap())as usize;
        let descriptors=u32::from_le_bytes(bytes[14..18].try_into().unwrap())as usize;
        let holds=std::array::from_fn(|i|{let d=descriptors+i*10;let base=u32::from_le_bytes(bytes[d..d+4].try_into().unwrap())as usize;(0..usize::from(bytes[d+8])).map(|j|bytes[frames+base+j*14+10]).collect()});
        let flags=std::array::from_fn(|i|bytes[descriptors+i*10+9]);Ok(Self{holds,flags,bones:None})
    }
    /// Explicit optional binding for corpse states26/27; no MULE fallback.
    pub fn with_bones_anb(mut self,bytes:&[u8])->Result<Self,CreatureError>{
        if bytes.len()!=4077||crate::identity::sha256_hex(bytes)!="2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f"{return Err(CreatureError::UnsupportedMotionState);}
        let frames=u32::from_le_bytes(bytes[10..14].try_into().unwrap())as usize;
        let descriptors=u32::from_le_bytes(bytes[14..18].try_into().unwrap())as usize;
        let holds=std::array::from_fn(|i|{let d=descriptors+i*10;let base=u32::from_le_bytes(bytes[d..d+4].try_into().unwrap())as usize;(0..usize::from(bytes[d+8])).map(|j|bytes[frames+base+j*14+10]).collect()});
        let flags=std::array::from_fn(|i|bytes[descriptors+i*10+9]);self.bones=Some(BonesAnimations{holds,flags});Ok(self)
    }
    fn metadata(&self,state:u8,id:usize)->Result<(&[u8],u8),CreatureError>{
        if state>=26{let bones=self.bones.as_ref().ok_or(CreatureError::UnsupportedMotionState)?;Ok((bones.holds.get(id).ok_or(CreatureError::UnsupportedMotionState)?,*bones.flags.get(id).ok_or(CreatureError::UnsupportedMotionState)?))}
        else{Ok((self.holds.get(id).ok_or(CreatureError::UnsupportedMotionState)?,*self.flags.get(id).ok_or(CreatureError::UnsupportedMotionState)?))}
    }
    /// Read the proved frame hold and animation flags for the state-selected
    /// database. States26+ require the separately hash-bound BONES table;
    /// there is no fallback to the corresponding MULE animation index.
    pub fn frame_metadata(&self,state:u8,id:usize,frame:usize)->Result<(u8,u8),CreatureError>{
        let (holds,flags)=self.metadata(state,id)?;
        let hold=*holds.get(frame).ok_or(CreatureError::UnsupportedMotionState)?;
        if flags&!1!=0 {return Err(CreatureError::UnsupportedMotionState);}
        Ok((hold,flags))
    }
}
/// Admitted straight-move portion of6FB60 through its6F980 steering call.
/// Virtual10/74 must both be false; owner-specific pathfinding, waypoint
/// resolution, flight and the periodic idle event search remain excluded.
/// Query booleans are evaluated by the owned world only when flag49 bit4 is
/// set. A blocked point/ray rejects instead of fabricating a waypoint.
pub fn plan_common_straight_motion_prefix<const N:usize>(before:&[u8;N],frame:u32,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    target_accepted:bool,ray_clear:bool,idle_event_search_suppressed:bool)->Result<[u8;N],CreatureError> {
    if N<0x55||before[0x16]&0x48!=0||motion_word(before,0x49)as u16&0x140b!=0||before[0x4f]!=0||before[0x52]!=0 {
        return Err(CreatureError::UnsupportedMotionState);
    }
    let mut a=*before;let counter=frame.wrapping_add(motion_word(&a,15)as i32 as u32);a[0x49]&=0x9f;
    if a[0x49]&0x10==0&&counter&31==0&&!idle_event_search_suppressed{return Err(CreatureError::UnsupportedMotionState);}
    if a[0x49]&0x10!=0 {
        if a[0x49]&4!=0 {if !target_accepted||!ray_clear{return Err(CreatureError::MotionRayBlocked);}a[0x49]&=0xf3;}
        if counter&1!=0 {
            let dx=motion_word(&a,0x0b).wrapping_sub(motion_word(&a,7)).wrapping_mul(2);
            let dy=motion_word(&a,0x0d).wrapping_sub(motion_word(&a,9)).wrapping_mul(2);
            motion_put(&mut a,0x4b,crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles())as i16);
        }
    }
    plan_common_ground_steering(&a,trig)
}
/// Subsequent owner4/state8 Mule callback while its positive idle/wander
/// timer and healthy energy branch remain admitted. The map is current;
/// `ray_clear` is the actual743C0 query, not an inferred straight path.
/// Cell-entry and intrusive-link changes must be applied by the owned world.
pub fn plan_initial_mule_followup<F,O>(before:&CreatureRecord,map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,target_accepted:bool,ray_clear:bool,idle_event_search_suppressed:bool,
    contact:F,obstacle:O,animations:&MuleAnimations)->Result<CommonPhysicsPlan<124>,CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError> {
    plan_initial_mule_followup_with_choice(before,map,context,physics,trig,target_accepted,ray_clear,idle_event_search_suppressed,contact,obstacle,animations,|_|Err(CreatureError::UnsupportedMotionState))
}
pub fn plan_initial_mule_followup_with_choice<F,O,C>(before:&CreatureRecord,map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,target_accepted:bool,ray_clear:bool,idle_event_search_suppressed:bool,
    contact:F,obstacle:O,animations:&MuleAnimations,choice:C)->Result<CommonPhysicsPlan<124>,CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,C:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError> {
    let prefix=plan_common_straight_motion_prefix(before,context.frame_counter,physics,trig,target_accepted,ray_clear,idle_event_search_suppressed)?;
    plan_mule_after_admitted_prefix(before,&prefix,[motion_word(&prefix,11),motion_word(&prefix,13)],map,context,physics,trig,contact,obstacle,animations,choice)
}
/// Source5ED20/5E0B0 cadence: wrapping frame plus signed native ID.
pub(crate) fn mule_callback_phase(record:&CreatureRecord,frame:u32)->u32 {
    frame.wrapping_add(i32::from(motion_word(record,0x0f)) as u32)
}
fn mule_skill_percent(record:&CreatureRecord,minimum:i32)->i32 {
    minimum+i32::from(record[0x76].min(100))*(100-minimum)/100
}
/// Source5A990 virtual80. Only callers that request a new speed use this;
/// serialized and already-running speed fields remain untouched.
pub(crate) fn mule_requested_speed(record:&CreatureRecord)->i16 {
    ((704*mule_skill_percent(record,75)/100)*(i32::from(record[0x7b])+1)>>8) as i16
}
/// Source6A2BD..6A48F after the health/action prefix, before timer6C.
fn refresh_mule_lifecycle(record:&mut CreatureRecord) {
    let energy=motion_word(record,0x66).wrapping_sub(1);motion_put(record,0x66,energy);
    let percent=mule_skill_percent(record,30);let life=19000*percent/100;
    let reserve=life>>3;let threshold=life-reserve;
    if i32::from(energy)<=threshold {
        record[0x77]=255;let stage=(i32::from(energy)-reserve)/(threshold/3)+1;
        if stage!=i32::from(record[0x79]as i8).abs(){record[0x79]=stage as u8;}
    }else{
        let hunger=((life-i32::from(energy))*255/reserve).max(16)as u8;record[0x77]=hunger;
        let scale62=160*mule_skill_percent(record,40)/100;let scale64=80*percent/100;
        motion_put(record,0x62,((scale62*(i32::from(hunger)+1)>>8)+32)as i16);
        motion_put(record,0x64,((scale64*(i32::from(hunger)+1)>>8)+16)as i16);
    }
}
/// Complete a separately admitted6FB60 route/steering prefix. The owned
/// adapter derives `prefix` and `working_target` from actual point/ray/walk
/// queries; this function does not invent them or change any map links.
pub fn plan_mule_after_admitted_prefix<F,O,C>(before:&CreatureRecord,prefix:&CreatureRecord,working_target:[i16;2],map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    contact:F,obstacle:O,animations:&MuleAnimations,choice:C)->Result<CommonPhysicsPlan<124>,CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,C:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError> {
    Ok(plan_mule_after_admitted_prefix_with_water_policy(before,prefix,working_target,map,context,physics,trig,contact,obstacle,animations,choice,false)?.0)
}
/// Optional typed water factory admission. The source Mule virtual90 is1;
/// requested factories are returned at the physics stage for owned commit.
pub fn plan_mule_after_admitted_prefix_with_water_policy<F,O,C>(before:&CreatureRecord,prefix:&CreatureRecord,working_target:[i16;2],map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    contact:F,obstacle:O,animations:&MuleAnimations,choice:C,admit_water:bool)->Result<(CommonPhysicsPlan<124>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,C:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError> {
    plan_mule_after_admitted_prefix_with_cell_entry(before,prefix,working_target,map,context,physics,trig,contact,obstacle,animations,choice,admit_water,|record,_,_|Ok(*record))
}
pub fn plan_mule_after_admitted_prefix_with_cell_entry<F,O,C,E>(before:&CreatureRecord,prefix:&CreatureRecord,working_target:[i16;2],map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    contact:F,obstacle:O,animations:&MuleAnimations,mut choice:C,admit_water:bool,cell_entry:E)->Result<(CommonPhysicsPlan<124>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError>,C:FnMut(&CreatureRecord)->Result<CreatureRecord,CreatureError>,E:FnMut(&CreatureRecord,usize,usize)->Result<CreatureRecord,CreatureError> {
    if matches!(before[0x5e],24..=27)||(before[0x5e]==8&&motion_word(before,0x66)==1&&mule_callback_phase(before,context.frame_counter)&1!=0) {
        return plan_mule_mortality_after_prefix(before,prefix,map,context,physics,trig,contact,obstacle,animations,admit_water);
    }
    // The serialized idle Mule starts with action flags0; frame0 performs
    // ordinary physics before the first odd-frame choice establishes bit8.
    let neutral=before[0x5e]==8&&matches!(before[0x61],0|0x40|0x80)&&matches!(before[0x60],0|8|0x88);
    let relocation=before[0x5e]==0&&before[0x61]==0x80&&before[0x60]==8&&before[0x57]==0;
    let food_approach=before[0x5e]==1&&before[0x61]==0x40&&matches!(before[0x60],0x48|0xc8)&&before[0x57]==2;
    let feeding_wait=before[0x5e]==7&&before[0x61]==0x40&&before[0x60]==8&&before[0x57]==2;
    if before[6]!=1||before[0x18]!=4||!(neutral||relocation||food_approach||feeding_wait)||before[0x7a]!=4||before[0x76]>90
       ||before[0x7b]!=255||before[0x15]!=0x80||!matches!(before[0x16],2|0x82)
       ||before[0x4f]!=0||before[0x52]!=0||motion_word(before,0x49)as u16&!0x337f!=0
       ||motion_word(before,0x6c)<0||motion_word(before,0x66)<=if mule_callback_phase(before,context.frame_counter)&1==0{0}else{2}{return Err(CreatureError::UnsupportedMotionState);}
    let waiting=before[0x49]&3==3;
    if waiting&&motion_word(prefix,0x3f)!=0{return Err(CreatureError::UnsupportedMotionState);}
    let mut a=*prefix;
    let(plan,water_effect)=plan_common_nonflight_physics_with_cell_entry(&a,map,context.frame_counter,context.gravity,context.water_height,physics,trig,contact,obstacle,admit_water,cell_entry)?;
    a=plan.after;
    if waiting {
        //6FD58..6FD80 restores the pre-steering target speed and returns
        //from6FB60 before ordinary arrival/waypoint postphysics checks.
        motion_put(&mut a,0x3f,motion_word(before,0x3f));
    }else if a[0x49]&3!=0 {
        //70631..7067A: long avoidance waits use the prephysics working
        //target and only complete when grounded, slow and within32 units.
        if a[0x54]>20 {
            if motion_word(&a,0x3d)<=256&&motion_dword(&a,0x27)<=motion_dword(&a,0x11)
                &&common_distance_squared(map,motion_point(&a),working_target)?<=1024{a[0x49]|=3;}
        }else if common_distance_squared(map,motion_point(&a),working_target)?<=4096{a[0x49]|=8;}
    }else if a[0x49]&0x10!=0&&motion_word(&a,0x3d)<=256&&motion_dword(&a,0x27)<=motion_dword(&a,0x11) {
        if common_distance_squared(map,motion_point(&a),working_target)?<=1024{
            //6E9E0 BL1 preserves physical height while snapping ground/shade,
            // then6FB60 clears velocity/speed and marks ordinary arrival.
            motion_put(&mut a,7,working_target[0]&0x7fff);motion_put(&mut a,9,working_target[1]&0x7fff);
            let(g,s,_)=motion_sample(map,motion_point(&a))?;motion_put32(&mut a,0x11,g);a[0x17]=s;
            a[0x43]=0;a[0x44]=0;motion_put(&mut a,0x41,0);a[0x2d..0x33].fill(0);motion_put(&mut a,0x3d,0);motion_put(&mut a,0x3f,0);a[0x54]=0;
            let flags=(motion_word(&a,0x49)as u16&0xfdcf)|0x220;motion_put(&mut a,0x49,flags as i16);a[0x16]&=0x3f;
        }
    }
    if mule_callback_phase(before,context.frame_counter)&1!=0 {
        let energy=motion_word(&a,0x66).wrapping_sub(1);motion_put(&mut a,0x66,energy);
        let reserve=motion_word(&a,0x68);let health=motion_word(&a,0x6a);let maximum=motion_word(&a,0x64);
        if reserve>0&&health<maximum&&(mule_callback_phase(before,context.frame_counter)%20)<=1 {
            motion_put(&mut a,0x6a,health.wrapping_add((maximum-health).min(2)));
            motion_put(&mut a,0x68,if reserve<3{0}else{reserve-3});
        }else if reserve<=0&&mule_callback_phase(before,context.frame_counter)&31<=1{return Err(CreatureError::UnsupportedMotionState);}
        // Type1 virtualB0=1EC10 tests49mask0100.5E0B0 resets the
        // failed action and chooses a one-draw wait before ordinary AI.
        if motion_word(&a,0x49)as u16&0x100!=0 {a=choice(&a)?;}
        else if a[0x60]&0x80!=0 {
            let class=crate::world::map_terrain_class(map,motion_point(&a)).map_err(|_|CreatureError::MotionEnvironment)?;
            let mask=if matches!(class,2|5){63}else{127};
            if motion_word(&a,0x6c)&mask==0{let reserve=motion_word(&a,0x68);motion_put(&mut a,0x68,if reserve<1{0}else{reserve-1});}
        }
        if mule_callback_phase(before,context.frame_counter)&15<=1&&context.climate.wrapping_add(motion_word(map,motion_cell(motion_point(&a))*14+9))<=25{return Err(CreatureError::MotionEnvironment);}
        if a[0x60]&0x80!=0&&motion_word(&a,0x49)as u16&0x300!=0 {a[0x60]&=0x7f;if a[0x61]&0x80!=0&&a[0x6e..0x72]==a[11..15]{a[0x61]&=0x7f;}motion_put(&mut a,0x6c,0);}
        if a[0x5e]==0&&motion_word(&a,0x6c)==0 {a=choice(&a)?;}
        if before[0x5e]==7 {
            if common_distance_squared(map,motion_point(&a),working_target)?>4096 {return Err(CreatureError::UnsupportedMotionState);}
            if motion_word(&a,0x6c)&15==0 {a=choice(&a)?;}
        }
        if a[0x5e]==8&&motion_word(&a,0x6c)==0 {a=choice(&a)?;}
        //65B4D's owner4/skill<=90 approach callback5D360 is a no-op.
        //Arrival requires the separately visible plant reservation/state7
        //mutation; do not silently continue the walking-only plan.
        if a[0x5e]==1&&motion_word(&a,0x49)as u16&0x200!=0&&a[0x60]&0x80==0 {a=choice(&a)?;}
        a[0x4a]&=0xfc;refresh_mule_lifecycle(&mut a);
        let timer=motion_word(&a,0x6c);if timer>0 {motion_put(&mut a,0x6c,timer-1);}
    }
    else if a[0x60]&0x80!=0&&motion_word(&a,0x49)as u16&0x300!=0 {a[0x60]&=0x7f;if a[0x61]&0x80!=0&&a[0x6e..0x72]==a[11..15]{a[0x61]&=0x7f;}motion_put(&mut a,0x6c,0);}
    let yaw=motion_word(&a,0x39);if !(0..=2048).contains(&yaw){return Err(CreatureError::UnsupportedMotionState);}
    if a[0x5e]==7 {
        if a[0x19]!=19 {a[0x19]=19;a[0x1a]=0;a[0x1b]=0;a[0x1c]=1;a[0x1d]=2;a[0x1e]=0;}
    }else{
        a[0x19]=(((((i32::from(yaw)>>7)+1)>>1)+1)&7)as u8;
        let speed=motion_word(&a,0x41);
        if speed==0&&a[0x5e]!=0 {a[0x1b]=0;a[0x1c]=0;a[0x1d]=0;}else{a[0x1c]=1;a[0x1d]=((1024-i32::from(speed))/256)as u8;}
    }
    let animation_holds=&animations.holds[usize::from(a[0x19])];let animation_flags=animations.flags[usize::from(a[0x19])];
    if animation_holds.is_empty()||animation_holds.len()>255||animation_flags&!1!=0{return Err(CreatureError::UnsupportedMotionState);}
    // Shared7AC00 also admits frozen step0, unlike the moving-only actor API.
    let last=animation_holds.len()-1;let frame=usize::from(a[0x1b]).min(last);a[0x1b]=frame as u8;
    if(a[0x1e]as i8)>(a[0x1d]as i8){a[0x1e]=a[0x1d];}
    let counter=a[0x1e]as i8;a[0x1e]=a[0x1e].wrapping_sub(1);
    if counter<=0 {a[0x1e]=a[0x1d];a[0x1a]=a[0x1a].min(animation_holds[frame]);let hold=a[0x1a];a[0x1a]=hold.wrapping_sub(1);
        if hold==0 {let next=frame as i32+i32::from(a[0x1c]as i8);let next=if next>last as i32 {if animation_flags&1!=0 {if last==0{return Err(CreatureError::UnsupportedMotionState);}a[0x1c]=255;last-1}else{0}}
          else if next<0 {if animation_flags&1!=0{a[0x1c]=1;1}else{last}}else{next as usize};a[0x1b]=next as u8;a[0x1a]=animation_holds[next];}
    }
    Ok((CommonPhysicsPlan{new_cell:motion_cell(motion_point(&a)),after:a,old_cell:plan.old_cell,physics_cell:plan.new_cell},water_effect))
}
/// Natural owner4 Mule mortality, grounded corpse friction and BONES decay.
/// External references, airborne corpse motion and final retirement reject.
fn plan_mule_mortality_after_prefix<F,O>(before:&CreatureRecord,prefix:&CreatureRecord,map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    contact:F,obstacle:O,animations:&MuleAnimations,admit_water:bool)->Result<(CommonPhysicsPlan<124>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError> {
    plan_mule_mortality_after_prefix_inner(before,prefix,map,context,physics,trig,contact,obstacle,animations,admit_water,false)
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct MuleRetirementPlan {
    /// Numeric epilogue afterimage retains kind1 for the owning transaction.
    /// Unlink8ACC0, cleanup724C0, count decrement and final kind clear are ports.
    pub motion:CommonPhysicsPlan<124>,
    pub retirement_required:bool,
}
/// Opt-in stationary final BONES callback. Legacy mortality planners retain
/// their atomic retirement-boundary rejection. The owner must commit source
/// unlink/cleanup/kind/count mutations together with this complete epilogue.
pub fn plan_mule_bones_retirement_after_prefix(before:&CreatureRecord,prefix:&CreatureRecord,map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,animations:&MuleAnimations)
    ->Result<MuleRetirementPlan,CreatureError>{
    if before[0x5e]!=27||mule_callback_phase(before,context.frame_counter)&1==0||motion_word(before,0x6c)&15!=0
        ||!(0..=1).contains(&motion_word(before,0x68))||motion_word(prefix,0x3d)!=0||motion_word(prefix,0x41)<0
        ||(0..3).any(|i|motion_word(prefix,0x2d+2*i)!=0){return Err(CreatureError::UnsupportedMotionState);}
    let (motion,effect)=plan_mule_mortality_after_prefix_inner(before,prefix,map,context,physics,trig,
        |_,_,_|Err(CreatureError::UnsupportedMotionState),|_|Ok(false),animations,true,true)?;
    if effect.is_some(){return Err(CreatureError::MotionEnvironment);}
    Ok(MuleRetirementPlan{motion,retirement_required:true})
}

fn plan_mule_mortality_after_prefix_inner<F,O>(before:&CreatureRecord,prefix:&CreatureRecord,map:&[u8],context:MuleStepContext,
    physics:&crate::physics_tables::PhysicsTables,trig:&crate::actor_activation::TrigTables,
    contact:F,obstacle:O,animations:&MuleAnimations,admit_water:bool,admit_retirement:bool)->Result<(CommonPhysicsPlan<124>,Option<CommonWaterEffect>),CreatureError>
where F:FnMut(&CreatureRecord,[i32;3],i16)->Result<i16,CreatureError>,O:FnMut([i16;2])->Result<bool,CreatureError> {
    let entering=before[0x5e]==8&&motion_word(before,0x66)==1&&mule_callback_phase(before,context.frame_counter)&1!=0;
    if before[6]!=1||before[0x18]!=4||before[0x7a]!=4||before[0x76]>90||before[0x7b]!=255
        ||before[0x15]!=0x80||!matches!(before[0x16],2|0x82)||before[0x57]!=0||before[0x5d]!=0||before[0x60]&2!=0
        ||before[0x4f]!=0||before[0x52]!=0||before[0x61]!=0||motion_word(before,0x6c)<0
        ||(!entering&&!matches!(before[0x5e],24..=27)){return Err(CreatureError::UnsupportedMotionState);}
    if before[0x5e]>=26&&(animations.bones.is_none()||before[0x19]!=4||before[0x1b]>1||before[0x1c]!=0||before[0x1d]!=0){return Err(CreatureError::UnsupportedMotionState);}
    let (plan,water_effect)=if entering {
        if admit_water{plan_common_nonflight_physics_with_water_tail(prefix,map,context.frame_counter,context.gravity,context.water_height,physics,trig,contact,obstacle,true)?}
        else{(plan_common_nonflight_physics(prefix,map,context.frame_counter,context.gravity,context.water_height,physics,trig,contact,obstacle)?,None)}
    }else{
        if motion_word(before,0x66)!=0||motion_word(prefix,0x3f)!=-1||motion_word(prefix,0x41)<0||motion_word(prefix,0x3d)<0
            ||(0..3).any(|i|motion_word(prefix,0x33+i*2)!=0)
            ||motion_dword(prefix,0x27)!=motion_dword(prefix,0x11){return Err(CreatureError::UnsupportedMotionState);}
        //Source skips A4 and the3D<-41 copy at desired=-1. For a stationary corpse,
        //the zero-speed contact arithmetic with desired0 is identical; the
        //constant closure has no world-query effects and restores sentinel.
        let stationary=(0..3).all(|i|motion_word(prefix,0x2d+i*2)==0)&&motion_word(prefix,0x3d)==0&&motion_word(prefix,0x41)==0;
        let mut numeric=*prefix;if stationary{motion_put(&mut numeric,0x3f,0);}
        let (mut result,effect)=if admit_water{plan_common_nonflight_physics_with_water_tail(&numeric,map,context.frame_counter,context.gravity,context.water_height,physics,trig,|_,_,_|Ok(0),obstacle,true)?}
            else{(plan_common_nonflight_physics(&numeric,map,context.frame_counter,context.gravity,context.water_height,physics,trig,|_,_,_|Ok(0),obstacle)?,None)};
        motion_put(&mut result.after,0x3f,-1);(result,effect)
    };
    let mut a=plan.after;
    if entering {
        if motion_word(&a,0x6a)<=0{return Err(CreatureError::UnsupportedMotionState);}
        a[0x5e]=24;a[0x15]&=0xdf;motion_put(&mut a,0x33,0);motion_put(&mut a,0x35,0);
        let flags=motion_word(&a,0x60)as u16&0xd71a;motion_put(&mut a,0x60,flags as i16);a[0x5d]=0;a[0x57]=0;
        let flags=motion_word(&a,0x15)as u16&0xf7df&0x3fdf;motion_put(&mut a,0x15,flags as i16);
        motion_put(&mut a,0x6c,32767);a[0x54]=0;motion_put(&mut a,0x3f,-1);a[0x78]=0;
        let flags=motion_word(&a,0x49)as u16&0x3bef;motion_put(&mut a,0x49,flags as i16);a[0x60]&=0xef;motion_put(&mut a,0x66,0);
    }else if a[0x5e]==24 {
        let id=usize::from(a[0x19]);let holds=animations.holds.get(id).ok_or(CreatureError::UnsupportedMotionState)?;
        if holds.is_empty(){return Err(CreatureError::UnsupportedMotionState);}
        let complete=if animations.flags[id]&1!=0{a[0x1a]==0&&a[0x1b]==0}
            else{a[0x1a]==0&&a[0x1e]==0&&usize::from(a[0x1b])==holds.len()-1};
        if complete{motion_put(&mut a,0x6c,0);}
        if mule_callback_phase(before,context.frame_counter)&1!=0&&motion_word(&a,0x6c)<=0{
            a[0x5e]=25;let reserve=motion_word(&a,0x68).wrapping_mul(2).max(motion_word(&a,0x62)>>2);motion_put(&mut a,0x68,reserve);motion_put(&mut a,0x6c,32767);
        }
    }
    if mule_callback_phase(before,context.frame_counter)&1!=0{
        match a[0x5e]{
          25 if motion_word(&a,0x6c)&31==0=>{
            let reserve=motion_word(&a,0x68).saturating_sub(1).max(0);motion_put(&mut a,0x68,reserve);motion_put(&mut a,0x6c,32767);
            if reserve<=motion_word(&a,0x62)>>1{
                if animations.bones.is_none(){return Err(CreatureError::UnsupportedMotionState);}
                a[0x5e]=26;a[0x19]=4;a[0x1b]=0;a[0x1c]=0;
            }
          },
          26 if motion_word(&a,0x6c)&31==0=>{
            let reserve=motion_word(&a,0x68).saturating_sub(1).max(0);motion_put(&mut a,0x68,reserve);motion_put(&mut a,0x6c,32767);
            if reserve<=motion_word(&a,0x62)>>2{a[0x5e]=27;a[0x1b]=1;}
          },
          27 if motion_word(&a,0x6c)&15==0=>{
            let reserve=motion_word(&a,0x68).saturating_sub(1).max(0);
            if reserve<=0&&!admit_retirement{return Err(CreatureError::UnsupportedMotionState);}
            motion_put(&mut a,0x68,reserve);motion_put(&mut a,0x6c,32767);
          },
          _=>{},
        }
        let timer=motion_word(&a,0x6c);if timer>0{motion_put(&mut a,0x6c,timer-1);}
    }
    let animation=match a[0x5e]{24=>17,25=>18,_=>4};
    if a[0x19]!=animation{a[0x19]=animation;a[0x1a]=0;a[0x1b]=0;a[0x1c]=if animation==17{1}else{0};a[0x1d]=0;a[0x1e]=0;}
    advance_mule_mortality_frame(&mut a,animations)?;
    Ok((CommonPhysicsPlan{new_cell:motion_cell(motion_point(&a)),after:a,old_cell:plan.old_cell,physics_cell:plan.new_cell},water_effect))
}
fn advance_mule_mortality_frame(a:&mut CreatureRecord,animations:&MuleAnimations)->Result<(),CreatureError>{
    let id=usize::from(a[0x19]);let (holds,flags)=animations.metadata(a[0x5e],id)?;
    if holds.is_empty()||holds.len()>255||flags&!1!=0{return Err(CreatureError::UnsupportedMotionState);}
    let last=holds.len()-1;let frame=usize::from(a[0x1b]).min(last);a[0x1b]=frame as u8;
    if(a[0x1e]as i8)>(a[0x1d]as i8){a[0x1e]=a[0x1d];}
    let counter=a[0x1e]as i8;a[0x1e]=a[0x1e].wrapping_sub(1);
    if counter<=0{a[0x1e]=a[0x1d];a[0x1a]=a[0x1a].min(holds[frame]);let hold=a[0x1a];a[0x1a]=hold.wrapping_sub(1);
        if hold==0{let next=frame as i32+i32::from(a[0x1c]as i8);let next=if next>last as i32{if flags&1!=0{if last==0{return Err(CreatureError::UnsupportedMotionState);}a[0x1c]=255;last-1}else{0}}
            else if next<0{if flags&1!=0{a[0x1c]=1;1}else{last}}else{next as usize};a[0x1b]=next as u8;a[0x1a]=holds[next];}
    }Ok(())
}
/// Source736F0 ground-aware squared distance, with signed15 modular XY.
pub fn common_distance_squared(map:&[u8],from:[i16;2],to:[i16;2])->Result<i32,CreatureError> {
    let a=genewars_core::cd_candidate::interpolate_height(map,from[0],from[1]).map_err(|_|CreatureError::MotionEnvironment)?;
    let b=genewars_core::cd_candidate::interpolate_height(map,to[0],to[1]).map_err(|_|CreatureError::MotionEnvironment)?;
    let z=b.wrapping_sub(a)>>3;
    Ok((genewars_core::cd_candidate::distance_squared_wrapping_15(from,to)as i32).wrapping_add(z.wrapping_mul(z)))
}

const WALK_STEPS:[[i16;2];8]=[[0,-128],[128,-128],[128,0],[128,128],[0,128],[-128,128],[-128,0],[-128,-128]];
/// Source305F0 with radius2048, eight cell rings and callback6D230.
/// Queries retain source order. A success skips later ring/minor groups,
/// while the current group continues and its last accepted point wins.
/// This variant has no75080 rounding or RNG; coordinates wrap as words.
pub fn find_common_idle_relocation<F>(current:[i16;2],mut accepts:F)->Result<Option<[i16;2]>,CreatureError>
where F:FnMut([i16;2])->Result<bool,CreatureError> {
    for ring in 0..=8i16 {
        for minor in 0..=ring {
            let (i,j)=(minor*256,ring*256);
            let mut offsets=Vec::with_capacity(8);
            if ring==0 {offsets.push([0,0]);}
            else {
                if minor!=0 {offsets.extend([[i,j],[j,-i],[-i,-j],[-j,i]]);}
                if minor!=ring {offsets.extend([[j,i],[i,-j],[-j,-i],[-i,j]]);}
            }
            let mut found=None;
            for delta in offsets {
                let point=std::array::from_fn(|k|current[k].wrapping_add(delta[k]));
                if genewars_core::cd_candidate::distance_squared_wrapping_15(current,point)<=0x400000&&accepts(point)? {found=Some(point);}
            }
            if found.is_some(){return Ok(found);}
        }
    }
    Ok(None)
}
/// Bounded type1 virtualA8 (5DF20 ->5DF50) from an idle state8 with no
/// live action references. The optional portrait refresh is caller-owned.
pub fn plan_mule_idle_relocation(before:&CreatureRecord,point:[i16;2])->Result<CreatureRecord,CreatureError> {
    if before[6]!=1||before[0x18]!=4||before[0x5e]!=8||before[0x60]&2!=0||before[0x57]!=0||before[0x5d]!=0 {return Err(CreatureError::UnsupportedMotionState);}
    let mut a=*before;a[0x15]&=0xdf;a[0x33..0x37].fill(0);let flags=motion_word(&a,0x60)as u16&0xd71a;motion_put(&mut a,0x60,flags as i16);a[0x5d]=0;a[0x57]=0;motion_put(&mut a,0x6c,0);
    a[0x5e]=0;a[0x60]|=8;motion_put(&mut a,0x72,point[0]);motion_put(&mut a,0x74,point[1]);motion_put(&mut a,0x6e,point[0]);motion_put(&mut a,0x70,point[1]);a[0x61]|=0x80;a[0x5f]=0;Ok(a)
}
fn walk_point(at:[i16;2],direction:u8)->[i16;2]{std::array::from_fn(|i|at[i].wrapping_add(WALK_STEPS[usize::from(direction)][i]))}
fn boundary_direction<F>(at:[i16;2],mut direction:u8,clockwise:bool,accept:&mut F)->Result<u8,CreatureError>
where F:FnMut([i16;2])->Result<bool,CreatureError> {
    let original=accept(walk_point(at,direction))?;
    for _ in 0..8 {
        let forward=if original{clockwise}else{!clockwise};direction=direction.wrapping_add(if forward{1}else{7})&7;
        let next=accept(walk_point(at,direction))?;
        if next!=original {return Ok(if original{direction.wrapping_add(if clockwise{7}else{1})&7}else{direction});}
    }Ok(direction)
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct CommonWaypoint {pub cost:u8,pub point:Option<[i16;2]>}
/// Source6F0C0/6EDE0 boundary walk for an admitted null associated obstacle.
/// Both predicates are actual source74D90/743C0 queries. The128-entry visited
/// table, cycle fallback, source step limit and output choice are retained.
pub fn plan_common_waypoint<F,R>(position:[i16;2],start:[i16;2],target:[i16;2],side:bool,limit:u8,
    physics:&crate::physics_tables::PhysicsTables,mut accept:F,mut ray:R)->Result<CommonWaypoint,CreatureError>
where F:FnMut([i16;2])->Result<bool,CreatureError>,R:FnMut([i16;2],[i16;2])->Result<bool,CreatureError> {
    if limit==0||limit>120{return Err(CreatureError::UnsupportedMotionState);}
    if !accept(target)? {return Ok(CommonWaypoint{cost:limit.wrapping_add(1),point:None});}
    let dx=target[0].wrapping_sub(start[0]).wrapping_mul(2);let dy=target[1].wrapping_sub(start[1]).wrapping_mul(2);
    let angle=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles());
    let mut direction=(((u32::from(angle)+128)>>8)&7)as u8;
    direction=boundary_direction(start,direction,!side,&mut accept)?;
    let mut visited:[Option<[i16;2]>;128]=[None;128];let mut point=start;let mut output=start;let mut first_point=None;let mut best_step=0u8;let mut steps=0u8;
    loop {
        let hash=point[0].wrapping_add(point[1])as u16&127;let mut at=usize::from(hash);let mut repeat=false;
        for _ in 0..128 {match visited[at]{None=>break,Some(old)if old==point=>{repeat=true;break;},_=>at=(at+1)&127}}
        if repeat {if best_step>steps/2 {output=first_point.ok_or(CreatureError::UnsupportedMotionState)?;}return Ok(CommonWaypoint{cost:limit+1,point:Some(output)});}
        direction=boundary_direction(point,direction,side,&mut accept)?;
        let mut at=usize::from(hash);for _ in 0..128 {if visited[at].is_none(){break;}at=(at+1)&127;}
        if visited[at].is_some(){return Err(CreatureError::UnsupportedMotionState);}visited[at]=Some(point);
        point=walk_point(point,direction);
        let choose=if first_point.is_none(){true}else{
            ray(point,position)?&&genewars_core::cd_candidate::distance_squared_wrapping_15(position,point)>=genewars_core::cd_candidate::distance_squared_wrapping_15(position,output)
        };
        if choose {if first_point.is_none(){first_point=Some(point);}output=point;best_step=steps;}
        let finished=ray(point,target)?;steps=steps.wrapping_add(1);
        if finished||steps>=limit{return Ok(CommonWaypoint{cost:steps,point:Some(output)});}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreatureRngPurpose { NeighborhoodX, NeighborhoodY, IdleDelay, WanderAngle, WanderRadius, RetryDelay }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureRngDraw { pub purpose: CreatureRngPurpose, pub seed_before:u16, pub seed_after:u16, pub value:u8 }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitialMuleChoice {
    pub before:CreatureRecord,
    pub after:CreatureRecord,
    pub rng_before:crate::blueprint::BlueprintRng,
    pub rng_after:crate::blueprint::BlueprintRng,
    pub draws:Vec<CreatureRngDraw>,
    pub sampled_points:Vec<([i16;2],bool)>,
    pub chosen_target:Option<[i16;2]>,
    /// 83140 is requested after the numeric update; admission is caller-owned.
    pub queue_requested:bool,
}
fn draw(rng:&mut crate::blueprint::BlueprintRng,purpose:CreatureRngPurpose,trace:&mut Vec<CreatureRngDraw>)->u8 {
    let before=rng.seed;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=rng.ring.wrapping_add(1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
    let value=(rng.seed>>8)as u8;trace.push(CreatureRngDraw{purpose,seed_before:before,seed_after:rng.seed,value});value
}
///5E0B0 retry-exhaustion reset with a separately proved false5B100 cell
/// predicate. The admitted idle Mule has no selected/live action to release.
/// Call after the source energy/regeneration prefix, before ordinary AI.
pub fn plan_mule_retry_wait(before:&CreatureRecord,rng_before:crate::blueprint::BlueprintRng)->Result<InitialMuleChoice,CreatureError> {
    if rng_before.ring>=32||before[6]!=1||before[0x18]!=4||before[0x5e]!=8||before[0x7a]!=4
       ||motion_word(before,0x49)as u16&0x100==0||before[0x60]&2!=0||before[0x57]!=0||before[0x5d]!=0 {return Err(CreatureError::UnsupportedMotionState);}
    let mut after=*before;let mut rng=rng_before;let mut draws=Vec::new();
    after[0x15]&=0xdf;after[0x33..0x37].fill(0);let flags=motion_word(&after,0x60)as u16&0xd71a;motion_put(&mut after,0x60,flags as i16);after[0x5d]=0;after[0x57]=0;
    let delay=i16::from(draw(&mut rng,CreatureRngPurpose::RetryDelay,&mut draws)%20)+10;after[0x5e]=8;motion_put(&mut after,0x6c,delay);
    Ok(InitialMuleChoice{before:*before,after,rng_before,rng_after:rng,draws,sampled_points:Vec::new(),chosen_target:None,queue_requested:true})
}
/// Owner4 living/neutral-neighborhood rechoice at timer0, after the common
/// movement and5E0B0 prefix. Caller establishes the neighborhood scorer's
/// no-action gates and false current-cell predicate before invoking this.
pub fn plan_neutral_mule_rechoice<F>(before:&CreatureRecord,rng_before:crate::blueprint::BlueprintRng,tables:&crate::actor_activation::TrigTables,mut accept:F)->Result<InitialMuleChoice,CreatureError>
where F:FnMut([i16;2])->Result<bool,CreatureError> {
    if rng_before.ring>=32||before[6]!=1||before[0x18]!=4||before[0x5e]!=8||before[0x7a]!=4||before[0x76]>90||before[0x7b]!=255
        ||motion_word(before,0x6a)<=0||motion_word(before,0x6c)!=0 {return Err(CreatureError::UnsupportedMotionState);}
    let mut after=*before;let mut rng=rng_before;let mut draws=Vec::new();let mut sampled_points=Vec::new();
    draw(&mut rng,CreatureRngPurpose::NeighborhoodX,&mut draws);draw(&mut rng,CreatureRngPurpose::NeighborhoodY,&mut draws);
    if after[0x60]&8==0 {after[0x72..0x76].copy_from_slice(&before[7..11]);after[0x60]|=8;}
    let mut chosen_target=None;
    let explicit_relocation=after[0x61]&0x80!=0;
    let delay=if explicit_relocation {false}else{draw(&mut rng,CreatureRngPurpose::IdleDelay,&mut draws)%10<4};
    if delay {motion_put(&mut after,0x6c,20);}else {
        if explicit_relocation {let point=[motion_word(&after,0x6e),motion_word(&after,0x70)];let accepted=accept(point)?;sampled_points.push((point,accepted));if !accepted{return Err(CreatureError::TargetSearchRequired);}chosen_target=Some(point);}
        else {
        for _ in 0..16 {
            let angle=usize::from(draw(&mut rng,CreatureRngPurpose::WanderAngle,&mut draws))*8;
            let radius=(2048*i32::from(draw(&mut rng,CreatureRngPurpose::WanderRadius,&mut draws)))>>8;let index=angle.wrapping_sub(512)&2047;
            let deltas=[(tables.cosine()[index]*(radius*2))>>16,(tables.sine()[index]*(radius*2))>>16];
            let anchor=[motion_word(&after,0x72),motion_word(&after,0x74)];let point=std::array::from_fn(|i|((anchor[i].wrapping_mul(2).wrapping_add(deltas[i]as i16)>>1)as u16&0x7fff)as i16);
            let accepted=accept(point)?;sampled_points.push((point,accepted));if accepted{chosen_target=Some(point);break;}after[0x49]|=0x40;
        }
        }
        let point=chosen_target.ok_or(CreatureError::TargetSearchRequired)?;
        motion_put(&mut after,11,point[0]);motion_put(&mut after,13,point[1]);after[0x16]&=0x3f;after[0x49]|=0x14;after[0x54]=20;
        let flags=motion_word(&after,0x49)as u16&0xeffc;motion_put(&mut after,0x49,flags as i16);after[0x4f]=0;after[0x52]=0;
        let speed=mule_requested_speed(&after);motion_put(&mut after,0x3f,speed);motion_put(&mut after,0x6c,32767);after[0x60]|=0x80;
    }
    Ok(InitialMuleChoice{before:*before,after,rng_before,rng_after:rng,draws,sampled_points,chosen_target,queue_requested:true})
}
/// Exact neutral type2 acquisition at5B580 for the admitted Mule. The
/// owned caller supplies the scorer-selected active plant and establishes
/// reserve eligibility; this plan consumes only the two scan-order draws.
pub fn plan_mule_plant_acquisition(before:&CreatureRecord,target_id:i16,target_position:[i16;2],rng_before:crate::blueprint::BlueprintRng)->Result<InitialMuleChoice,CreatureError> {
    if rng_before.ring>=32||target_id<0||target_id>=512||before[6]!=1||before[0x18]!=4||before[0x5e]!=8||before[0x7a]!=4||before[0x76]>90||before[0x7b]!=255
       ||motion_word(before,0x6c)!=0||before[0x78]!=0||before[0x60]&2!=0||before[0x55..0x58][2]!=0
       ||(motion_word(before,0x68)>motion_word(before,0x62)>>1&&before[0x61]&0x40==0) {return Err(CreatureError::UnsupportedMotionState);}
    let mut a=*before;let mut rng=rng_before;let mut draws=Vec::new();
    draw(&mut rng,CreatureRngPurpose::NeighborhoodX,&mut draws);draw(&mut rng,CreatureRngPurpose::NeighborhoodY,&mut draws);
    a[0x61]|=0x40;
    //5CC60 state8 reset. Null reference indices remain opaque.
    a[0x15]&=0xdf;a[0x33..0x37].fill(0);let flags=motion_word(&a,0x60)as u16&0xd71a;motion_put(&mut a,0x60,flags as i16);a[0x5d]=0;a[0x57]=0;motion_put(&mut a,0x6c,0);
    motion_put(&mut a,0x55,target_id);a[0x57]=2;
    //Mapped5B56C[1]=5B6F9: plant uses a literal destination, unlike the
    //kind1/kind3 branches that retain a live movement reference50.
    motion_put(&mut a,11,target_position[0]);motion_put(&mut a,13,target_position[1]);a[0x16]&=0x3f;a[0x49]|=0x14;let flags=motion_word(&a,0x49)as u16&0xeffc;motion_put(&mut a,0x49,flags as i16);a[0x54]=20;a[0x4f]=0;a[0x52]=0;
    let speed=mule_requested_speed(&a);motion_put(&mut a,0x3f,speed);motion_put(&mut a,0x6c,32767);a[0x60]|=0xc0;a[0x5e]=1;
    Ok(InitialMuleChoice{before:*before,after:a,rng_before,rng_after:rng,draws,sampled_points:Vec::new(),chosen_target:Some(target_position),queue_requested:true})
}
///6C900/5C730 numeric feeding prefix for a live neutral plant. The owned
/// caller decrements planthealth by3 and emits the source virtual44 audio
/// request in the same transaction, after proving a no-action neighborhood.
pub fn plan_mule_plant_feed(before:&CreatureRecord,nutrition:u8,rng_before:crate::blueprint::BlueprintRng)->Result<InitialMuleChoice,CreatureError> {
    plan_mule_plant_feed_inner(before,nutrition,rng_before,false)
}
/// Feeding completion after the owned adapter proves immediate plant death
/// and the no-action scorer/current-cell predicates. UI cleanup stays pending.
pub fn plan_mule_plant_feed_after_retirement(before:&CreatureRecord,nutrition:u8,rng_before:crate::blueprint::BlueprintRng)->Result<InitialMuleChoice,CreatureError> {
    plan_mule_plant_feed_inner(before,nutrition,rng_before,true)
}
fn plan_mule_plant_feed_inner(before:&CreatureRecord,nutrition:u8,rng_before:crate::blueprint::BlueprintRng,retired:bool)->Result<InitialMuleChoice,CreatureError> {
    if rng_before.ring>=32||before[6]!=1||before[0x18]!=4||before[0x5e]!=7||before[0x7a]!=4||before[0x76]>90||before[0x61]!=0x40||before[0x60]!=8
       ||before[0x57]!=2||motion_word(before,0x6c)&15!=0{return Err(CreatureError::UnsupportedMotionState);}
    let gain=i16::from(nutrition)*2;let reserve=motion_word(before,0x68).wrapping_add(gain);let energy=if before[0x77]==255 {motion_word(before,0x66)}else{motion_word(before,0x66).wrapping_sub(gain*2)};
    if energy<=2{return Err(CreatureError::UnsupportedMotionState);}
    let full=reserve>motion_word(before,0x62);
    let mut after=*before;motion_put(&mut after,0x68,if full{motion_word(before,0x62)}else{reserve});motion_put(&mut after,0x66,energy);if full{after[0x61]&=0xbf;}
    let mut rng=rng_before;let mut draws=Vec::new();draw(&mut rng,CreatureRngPurpose::NeighborhoodX,&mut draws);draw(&mut rng,CreatureRngPurpose::NeighborhoodY,&mut draws);
    if retired||full {
        //724C0 clears the selected target kind.5CC60 then resets the
        //state7 action and69500 consumes one idle-delay draw.
        after[0x57]=0;after[0x15]&=0xdf;after[0x33..0x37].fill(0);let flags=motion_word(&after,0x60)as u16&0xd71a;motion_put(&mut after,0x60,flags as i16);after[0x5d]=0;
        let delay=i16::from(draw(&mut rng,CreatureRngPurpose::IdleDelay,&mut draws)%20)+10;after[0x5e]=8;motion_put(&mut after,0x6c,delay);
    }
    Ok(InitialMuleChoice{before:*before,after,rng_before,rng_after:rng,draws,sampled_points:Vec::new(),chosen_target:None,queue_requested:true})
}
/// Numeric first-counter choice at 5ED20 for the exact initial Mule record.
///
/// The caller must establish frame1, the original ground/temperature context,
/// self-only current cell, and a neighborhood containing only this creature
/// and neutral owner4 plants. Under these explicit gates the scorer consumes
/// its two scan-order draws, while all hunger/enemy/reproduction actions are
/// ineligible. `accept` is the source 74CC0 terrain/plant predicate at each
/// sampled point, in source order. It must query immutable owned state.
/// Sixteen rejected points require the unimplemented backward-search branch;
/// this pure plan returns an error and commits neither bytes nor RNG.
pub fn plan_initial_mule_choice<F>(before:&CreatureRecord,rng_before:crate::blueprint::BlueprintRng,tables:&crate::actor_activation::TrigTables,mut accept:F)->Result<InitialMuleChoice,CreatureError>
where F:FnMut([i16;2])->Result<bool,CreatureError> {
    if rng_before.ring>=32 { return Err(CreatureError::InvalidRngRing(rng_before.ring)); }
    let mut packed=Vec::with_capacity(116);packed.extend_from_slice(&before[..31]);packed.extend_from_slice(&before[39..]);
    if crate::identity::sha256_hex(&packed)!="5d2bf29edb8851d6c825f04cd6c9469224cb44907ea02eef57d1883bc95cbbaa" { return Err(CreatureError::UnsupportedInitialState); }
    let mut after=*before;let mut rng=rng_before;let mut draws=Vec::new();let mut sampled_points=Vec::new();
    // Ground contact zeroes the vertical velocity and gives the zero-vector
    // pitch angle512. Position/ground/shade/link bytes stay unchanged here.
    after[0x3b..0x3d].copy_from_slice(&512i16.to_le_bytes());
    // 5E0B0 and the common odd-counter tail each decrement signed field66.
    let counter=i16::from_le_bytes(before[0x66..0x68].try_into().unwrap()).wrapping_sub(2);
    after[0x66..0x68].copy_from_slice(&counter.to_le_bytes());
    draw(&mut rng,CreatureRngPurpose::NeighborhoodX,&mut draws);
    draw(&mut rng,CreatureRngPurpose::NeighborhoodY,&mut draws);
    // The initial current-cell identity predicate is false. 64E0E first
    // saves the original anchor, then the delay decision consumes one draw.
    after[0x72..0x76].copy_from_slice(&before[7..11]);after[0x60]|=8;
    let delay=draw(&mut rng,CreatureRngPurpose::IdleDelay,&mut draws)%10<4;
    let mut chosen_target=None;
    let timer=if delay {20i16} else {
        for _ in 0..16 {
            let angle=usize::from(draw(&mut rng,CreatureRngPurpose::WanderAngle,&mut draws))*8;
            let radius=(2048i32*i32::from(draw(&mut rng,CreatureRngPurpose::WanderRadius,&mut draws)))>>8;
            let index=angle.wrapping_sub(512)&2047;
            let dx=(tables.cosine()[index]*(radius*2))>>16;
            let dy=(tables.sine()[index]*(radius*2))>>16;
            let center=[i16::from_le_bytes(before[7..9].try_into().unwrap()),i16::from_le_bytes(before[9..11].try_into().unwrap())];
            let point=[((center[0].wrapping_mul(2).wrapping_add(dx as i16)>>1)as u16&0x7fff)as i16,((center[1].wrapping_mul(2).wrapping_add(dy as i16)>>1)as u16&0x7fff)as i16];
            let accepted=accept(point)?;sampled_points.push((point,accepted));
            if accepted { chosen_target=Some(point);break; }
            // 64FAA leaves the rejected-attempt bit set even when a later
            // point succeeds; 13E70 does not clear mask40.
            after[0x49]|=0x40;
        }
        let point=chosen_target.ok_or(CreatureError::TargetSearchRequired)?;
        after[11..13].copy_from_slice(&point[0].to_le_bytes());after[13..15].copy_from_slice(&point[1].to_le_bytes());
        after[0x49]|=0x14;after[0x54]=20;after[0x60]|=0x80;
        // Selector4 speed704, skill30 gives82%, initial warmth255 gives256/256.
        let speed=mule_requested_speed(&after);
        after[0x3f..0x41].copy_from_slice(&(speed as i16).to_le_bytes());
        32767i16
    };
    after[0x6c..0x6e].copy_from_slice(&timer.wrapping_sub(1).to_le_bytes());
    // 5E640 state8 selects heading animation6 and freezes frame0. MULE's
    // animation6/frame0 duration byte is zero; 7AC00 preserves the other zeros.
    after[0x1c]=0;
    Ok(InitialMuleChoice{before:*before,after,rng_before,rng_after:rng,draws,sampled_points,chosen_target,queue_requested:true})
}
impl core::fmt::Display for CreatureError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f,"{self:?}") }
}
impl std::error::Error for CreatureError {}
impl From<crate::actor_motion::ActorMotionError> for CreatureError {
    fn from(_:crate::actor_motion::ActorMotionError)->Self{Self::UnsupportedMotionState}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreaturePool {
    records: [CreatureRecord; CREATURE_CAPACITY],
    bindings: [Option<CreatureAnimationBinding>; CREATURE_CAPACITY],
}
impl CreaturePool {
    /// The caller supplies already-constructed native priors. The serialized
    /// reader leaves [31,39) untouched except for the active 5A1C0 animation
    /// pointer assignment, represented here by a zero placeholder plus handle.
    /// The vtable bytes [35,39) remain opaque; the known Kind1 class is enforced
    /// by this type rather than by a forged process address.
    pub fn from_mission_source(source: &ConqMissionSource<'_>, prior: &[CreatureRecord; CREATURE_CAPACITY]) -> Result<Self,CreatureError> {
        if crate::identity::sha256_hex(source.bytes()) != MISSION_SHA256 { return Err(CreatureError::MissionHash); }
        let group=source.group(0).map_err(CreatureError::MissionSource)?;
        Self::from_packed(group.bytes(),prior)
    }
    /// Exact loader copy plus the explicitly admitted Mule binding. Other
    /// active resource selectors and the state>=26 Bones route reject.
    pub fn from_packed(packed: &[u8], prior: &[CreatureRecord; CREATURE_CAPACITY]) -> Result<Self,CreatureError> {
        if packed.len()!=CREATURE_CAPACITY*PACKED_CREATURE_BYTES { return Err(CreatureError::PackedLength(packed.len())); }
        let mut records=*prior;
        let mut bindings=[None;CREATURE_CAPACITY];
        for (i,source) in packed.chunks_exact(PACKED_CREATURE_BYTES).enumerate() {
            records[i][..31].copy_from_slice(&source[..31]);
            records[i][39..].copy_from_slice(&source[31..]);
            if records[i][6]!=0 {
                if records[i][6]!=1 || records[i][0x5e]>=26 || records[i][0x7a] as i8 !=4 { return Err(CreatureError::UnsupportedActiveRecord(i)); }
                if u16::from_le_bytes([records[i][15],records[i][16]]) as usize !=i || records[i][24]>4 { return Err(CreatureError::InvalidIdentity(i)); }
                records[i][31..35].fill(0);
                bindings[i]=Some(CreatureAnimationBinding::MuleSelector4);
            }
        }
        Ok(Self{records,bindings})
    }
    pub fn records(&self)->&[CreatureRecord;CREATURE_CAPACITY] { &self.records }
    pub fn record(&self,slot:usize)->Result<&CreatureRecord,CreatureError> { self.records.get(slot).ok_or(CreatureError::InvalidSlot(slot)) }
    pub fn binding(&self,slot:usize)->Result<Option<CreatureAnimationBinding>,CreatureError> { self.bindings.get(slot).copied().ok_or(CreatureError::InvalidSlot(slot)) }
    /// Transaction boundary used by the world adapter for source-proved
    /// native-field/link updates. No packed mirror or second map is written.
    pub(crate) fn replace_record_checked(&mut self,slot:usize,before:&CreatureRecord,after:&CreatureRecord)->Result<(),CreatureError> {
        if self.record(slot)?!=before { return Err(CreatureError::StaleRecord(slot)); }
        if before[6]!=after[6] || before[15..17]!=after[15..17] || before[24]!=after[24] || before[0x7a]!=after[0x7a] { return Err(CreatureError::IdentityChanged(slot)); }
        if before[31..39]!=after[31..39] { return Err(CreatureError::PointerBytesChanged(slot)); }
        if after[6]==1&&after[0x5e]>=26{
            if before[0x5e]<25||after[0x7a]!=4{return Err(CreatureError::UnsupportedMotionState);}
            self.bindings[slot]=Some(CreatureAnimationBinding::BonesSelector4);
        }
        self.records[slot]=*after;
        Ok(())
    }
    pub(crate) fn retire_bones_checked(&mut self,slot:usize,before:&CreatureRecord)->Result<(),CreatureError>{
        if self.record(slot)?!=before{return Err(CreatureError::StaleRecord(slot));}
        if self.binding(slot)?!=Some(CreatureAnimationBinding::BonesSelector4)||before[6]!=1||before[24]!=4||before[0x7a]!=4
            ||before[0x5e]!=27||motion_word(before,0x68)!=0||motion_word(before,15)as u16 as usize!=slot{return Err(CreatureError::UnsupportedMotionState);}
        self.records[slot][6]=0;self.bindings[slot]=None;Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mule_source_skill_vectors_and_wrapping_slot_phase() {
        // Literal6A2BD/5A990 receipt: actual Krell skills25/50, legacy30.
        for (skill,energy,hunger,max62,max64,stage,speed) in [
            (25,8930,16,37,18,0,570),(30,9690,16,38,18,0,577),
            (50,12350,16,39,19,0,612),(25,7814,255,88,42,3,570),
            (50,10807,255,88,42,3,612)] {
            let mut record=[0;124];record[0x76]=skill;record[0x7b]=255;
            motion_put(&mut record,0x66,energy+1);motion_put(&mut record,0x62,88);motion_put(&mut record,0x64,42);
            refresh_mule_lifecycle(&mut record);
            assert_eq!((motion_word(&record,0x66),record[0x77],motion_word(&record,0x62),motion_word(&record,0x64),record[0x79]),(energy,hunger,max62,max64,stage));
            assert_eq!(mule_requested_speed(&record),speed);
        }
        let mut record=[0;124];motion_put(&mut record,0x0f,1);
        assert_eq!(mule_callback_phase(&record,0),1);assert_eq!(mule_callback_phase(&record,u32::MAX),0);
        motion_put(&mut record,0x0f,-1);assert_eq!(mule_callback_phase(&record,0),u32::MAX);
    }
    #[test]
    fn natural_mortality_preserves_pointers_and_corpse_skips_contact_query() {
        let mut map=vec![0;229376];for cell in map.chunks_exact_mut(14){cell[2..4].copy_from_slice(&100i16.to_le_bytes());}
        let mut before=[0u8;124];before[6]=1;before[24]=4;before[0x7a]=4;before[0x76]=30;before[0x7b]=255;before[0x15]=0x80;before[0x16]=2;before[0x5e]=8;before[0x60]=8;
        before[31..39].fill(0x55);motion_put32(&mut before,0x11,200);motion_put32(&mut before,0x27,200);motion_put(&mut before,0x66,1);motion_put(&mut before,0x6a,55);motion_put(&mut before,0x62,100);motion_put(&mut before,0x68,100);
        let mut animations=MuleAnimations{holds:std::array::from_fn(|_|vec![0]),flags:[0;24],bones:None};animations.holds[17]=vec![0;7];
        let physics=crate::physics_tables::PhysicsTables::fixture();let trig=crate::actor_activation::TrigTables::generate_pinned().unwrap();let context=MuleStepContext{frame_counter:3,gravity:1024,water_height:0,climate:106};
        let (death,request)=plan_mule_mortality_after_prefix(&before,&before,&map,context,&physics,&trig,|_,_,_|Ok(0),|_|Ok(false),&animations,true).unwrap();
        assert!(request.is_none());assert_eq!(death.after[0x5e],24);assert_eq!(death.after[0x19],17);assert_eq!(death.after[0x1b],1);assert_eq!(motion_word(&death.after,0x66),0);assert_eq!(motion_word(&death.after,0x3f),-1);assert_eq!(&death.after[31..39],&before[31..39]);
        let context=MuleStepContext{frame_counter:4,..context};
        let (next,_)=plan_mule_mortality_after_prefix(&death.after,&death.after,&map,context,&physics,&trig,|_,_,_|panic!("source skips A4 at sentinel-1"),|_|Ok(false),&animations,true).unwrap();assert_eq!(next.after[0x1b],2);
        let mut finished=next.after;finished[0x1b]=6;finished[0x1a]=0;finished[0x1e]=0;let context=MuleStepContext{frame_counter:5,..context};
        let (corpse,_)=plan_mule_mortality_after_prefix(&finished,&finished,&map,context,&physics,&trig,|_,_,_|panic!("source skips A4"),|_|Ok(false),&animations,true).unwrap();assert_eq!(corpse.after[0x5e],25);assert_eq!(corpse.after[0x19],18);assert_eq!(motion_word(&corpse.after,0x68),200);
        let mut bones=corpse.after;motion_put(&mut bones,0x6c,0);motion_put(&mut bones,0x68,51);
        assert!(plan_mule_mortality_after_prefix(&bones,&bones,&map,context,&physics,&trig,|_,_,_|panic!("source skips A4"),|_|Ok(false),&animations,true).is_err());
        animations.bones=Some(BonesAnimations{holds:std::array::from_fn(|_|vec![0;2]),flags:[0;5]});
        let (converted,_)=plan_mule_mortality_after_prefix(&bones,&bones,&map,context,&physics,&trig,|_,_,_|panic!("source skips A4"),|_|Ok(false),&animations,true).unwrap();
        assert_eq!((converted.after[0x5e],converted.after[0x19],converted.after[0x1b]),(26,4,0));assert_eq!(&converted.after[31..39],&bones[31..39]);
        let mut pool=CreaturePool{records:[[0;124];300],bindings:[None;300]};pool.records[0]=bones;pool.bindings[0]=Some(CreatureAnimationBinding::MuleSelector4);
        pool.replace_record_checked(0,&bones,&converted.after).unwrap();assert_eq!(pool.binding(0).unwrap(),Some(CreatureAnimationBinding::BonesSelector4));
        let mut faded=converted.after;motion_put(&mut faded,0x6c,0);motion_put(&mut faded,0x68,26);
        let (last,_)=plan_mule_mortality_after_prefix(&faded,&faded,&map,context,&physics,&trig,|_,_,_|panic!("source skips A4"),|_|Ok(false),&animations,true).unwrap();assert_eq!((last.after[0x5e],last.after[0x1b]),(27,1));
        let mut retired=last.after;motion_put(&mut retired,0x6c,0);motion_put(&mut retired,0x68,1);let prior=retired;
        assert!(plan_mule_mortality_after_prefix(&retired,&retired,&map,context,&physics,&trig,|_,_,_|panic!("source skips A4"),|_|Ok(false),&animations,true).is_err());assert_eq!(retired,prior);
    }
    #[test]
    fn retry_wait_consumes_one_draw_preserves_motion_and_null_indices() {
        let mut before=[0x55;124];before[6]=1;before[24]=4;before[0x5e]=8;before[0x7a]=4;before[0x60]=8;before[0x57]=0;before[0x5d]=0;motion_put(&mut before,0x49,0x2150);
        let rng=crate::blueprint::BlueprintRng{seed:49876,ring:31,draw_count:u32::MAX};let plan=plan_mule_retry_wait(&before,rng).unwrap();
        assert_eq!(plan.rng_after,crate::blueprint::BlueprintRng{seed:46916,ring:0,draw_count:0});assert_eq!(plan.draws.len(),1);assert_eq!(motion_word(&plan.after,0x6c),13);
        assert_eq!(&plan.after[0x2d..0x33],&before[0x2d..0x33]);assert_eq!(&plan.after[0x3d..0x43],&before[0x3d..0x43]);assert_eq!(&plan.after[0x55..0x57],&before[0x55..0x57]);
        before[0x57]=2;assert!(plan_mule_retry_wait(&before,rng).is_err());before[0x57]=0;motion_put(&mut before,0x49,0x2050);assert!(plan_mule_retry_wait(&before,rng).is_err());
    }
    #[test]
    fn relocation_finishes_current_group_and_keeps_last_accepted_point() {
        let mut queries=Vec::new();
        let result=find_common_idle_relocation([32767,-32768],|point|{queries.push(point);Ok(queries.len()>1)}).unwrap();
        assert_eq!(queries,vec![[32767,-32768],[-32513,-32768],[32767,32512],[32511,-32768],[32767,-32512]]);
        assert_eq!(result,Some([32767,-32512]));
        let mut calls=0;assert!(find_common_idle_relocation([0,0],|_|{calls+=1;Err(CreatureError::MotionEnvironment)}).is_err());assert_eq!(calls,1);
    }
    #[test]
    fn relocation_preserves_opaque_null_indices_and_resets_only_admitted_action() {
        let mut before=[0x55;124];before[6]=1;before[24]=4;before[0x5e]=8;before[0x60]=8;before[0x57]=0;before[0x5d]=0;
        let after=plan_mule_idle_relocation(&before,[123,-456]).unwrap();
        assert_eq!(&after[0x55..0x57],&before[0x55..0x57]);assert_eq!(&after[0x5b..0x5d],&before[0x5b..0x5d]);
        assert_eq!(after[0x5e],0);assert_eq!(&after[0x6e..0x72],&after[0x72..0x76]);assert_eq!(motion_word(&after,0x6c),0);assert_eq!(after[0x5f],0);
        before[0x57]=2;assert!(plan_mule_idle_relocation(&before,[0,0]).is_err());
    }
    #[test]
    fn split_copy_preserves_runtime_gap_and_binds_active_record() {
        let prior=core::array::from_fn(|slot|core::array::from_fn(|i|(slot*37+i*19)as u8));
        let mut raw=vec![0;CREATURE_CAPACITY*PACKED_CREATURE_BYTES];
        raw[6]=1;raw[24]=4;raw[0x7a-8]=4;raw[0x5e-8]=8;
        let pool=CreaturePool::from_packed(&raw,&prior).unwrap();
        assert_eq!(&pool.records[0][31..35],&[0;4]);assert_eq!(&pool.records[0][35..39],&prior[0][35..39]);
        assert_eq!(&pool.records[1][31..39],&prior[1][31..39]);assert_eq!(pool.binding(0).unwrap(),Some(CreatureAnimationBinding::MuleSelector4));
        assert_eq!(pool.binding(1).unwrap(),None);
    }
    #[test]
    fn stale_and_pointer_changes_reject_without_mutation() {
        let prior=[[0;NATIVE_CREATURE_BYTES];CREATURE_CAPACITY];let mut pool=CreaturePool::from_packed(&vec![0;CREATURE_CAPACITY*PACKED_CREATURE_BYTES],&prior).unwrap();let before=*pool.record(0).unwrap();let snapshot=pool.clone();let mut after=before;after[35]=1;
        assert!(pool.replace_record_checked(0,&before,&after).is_err());assert_eq!(pool,snapshot);
        after=before;after[0]=9;pool.replace_record_checked(0,&before,&after).unwrap();assert!(pool.replace_record_checked(0,&before,&after).is_err());
    }
}

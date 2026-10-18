//! Type3 pending move consumption after the source movement callback.
//! This slice is 57E0F..57F92 and 52340 case0. It is deliberately not a
//! substitute for the preceding 6FB60 movement or subsequent animation pass.

pub type ActorMotionRecord = [u8; 200];

///587BB re-reads signed count3 and587A4 sign-extends byte4+index on every
/// callback. Negative/zero counts terminate the source loop immediately.
pub fn next_player_actor_id(player:&[u8],index:usize)->Result<Option<i16>,ActorMotionError> {
    if player.len()<9 {return Err(ActorMotionError::UnsupportedState)}
    let count=player[3]as i8;
    if count<=0||index>=count as usize{return Ok(None)}
    if count>5{return Err(ActorMotionError::UnsupportedState)}
    Ok(Some(i16::from(player[4+index]as i8)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorMotionError { Inactive, UnsupportedState, InvalidBlueprint, InvalidAngleTable, InvalidAnimation, RayLimit, IdleNeighborhoodRequired, InvalidRng }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMovePlan {
    pub after: ActorMotionRecord,
    /// 419E0 formatted move acknowledgement; the host resolves owned text/UI.
    pub acknowledgement: Option<MoveAcknowledgement>,
    pub consumed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveAcknowledgement { pub owner: u8, pub actor_id: i16, pub class: u8, pub mode: u8 }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorPostMotionPrefix {
    pub after: ActorMotionRecord,
    pub blueprint_after: [u8;36],
    pub acknowledgement: Option<MoveAcknowledgement>,
}

fn word(r:&ActorMotionRecord,p:usize)->i16 { i16::from_le_bytes([r[p],r[p+1]]) }
fn put(r:&mut ActorMotionRecord,p:usize,v:i16) { r[p..p+2].copy_from_slice(&v.to_le_bytes()); }
/// AE028 octant lookup; callers must admit the hash-bound AE134 table whose
/// entries are all0..256. This primitive does not silently substitute a table.
pub(crate) fn angle(x:i32,y:i32,t:&[u16;257])->u16 {
    if x==0 && y==0 { return 0; }
    let ax=x.unsigned_abs(); let ay=y.unsigned_abs();
    let a=if ax<ay { t[(u64::from(ax)*256/u64::from(ay)) as usize] }
          else { t[(u64::from(ay)*256/u64::from(ax)) as usize] };
    match (x>=0,y>=0,ax<ay) {
        (true,false,true)=>a,(true,false,false)=>512-a,
        (true,true,true)=>1024-a,(true,true,false)=>512+a,
        (false,false,true)=>2048-a,(false,false,false)=>1536+a,
        (false,true,true)=>1024+a,(false,true,false)=>1536-a,
    }
}

/// Inputs must be the post-6FB60, post-blueprint-update state at 57E0F.
/// `class_speeds` binds the four signed words at 104AE8+16*class;
/// `atan` binds AE134. A fresh opcode3 has pending command byte5B zero.
/// Reinitialization via 51E80 and non-move commands remain unsupported.
pub fn plan_pending_move_after_motion(before:&ActorMotionRecord,blueprint:&[u8;36],class_speeds:&[i16;4],atan:&[u16;257])->Result<PendingMovePlan,ActorMotionError> {
    if before[6]!=3 { return Err(ActorMotionError::Inactive); }
    let unchanged=||PendingMovePlan{after:*before,acknowledgement:None,consumed:false};
    if before[0x71]&0x10!=0 || matches!(before[0x55],2|3|4|5) || before[0x70]&1==0 { return Ok(unchanged()); }
    if before[0x57]&2==0 { return Ok(unchanged()); }
    if before[0x58]&1!=0 || before[0x5b]!=0 { return Err(ActorMotionError::UnsupportedState); }
    let class=blueprint[0x15];let mode=blueprint[0x16];
    if class>3 || mode>3 { return Err(ActorMotionError::InvalidBlueprint); }
    if atan.iter().any(|&v|v>256) { return Err(ActorMotionError::InvalidAngleTable); }
    let mut a=*before;
    a[0x7a]=0;a[0x70]&=!0x10;a[0x55]=8;a[0x5a]=0;
    put(&mut a,0x3f,class_speeds[class as usize]);a[0x68..0x6c].fill(0);
    a[0x58]|=2;a[0x49]|=4;
    a[0x0b..0x0f].copy_from_slice(&before[0x5c..0x60]);
    if a[0x49]&0x10==0 {
        // 73970 doubles each coordinate then uses the signed low16 delta.
        let dx=word(&a,0x0b).wrapping_sub(word(&a,7)).wrapping_mul(2);
        let dy=word(&a,0x0d).wrapping_sub(word(&a,9)).wrapping_mul(2);
        let yaw=angle(i32::from(dx),i32::from(dy),atan) as i16;
        put(&mut a,0x4b,yaw);put(&mut a,0x39,yaw);
    }
    a[0x54]=20;a[0x49]|=0x14;a[0x16]&=0x3f;
    let flags=(word(&a,0x49) as u16)&0xeffc;put(&mut a,0x49,flags as i16);
    a[0x4f]=0;a[0x52]=0;a[0x57]&=!2;
    Ok(PendingMovePlan{after:a,acknowledgement:Some(MoveAcknowledgement{owner:a[0x18],actor_id:word(&a,0x0f),class,mode}),consumed:true})
}

/// Exact 58435 directional selection preceding 7AC00, for state8 without
/// the flag70 bit40 animation override. Resource metadata must describe the
/// selected animation, not the previous animation. Each frame has one hold byte.
pub fn plan_actor_moving_animation(before:&ActorMotionRecord,holds:&[u8],flags:u8)->Result<ActorMotionRecord,ActorMotionError> {
    if before[6]!=3 { return Err(ActorMotionError::Inactive); }
    if before[0x55]!=8 || before[0x70]&0x40!=0 { return Err(ActorMotionError::UnsupportedState); }
    let yaw=word(before,0x39);
    if !(0..=2048).contains(&yaw) { return Err(ActorMotionError::InvalidAngleTable); }
    let animation=((((i32::from(yaw)>>7)+1)>>1)+1) as u8&7;
    let mut a=*before;
    if animation!=a[0x19] { a[0x19]=animation;a[0x1c]=1; }
    a[0x1d]=((1024-i32::from(word(&a,0x41)))/256) as u8;
    advance_actor_animation(&a,holds,flags)
}

/// 7AC00 on contiguous source animation frames. This preserves signed delay
/// and counter comparisons, unsigned hold comparisons, and byte wrapping.
pub fn advance_actor_animation(before:&ActorMotionRecord,holds:&[u8],flags:u8)->Result<ActorMotionRecord,ActorMotionError> {
    if holds.is_empty() || holds.len()>255 || flags&!1!=0 || (flags&1!=0 && holds.len()<2) || !matches!(before[0x1c],1|255) { return Err(ActorMotionError::InvalidAnimation); }
    let mut a=*before;let last=holds.len()-1;
    let frame=usize::from(a[0x1b]).min(last);a[0x1b]=frame as u8;
    if (a[0x1e] as i8)>(a[0x1d] as i8) { a[0x1e]=a[0x1d]; }
    let counter=a[0x1e] as i8;a[0x1e]=a[0x1e].wrapping_sub(1);
    if counter>0 { return Ok(a); }
    a[0x1e]=a[0x1d];a[0x1a]=a[0x1a].min(holds[frame]);
    let hold=a[0x1a];a[0x1a]=hold.wrapping_sub(1);
    if hold!=0 { return Ok(a); }
    let target=frame as i32+i32::from(a[0x1c] as i8);
    let next=if target>last as i32 {
        if flags&1!=0 { a[0x1c]=255;last-1 } else { 0 }
    } else if target<0 {
        if flags&1!=0 { a[0x1c]=1;1 } else { last }
    } else { target as usize };
    a[0x1b]=next as u8;a[0x1a]=holds[next];Ok(a)
}

/// 6F760/736F0 null-target, non-flying query destination. Call7B140 at this
/// point and the current actor position before evaluating the squared metric.
pub fn actor_contact_distance_target(before:&ActorMotionRecord)->Result<[i16;2],ActorMotionError> {
    if before[6]!=3 { return Err(ActorMotionError::Inactive); }
    if before[0x52]!=0 || word(before,0x49) as u16&0x400!=0 { return Err(ActorMotionError::UnsupportedState); }
    let p=if before[0x49]&3!=0 {0x45}else{0x0b};
    Ok([word(before,p),word(before,p+2)])
}

/// Source736F0 returns a wrapping signed32 squared metric. Heights are the
/// exact7B140 samples at current7 and actor_contact_distance_target().
pub fn actor_contact_distance_squared(before:&ActorMotionRecord,current_ground:i32,target_ground:i32)->Result<i32,ActorMotionError> {
    let t=actor_contact_distance_target(before)?;
    let dx=i32::from(t[0].wrapping_sub(word(before,7)).wrapping_mul(2)>>1);
    let dy=i32::from(t[1].wrapping_sub(word(before,9)).wrapping_mul(2)>>1);
    let dz=target_ground.wrapping_sub(current_ground)>>3;
    Ok(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)).wrapping_add(dz.wrapping_mul(dz)))
}

/// 57A44..583D1 for the ordinary activated idle/move states, after6FB60.
/// Arrival/error flags49 mask60 require the separate idle/reinitialization/RNG
/// path and are rejected. Class3's charging callback is also a separate path.
pub fn plan_actor_post_motion_prefix(before:&ActorMotionRecord,blueprint:&[u8;36],frame:u32,class_speeds:&[i16;4],atan:&[u16;257])->Result<ActorPostMotionPrefix,ActorMotionError> {
    plan_actor_post_motion_prefix_with_class3(before,blueprint,frame,class_speeds,atan,false)
}
/// Class3 is admitted only when the owning player's source byte610 is zero.
/// That branch clears actor73 and never invokes the charge callback57860.
pub fn plan_actor_post_motion_prefix_with_class3(before:&ActorMotionRecord,blueprint:&[u8;36],frame:u32,class_speeds:&[i16;4],atan:&[u16;257],class3_unpowered:bool)->Result<ActorPostMotionPrefix,ActorMotionError> {
    if before[6]!=3 || before[0x70]&1==0 { return Err(ActorMotionError::Inactive); }
    let queued_architect=before[0x7e]==0&&before[0x82]==1&&matches!(before[0x83],0|1|3|6|11)&&blueprint[0x16]==0;
    if !matches!(before[0x55],1|8) || before[0x58]&1!=0 || (before[0x57]&1!=0&&!queued_architect) || before[0x70]&0x40!=0 || blueprint[0x15]>3 || (blueprint[0x15]==3&&!class3_unpowered) || blueprint[0x16]>3 { return Err(ActorMotionError::UnsupportedState); }
    if before[0x55]==8 && (before[0x49]&0x60!=0 || before[0x58]==0) { return Err(ActorMotionError::UnsupportedState); }
    let mut a=*before;let mut b=*blueprint;
    if matches!(b[0x16],2|3) {
        let v=i32::from_le_bytes(a[0x64..0x68].try_into().unwrap());
        if v>0 { a[0x64..0x68].copy_from_slice(&(v-1).to_le_bytes()); }
    }
    let mask=if b[0x15]==2 {127}else{255};
    if frame.wrapping_add(i32::from(word(&a,0x0f)) as u32)&mask==0 && (b[0x1a] as i8)<(b[0x19] as i8) { b[0x1a]=b[0x1a].wrapping_add(1); }
    if a[0x55]==8 && a[0x70]&0x20==0 && a[0x49]&0x13==0 {
        //526C0 reissues52340 case0 when no path/arrival flag remains.
        if a[0x5a]!=0 { return Err(ActorMotionError::UnsupportedState); }
        if atan.iter().any(|&v|v>256) { return Err(ActorMotionError::InvalidAngleTable); }
        a[0x0b..0x0f].copy_from_slice(&before[0x5c..0x60]);
        let yaw=angle(i32::from(word(&a,0x0b).wrapping_sub(word(&a,7)).wrapping_mul(2)),i32::from(word(&a,0x0d).wrapping_sub(word(&a,9)).wrapping_mul(2)),atan) as i16;
        put(&mut a,0x4b,yaw);put(&mut a,0x39,yaw);a[0x54]=20;a[0x49]|=0x14;a[0x16]&=0x3f;
        let f=word(&a,0x49) as u16&0xeffc;put(&mut a,0x49,f as i16);a[0x4f]=0;a[0x52]=0;
    }
    let pending=plan_pending_move_after_motion(&a,&b,class_speeds,atan)?;a=pending.after;
    if a[0x55]!=8 { return Err(ActorMotionError::UnsupportedState); }
    if b[0x15]==3 {a[0x73]=0;}
    if (b[0x1b] as i8)<(b[0x18] as i8) && frame&127==0 { b[0x1b]=b[0x1b].wrapping_add(1); }
    Ok(ActorPostMotionPrefix{after:a,blueprint_after:b,acknowledgement:pending.acknowledgement})
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActorFootstep {
    pub position: [i16;2],
    pub physical_height: i32,
    /// Code6 factory arguments; the source does not mark this effect newborn.
    pub angles: [u16;3],
    /// The actor virtual44 audio request is channel4B, mode2, volume127,100ticks.
    pub audio_channel: u8,
}

/// 5861E..586DE requests8B520 only on this admitted grounded/moving parity.
///583D1..5841C selects the source BL animation state before83140/7AC00.
/// It differs from actor55 during movement/override and settled construction.
pub fn actor_selected_animation_state(a:&ActorMotionRecord)->Result<u8,ActorMotionError> {
    if a[6]!=3{return Err(ActorMotionError::Inactive);}
    Ok(if a[0x70]&0x40!=0{6}
        else if a[0x70]&0x10!=0&&a[0x55]!=5{8}
        else if (a[0x55]as i8)<9||u16::from_le_bytes(a[0x70..0x72].try_into().unwrap())&0x180!=0{a[0x55]}
        else{1})
}
pub fn actor_footstep_query_position(a:&ActorMotionRecord,frame:u32)->Result<Option<[i16;2]>,ActorMotionError> {
    if a[6]!=3 { return Err(ActorMotionError::Inactive); }
    //58652 tests the selected BL after virtual4 returns zero, not actor55.
    if actor_selected_animation_state(a)?!=1{return Ok(None);}
    let physical=i32::from_le_bytes(a[0x27..0x2b].try_into().unwrap());
    let ground=i32::from_le_bytes(a[0x11..0x15].try_into().unwrap());
    let moving=[0x2d,0x2f,0x31].iter().any(|&p|word(a,p)!=0)||physical>ground;
    Ok(if frame.wrapping_add(i32::from(word(a,0x0f)) as u32)&3==3 && physical.wrapping_sub(ground)==0 && moving {Some([word(a,7),word(a,9)])}else{None})
}

/// Call after the actual8B520 query; factory and audio are ordered host effects.
pub fn plan_actor_footstep(a:&ActorMotionRecord,frame:u32,terrain_class:u8)->Result<Option<ActorFootstep>,ActorMotionError> {
    let Some(position)=actor_footstep_query_position(a,frame)? else{return Ok(None)};
    if terrain_class<=1 {return Ok(None)}
    let yaw=word(a,0x39);let pitch=word(a,0x3b);let speed=word(a,0x3d);
    if !(0..=2048).contains(&yaw)||!(0..=2048).contains(&pitch)||speed<0 {return Err(ActorMotionError::InvalidAngleTable)}
    Ok(Some(ActorFootstep{position,physical_height:i32::from_le_bytes(a[0x27..0x2b].try_into().unwrap()),angles:[yaw as u16,(i32::from(pitch)-256).max(0) as u16,(speed>>1) as u16],audio_channel:0x4b}))
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActorStraightRay {
    pub clear: bool,
    ///743C0 writes the point preceding a rejected sample only on loop failure.
    pub failure_point: Option<[i16;2]>,
    pub sampled_points: Vec<[i16;2]>,
}

/// Exact743C0 numeric sample walk for a type3 actor. `accept` performs the
/// source74270 -> terrain-mask ->8B130 predicate against the live owned world.
/// A rejected initial point returns true without entering the walk in source.
/// Source table binding is caller-owned; the finite1024 bound rejects runaway
/// inputs rather than inventing a line-of-sight result.
pub fn actor_straight_ray<E,F>(from:[i16;2],to:[i16;2],atan:&[u16;257],sine:&[i32;2048],cosine:&[i32;2048],mut accept:F)->Result<ActorStraightRay,E>
where E:From<ActorMotionError>,F:FnMut([i16;2])->Result<bool,E> {
    if atan.iter().any(|&v|v>256) {return Err(ActorMotionError::InvalidAngleTable.into())}
    let mut samples=vec![from];
    if !accept(from)? {return Ok(ActorStraightRay{clear:true,failure_point:None,sampled_points:samples})}
    let mut fixed=[i32::from(from[0])<<16,i32::from(from[1])<<16];let mut step=[0i32;2];
    for n in 0..1024 {
        let point=fixed.map(|v|(v>>16) as i16);
        if n&63==0 {
            let dx=to[0].wrapping_sub(point[0]).wrapping_mul(2);let dy=to[1].wrapping_sub(point[1]).wrapping_mul(2);
            let yaw=angle(i32::from(dx),i32::from(dy),atan);
            let index=if yaw<512 {usize::from(yaw)+1536}else{usize::from(yaw)-512};
            step=[cosine[index].wrapping_shl(6),sine[index].wrapping_shl(6)];
        }
        let dx=i32::from(to[0].wrapping_sub(point[0]).wrapping_mul(2)>>1);let dy=i32::from(to[1].wrapping_sub(point[1]).wrapping_mul(2)>>1);
        let done=((dx*dx+dy*dy) as u32)<4097;let sample=if done {to}else{point};samples.push(sample);
        if !accept(sample)? {return Ok(ActorStraightRay{clear:false,failure_point:Some(std::array::from_fn(|i|(fixed[i].wrapping_sub(step[i])>>16) as i16)),sampled_points:samples})}
        fixed=std::array::from_fn(|i|fixed[i].wrapping_add(step[i]));
        if done {return Ok(ActorStraightRay{clear:true,failure_point:None,sampled_points:samples})}
    }
    Err(ActorMotionError::RayLimit.into())
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ActorCallbackRng {pub seed:u16,pub ring:u8,pub draw_count:u32}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ActorCallbackDraw {pub seed_before:u16,pub seed_after:u16,pub ring_slot:u8,pub journal_code:u16}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ActorIdleContext {pub runtime_mode:u16,pub override_180d80:u8,pub runtime_count_11c2bc:u16,pub player_mode:u8}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ActorIdlePlan {pub after:ActorMotionRecord,pub blueprint_after:[u8;36],pub rng_after:ActorCallbackRng,pub draws:Vec<ActorCallbackDraw>}
fn actor_draw(rng:&mut ActorCallbackRng,draws:&mut Vec<ActorCallbackDraw>,journal_code:u16)->u8 {
    let before=*rng;rng.seed=rng.seed.wrapping_mul(0xe62d);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
    draws.push(ActorCallbackDraw{seed_before:before.seed,seed_after:rng.seed,ring_slot:before.ring,journal_code});(rng.seed>>8)as u8
}
/// 6E9E0 BL1 plus6FB60's ordinary arrival tail, excluding detached flag16/20.
/// The host performs both intermediate physics relocation and this relocation.
pub fn plan_actor_arrival_snap(before:&ActorMotionRecord,target:[i16;2],ground:i32,shade:u8)->Result<ActorMotionRecord,ActorMotionError> {
    if before[6]!=3||before[0x16]&0x20!=0 {return Err(ActorMotionError::UnsupportedState)}
    let mut a=*before;put(&mut a,7,target[0]&0x7fff);put(&mut a,9,target[1]&0x7fff);
    a[0x11..0x15].copy_from_slice(&ground.to_le_bytes());a[0x17]=shade;a[0x43]=0;a[0x44]=0;put(&mut a,0x41,0);
    a[0x2d..0x33].fill(0);put(&mut a,0x3d,0);put(&mut a,0x3f,0);a[0x54]=0;
    let flags=(word(&a,0x49)as u16&0xfdcf)|0x220;put(&mut a,0x49,flags as i16);a[0x16]&=0x3f;Ok(a)
}
/// Ordinary state8 arrival -> state1 and52780 idle callback, stopping before
/// any required neighborhood scorer. Runtime globals remain explicit inputs.
/// Animation resource selection and7AC00 follow this plan.
pub fn plan_actor_arrival_idle(before:&ActorMotionRecord,blueprint:&[u8;36],frame:u32,rng:ActorCallbackRng,context:ActorIdleContext)->Result<ActorIdlePlan,ActorMotionError> {
    plan_actor_arrival_idle_with_empty_scorer(before,blueprint,frame,rng,context,false,false)
}
///57D4F..57E0F state8 arrival prefix for a queued architect. The following
///57F72 dispatch must run507D0 and construction in this same callback, not52780.
///Periodic blueprint recovery remains the owning callback's responsibility.
pub fn plan_actor_arrival_pending_architect(before:&ActorMotionRecord,blueprint:&[u8;36],mut rng:ActorCallbackRng)->Result<ActorIdlePlan,ActorMotionError>{
    if rng.ring>=32{return Err(ActorMotionError::InvalidRng);}
    if before[6]!=3||before[0x55]!=8||before[0x49]&0x73!=0x20||before[0x58]!=2||before[0x57]&3!=1||before[0x70]&0x70!=0||before[0x71]&16!=0||before[0x7e]!=0||before[0x82]!=1||!matches!(before[0x83],0|1|3|6|11)||blueprint[0x16]!=0||blueprint[0x15]>=3{return Err(ActorMotionError::UnsupportedState);}
    let mut a=*before;let mut draws=Vec::new();
    //526C0 success withDL1 precedes the57D4F state reset.
    a[0x16]&=0x3f;a[0x54]=0;a[0x49]&=!16;a[0x58]&=!2;
    a[0x55]=1;a[0x68..0x6c].fill(0);a[0x19]=8;a[0x1c]=1;a[0x1b]=0;a[0x1d]=16;
    if actor_draw(&mut rng,&mut draws,0x21b)%4<2{a[0x70]|=4}else{a[0x70]&=!4}
    Ok(ActorIdlePlan{after:a,blueprint_after:*blueprint,rng_after:rng,draws})
}
/// `empty` must be proved by the source512-radius cell scorer over the current
/// world; it is not an instruction to discard eligible targets.
pub fn plan_actor_arrival_idle_with_empty_scorer(before:&ActorMotionRecord,blueprint:&[u8;36],frame:u32,mut rng:ActorCallbackRng,context:ActorIdleContext,empty:bool,class3_unpowered:bool)->Result<ActorIdlePlan,ActorMotionError> {
    if rng.ring>=32 {return Err(ActorMotionError::InvalidRng)}
    if before[6]!=3||before[0x55]!=8||before[0x49]&0x73!=0x20||before[0x58]&3!=2||before[0x57]&3!=0||before[0x70]&0x70!=0||before[0x71]&0x10!=0||blueprint[0x15]>3||(blueprint[0x15]==3&&!class3_unpowered)||blueprint[0x16]>3 {return Err(ActorMotionError::UnsupportedState)}
    let mut a=*before;let mut b=*blueprint;let mut draws=Vec::new();
    if matches!(b[0x16],2|3) {let v=i32::from_le_bytes(a[0x64..0x68].try_into().unwrap());if v>0{a[0x64..0x68].copy_from_slice(&(v-1).to_le_bytes())}}
    let mask=if b[0x15]==2{127}else{255};let parity=frame.wrapping_add(i32::from(word(&a,15))as u32);
    if parity&mask==0&&(b[0x1a]as i8)<(b[0x19]as i8){b[0x1a]=b[0x1a].wrapping_add(1)}
    a[0x16]&=0x3f;a[0x54]=0;a[0x49]&=!0x10;a[0x58]&=!2;
    a[0x55]=1;a[0x68..0x6c].fill(0);a[0x19]=8;a[0x1c]=1;a[0x1b]=0;a[0x1d]=16;
    if actor_draw(&mut rng,&mut draws,0x21b)%4<2 {a[0x70]|=4}else{a[0x70]&=!4}
    run_actor_idle(&mut a,&b,frame,&mut rng,context,&mut draws,empty)?;
    if b[0x15]==3 {a[0x73]=0;}
    if (b[0x1b]as i8)<(b[0x18]as i8)&&frame&31==0{b[0x1b]=b[0x1b].wrapping_add(1)}
    Ok(ActorIdlePlan{after:a,blueprint_after:b,rng_after:rng,draws})
}
fn run_actor_idle(a:&mut ActorMotionRecord,b:&[u8;36],frame:u32,rng:&mut ActorCallbackRng,context:ActorIdleContext,draws:&mut Vec<ActorCallbackDraw>,empty:bool)->Result<(),ActorMotionError> {
    let parity=frame.wrapping_add(i32::from(word(a,15))as u32);
    let chance=(i32::from(b[0x17]as i8)>>if context.player_mode==0{1}else{3})*if matches!(b[0x16],2|3){2}else{1};
    if parity&15==0&&i32::from(actor_draw(rng,draws,0x666)%100)<chance {
        if !empty {return Err(ActorMotionError::IdleNeighborhoodRequired)}
        actor_draw(rng,draws,0x6bd);actor_draw(rng,draws,0x6be);
    }
    let override_value=if context.runtime_mode&1==0{context.override_180d80}else{0};
    if context.runtime_count_11c2bc!=0&&override_value==0 {a[0x19]=8}
    else {
        let timer=i32::from_le_bytes(a[0x68..0x6c].try_into().unwrap()).wrapping_sub(1);a[0x68..0x6c].copy_from_slice(&timer.to_le_bytes());
        if timer<=0 {
            a[0x19]=if override_value!=0{11}else if actor_draw(rng,draws,0x6a0)%10<6{8}else{11};
            let timer=20+i32::from(actor_draw(rng,draws,0x6a3)%30);a[0x68..0x6c].copy_from_slice(&timer.to_le_bytes());
            if actor_draw(rng,draws,0x6a5)%4<2 {a[0x70]|=4}else{a[0x70]&=!4}
        }
    }
    Ok(())
}
/// Ordinary state1 callback while no pending order/action or neighborhood
/// scorer is reached. The source's idle timer and all reachable draws are real.
pub fn plan_actor_idle_after_motion(before:&ActorMotionRecord,blueprint:&[u8;36],frame:u32,rng:ActorCallbackRng,context:ActorIdleContext)->Result<ActorIdlePlan,ActorMotionError> {
    plan_actor_idle_with_empty_scorer(before,blueprint,frame,rng,context,false,false)
}
/// Continuing ordinary idle callback with an independently checked empty
/// source scorer. The iterator's two axis-order RNG draws are preserved.
pub fn plan_actor_idle_with_empty_scorer(before:&ActorMotionRecord,blueprint:&[u8;36],frame:u32,mut rng:ActorCallbackRng,context:ActorIdleContext,empty:bool,class3_unpowered:bool)->Result<ActorIdlePlan,ActorMotionError> {
    if rng.ring>=32{return Err(ActorMotionError::InvalidRng)}
    if before[6]!=3||before[0x55]!=1||before[0x57]&3!=0||before[0x58]!=0||before[0x70]&0x70!=0||blueprint[0x15]>3||(blueprint[0x15]==3&&!class3_unpowered)||blueprint[0x16]>3{return Err(ActorMotionError::UnsupportedState)}
    let mut a=*before;let mut b=*blueprint;let mut draws=Vec::new();
    if matches!(b[0x16],2|3){let v=i32::from_le_bytes(a[0x64..0x68].try_into().unwrap());if v>0{a[0x64..0x68].copy_from_slice(&(v-1).to_le_bytes())}}
    let mask=if b[0x15]==2{127}else{255};let parity=frame.wrapping_add(i32::from(word(&a,15))as u32);
    if parity&mask==0&&(b[0x1a]as i8)<(b[0x19]as i8){b[0x1a]=b[0x1a].wrapping_add(1)}
    run_actor_idle(&mut a,&b,frame,&mut rng,context,&mut draws,empty)?;
    if b[0x15]==3 {a[0x73]=0;}
    if (b[0x1b]as i8)<(b[0x18]as i8)&&frame&31==0{b[0x1b]=b[0x1b].wrapping_add(1)}
    Ok(ActorIdlePlan{after:a,blueprint_after:b,rng_after:rng,draws})
}

#[cfg(test)]
mod footstep_selection_tests {
    use super::*;
    #[test]
    fn footsteps_follow_selected_animation_bl() {
        let mut a=[0u8;200]; a[6]=3;
        a[0x2d..0x2f].copy_from_slice(&1i16.to_le_bytes());
        for (state, flags, selected, dust) in [
            (8,1,8,false),(1,1,1,true),(9,1,1,true),
            (9,0x81,9,false),(9,0x101,9,false),
            (1,0x11,8,false),(1,0x41,6,false),(5,0x11,5,false),
        ] {
            a[0x55]=state; a[0x70..0x72].copy_from_slice(&(flags as u16).to_le_bytes());
            assert_eq!(actor_selected_animation_state(&a).unwrap(),selected);
            assert_eq!(plan_actor_footstep(&a,3,7).unwrap().is_some(),dust);
        }
    }
}

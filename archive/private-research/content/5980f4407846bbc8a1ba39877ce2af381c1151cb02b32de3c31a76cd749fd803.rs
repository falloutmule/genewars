//! Bounded451F0 selector1/grade0 callback with no accepted attack target.
//! The cell reader supplies8B000's first eligible live kind1; target attack,
//! destruction and disabled flags remain explicit errors. No scene claim.
use crate::blueprint::BlueprintRng;
pub type BuildingRecord=[u8;67];
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum QuietBuildingError { UnsupportedRecord,InvalidPlayer,InvalidRng,AcceptedAttackTarget,ProjectileRequired,Environment }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct TurretCellCandidate {pub position:[i16;2],pub state:u8,pub timer:i16}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct TurretTarget {pub reference:[u8;3],pub candidate:TurretCellCandidate}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum TurretAudioRequest {
 /// Virtual44:code23,mode2,parameter255,volume127,range100.
 Rotate23,
 /// Virtual48 stops code23.
 Stop23,
}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct TurretTrackingPlan {pub numeric:QuietBuildingPlan,pub audio:Option<TurretAudioRequest>,pub target:Option<[u8;3]>}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct QuietBuildingPlan {
 pub before:BuildingRecord,pub after:BuildingRecord,pub player_before:[u8;1805],pub player_after:[u8;1805],
 pub rng_before:BlueprintRng,pub rng_after:BlueprintRng,pub search_cells:usize,pub queue_requested:bool,
}
fn word(r:&[u8],at:usize)->i16{i16::from_le_bytes(r[at..at+2].try_into().unwrap())}
fn setword(r:&mut[u8],at:usize,n:i16){r[at..at+2].copy_from_slice(&n.to_le_bytes())}
fn delta(a:i16,b:i16)->i32 {i32::from(a.wrapping_mul(2).wrapping_sub(b.wrapping_mul(2))>>1)}
fn distance(a:[i16;2],b:[i16;2])->i32 {let dx=delta(a[0],b[0]);let dy=delta(a[1],b[1]);dx*dx+dy*dy}
fn direction(rng:&mut BlueprintRng)->i16 {
 rng.seed=rng.seed.wrapping_mul(0xE62D);rng.ring=(rng.ring+1)&31;rng.draw_count=rng.draw_count.wrapping_add(1);
 if (rng.seed>>8)&1==0 {256}else{-256}
}
/// `cell` implements8B000: FIRST kind1 with nonallied foreign owner and
/// virtual68 false. A grade0 state18/timer exclusion happens AFTER that read,
/// so callers must not skip to a later entity in the same cell.
/// The admitted exact BTURR animation1 has8 frames, zero holds and flags0.
pub fn plan_quiet_selector1<F>(before:&BuildingRecord,player:&[u8;1805],rng:BlueprintRng,mut cell:F)
 ->Result<QuietBuildingPlan,QuietBuildingError>
where F:FnMut([i16;2])->Result<Option<TurretCellCandidate>,QuietBuildingError> {
 let flags=u16::from_le_bytes(before[49..51].try_into().unwrap());
 if before[6]!=4||before[24]>=4||before[45]!=1||before[46]!=0||flags&!4!=1||before[66]!=0||before[28]!=0 {
  return Err(QuietBuildingError::UnsupportedRecord)
 }
 if player[1]!=before[24]||player[0x4a1]!=before[24]{return Err(QuietBuildingError::InvalidPlayer)}
 if rng.ring>=32{return Err(QuietBuildingError::InvalidRng)}
 let mut after=*before;let mut player_after=*player;let mut rng_after=rng;let mut search_cells=0;
 let timer=word(before,61).wrapping_sub(2+i16::from(before[63]));setword(&mut after,61,timer);
 if timer<=0 {
  setword(&mut after,61,0);after[49]&=!4;
  if word(&player_after,0x5ea)==0{setword(&mut player_after,0x5ea,1000)}
 }
 if word(&after,61)!=0 {
  let center=[word(before,7),word(before,9)];let sx=direction(&mut rng_after);let sy=direction(&mut rng_after);
  let end_x=center[0].wrapping_add(sx*7);let end_y=center[1].wrapping_add(sy*7);
  let mut x=center[0].wrapping_sub(sx*6);
  while x!=end_x {
   let mut y=center[1].wrapping_sub(sy*6);
   while y!=end_y {
    let point=[x,y];if distance(point,center)<=1536*1536 {
     search_cells+=1;
     if let Some(candidate)=cell(point)? {
      if candidate.state!=18||candidate.timer==0 {
       //48540 excludes the inner256 radius and compares actual entity
       // coordinates, not the sampled cell center. Any accepted target
       // enters attack/rotation branches excluded from this quiet plan.
       if distance(candidate.position,center)>65536{return Err(QuietBuildingError::AcceptedAttackTarget)}
      }
     }
    }
    y=y.wrapping_add(sy);
   }
   x=x.wrapping_add(sx);
  }
  after[66]=0;
  after[27]=((((i32::from(word(before,52))>>7)+1)>>1)+1)as u8&7;after[28]=0;
 }
 if word(&after,61)>200{setword(&mut after,61,200)}
 if after[63]!=0{after[63]-=1}
 if before[25]!=after[25]||after[28]!=0||(before[49]^after[49])&4!=0||after[48]!=0{after[21]|=4}
 if after[48]!=0{after[48]-=1}
 after[25]=1;after[28]=0;
 after[27]=after[27].min(7);
 if (after[30]as i8)>(after[29]as i8){after[30]=after[29]}
 let count=after[30]as i8;after[30]=after[30].wrapping_sub(1);
 if count<=0 {after[30]=after[29];after[26]=0}
 Ok(QuietBuildingPlan{before:*before,after,player_before:*player,player_after,rng_before:rng,rng_after,search_cells,queue_requested:true})
}

fn angle_difference(current:i16,desired:i16)->i32 {
 let difference=i32::from(desired)-i32::from(current);let alternative=-(2048-difference.abs())*difference.signum();
 if difference.abs()>alternative.abs(){alternative}else{difference}
}
/// Source47387 target assignment/rotation. The caller resolves an existing
/// kind1 target and8B000 cell results from the actual owned world. Aligned
/// targets with zero prior cooldown require the separate475D5 projectile
/// body and reject before producing an afterimage.
pub fn plan_tracking_selector1<F>(before:&BuildingRecord,player:&[u8;1805],rng:BlueprintRng,
 frame_counter:u32,physics:&crate::physics_tables::PhysicsTables,existing:Option<TurretTarget>,mut cell:F)
 ->Result<TurretTrackingPlan,QuietBuildingError>
where F:FnMut([i16;2])->Result<Option<TurretTarget>,QuietBuildingError> {
 if before[66]!=0&&before[66]!=1{return Err(QuietBuildingError::UnsupportedRecord)}
 if before[66]!=0&&existing.is_none(){return Err(QuietBuildingError::Environment)}
 if before[66]==0&&existing.is_some(){return Err(QuietBuildingError::Environment)}
 if let Some(target)=existing {
  if target.reference!=before[64..67]||target.reference[2]!=1{return Err(QuietBuildingError::Environment)}
 }
 let center=[word(before,7),word(before,9)];let remaining=word(before,61).wrapping_sub(2+i16::from(before[63]));
 let search=remaining>0&&(frame_counter.wrapping_add(word(before,15)as i32 as u32)&63==0
  ||existing.is_none()||existing.is_some_and(|target|{let d=distance(target.candidate.position,center);d>=1536*1536||d<=65536}));
 let mut selected=existing;let mut best=i32::MAX;let mut synthetic=*before;synthetic[66]=0;
 if search{selected=None}
 let mut numeric=plan_quiet_selector1(&synthetic,player,rng,|point| {
  if search {
   if let Some(target)=cell(point)? {
    if target.reference[2]!=1||word(&target.reference,0)<0{return Err(QuietBuildingError::Environment)}
    let candidate=target.candidate;let d=distance(candidate.position,center);
    if (candidate.state!=18||candidate.timer==0)&&d>65536&&d<best {selected=Some(target);best=d;}
   }
  }
  Ok(None)
 })?;
 numeric.before=*before;
 if !search{numeric.rng_after=rng;numeric.search_cells=0;}
 //47383 clears stored kind but leaves old resolved ESI alive this callback.
 let active=if search{selected.or(existing)}else{existing};
 let mut audio=None;
 if remaining>0 {
  if search {
   if let Some(target)=selected{numeric.after[64..67].copy_from_slice(&target.reference)}else{numeric.after[66]=0}
  }else{numeric.after[64..67].copy_from_slice(&before[64..67]);}
  if let Some(target)=active {
   let dx=target.candidate.position[0].wrapping_sub(center[0]).wrapping_mul(2);
   let dy=target.candidate.position[1].wrapping_sub(center[1]).wrapping_mul(2);
   let desired=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles())as i16;
   let current=word(before,52);let delta=angle_difference(current,desired);let speed=i32::from(remaining);
   let yaw=if delta>speed {
    audio=Some(TurretAudioRequest::Rotate23);current.wrapping_add(remaining)as u16&2047
   }else if delta< -speed {
    audio=Some(TurretAudioRequest::Rotate23);current.wrapping_sub(remaining)as u16&2047
   }else{audio=Some(TurretAudioRequest::Stop23);desired as u16};
   setword(&mut numeric.after,52,yaw as i16);numeric.after[21]|=4;
   if before[51]==0&&yaw as i16==desired{return Err(QuietBuildingError::ProjectileRequired)}
   if before[51]!=0{numeric.after[51]-=1}
   numeric.after[27]=((((i32::from(yaw as i16)>>7)+1)>>1)+1)as u8&7;
  }
 }else{numeric.after[64..67].copy_from_slice(&before[64..67]);}
 Ok(TurretTrackingPlan{numeric,audio,target:if remaining>0{active.map(|target|target.reference)}else{None}})
}

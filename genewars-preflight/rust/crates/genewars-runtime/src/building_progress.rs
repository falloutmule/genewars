//! Source441C0 admitted grade0 progress and43F60/43620/6A6D0 completion.
//! UI native pointer state remains an explicit policy request; audio4D uses2/0/127/100.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BuildingProgressError{InvalidBuilding,InvalidPlayer,CountMismatch}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CreatureTargetChange{pub slot:u16,pub before:[u8;124],pub after:[u8;124]}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingProgressPlan{pub before:[u8;67],pub after:[u8;67],pub player_before:[u8;1805],pub player_after:[u8;1805],pub creature_changes:Vec<CreatureTargetChange>,pub completed:bool,pub pending_audio_4d:bool,pub ui_refresh_required:bool}
pub fn plan_selector1_progress(pool:&[[u8;67];256],slot:u16,player:&[u8;1805],creatures:&[[u8;124];300],creature_count:i16,increment:i16,loading:bool)->Result<BuildingProgressPlan,BuildingProgressError>{
 if pool.get(usize::from(slot)).ok_or(BuildingProgressError::InvalidBuilding)?[45]!=1{return Err(BuildingProgressError::InvalidBuilding);}
 plan_admitted_grade0_progress(pool,slot,player,creatures,creature_count,increment,loading)
}
/// Common completion path for independently reviewed selectors0,1,3,6 and11.
/// Other selectors may have additional43F60 effects and remain unsupported.
pub fn plan_admitted_grade0_progress(pool:&[[u8;67];256],slot:u16,player:&[u8;1805],creatures:&[[u8;124];300],creature_count:i16,increment:i16,loading:bool)->Result<BuildingProgressPlan,BuildingProgressError>{
 let before=*pool.get(usize::from(slot)).ok_or(BuildingProgressError::InvalidBuilding)?;
 let selector=before[45];let maximum=match selector{0=>280i16,1=>120i16,3=>240i16,6=>80i16,11=>280i16,_=>return Err(BuildingProgressError::InvalidBuilding)};
 let health=i16::from_le_bytes(before[0x2b..0x2d].try_into().unwrap());
 if before[6]!=4||before[24]>=4||before[46]!=0||before[47]!=0||u16::from_le_bytes(before[49..51].try_into().unwrap())!=0||!(0..maximum).contains(&health)||i16::from_le_bytes(before[15..17].try_into().unwrap())!=slot as i16||usize::from(before[24])!=usize::from(slot)/64{return Err(BuildingProgressError::InvalidBuilding);}
 let owner=usize::from(before[24]);if player[1]as usize!=owner||player[0x4a1]as usize!=owner{return Err(BuildingProgressError::InvalidPlayer);}
 let active=pool[owner*64..owner*64+64].iter().filter(|r|r[6]!=0).count();
 if usize::from(player[0x4a2])!=active{return Err(BuildingProgressError::CountMismatch);}
 let mut after=before;let mut player_after=*player;let mut creature_changes=Vec::new();let health=health.wrapping_add(increment);after[0x2b..0x2d].copy_from_slice(&health.to_le_bytes());let completed=health>=maximum;
 if completed{
  after[46]=0;after[0x1b]=0;after[0x2b..0x2d].copy_from_slice(&maximum.to_le_bytes());after[0x31..0x33].copy_from_slice(&1u16.to_le_bytes());after[0x15]|=4;after[0x3d..0x3f].copy_from_slice(&200i16.to_le_bytes());
  let count_at=0x600+usize::from(selector);player_after[count_at]=player_after[count_at].wrapping_add(1);let owned=u16::from_le_bytes(player_after[0x60e..0x610].try_into().unwrap())|(1u16<<selector);player_after[0x60e..0x610].copy_from_slice(&owned.to_le_bytes());
  let mut completed_pool=*pool;completed_pool[usize::from(slot)]=after;
  player_after=recompute_player_capacity(&completed_pool,&player_after)?;
  let active=creatures.iter().filter(|r|r[6]!=0).count();if creature_count<0||creature_count as usize!=active||creatures.iter().any(|r|r[6]!=0&&r[6]!=1){return Err(BuildingProgressError::CountMismatch);}
  for (i,r)in creatures.iter().enumerate(){if r[6]!=0&&r[0x5a]==4&&i16::from_le_bytes(r[0x58..0x5a].try_into().unwrap())==slot as i16{let mut a=*r;a[0x5a]=0;a[0x78]=0;a[0x5f]=0;a[0x60]&=!16;creature_changes.push(CreatureTargetChange{slot:i as u16,before:*r,after:a});}}
 }
 Ok(BuildingProgressPlan{before,after,player_before:*player,player_after,creature_changes,completed,pending_audio_4d:completed&&!loading&&selector!=6,ui_refresh_required:true})
}

/// Source43620 capacity and maximum operational HQ grade over the occupied owner pool.
pub fn recompute_player_capacity(pool:&[[u8;67];256],player:&[u8;1805])->Result<[u8;1805],BuildingProgressError>{
 let owner=usize::from(player[0x4a1]);if owner>=4||usize::from(player[1])!=owner{return Err(BuildingProgressError::InvalidPlayer);}
 let occupied=pool[owner*64..owner*64+64].iter().filter(|r|r[6]!=0).count();
 if usize::from(player[0x4a2])!=occupied{return Err(BuildingProgressError::CountMismatch);}
 let mut player_after=*player;
 let mut supply=0i32;let mut max_grade=0u8;
  for (_i,prior)in pool[owner*64..owner*64+64].iter().enumerate(){let r=prior;if r[6]==0{continue;}if r[6]!=4||usize::from(r[24])!=owner{return Err(BuildingProgressError::InvalidBuilding);}
   let flags=u16::from_le_bytes(r[49..51].try_into().unwrap());
   if flags&0x840==0{supply=supply.wrapping_add(match r[45]{11=>600,4=>(i32::from(r[46])+1)*400,13=>(i32::from(r[46])+1)*500,_=>0});}
   if r[45]==0&&flags&0x141==1{max_grade=max_grade.max(r[46]);}
  }
  player_after[0x5e6..0x5ea].copy_from_slice(&supply.to_le_bytes());player_after[0x6e1]=max_grade;
 Ok(player_after)
}

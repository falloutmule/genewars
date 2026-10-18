use crate::physics_tables::PhysicsTables;
#[derive(Debug,Clone,PartialEq,Eq)]pub struct TurretAimPlan{pub angles:[u16;3],pub forecast_position:[i16;2],pub forecast_height:i32,pub launch_position:[i16;2],pub launch_height:i32}
fn w(r:&[u8],i:usize)->i16{i16::from_le_bytes(r[i..i+2].try_into().unwrap())}fn d(r:&[u8],i:usize)->i32{i32::from_le_bytes(r[i..i+4].try_into().unwrap())}
fn delta(a:i16,b:i16)->i32{i32::from(a.wrapping_mul(2).wrapping_sub(b.wrapping_mul(2))>>1)}
fn range(a:[i16;2],b:[i16;2],ah:i32,bh:i32)->u32{let dx=delta(b[0],a[0]);let dy=delta(b[1],a[1]);let dz=bh.wrapping_sub(ah)>>3;genewars_core::cd_candidate::integer_sqrt(dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy)).wrapping_add(dz.wrapping_mul(dz))as u32)}
pub fn plan_mule_turret_aim(shooter:&[u8;67],target:&[u8;124],map:&[u8],physics:&PhysicsTables)->Result<TurretAimPlan,String>{
 if shooter[6]!=4||shooter[45]!=1||shooter[46]!=0||target[6]!=1{return Err("unsupported typed records".into())}
 let source=[w(shooter,7),w(shooter,9)];let at=[w(target,7),w(target,9)];let source_height=d(shooter,17);let physical=d(target,39);let velocity=[w(target,45),w(target,47),w(target,49)];
 let predict=|time:i32|->Result<([i16;2],i32),String>{let offsets=velocity.map(|v|(i32::from(v).wrapping_mul(time)>>8)as i16);let point=[at[0].wrapping_add(offsets[0]>>6),at[1].wrapping_add(offsets[1]>>6)];let height=physical.wrapping_add(i32::from(offsets[2]>>3));let ground=genewars_core::cd_candidate::interpolate_height(map,point[0],point[1]).map_err(|e|e.to_string())?;Ok((point,height.max(ground)))};
 let time1=(range(source,at,source_height,physical).wrapping_shl(8)/100)as i32;let(point,height)=predict(time1)?;let time2=(range(source,point,source_height,height).wrapping_shl(8)/100)as i32;let time=time1.wrapping_add(time2)/2;let(point,height)=predict(time)?;
 let dx=point[0].wrapping_sub(source[0]).wrapping_mul(2);let dy=point[1].wrapping_sub(source[1]).wrapping_mul(2);let yaw=crate::actor_motion::angle(i32::from(dx),i32::from(dy),physics.angles());let hx=i32::from(dx>>1);let hy=i32::from(dy>>1);let horizontal=genewars_core::cd_candidate::integer_sqrt(hx.wrapping_mul(hx).wrapping_add(hy.wrapping_mul(hy))as u32);let vertical=source_height.wrapping_add(6400).wrapping_sub(height)>>3;let pitch=crate::actor_motion::angle(horizontal as i32,vertical,physics.angles());let launch_height=genewars_core::cd_candidate::interpolate_height(map,source[0],source[1]).map_err(|e|e.to_string())?.wrapping_add(6400);
 Ok(TurretAimPlan{angles:[yaw,pitch,6400],forecast_position:point,forecast_height:height,launch_position:source,launch_height})
}

/// Free-slot source48790 code42 path only. Common constructor is independently
/// validated by the water factory gate; occupied-pool replacement is excluded.
pub fn plan_turret_projectile_factory(pool:&[crate::landing::LandingRecord;200],count:i16,owner:u8,at:[i16;2],ground:i32,shade:u8,height:i32,angles:[u16;3],shooter_index:i16,head:Option<crate::landing::ResolvedCellHead>,sine:&[i32;2048],cosine:&[i32;2048])->Result<crate::landing::LandingFactoryPlan,crate::landing::LandingError>{
 if !(0..256).contains(&shooter_index){return Err(crate::landing::LandingError::InvalidRecord)}
 let mut plan=crate::water_effect::plan_factory(22,pool,count,owner,at,ground,shade,height,angles,head,sine,cosine)?;
 let a=&mut plan.after_nonpointer;a[69]=42;a[71..73].copy_from_slice(&shooter_index.to_le_bytes());a[73]=4;a[74..76].copy_from_slice(&128i16.to_le_bytes());a[76]=0;
 plan.animation_index=42;plan.events.push(crate::landing::LandingEvent::PlayAudio(crate::landing::LandingAudioRequest{channel:33,mode:2,parameter:0,stack_byte:127,stack_dword:100}));Ok(plan)
}
#[derive(Debug,Clone,PartialEq,Eq)]pub struct ProjectileLifetimePrefix{pub after:crate::landing::LandingRecord,pub retirement_required:bool,pub near_query_required:bool}
/// Postphysics prefix only: surviving code42 callbacks must still perform
/// source4B9AC proximity query and its ordered RNG effects.
pub fn plan_projectile_lifetime_prefix(before:&crate::landing::LandingRecord)->Result<ProjectileLifetimePrefix,crate::landing::LandingError>{
 if before[6]!=5||before[69]!=42||before[28]!=1||before[29]!=0{return Err(crate::landing::LandingError::UnsupportedAnimationState)}
 let mut after=*before;let timer=i16::from_le_bytes(before[74..76].try_into().unwrap());let timer=if timer>=0{timer.wrapping_sub(1)}else{timer};after[74..76].copy_from_slice(&timer.to_le_bytes());after[26]=0;after[27]=0;after[30]=0;
 let retirement_required=before[17..21]==before[39..43]||before[76]!=0||timer<0;
 Ok(ProjectileLifetimePrefix{after,retirement_required,near_query_required:!retirement_required})
}

//! Source4B630/6DF30 code2C on unchanged flat land, with EFF44 admitted.
//! Generic49D30/724C0 cleanup must commit before a retired plan is installed.
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BuildingEffectLifecycleError{InvalidRecord,UnsupportedPhysics,UnsupportedAnimation}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingEffectUpdatePlan{pub before:[u8;81],pub after:[u8;81],pub retirement_required:bool}
fn word(b:&[u8;81],p:usize)->i16{i16::from_le_bytes(b[p..p+2].try_into().unwrap())}
fn dword(b:&[u8;81],p:usize)->i32{i32::from_le_bytes(b[p..p+4].try_into().unwrap())}
///The caller samples both source ground queries at the unchanged position.
///Four equal raw heights prove the admitted flat surface. EFF44 is9frames,
///flags0, holds0; caller must bind its hash-admitted animation resource.
pub fn plan_building_effect_quiet_grounded(before:&[u8;81],ground:i32,shade:u8,water_height:i32,terrain_height_words:[i16;4])->Result<BuildingEffectUpdatePlan,BuildingEffectLifecycleError>{
 if before[6]!=5||before[69]!=44||before[24]>=4||!(0..200).contains(&word(before,15)){return Err(BuildingEffectLifecycleError::InvalidRecord);}
 if before[21]&!0x21!=0||before[21]&0x20==0||before[22]!=0||word(before,63)!=-1||word(before,43)!=16||before[45..57]!=[0;12]||word(before,61)!=0||!(0..=2048).contains(&word(before,57))||![0,512].contains(&word(before,59))||ground<=water_height||water_height<0||dword(before,17)!=ground||dword(before,39)!=ground||terrain_height_words.iter().any(|h|*h!=terrain_height_words[0])||i32::from(terrain_height_words[0])*2!=ground{return Err(BuildingEffectLifecycleError::UnsupportedPhysics);}
 if before[25]!=0||before[26]!=0||before[27]>=9||before[28]!=1||before[29]!=0{return Err(BuildingEffectLifecycleError::UnsupportedAnimation);}
 let mut after=*before;after[23]=shade;after[59..61].copy_from_slice(&512i16.to_le_bytes());
 let timer=word(before,74);let timer=if timer>=0{timer-1}else{timer};after[74..76].copy_from_slice(&timer.to_le_bytes());
 after[27]=(after[27]+1)%9;after[30]=0;let retirement_required=timer<0&&after[27]==8;
 Ok(BuildingEffectUpdatePlan{before:*before,after,retirement_required})
}

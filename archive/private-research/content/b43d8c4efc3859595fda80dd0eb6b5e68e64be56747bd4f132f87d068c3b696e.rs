//! Source451F0 incomplete selector1 callback; completed/destroying buildings stay explicit.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BuildingCallbackError{UnsupportedBuilding,InvalidAnimation}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct IncompleteBuildingPlan{pub before:[u8;67],pub after:[u8;67],pub stop_audio_23:bool,pub stop_audio_2f:bool,pub queue_requested:bool}
pub fn plan_incomplete_selector1(before:&[u8;67],animation0_holds:&[u8])->Result<IncompleteBuildingPlan,BuildingCallbackError>{
    if before[0x2d]!=1{return Err(BuildingCallbackError::UnsupportedBuilding);}
    plan_incomplete_admitted(before,animation0_holds)
}
/// Common451F0 incomplete path with source selector-specific audio stops.
pub fn plan_incomplete_admitted(before:&[u8;67],animation0_holds:&[u8])->Result<IncompleteBuildingPlan,BuildingCallbackError>{
    let flags=u16::from_le_bytes(before[0x31..0x33].try_into().unwrap());
    if before[6]!=4||before[0x18]>=4||!matches!(before[0x2d],1|3|6|11)||flags&0x841!=0 {
        return Err(BuildingCallbackError::UnsupportedBuilding);
    }
    let mut after=*before;
    let timer=i16::from_le_bytes(after[0x3d..0x3f].try_into().unwrap());
    if timer>0 {after[0x3d..0x3f].copy_from_slice(&(timer-1).to_le_bytes());}
    if before[0x1c]!=0||before[0x30]!=0 {after[0x15]|=4;}
    if after[0x30]>0{after[0x30]-=1;}
    let queue_requested=i16::from_le_bytes(after[0x2b..0x2d].try_into().unwrap())>0;
    if queue_requested{
        after[0x19]=0;after[0x1b]=after[0x2f];after[0x1c]=0;
        if animation0_holds.is_empty()||animation0_holds.len()>255{return Err(BuildingCallbackError::InvalidAnimation);}
        let frame=usize::from(after[0x1b]).min(animation0_holds.len()-1);after[0x1b]=frame as u8;
        if (after[0x1e]as i8)>(after[0x1d]as i8){after[0x1e]=after[0x1d];}
        let counter=after[0x1e]as i8;after[0x1e]=after[0x1e].wrapping_sub(1);
        if counter<=0{
            after[0x1e]=after[0x1d];after[0x1a]=after[0x1a].min(animation0_holds[frame]);
            let hold=after[0x1a];after[0x1a]=hold.wrapping_sub(1);
            if hold==0{after[0x1a]=animation0_holds[frame];}
        }
    }
    Ok(IncompleteBuildingPlan{before:*before,after,stop_audio_23:matches!(before[0x2d],1|6),stop_audio_2f:before[0x2d]==11,queue_requested})
}

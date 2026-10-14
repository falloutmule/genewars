//!534D0 stage4 waits for action14 then requests48790 code2C.
#[derive(Debug,Clone,PartialEq,Eq)]
pub enum ConstructionStage4Error{UnsupportedActor,InvalidAnimation,MissingRequest,InvalidBuilding}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct BuildingEffectRequest{pub code:u8,pub owner:u8,pub selector:u8,pub position:[i16;2]}
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ConstructionStage4Plan{pub after:[u8;200],pub request:Option<BuildingEffectRequest>}
pub fn plan_architect_stage4(before:&[u8;200],count:u8,flags:u8)->Result<ConstructionStage4Plan,ConstructionStage4Error>{
    if before[6]!=3||before[0x55]!=9||before[0x7e]!=0||before[0x82]!=1||!matches!(before[0x83],1|3|6|11)
        ||i16::from_le_bytes(before[0x88..0x8a].try_into().unwrap())!=4||before[0x59]!=14||before[0x19]!=14{return Err(ConstructionStage4Error::UnsupportedActor);}
    if count==0||flags&!1!=0||before[0x1b]>=count{return Err(ConstructionStage4Error::InvalidAnimation);}
    let complete=before[0x1a]==0&&if flags&1!=0{before[0x1b]==0}else{before[0x1e]==0&&before[0x1b]==count-1};
    let mut after=*before;if complete{after[0x4a]&=!8;}
    Ok(ConstructionStage4Plan{after,request:complete.then_some(BuildingEffectRequest{code:0x2c,owner:before[24],selector:before[0x83],position:[i16::from_le_bytes(before[0x8c..0x8e].try_into().unwrap()),i16::from_le_bytes(before[0x8e..0x90].try_into().unwrap())]})})
}
///After a nonNULL effect resolves its nativekind4 child. The caller applies441C0+1.
pub fn finish_architect_stage4(plan:&ConstructionStage4Plan,building:&[u8;67],desired_speed:i16)->Result<[u8;200],ConstructionStage4Error>{
    if plan.request.is_none(){return Err(ConstructionStage4Error::MissingRequest);}
    let id=i16::from_le_bytes(building[15..17].try_into().unwrap());
    if building[6]!=4||building[24]!=plan.after[24]||building[45]!=plan.after[0x83]||!(0..256).contains(&id)
        ||id/64!=i16::from(building[24]){return Err(ConstructionStage4Error::InvalidBuilding);}
    let mut after=plan.after;after[0x5a]=1;after[0x60..0x64].copy_from_slice(&building[7..11]);
    after[0x3f..0x41].copy_from_slice(&desired_speed.to_le_bytes());after[0x70]|=16;
    after[0x7f..0x81].copy_from_slice(&id.to_le_bytes());after[0x81]=4;
    after[0x88..0x8a].copy_from_slice(&5i16.to_le_bytes());after[0x59]=15;after[0x70]&=0x7b;Ok(after)
}

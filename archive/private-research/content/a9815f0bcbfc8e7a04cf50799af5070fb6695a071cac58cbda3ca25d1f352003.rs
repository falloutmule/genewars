//! CD selector11 harvest substep46F0C..4707B, not the whole building callback.
//! The enclosing powered/animation gates and particle/contact pass run first.
//! Returned terrain and presentation requests must be resolved before commit.

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ExtractorHarvestError { MapLength(usize), Building, Player }

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ExtractorCallbackError { Building, Player, HarvestContactRequired }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ExtractorAudio { StopWork2f, Finish30, StartWork2f }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ExtractorIdlePlan {
    pub building_after:[u8;67],pub player_after:[u8;1805],
    pub audio:Vec<ExtractorAudio>,pub queue_requested:bool,
}

/// Whole451F0 completed grade0 Extractor callback outside the powered
/// harvest/contact frames7..10. Audio-service state is an explicit input.
/// Exact BMINE animation1:20 frames, hold1 only at frame12, flags0.
pub fn plan_completed_idle(before:&[u8;67],player:&[u8;1805],work_audio_active:bool)->Result<ExtractorIdlePlan,ExtractorCallbackError>{
    let flags=u16::from_le_bytes(before[49..51].try_into().unwrap());
    if before[6]!=4||before[24]>=4||before[45]!=11||before[46]!=0||flags&!4!=1{return Err(ExtractorCallbackError::Building);}
    if player[1]!=before[24]||player[0x4a1]!=before[24]{return Err(ExtractorCallbackError::Player);}
    let mut a=*before;let mut p=*player;let mut audio=Vec::new();
    let word=|r:&[u8],at|i16::from_le_bytes([r[at],r[at+1]]);
    let mut timer=word(before,61).wrapping_sub(1+i16::from(before[63]));
    if timer<=0{timer=0;a[49]&=!4;if word(&p,0x5ea)==0{p[0x5ea..0x5ec].copy_from_slice(&1000i16.to_le_bytes());}}
    if a[27]!=0{a[28]=1;a[21]|=4;}else{a[28]=0;if work_audio_active{audio.extend([ExtractorAudio::StopWork2f,ExtractorAudio::Finish30]);}}
    if a[27]==17{a[27]=0;a[28]=0;a[21]|=4;}
    if timer!=0{
        let cycle=a[51];a[51]=cycle.wrapping_add(1);
        if cycle>=240{a[51]=0;let funds=u32::from_le_bytes(p[0x4a7..0x4ab].try_into().unwrap());let cap=u32::from_le_bytes(p[0x5e6..0x5ea].try_into().unwrap());if funds<cap{a[27]=1;a[29]=5;audio.push(ExtractorAudio::StartWork2f);}}
        if (7..11).contains(&a[27]){return Err(ExtractorCallbackError::HarvestContactRequired);}
    }else{a[49]&=!4;}
    timer=timer.min(200);a[61..63].copy_from_slice(&timer.to_le_bytes());
    if a[63]!=0{a[63]-=1;}
    if before[25]!=a[25]||a[28]!=0||(before[49]^a[49])&4!=0||a[48]!=0{a[21]|=4;}
    if a[48]!=0{a[48]-=1;}
    a[25]=1;
    a[27]=a[27].min(19);
    if (a[30]as i8)>(a[29]as i8){a[30]=a[29];}
    let delay=a[30]as i8;a[30]=a[30].wrapping_sub(1);
    if delay<=0{
        a[30]=a[29];let hold=if a[27]==12{1}else{0};a[26]=a[26].min(hold);
        let old=a[26];a[26]=old.wrapping_sub(1);
        if old==0{let next=usize::from(a[27])+usize::from(a[28]);a[27]=if next>=20{0}else{next as u8};a[26]=if a[27]==12{1}else{0};}
    }
    Ok(ExtractorIdlePlan{building_after:a,player_after:p,audio,queue_requested:true})
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct ResourceChange { pub cell:u16, pub before:u8, pub after:u8 }

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct ExtractorHarvestPlan {
    pub building_before:[u8;67], pub building_after:[u8;67],
    pub player_before:[u8;1805], pub player_after:[u8;1805],
    pub resource:Option<ResourceChange>,
    /// Source8CBA0(cell,-1), required when the chosen cell is exhausted.
    pub depleted_terrain:Option<u16>,
    /// Source84E10 receives the final scanned position, not the chosen cell.
    pub dirty_position:Option<[i16;2]>,
    /// Source44E50(building,0x4E) followed by its audio request.
    pub empty_resource_notice:bool,
}

pub fn plan_harvest_slice(building:&[u8;67],player:&[u8;1805],map:&[u8]) -> Result<ExtractorHarvestPlan,ExtractorHarvestError> {
    if map.len()!=128*128*14 {return Err(ExtractorHarvestError::MapLength(map.len()));}
    if building[6]!=4||building[45]!=11||building[46]>2||building[24]>=4 {return Err(ExtractorHarvestError::Building);}
    if player[1]!=building[24]||player[0x4a1]!=building[24] {return Err(ExtractorHarvestError::Player);}
    let mut q=ExtractorHarvestPlan{building_before:*building,building_after:*building,player_before:*player,player_after:*player,resource:None,depleted_terrain:None,dirty_position:None,empty_resource_notice:false};
    q.building_after[0x3f]=3;
    let funds=u32::from_le_bytes(player[0x4a7..0x4ab].try_into().unwrap());
    let capacity=u32::from_le_bytes(player[0x5e6..0x5ea].try_into().unwrap());
    if funds>=capacity {return Ok(q);}
    let x=i16::from_le_bytes(building[7..9].try_into().unwrap());
    let y=i16::from_le_bytes(building[9..11].try_into().unwrap());
    let mut chosen=None;
    let mut best=0;
    for dy in -3i16..=3 {for dx in -3i16..=3 {
        let cx=usize::from((x.wrapping_add(dx*256)>>8) as u16&127);
        let cy=usize::from((y.wrapping_add(dy*256)>>8) as u16&127);
        let cell=cy*128+cx;
        let amount=map[cell*14+13];
        if amount>best {best=amount;chosen=Some(cell as u16);}
    }}
    if let Some(cell)=chosen {
        let take=best.min(building[46]*2+2);
        q.resource=Some(ResourceChange{cell,before:best,after:best-take});
        if best<=take {q.depleted_terrain=Some(cell);}
        let after=funds.wrapping_add(u32::from(take)*2).min(capacity);
        q.player_after[0x4a7..0x4ab].copy_from_slice(&after.to_le_bytes());
        q.dirty_position=Some([x.wrapping_add(3*256),y.wrapping_add(3*256)]);
    } else {
        q.building_after[0x32]|=3;
        q.empty_resource_notice=true;
    }
    Ok(q)
}

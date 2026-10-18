//! CD selector11 harvest substep46F0C..4707B, not the whole building callback.
//! The enclosing powered/animation gates and particle/contact pass run first.
//! Returned terrain and presentation requests must be resolved before commit.

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum ExtractorHarvestError { MapLength(usize), Building, Player }

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

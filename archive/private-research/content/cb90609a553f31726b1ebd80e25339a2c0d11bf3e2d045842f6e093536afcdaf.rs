//! Numeric empty-building and empty-family8 scheduler branches. Presentation
//! requests are returned explicitly; expiry requiring factories is rejected.
use crate::player_state::PLAYER_RECORD_BYTES;

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum BookkeepingError { InvalidPlayer, BuildingsPresent, CountdownExpiry, UnsupportedBuilding }
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum BookkeepingNotice { UnusedResource, Narr087, Narr064, Narr065, Effect4f }
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PlayerBookkeepingPlan {
    pub after:[u8;PLAYER_RECORD_BYTES],
    pub notices:Vec<BookkeepingNotice>,
}
fn word(r:&[u8],p:usize)->u16 {u16::from_le_bytes([r[p],r[p+1]])}
fn put(r:&mut[u8],p:usize,v:u16) {r[p..p+2].copy_from_slice(&v.to_le_bytes());}

/// 47F80 and its 437A0 call, restricted to zero buildings for this player.
/// `countdown_period` is the unsigned source word11A9DA (mission+1C7A).
pub fn plan_empty_building_player(before:&[u8;PLAYER_RECORD_BYTES],selected:u8,countdown_period:u16)
    ->Result<PlayerBookkeepingPlan,BookkeepingError> {
    if before[0x4a2]!=0 {return Err(BookkeepingError::BuildingsPresent)}
    plan_incomplete_player_tail(before,selected,countdown_period)
}

/// Incomplete selector1 buildings contribute no completed-building timer
/// sample and cannot receive437A0 storage allocation. Their451F0 callbacks
/// must already have run in source order. Other selectors/completion reject.
pub fn plan_incomplete_selector1_player(before:&[u8;PLAYER_RECORD_BYTES],records:&[[u8;67]],
    selected:u8,countdown_period:u16)->Result<PlayerBookkeepingPlan,BookkeepingError> {
    plan_selector1_player_inner(before,records,selected,countdown_period,false)
}

///47F80 numeric maintenance over caller-supplied post-callback selector1
/// records. Completed buildings contribute signed timer samples unless100h
/// is set. This does not execute451F0 or its target-search/RNG behavior.
pub fn plan_selector1_player(before:&[u8;PLAYER_RECORD_BYTES],records:&[[u8;67]],
    selected:u8,countdown_period:u16)->Result<PlayerBookkeepingPlan,BookkeepingError> {
    plan_selector1_player_inner(before,records,selected,countdown_period,true)
}

fn plan_selector1_player_inner(before:&[u8;PLAYER_RECORD_BYTES],records:&[[u8;67]],
    selected:u8,countdown_period:u16,completed_allowed:bool)->Result<PlayerBookkeepingPlan,BookkeepingError> {
    if records.len()!=64 {return Err(BookkeepingError::UnsupportedBuilding)}
    let declared=usize::from(before[0x4a2]);
    if declared>64{return Err(BookkeepingError::BuildingsPresent)}
    let mut active=0usize;let mut samples=0i32;let mut sum=0i32;
    let mut minimum=1000i16;let mut maximum=-1000i16;
    for (slot,r) in records.iter().enumerate() {
        // This adapter deliberately admits a dense active prefix. Source
        //47F80 skips inactive/newborn slots without decrementing ECX; its
        // wider sparse/newborn scan remains outside this bounded contract.
        if (r[6]!=0)!=(slot<declared){return Err(BookkeepingError::UnsupportedBuilding)}
        if r[6]==0 {continue}
        active+=1;
        if r[6]!=4||r[24]!=before[1]||r[0x2d]!=1||r[0x2e]!=0||r[0x15]&1!=0
            ||word(r,0x31)&(if completed_allowed{0x840}else{0x841})!=0
            ||word(r,15)as usize!=usize::from(before[1])*64+slot {
            return Err(BookkeepingError::UnsupportedBuilding)
        }
        if word(r,0x31)&0x101==1 {
            let timer=word(r,0x3d)as i16;
            sum+=i32::from(timer);samples+=1;
            minimum=minimum.min(timer);maximum=maximum.max(timer);
        }
    }
    if active!=declared {return Err(BookkeepingError::BuildingsPresent)}
    let mut plan=plan_incomplete_player_tail(before,selected,countdown_period)?;
    if samples>0 {
        let ring=usize::from(plan.after[0x5e5]);
        for (offset,value) in [(0x18,(sum/samples)as i16),(0x7c,maximum),(0xe0,minimum)] {
            put(&mut plan.after,0x4a1+offset+ring*2,value as u16);
        }
    }
    Ok(plan)
}

fn plan_incomplete_player_tail(before:&[u8;PLAYER_RECORD_BYTES],selected:u8,countdown_period:u16)
    ->Result<PlayerBookkeepingPlan,BookkeepingError> {
    const B:usize=0x4a1;
    if selected>=4||before[1]>=4||before[B]!=before[1] {return Err(BookkeepingError::InvalidPlayer)}
    let mut after=*before;let mut notices=Vec::new();let local=before[1]==selected;
    put(&mut after,B+0xc,0);put(&mut after,B+0xe,0);after[B+0x16f]=0;
    after[B+0x14..B+0x18].copy_from_slice(&600u32.to_le_bytes());
    let ring=(u16::from(after[B+0x144])+1)%50;after[B+0x144]=ring as u8;
    for offset in [0x18,0x7c,0xe0] {put(&mut after,B+offset+usize::from(ring)*2,0);}
    // 437A0 cannot allocate positive resources without a building; it clears
    // that residual and creates its warning timer, preserving nonpositive data.
    if i32::from_le_bytes(after[B+6..B+10].try_into().unwrap())>0 {
        if word(&after,B+0x14f)==0 {put(&mut after,B+0x14f,1000);}
        after[B+6..B+10].fill(0);
        if word(&after,B+0x14f)==1000&&local {notices.push(BookkeepingNotice::UnusedResource);}
    }
    if word(&after,B+0x149)==1000 {
        put(&mut after,B+0x149,800);
        if local {notices.push(if after[B+0x162]==0{BookkeepingNotice::Narr087}else{BookkeepingNotice::Narr064});}
    }
    if word(&after,B+0x14b)==1000 {put(&mut after,B+0x14b,500);if local{notices.push(BookkeepingNotice::Narr065);}}
    if word(&after,B+0x153)!=0&&local {notices.push(BookkeepingNotice::Effect4f);}
    if word(&after,B+0x14d)==1000&&countdown_period!=0 {put(&mut after,B+0x14d,countdown_period/4);}
    for i in 0..8 {let p=B+0x149+i*2;let n=word(&after,p);if n!=0{put(&mut after,p,n-1);}}
    let timer=u32::from_le_bytes(after[0x499..0x49d].try_into().unwrap());
    if timer!=0 {after[0x499..0x49d].copy_from_slice(&(timer-1).to_le_bytes());}
    Ok(PlayerBookkeepingPlan{after,notices})
}

#[cfg(test)]
mod incomplete_tests {
    use super::*;
    #[test]
    fn sparse_building_partition_cannot_be_treated_as_a_dense_source_scan(){
        let mut player=[0;PLAYER_RECORD_BYTES];player[0x4a2]=1;
        let mut records=[[0;67];64];records[1][6]=4;records[1][0x2d]=1;records[1][15]=1;
        assert_eq!(plan_incomplete_selector1_player(&player,&records,0,6000),Err(BookkeepingError::UnsupportedBuilding));
        records[0]=records[1];records[0][15]=0;records[1].fill(0);
        assert!(plan_incomplete_selector1_player(&player,&records,0,6000).is_ok());
        records[0][0x15]|=1;
        assert_eq!(plan_incomplete_selector1_player(&player,&records,0,6000),Err(BookkeepingError::UnsupportedBuilding));
    }
    #[test]
    fn incomplete_partition_preserves_count_and_funds_while_clearing_unallocatable_resource(){
        let mut player=[0;PLAYER_RECORD_BYTES];player[0x4a2]=2;
        player[0x4a3..0x4a7].copy_from_slice(&1312i32.to_le_bytes());
        player[0x4a7..0x4ab].copy_from_slice(&125i32.to_le_bytes());
        let mut records=[[0;67];64];
        for (slot,r) in records[..2].iter_mut().enumerate(){r[6]=4;r[0x2d]=1;r[15]=slot as u8;}
        let plan=plan_incomplete_selector1_player(&player,&records,0,6000).unwrap();
        assert_eq!(plan.after[0x4a2],2);assert_eq!(&plan.after[0x4a3..0x4a7],&1312i32.to_le_bytes());
        assert_eq!(&plan.after[0x4a7..0x4ab],&0i32.to_le_bytes());
        assert_eq!(plan.notices,[BookkeepingNotice::UnusedResource]);
        assert_eq!(word(&plan.after,0x5f0),999);
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct CountdownPlan {pub before:u16,pub after:u16,pub started:bool}
/// 13780 numeric count branch with no active family8 records. The host must
/// validate that pool. A transition through expiry requires original effect
/// factories and remains an error rather than discarding their side effects.
pub fn plan_empty_family8_countdown(count:u16,period:u16,player_progress:i8,player_byte607:u8,runtime_flags:u8)
    ->Result<CountdownPlan,BookkeepingError> {
    let special=matches!(player_progress,14|20);
    let after=if count==0 {
        if special {3000}else if runtime_flags&0xc0!=0 {65535}else{period}
    }else if (special&&player_byte607>0)||(!special&&count<65535) {
        if count==1 {return Err(BookkeepingError::CountdownExpiry)}count-1
    }else{count};
    Ok(CountdownPlan{before:count,after,started:count==0})
}

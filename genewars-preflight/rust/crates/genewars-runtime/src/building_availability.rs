//! CD QA BETA V5 construction menu predicate43960.
//! Eligibility does not imply an implemented callback or a valid placement.

const THRESHOLDS: [u8;14] = [1,9,4,0,2,3,0,10,6,30,5,0,6,2];
const DEPENDENCIES: [u8;14] = [3,10,0,11,0,0,3,10,6,10,0,255,10,0];
pub const MENU_ORDER: [u8;14] = [11,3,6,8,4,2,0,13,10,5,1,12,7,9];

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum AvailabilityError { InvalidSelector(u8), InvalidOwner(u8) }

/// Source takes the owner's building state at player+4A1 and reads the
/// signed campaign progress from that owner's player+6E7. A completed
/// building count unlocks dependents; the owned bit also allows rebuilding.
pub fn available(player:&[u8;1805],selector:u8)->Result<bool,AvailabilityError> {
    let index=usize::from(selector);
    let threshold=*THRESHOLDS.get(index).ok_or(AvailabilityError::InvalidSelector(selector))?;
    if player[0x4a1]>=4||player[1]!=player[0x4a1]{return Err(AvailabilityError::InvalidOwner(player[0x4a1]));}
    if i16::from(player[0x6e7] as i8)<i16::from(threshold)||threshold>=30{return Ok(false);}
    let dependency=DEPENDENCIES[index];
    let owned=u16::from_le_bytes(player[0x60e..0x610].try_into().unwrap());
    Ok(dependency==255||player[0x600+usize::from(dependency)]!=0||owned&(1u16<<selector)!=0)
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn first_mission_unlocks_only_from_real_completed_counts() {
        let mut player=[0;1805];
        let choices=|p:&[u8;1805]|MENU_ORDER.into_iter().filter(|s|available(p,*s)==Ok(true)).collect::<Vec<_>>();
        assert_eq!(choices(&player),[11]);
        player[0x60b]=1;assert_eq!(choices(&player),[11,3]);
        player[0x603]=1;assert_eq!(choices(&player),[11,3,6]);
        player[0x603]=0;player[0x60e]=1<<6;assert_eq!(choices(&player),[11,3,6]);
        player[0x6e7]=255;assert!(choices(&player).is_empty());
        assert_eq!(available(&player,14),Err(AvailabilityError::InvalidSelector(14)));
    }
}

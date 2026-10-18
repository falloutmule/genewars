//! Bounded mission-start funds assignment at source7D744..7D58F.
//! Earlier player identity/mode setup and later startup calls are separate.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionFundsError {
    InvalidSlot(u8),
    InvalidActiveCount(u8),
    PlayerIdentity { slot: u8, actual: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionFundsPlan {
    pub before: [u8; 1805],
    pub after: [u8; 1805],
    pub applied: bool,
}

/// The source uses the unsigned mission byte11A9BC (packed offset1C5C).
/// Mode0 receives100 times that byte; every nonzero mode receives2000.
/// The separate balance at player+4A7 and all other fields remain unchanged.
pub fn plan_mission_player_funds(
    before: &[u8; 1805],
    slot: u8,
    active_players: u8,
    mission_units: u8,
) -> Result<MissionFundsPlan, MissionFundsError> {
    if slot >= 4 { return Err(MissionFundsError::InvalidSlot(slot)); }
    if active_players > 4 { return Err(MissionFundsError::InvalidActiveCount(active_players)); }
    if before[1] != slot { return Err(MissionFundsError::PlayerIdentity { slot, actual: before[1] }); }
    let mut after = *before;
    let applied = slot < active_players;
    if applied {
        let funds = if before[2] == 0 { u32::from(mission_units) * 100 } else { 2000 };
        after[0x4a3..0x4a7].copy_from_slice(&funds.to_le_bytes());
    }
    Ok(MissionFundsPlan { before: *before, after, applied })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_invalid_player_admission_without_a_plan() {
        let mut before = [0xa5; 1805];
        before[1] = 0;
        assert_eq!(plan_mission_player_funds(&before, 4, 4, 20), Err(MissionFundsError::InvalidSlot(4)));
        assert_eq!(plan_mission_player_funds(&before, 0, 5, 20), Err(MissionFundsError::InvalidActiveCount(5)));
        assert_eq!(plan_mission_player_funds(&before, 1, 4, 20), Err(MissionFundsError::PlayerIdentity { slot: 1, actual: 0 }));
    }
}

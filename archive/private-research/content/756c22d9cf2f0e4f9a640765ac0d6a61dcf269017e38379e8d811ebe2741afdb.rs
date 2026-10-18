use super::*;
use crate::mission_player_funds::{plan_mission_player_funds, MissionFundsPlan};

impl ActorWorld {
    /// Apply the proved mission-start funds block once at its explicit load
    /// boundary. The active count comes from the admitted mission globals.
    /// This initializes funds; it is not a per-frame refill or save loader.
    pub fn initialize_mission_player_funds(&mut self, active_players:u8)
        ->Result<Vec<MissionFundsPlan>,ActorWorldError> {
        let units=self.backing[0x1c5c];
        let mut players=self.players.clone();
        let mut plans=Vec::with_capacity(4);
        for (slot,player) in players.iter_mut().enumerate() {
            let plan=plan_mission_player_funds(player.bytes(),slot as u8,active_players,units)
                .map_err(ActorWorldError::MissionFunds)?;
            player.replace_bookkeeping_checked(&plan.before,&plan.after)?;
            plans.push(plan);
        }
        self.players=players;
        Ok(plans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->ActorWorld {
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());
        bytes[337_054..337_058].copy_from_slice(&0x6beu32.to_le_bytes());bytes[0x1c5c]=20;
        let source=ConqMissionSource::parse(&bytes).unwrap();
        let players=core::array::from_fn(|slot|PlayerState::fresh_campaign(slot as u8,[4096,4096],b"fixture\0").unwrap());
        ActorWorld::from_parts(&source,&[[0;200];20],core::array::from_fn(|_|ActorHandleBindings::default()),players,
            BlueprintRng{seed:5428,ring:0,draw_count:0}).unwrap()
    }
    #[test]
    fn mission_funds_changes_only_active_player_and_preserves_origin() {
        let mut world=fixture();let before=world.clone();
        let plans=world.initialize_mission_player_funds(1).unwrap();
        assert!(plans[0].applied);assert!(plans[1..].iter().all(|plan|!plan.applied));
        let mut expected=before.clone();let mut player=*expected.players[0].bytes();
        player[0x4a3..0x4a7].copy_from_slice(&2000u32.to_le_bytes());
        let old=*expected.players[0].bytes();expected.players[0].replace_bookkeeping_checked(&old,&player).unwrap();
        assert_eq!(world,expected);
    }
    #[test]
    fn invalid_last_player_rolls_back_earlier_funding() {
        let mut world=fixture();let mut bad=*world.players[3].bytes();bad[1]=2;
        world.players[3]=PlayerState::from_record(&bad).unwrap();let before=world.clone();
        assert_eq!(world.initialize_mission_player_funds(4).unwrap_err(),ActorWorldError::MissionFunds(
            crate::mission_player_funds::MissionFundsError::PlayerIdentity{slot:3,actual:2}));
        assert_eq!(world,before);
    }
}

//! Bridge live world state to source72ED0 without applying result-screen effects.
use super::*;
use crate::mission_outcome::{MissionOutcomeContext, MissionOutcomeError, MissionOutcomePlan};

/// Runtime globals that are not serialized mission fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutcomeRuntimeInputs {
    pub flags: u8,
    pub mode: u8,
    pub participating_players: u8,
    pub selected_player: u8,
    pub frame: u32,
}

impl ActorWorld {
    /// Read-only plan using live backing countdown and actual actor capability.
    /// The host must handle the result request and player changes together;
    /// obtaining this plan alone does not complete a mission or transition.
    pub fn plan_mission_outcome(&self, runtime: OutcomeRuntimeInputs)
        -> Result<MissionOutcomePlan, MissionOutcomeError<ActorWorldError>>
    {
        let b = &self.backing;
        let context = MissionOutcomeContext {
            flags: runtime.flags, mode: runtime.mode,
            active: runtime.participating_players, selected: runtime.selected_player,
            frame: runtime.frame,
            mission_counter: u32::from_le_bytes(b[0x1ec2..0x1ec6].try_into().unwrap()),
            target_count: word(b, 0x1ea6) as u16,
            mission_limit: word(b, 0x1c7c) as u16,
            mission_timer: word(b, 0x355c) as u16,
            terrain_count: b[0x34b8], field_34b9: b[0x34b9],
            terrain_limit: i32::from_le_bytes(b[0x1eaa..0x1eae].try_into().unwrap()),
        };
        let players = core::array::from_fn(|id| *self.players[id].bytes());
        crate::mission_outcome::evaluate(&players, &context,
            |id| self.construction_capability_exhausted(id))
    }
}

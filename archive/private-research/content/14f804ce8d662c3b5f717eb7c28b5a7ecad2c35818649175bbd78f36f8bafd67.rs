//! Bridge live world state to source72ED0 without applying result-screen effects.
use super::*;
use crate::mission_outcome::{MissionOutcomeContext, MissionOutcomeError, MissionOutcomePlan};
use crate::mission_context::{MissionRuntimeContext,CallbackGate};

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum OwnedMissionOutcomeError {
    CallbackNotReady(CallbackGate),
    Source(MissionOutcomeError<ActorWorldError>),
    Player(crate::player_state::PlayerStateError),
    Context(&'static str),
}

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
    /// Evaluate source72ED0 from current state, then atomically commit all
    /// player afterimages and the flags/typed outcome. Result UI requests are
    /// returned for the host; this does not execute campaign continuation.
    pub fn step_mission_outcome(&mut self,context:&mut MissionRuntimeContext,selected_player:u8)
        ->Result<MissionOutcomePlan,OwnedMissionOutcomeError>{
        if context.callback_gate()!=CallbackGate::Ready {
            return Err(OwnedMissionOutcomeError::CallbackNotReady(context.callback_gate()));
        }
        let before_context=context.clone();
        let before_players=core::array::from_fn::<_,4,_>(|id|*self.players[id].bytes());
        let plan=self.plan_mission_outcome(OutcomeRuntimeInputs{
            flags:context.known_runtime_flags(),mode:context.runtime_mode().ok_or(OwnedMissionOutcomeError::Context("runtime mode unknown"))?,
            participating_players:context.participating_players(),selected_player,frame:context.frame(),
        }).map_err(OwnedMissionOutcomeError::Source)?;
        let mut staged_players=self.players.clone();
        for id in 0..4 {
            staged_players[id].replace_bookkeeping_checked(&before_players[id],&plan.scan.players[id])
                .map_err(OwnedMissionOutcomeError::Player)?;
        }
        let mut staged_context=context.clone();
        staged_context.apply_outcome_checked(&before_context,&plan).map_err(OwnedMissionOutcomeError::Context)?;
        self.players=staged_players;*context=staged_context;Ok(plan)
    }
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

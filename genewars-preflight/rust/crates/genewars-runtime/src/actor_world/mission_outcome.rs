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

#[cfg(test)]
mod adapter_tests {
    use super::*;
    use crate::mission_context::DevelopmentContext;
    use crate::mission_outcome::MissionOutcome;
    // Synthetic adapter fixtures only; these are not campaign acceptance.
    fn fixture() -> ActorWorld {
        let mut bytes=vec![0;343_958];bytes[..4].copy_from_slice(&0x55336u32.to_le_bytes());
        bytes[337_054..337_058].copy_from_slice(&0x6BEu32.to_le_bytes());
        ActorWorld::from_parts(&ConqMissionSource::parse(&bytes).unwrap(),&[[0;200];20],
            core::array::from_fn(|_|ActorHandleBindings::default()),
            core::array::from_fn(|id|PlayerState::fresh_campaign(id as u8,[4096,4096],b"fixture\0").unwrap()),
            BlueprintRng{seed:0,ring:31,draw_count:0}).unwrap()
    }
    fn context(mode:u8)->MissionRuntimeContext {
        MissionRuntimeContext::development(DevelopmentContext{participating_players:1,runtime_flags:4,runtime_mode:mode,countdown:0,frame:0,replay_seed:0}).unwrap()
    }
    #[test]fn outcome_adapter_running_matches_plan(){
        let mut w=fixture();let mut c=context(0);let before=w.clone();
        let plan=w.plan_mission_outcome(OutcomeRuntimeInputs{flags:4,mode:0,participating_players:1,selected_player:0,frame:0}).unwrap();
        assert_eq!(plan.outcome,MissionOutcome::Running);
        assert_eq!(w.step_mission_outcome(&mut c,0).unwrap(),plan);
        for id in 0..4{assert_eq!(w.players[id].bytes(),&plan.scan.players[id]);}
        assert_eq!(w.backing,before.backing);assert_eq!(w.actors,before.actors);assert_eq!(c.outcome(),MissionOutcome::Running);
    }
    #[test]fn outcome_adapter_rejects_context_atomically(){
        for (mut c,selected) in [(context(0),4),(context(2),0),(MissionRuntimeContext::partial_cd_conq001(None),0)] {
            let mut w=fixture();let before=w.clone();let old=c.clone();
            assert!(w.step_mission_outcome(&mut c,selected).is_err());assert_eq!(w,before);assert_eq!(c,old);
        }
    }
    #[test]fn outcome_adapter_terminal_latches_once(){
        for won in [false,true]{
            let mut w=fixture();let mut c=context(0);let mut player=*w.players[0].bytes();
            if won {player[0x49d..0x4a1].copy_from_slice(&1i32.to_le_bytes());player[3]=1;player[0x4a7..0x4ab].copy_from_slice(&1i32.to_le_bytes());w.backing[0x1ea6..0x1ea8].copy_from_slice(&1u16.to_le_bytes());}
            else {player[0x6e3]=1;}
            w.players[0]=PlayerState::from_record(&player).unwrap();
            let plan=w.step_mission_outcome(&mut c,0).unwrap();
            let expected=if won{MissionOutcome::Won{player:0}}else{MissionOutcome::Lost{player:0,winner:None}};
            assert_eq!(plan.outcome,expected);assert_eq!(c.outcome(),expected);
            let before=w.clone();let old=c.clone();assert!(w.step_mission_outcome(&mut c,0).is_err());assert_eq!(w,before);assert_eq!(c,old);
        }
    }
}

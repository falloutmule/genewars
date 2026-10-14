//! Explicit scheduling context for development and bounded fresh CD entry.
//!
//! Evidence: research/ghidra/mission-bootstrap-context-evidence.json. The partial
//! reference constructor records only the CONQ001 tail and the known 7D090
//! writes. Transitive startup effects remain unresolved; it cannot admit ticks.

pub const CD_EXECUTABLE_SHA256: &str = "4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba";
pub const CONQ001_SHA256: &str = "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextOrigin {
    Development,
    /// Bound to the constants above; not interchangeable with other missions.
    PartialCdConq001,
    /// First mission after fresh constructors and completed source bootstrap.
    FreshCdConq001,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RngProvenance {
    OriginalStartupUnknown,
    /// Serialized5428/count16855 followed by exactly64 blueprint draws.
    /// Ring position is diagnostic metadata, explicitly retained by caller.
    CdConq001SerializedThenBlueprints { seed:u16, draw_count:u32, diagnostic_ring:u8 },
    /// Caller-specified replay input. Does not prove original seed generation,
    /// RNG ring contents, draw count, or transitive startup draws.
    ReplayOverride { seed: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartupBlocker {
    TransitiveStartupEffects,
    RuntimeMode,
    ConditionalMainPanelEntry,
    InheritedNativePriors,
    OriginalRngStream,
}

const REFERENCE_BLOCKERS: [StartupBlocker; 5] = [
    StartupBlocker::TransitiveStartupEffects,
    StartupBlocker::RuntimeMode,
    StartupBlocker::ConditionalMainPanelEntry,
    StartupBlocker::InheritedNativePriors,
    StartupBlocker::OriginalRngStream,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallbackGate {
    Ready,
    Disabled,
    Blocked(&'static [StartupBlocker]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevelopmentContext {
    pub participating_players: u8,
    pub runtime_flags: u8,
    pub runtime_mode: u8,
    pub countdown: u16,
    pub frame: u32,
    pub replay_seed: u16,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionRuntimeContext {
    origin: ContextOrigin,
    participating_players: u8,
    /// Known-write projection for partial bootstrap, not a final flags claim.
    runtime_flags: u8,
    runtime_mode: Option<u8>,
    countdown: u16,
    frame: u32,
    rng: RngProvenance,
}

impl MissionRuntimeContext {
    /// Explicit opt-in; preserves caller policy rather than choosing defaults.
    pub fn development(input: DevelopmentContext) -> Result<Self, &'static str> {
        if !(1..=4).contains(&input.participating_players) {
            return Err("participating player count must be in 1..=4");
        }
        Ok(Self {
            origin: ContextOrigin::Development,
            participating_players: input.participating_players,
            runtime_flags: input.runtime_flags,
            runtime_mode: Some(input.runtime_mode),
            countdown: input.countdown,
            frame: input.frame,
            rng: RngProvenance::ReplayOverride { seed: input.replay_seed },
        })
    }

    /// 71B70 loads CONQ001 flags 0x38, player count 1 and frame 0. 7D090
    /// writes countdown 1000 and ORs flags with 2 after calling 884D0.
    /// 0x3a is only the known-write projection, since that call and conditional
    /// MAINPANL entry are not fully reconstructed. No callbacks are admitted.
    pub fn partial_cd_conq001(replay_seed: Option<u16>) -> Self {
        Self {
            origin: ContextOrigin::PartialCdConq001,
            participating_players: 1,
            runtime_flags: 0x38 | 2,
            runtime_mode: None,
            countdown: 1000,
            frame: 0,
            rng: replay_seed.map_or(RngProvenance::OriginalStartupUnknown,
                |seed| RngProvenance::ReplayOverride { seed }),
        }
    }

    /// Admit the first MAINPANL frame after the bounded fresh CD pipeline.
    ///
    /// Caller must complete restored-player preparation, the landing factory
    /// and72AD0 visibility/shade initialization before using this context.
    /// This is fresh process/no special CLI mode/no previous gameplay only.
    /// Static constructors leave185EC4 null; the initial MAINPANL entry calls
    ///25150 and sets flags4. Old mission-view exit7D020 resets frame to0 before
    /// that entry. It does not continue a saved or previously played world.
    /// Opaque blueprint scratch bytes are preserved by the separate bootstrap
    /// input; this context makes no claim about their values.
    pub fn fresh_cd_conq001(rng_after_blueprints:crate::blueprint::BlueprintRng)
        ->Result<Self,&'static str>{
        if rng_after_blueprints.seed!=10548 || rng_after_blueprints.draw_count!=16919 {
            return Err("fresh CD context requires serialized RNG followed by64 blueprint draws");
        }
        if rng_after_blueprints.ring>=32 {return Err("diagnostic RNG ring must be below32");}
        Ok(Self{origin:ContextOrigin::FreshCdConq001,participating_players:1,
            runtime_flags:0x3e,runtime_mode:Some(0),countdown:1000,frame:0,
            rng:RngProvenance::CdConq001SerializedThenBlueprints{
                seed:10548,draw_count:16919,diagnostic_ring:rng_after_blueprints.ring}})
    }

    pub fn origin(&self) -> ContextOrigin { self.origin }
    pub fn participating_players(&self) -> u8 { self.participating_players }
    pub fn known_runtime_flags(&self) -> u8 { self.runtime_flags }
    pub fn runtime_mode(&self) -> Option<u8> { self.runtime_mode }
    pub fn countdown(&self) -> u16 { self.countdown }
    pub fn frame(&self) -> u32 { self.frame }
    pub fn rng_provenance(&self) -> RngProvenance { self.rng }
    pub fn startup_blockers(&self) -> &'static [StartupBlocker] {
        match self.origin {
            ContextOrigin::Development | ContextOrigin::FreshCdConq001 => &[],
            ContextOrigin::PartialCdConq001 => &REFERENCE_BLOCKERS,
        }
    }

    /// Source 1C120: flags mask 0x04 set AND mode mask 0x02 clear.
    /// A replay seed never resolves the partial startup's other unknowns.
    pub fn callback_gate(&self) -> CallbackGate {
        if !self.startup_blockers().is_empty() {
            return CallbackGate::Blocked(self.startup_blockers());
        }
        match self.runtime_mode {
            Some(mode) if self.runtime_flags & 4 != 0 && mode & 2 == 0 => CallbackGate::Ready,
            Some(_) => CallbackGate::Disabled,
            None => CallbackGate::Blocked(&[StartupBlocker::RuntimeMode]),
        }
    }

    /// Caller invokes only after the entire outer frame's callbacks AND
    /// presentation have completed (1BBC0). Also advances gated-off frames.
    /// Do not call after a failed transactional frame. This is not a world tick.
    pub fn finish_outer_frame(&mut self) { self.frame = self.frame.wrapping_add(1); }
    /// Mirror the live serialized countdown after a completed callback transaction.
    pub fn observe_countdown_after_callbacks(&mut self, countdown:u16) { self.countdown=countdown; }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn development(flags: u8, mode: u8) -> MissionRuntimeContext {
        MissionRuntimeContext::development(DevelopmentContext {
            participating_players: 1, runtime_flags: flags, runtime_mode: mode,
            countdown: 1000, frame: 0, replay_seed: 5428,
        }).unwrap()
    }
    #[test]
    fn fresh_reference_requires_completed_blueprint_rng_and_starts_at_frame_zero(){
        let rng=crate::blueprint::BlueprintRng{seed:10548,ring:31,draw_count:16919};
        let mut context=MissionRuntimeContext::fresh_cd_conq001(rng).unwrap();
        assert_eq!((context.participating_players(),context.known_runtime_flags(),context.runtime_mode(),context.countdown(),context.frame()),(1,0x3e,Some(0),1000,0));
        assert_eq!(context.callback_gate(),CallbackGate::Ready);
        context.finish_outer_frame();assert_eq!(context.frame(),1);
        for invalid in [crate::blueprint::BlueprintRng{seed:5428,..rng},crate::blueprint::BlueprintRng{draw_count:0,..rng},crate::blueprint::BlueprintRng{ring:32,..rng}]{assert!(MissionRuntimeContext::fresh_cd_conq001(invalid).is_err());}
    }
    #[test]
    fn partial_reference_preserves_known_writes_without_admitting_startup() {
        let context = MissionRuntimeContext::partial_cd_conq001(None);
        assert_eq!(context.participating_players(), 1);
        assert_eq!(context.known_runtime_flags(), 0x3a);
        assert_eq!(context.countdown(), 1000);
        assert_eq!(context.frame(), 0);
        assert_eq!(context.runtime_mode(), None);
        assert_eq!(context.rng_provenance(), RngProvenance::OriginalStartupUnknown);
        assert!(matches!(context.callback_gate(), CallbackGate::Blocked(_)));
    }
    #[test]
    fn replay_override_does_not_clear_reference_blockers() {
        let context = MissionRuntimeContext::partial_cd_conq001(Some(65535));
        assert_eq!(context.rng_provenance(), RngProvenance::ReplayOverride { seed: 65535 });
        assert_eq!(context.startup_blockers(), &REFERENCE_BLOCKERS);
        assert!(matches!(context.callback_gate(), CallbackGate::Blocked(_)));
    }
    #[test]
    fn explicit_gate_matches_both_source_masks() {
        for flags in 0..=255 {
            for mode in 0..=255 {
                assert_eq!(development(flags, mode).callback_gate(),
                    if flags & 4 != 0 && mode & 2 == 0 { CallbackGate::Ready }
                    else { CallbackGate::Disabled });
            }
        }
    }
    #[test]
    fn callbacks_observe_current_frame_before_outer_completion() {
        let mut context = development(4, 0);
        let callback_frame = context.frame();
        assert_eq!(callback_frame, 0);
        assert_eq!(context.frame(), 0);
        context.finish_outer_frame();
        assert_eq!(context.frame(), 1);
        context.frame = u32::MAX;
        context.finish_outer_frame();
        assert_eq!(context.frame(), 0);
    }
    #[test]
    fn gated_frames_advance_but_invalid_player_counts_are_rejected() {
        let mut context = development(0, 0);
        assert_eq!(context.callback_gate(), CallbackGate::Disabled);
        context.finish_outer_frame();
        assert_eq!(context.frame(), 1);
        for count in [0, 5, 255] {
            assert!(MissionRuntimeContext::development(DevelopmentContext {
                participating_players: count, runtime_flags: 4, runtime_mode: 0,
                countdown: 0, frame: 0, replay_seed: 0,
            }).is_err());
        }
    }
}

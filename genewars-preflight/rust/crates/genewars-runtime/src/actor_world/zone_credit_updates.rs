//! Future single-Mule Krell credit transaction with ordered source tails.
//! Caller owns7A280 zone/slot scheduling and quota-exhaustion continuation.
//! No zone activation, countdown change or campaign transition occurs here.
use super::*;
use crate::creature_world::CreatureAnimationBinding;
use crate::mule_zone_credit::{self, Continuation, CreditContext, CreditError, ParticlePlan};
use super::zone_marker_factory::{ZoneScoreMarkerFactoryError, ZoneScoreMarkerReceipt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneCreditError {
    World(ActorWorldError),
    Credit(CreditError),
    Marker(ZoneScoreMarkerFactoryError),
    InvalidZone(u8),
    InvalidMule(u16),
    InvalidCreatureCount,
    UnsupportedContinuation,
}
impl From<ActorWorldError> for ZoneCreditError {
    fn from(error: ActorWorldError) -> Self { Self::World(error) }
}
impl From<CreditError> for ZoneCreditError {
    fn from(error: CreditError) -> Self { Self::Credit(error) }
}
impl From<ZoneScoreMarkerFactoryError> for ZoneCreditError {
    fn from(error: ZoneScoreMarkerFactoryError) -> Self { Self::Marker(error) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneCreditAudio {
    pub creature_slot: u16,
    pub code: u8,
    pub ebx: u8,
    pub ecx: u8,
    pub stack: [i32; 2],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneCreditParticle {
    pub plan: ParticlePlan,
    pub created_effect: Option<u16>,
    pub factory_draw: Option<landing::LandingDraw>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneCreditReceipt {
    pub zone_index: u8,
    pub mule_slot: u16,
    pub credits: [i32; 2],
    pub pending_audio: Vec<ZoneCreditAudio>,
    pub markers: Vec<ZoneScoreMarkerReceipt>,
    pub particles: Vec<ZoneCreditParticle>,
    pub rng_before: BlueprintRng,
    pub rng_after: BlueprintRng,
}

impl ActorWorld {
    /// One existing active zone and one actual living Mule. This adapter does
    /// not establish the source7A280 countdown gate or remaining zone/slot order;
    /// the live scheduler must prove them before calling it. All failures,
    /// including later marker/particle factories, roll back the entire credit.
    #[allow(clippy::too_many_arguments)]
    pub fn credit_one_krell_mule(
        &mut self,
        zone_index: u8,
        mule_slot: u16,
        source_frame: u32,
        participating_players: u8,
        selected_player: u8,
        context: LandingPhysicsContext,
        trig: &TrigTables,
    ) -> Result<ZoneCreditReceipt, ZoneCreditError> {
        let linked = self.validate_links()?;
        self.validate_effect_count()?;
        let zone_count = usize::from(self.backing[0x34b8]);
        if zone_count > 4 || usize::from(zone_index) >= zone_count {
            return Err(ZoneCreditError::InvalidZone(zone_index));
        }
        let pool = self.creatures.as_ref().ok_or(ZoneCreditError::InvalidMule(mule_slot))?;
        let active = pool.records().iter().filter(|record| record[6] != 0).count();
        if word(&self.backing, 0x3550) < 0 || word(&self.backing, 0x3550) as usize != active {
            return Err(ZoneCreditError::InvalidCreatureCount);
        }
        if mule_slot >= 300 || !linked.contains(&(1, mule_slot as i16)) {
            return Err(ZoneCreditError::InvalidMule(mule_slot));
        }
        let mule = *pool.record(usize::from(mule_slot))
            .map_err(|error| ZoneCreditError::World(error.into()))?;
        if pool.binding(usize::from(mule_slot))
            .map_err(|error| ZoneCreditError::World(error.into()))?
            != Some(CreatureAnimationBinding::MuleSelector4)
            || mule[6] != 1 || mule[122] != 4 || mule[94] >= 24
            || word(&mule, 15) != mule_slot as i16
        {
            return Err(ZoneCreditError::InvalidMule(mule_slot));
        }
        let offset = 0x2ff8 + usize::from(zone_index) * 19;
        let zone: [u8; 19] = self.backing[offset..offset + 19].try_into().unwrap();
        let player = *self.player(selected_player)?.bytes();
        let approval_limit = u16::from_le_bytes(self.backing[0x1c7c..0x1c7e].try_into().unwrap());
        let credit = mule_zone_credit::plan_one_mule(&zone, &player, &mule, CreditContext {
            frame: source_frame, participating_players, selected_player, approval_limit,
        })?;
        let rng_before = self.rng;
        let mut staged = self.clone();
        staged.backing[offset..offset + 19].copy_from_slice(&credit.zone_after);
        staged.players[usize::from(selected_player)]
            .replace_bookkeeping_checked(&player, &credit.player_after)
            .map_err(ActorWorldError::from)?;
        let mut pending_audio = Vec::new();
        let mut markers = Vec::new();
        let mut particles = Vec::new();
        // plan_one_mule returns source order, including zero-credit neutral
        // marker requests. Never batch3N draws before committing each factory.
        for continuation in credit.continuations {
            match continuation {
                Continuation::Audio { creature_slot, code, ebx, ecx, stack } => {
                    pending_audio.push(ZoneCreditAudio { creature_slot, code, ebx, ecx, stack });
                }
                Continuation::Marker { creature_slot, code, ebx, ecx, stack } => {
                    if creature_slot != mule_slot || code != 0x3c || ebx != 0 || ecx != 10
                        || stack[0] != 10 || !(1..=2).contains(&stack[1]) || stack[2] != -1
                    {
                        return Err(ZoneCreditError::UnsupportedContinuation);
                    }
                    for _ in 0..stack[1] {
                        markers.push(staged.create_zone_score_marker(mule_slot, 10, 10, -1, context.gravity, trig)?);
                    }
                }
                Continuation::ParticleTail(tail) => {
                    let plan = mule_zone_credit::plan_particle_tail(&tail, staged.rng)?;
                    staged.rng = plan.rng_after;
                    let created = staged.create_particle(plan.code, plan.owner, plan.position,
                        plan.physical_height, plan.angles, false, context, trig)?;
                    let (created_effect, factory_draw) = match created {
                        Some((slot, draw)) => (Some(slot), Some(draw)),
                        None => (None, None),
                    };
                    particles.push(ZoneCreditParticle { plan, created_effect, factory_draw });
                }
            }
        }
        staged.validate_links()?;
        staged.validate_effect_count()?;
        let receipt = ZoneCreditReceipt { zone_index, mule_slot, credits: credit.credits,
            pending_audio, markers, particles, rng_before, rng_after: staged.rng };
        *self = staged;
        Ok(receipt)
    }
}

//! One atomic source6D500 code3C marker factory iteration in the live world.
//! Zone credit, loop counts, lifecycle and presentation are separate adapters.
use super::*;
use crate::creature_world::CreatureAnimationBinding;
use crate::zone_score_marker::{self, MarkerError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneScoreMarkerFactoryError {
    World(ActorWorldError),
    Marker(MarkerError),
    InvalidMule(u16),
}
impl From<ActorWorldError> for ZoneScoreMarkerFactoryError {
    fn from(error: ActorWorldError) -> Self { Self::World(error) }
}
impl From<MarkerError> for ZoneScoreMarkerFactoryError {
    fn from(error: MarkerError) -> Self { Self::Marker(error) }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneScoreMarkerReceipt {
    pub mule_slot: u16,
    pub effect_slot: u16,
    pub angles: [u16; 3],
    /// Source threeF6 draws immediately followed by the factory229 draw.
    pub draws: [landing::LandingDraw; 4],
    pub rng_before: BlueprintRng,
    pub rng_after: BlueprintRng,
}

impl ActorWorld {
    /// Creates exactly one marker. The caller supplies source6D500 parameters;
    /// subsequent iterations must observe this committed pool, map and RNG.
    /// No score/player/zone fields are changed. Full-pool replacement rejects
    /// atomically rather than dropping a required marker or altering another.
    pub fn create_zone_score_marker(
        &mut self,
        mule_slot: u16,
        speed_base: i32,
        divisor: i32,
        owner_argument: i8,
        gravity: i16,
        trig: &TrigTables,
    ) -> Result<ZoneScoreMarkerReceipt, ZoneScoreMarkerFactoryError> {
        let linked = self.validate_links()?;
        self.validate_effect_count()?;
        if mule_slot >= 300 || !linked.contains(&(1, mule_slot as i16)) {
            return Err(ZoneScoreMarkerFactoryError::InvalidMule(mule_slot));
        }
        let creatures = self.creatures.as_ref()
            .ok_or(ZoneScoreMarkerFactoryError::InvalidMule(mule_slot))?;
        let mule = creatures.record(usize::from(mule_slot))
            .map_err(|error| ZoneScoreMarkerFactoryError::World(error.into()))?;
        if creatures.binding(usize::from(mule_slot))
            .map_err(|error| ZoneScoreMarkerFactoryError::World(error.into()))?
            != Some(CreatureAnimationBinding::MuleSelector4)
            || mule[6] != 1 || word(mule, 15) != mule_slot as i16
            || mule[0x7a] != 4 || mule[24] > 4 || mule[0x5e] >= 24
        {
            return Err(ZoneScoreMarkerFactoryError::InvalidMule(mule_slot));
        }
        let position = [word(mule, 7), word(mule, 9)];
        let cell = floor_cell(position);
        let raw = reference(self.map(), cell * 14 + 6);
        let head = if raw[2] == 0 {
            None
        } else {
            self.entity(raw)?;
            Some(landing::ResolvedCellHead {
                identity: landing::LandingReference { index: word(&raw, 0), kind: raw[2] },
            })
        };
        let (ground, shade) = self.effect_ground_sample(position)?;
        let effects = self.effects.as_ref().ok_or(ActorWorldError::EffectsNotAdmitted)?;
        let rng_before = self.rng;
        let rng = landing::LandingRng {
            seed: rng_before.seed, ring: rng_before.ring, draw_count: rng_before.draw_count,
        };
        let plan = zone_score_marker::plan_one(
            mule, effects.records(), word(&self.backing, 0x3554), speed_base,
            divisor, owner_argument, ground, shade, head, gravity, rng,
            trig.sine().try_into().unwrap(), trig.cosine().try_into().unwrap(),
        )?;
        let mut staged = self.clone();
        staged.effects.as_mut().unwrap().commit_landing_factory(&plan.factory)
            .map_err(|error| ZoneScoreMarkerFactoryError::World(error.into()))?;
        let effect_slot = plan.factory.slot;
        let identity = [effect_slot as u8, (effect_slot >> 8) as u8, 5];
        if raw[2] != 0 { staged.replace_previous(raw, identity)?; }
        staged.backing[MAP_START + cell * 14 + 6..MAP_START + cell * 14 + 9]
            .copy_from_slice(&identity);
        staged.backing[0x3554..0x3556]
            .copy_from_slice(&plan.factory.pool_count_after.to_le_bytes());
        staged.rng = BlueprintRng {
            seed: plan.rng_after.seed,
            ring: plan.rng_after.ring,
            draw_count: plan.rng_after.draw_count,
        };
        staged.validate_links()?;
        staged.validate_effect_count()?;
        let receipt = ZoneScoreMarkerReceipt {
            mule_slot, effect_slot, angles: plan.angles, draws: plan.draws,
            rng_before, rng_after: staged.rng,
        };
        *self = staged;
        Ok(receipt)
    }
}

//! One bounded source6D500 iteration, including its interleaved code3C factory.
//! Map insertion/resource binding/lifecycle and full-pool replacement are external.
use crate::landing::{self, LandingDraw, LandingError, LandingFactoryPlan, LandingRecord, LandingRng, ResolvedCellHead};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarkerError { InvalidMule, UnsupportedParameters, FullPoolReplacement, Factory(LandingError) }
impl From<LandingError> for MarkerError { fn from(e: LandingError) -> Self { Self::Factory(e) } }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkerPlan {
    pub factory: LandingFactoryPlan,
    pub angles: [u16; 3],
    pub rng_before: LandingRng,
    pub rng_after: LandingRng,
    /// Three6D500 journalF6 draws followed immediately by factory journal229.
    pub draws: [LandingDraw; 4],
}
fn draw(rng: &mut LandingRng) -> LandingDraw {
    let before = *rng;
    rng.seed = rng.seed.wrapping_mul(0xe62d);
    rng.ring = (rng.ring + 1) & 31;
    rng.draw_count = rng.draw_count.wrapping_add(1);
    LandingDraw { ring_slot: before.ring, seed_before: before.seed, seed_after: rng.seed, journal_code: 0xf6 }
}
/// Fixed code3C, variation0 and one iteration. Negative owner-1 inherits the
/// Mule owner. The supplied ground/shade belong to the factory's terrain sample;
/// the marker angle branch uses the Mule's own relative physical height.
/// Invoke again ONLY with the prior factory committed and its resulting RNG.
#[allow(clippy::too_many_arguments)]
pub fn plan_one(mule: &[u8; 124], pool: &[LandingRecord; 200], count: i16,
    speed_base: i32, divisor: i32, owner_argument: i8,
    ground: i32, shade: u8, head: Option<ResolvedCellHead>, gravity: i16,
    rng: LandingRng, sine: &[i32; 2048], cosine: &[i32; 2048],
) -> Result<MarkerPlan, MarkerError> {
    if mule[6] != 1 || mule[122] != 4 || mule[24] > 4 { return Err(MarkerError::InvalidMule); }
    if count == 200 { return Err(MarkerError::FullPoolReplacement); }
    if divisor <= 0 || speed_base < 0 || speed_base.checked_add(divisor).is_none_or(|v| v > 512)
        || !(-1..=4).contains(&owner_argument) { return Err(MarkerError::UnsupportedParameters); }
    if rng.ring >= 32 { return Err(LandingError::InvalidRing(rng.ring).into()); }
    let owner = if owner_argument == -1 { mule[24] } else { owner_argument as u8 };
    let height = i32::from_le_bytes(mule[39..43].try_into().unwrap());
    let relative = height.wrapping_sub(i32::from_le_bytes(mule[17..21].try_into().unwrap()));
    let mut next = rng;
    let d0 = draw(&mut next);
    let speed = (((next.seed >> 8) as i32 % divisor + speed_base) * 64) as u16;
    let d1 = draw(&mut next);
    let second = if relative < 128 { (next.seed >> 8) % 150 } else { ((next.seed >> 8) % 128) * 8 };
    let d2 = draw(&mut next);
    let angles = [((next.seed >> 8) % 128) * 16, second, speed];
    let at = [i16::from_le_bytes(mule[7..9].try_into().unwrap()), i16::from_le_bytes(mule[9..11].try_into().unwrap())];
    let mut p = landing::plan_landing_effect6_factory(pool, count, at, ground, shade, height, angles, head, gravity, next, sine, cosine)?;
    p.factory.after_nonpointer[24] = owner;
    p.factory.after_nonpointer[69] = 0x3c;
    p.factory.animation_index = 60;
    p.draw.journal_code = 0x229;
    Ok(MarkerPlan { factory: p.factory, angles, rng_before: rng, rng_after: p.rng_after, draws: [d0, d1, d2, p.draw] })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mule(relative: i32) -> [u8; 124] { let mut m = [0; 124]; m[6]=1;m[24]=4;m[122]=4;m[39..43].copy_from_slice(&relative.to_le_bytes());m }
    fn plan(m:&[u8;124],pool:&[LandingRecord;200],count:i16,seed:u16)->Result<MarkerPlan,MarkerError>{plan_one(m,pool,count,10,10,-1,0,0,None,1024,LandingRng{seed,ring:31,draw_count:u32::MAX-1},&[0;2048],&[0;2048])}
    #[test] fn relative_height_boundary_and_interleaved_factory_rng() {
        let pool=[[0;81];200];let low=plan(&mule(127),&pool,0,5428).unwrap();let high=plan(&mule(128),&pool,0,5428).unwrap();
        assert_eq!(low.draws.map(|d|d.journal_code),[0xf6,0xf6,0xf6,0x229]);
        assert_eq!(low.draws.map(|d|d.ring_slot),[31,0,1,2]);assert_eq!(low.rng_after.draw_count,2);
        let second=low.draws[1].seed_after>>8;assert_eq!(low.angles[1],second%150);assert_eq!(high.angles[1],(second%128)*8);
        assert_eq!(low.factory.after_nonpointer[24],4);assert_eq!(low.factory.animation_index,60);
        assert_eq!(low.factory.after_nonpointer[29],1+((low.draws[3].seed_after>>8)%3)as u8);
    }
    #[test] fn capacity_and_invalid_domains_do_not_mutate_inputs() {
        let pool=[[0;81];200];let m=mule(0);assert_eq!(plan(&m,&pool,200,1),Err(MarkerError::FullPoolReplacement));
        let mut occupied=pool;for r in &mut occupied{r[6]=5;}
        assert_eq!(plan(&m,&occupied,199,1),Err(MarkerError::Factory(LandingError::NoInactiveSlot)));
        assert_eq!(plan_one(&m,&pool,0,10,0,-1,0,0,None,1024,LandingRng{seed:1,ring:0,draw_count:0},&[0;2048],&[0;2048]),Err(MarkerError::UnsupportedParameters));
        assert_eq!(pool,[[0;81];200]);
    }
}

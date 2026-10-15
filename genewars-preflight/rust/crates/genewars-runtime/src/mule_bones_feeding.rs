//! Bounded source679CE/67A22..67AC8 state4 Bones feeding/query prefix.
//! This is not a complete callback: the caller proves postphysics distance
//! <=4096, exact Bones resource binding, and that the subsequent source query
//! has no action. Physics, query evaluation, animation and timer tail are external.
use crate::blueprint::BlueprintRng;
use crate::creature_world::{CreatureRecord, CreatureRngDraw, CreatureRngPurpose};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BonesFeedingError {
    Identity,
    Current,
    Target,
    References,
    Timing,
    Reserve,
    Rng,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BonesFeedingPlan {
    pub current_after: CreatureRecord,
    pub target_after: CreatureRecord,
    pub rng_after: BlueprintRng,
    pub draws: [CreatureRngDraw; 2],
    /// Source67A94 virtual44: code12h, EBX2, ECX0, stack127/100.
    /// Dispatch this before evaluating the neighborhood query.
    pub pending_audio: u8,
}

fn word(record: &CreatureRecord, offset: usize) -> i16 {
    i16::from_le_bytes([record[offset], record[offset + 1]])
}

/// Source67A41 removes2 from the Bones reserve, then5C730(EDX2,BL2)
/// applies Mule coefficient100: gain4. Only mature77==255/no-full nutrition
/// is admitted, so energy, hunger flags, references and state stay unchanged.
/// Two query-order draws follow. Caller must reject any query action it cannot
/// faithfully execute; a successful prefix alone is not permission to skip it.
pub fn plan_feed(
    current: &CreatureRecord,
    target: &CreatureRecord,
    rng_before: BlueprintRng,
) -> Result<BonesFeedingPlan, BonesFeedingError> {
    let id = word(current, 15);
    let target_id = word(target, 15);
    if !(0..300).contains(&id) || !(0..300).contains(&target_id) || id == target_id {
        return Err(BonesFeedingError::Identity);
    }
    if current[6] != 1 || current[24] != 4 || current[0x7a] != 4
        || current[0x5e] != 4 || current[0x77] != 255 || current[0x76] > 90
        || current[0x78] != 0 || current[0x60] != 8 || current[0x61] != 0x40
        || word(current, 0x6a) <= 0 || word(current, 0x66) <= 0
    {
        return Err(BonesFeedingError::Current);
    }
    if target[6] != 1 || target[24] != 4 || target[0x7a] != 4
        || !matches!(target[0x5e], 26 | 27) || word(target, 0x68) < 2
    {
        return Err(BonesFeedingError::Target);
    }
    if current[0x4f] != 0 || current[0x5d] != 0
        || current[0x52] != 1 || current[0x57] != 1
        || word(current, 0x50) != target_id || word(current, 0x55) != target_id
    {
        return Err(BonesFeedingError::References);
    }
    if word(current, 0x6c) < 0 || word(current, 0x6c) & 15 != 0 {
        return Err(BonesFeedingError::Timing);
    }
    let reserve = word(current, 0x68);
    let maximum = word(current, 0x62);
    let gained = reserve.checked_add(4).ok_or(BonesFeedingError::Reserve)?;
    if reserve < 0 || maximum <= 0 || gained > maximum {
        return Err(BonesFeedingError::Reserve);
    }
    if rng_before.ring >= 32 {
        return Err(BonesFeedingError::Rng);
    }
    let mut current_after = *current;
    let mut target_after = *target;
    current_after[0x68..0x6a].copy_from_slice(&gained.to_le_bytes());
    target_after[0x68..0x6a].copy_from_slice(&(word(target, 0x68) - 2).to_le_bytes());
    let mut rng_after = rng_before;
    let draws = [CreatureRngPurpose::NeighborhoodX, CreatureRngPurpose::NeighborhoodY].map(|purpose| {
        let seed_before = rng_after.seed;
        rng_after.seed = rng_after.seed.wrapping_mul(0xe62d);
        rng_after.ring = rng_after.ring.wrapping_add(1) & 31;
        rng_after.draw_count = rng_after.draw_count.wrapping_add(1);
        CreatureRngDraw { purpose, seed_before, seed_after: rng_after.seed, value: (rng_after.seed >> 8) as u8 }
    });
    Ok(BonesFeedingPlan { current_after, target_after, rng_after, draws, pending_audio: 0x12 })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (CreatureRecord, CreatureRecord, BlueprintRng) {
        let mut a = [0xa5; 124];
        let mut b = [0x5a; 124];
        for (record, id) in [(&mut a, 0i16), (&mut b, 7i16)] {
            record[6] = 1; record[24] = 4; record[0x7a] = 4;
            record[15..17].copy_from_slice(&id.to_le_bytes());
        }
        a[0x5e] = 4; a[0x77] = 255; a[0x76] = 50; a[0x78] = 0;
        a[0x60] = 8; a[0x61] = 0x40; a[0x4f] = 0; a[0x5d] = 0;
        a[0x50..0x53].copy_from_slice(&[7, 0, 1]);
        a[0x55..0x58].copy_from_slice(&[7, 0, 1]);
        for (at, value) in [(0x62, 143i16), (0x66, 7121), (0x68, 0), (0x6a, 16), (0x6c, 32752)] {
            a[at..at + 2].copy_from_slice(&value.to_le_bytes());
        }
        b[0x5e] = 26; b[0x68..0x6a].copy_from_slice(&34i16.to_le_bytes());
        (a, b, BlueprintRng { seed: 43097, ring: 16, draw_count: 43449 })
    }
    #[test]
    fn source_gate_consumes_two_reserve_and_preserves_opaque_state() {
        let (a, b, rng) = fixture();
        let p = plan_feed(&a, &b, rng).unwrap();
        assert_eq!(word(&p.current_after, 0x68), 4);
        assert_eq!(word(&p.target_after, 0x68), 32);
        assert_eq!(p.rng_after, BlueprintRng { seed: 9217, ring: 18, draw_count: 43451 });
        assert_eq!(p.pending_audio, 0x12);
        for i in 0..124 {
            if !matches!(i, 0x68 | 0x69) {
                assert_eq!(p.current_after[i], a[i], "current{i:x}");
                assert_eq!(p.target_after[i], b[i], "target{i:x}");
            }
        }
    }
    #[test]
    fn exact_capacity_and_empty_bones_boundary_keep_rng_wrap() {
        let (mut a, mut b, _) = fixture();
        a[0x68..0x6a].copy_from_slice(&139i16.to_le_bytes());
        b[0x5e] = 27; b[0x68..0x6a].copy_from_slice(&2i16.to_le_bytes());
        let p = plan_feed(&a, &b, BlueprintRng { seed: 1, ring: 31, draw_count: u32::MAX }).unwrap();
        assert_eq!(word(&p.current_after, 0x68), 143);
        assert_eq!(p.current_after[0x61], 0x40); // Equal max is not source strict-full.
        assert_eq!(word(&p.target_after, 0x68), 0);
        assert_eq!(p.rng_after.ring, 1); assert_eq!(p.rng_after.draw_count, 1);
        assert_eq!(p.draws[0].seed_after, p.draws[1].seed_before);
    }
    #[test]
    fn unsupported_full_nutrition_signed_reserves_and_references_reject() {
        let (a, b, rng) = fixture();
        for value in [-1i16, 140, 32767] {
            let mut bad = a; bad[0x68..0x6a].copy_from_slice(&value.to_le_bytes());
            assert_eq!(plan_feed(&bad, &b, rng), Err(BonesFeedingError::Reserve));
        }
        for value in [-1i16, 0, 1] {
            let mut bad = b; bad[0x68..0x6a].copy_from_slice(&value.to_le_bytes());
            assert_eq!(plan_feed(&a, &bad, rng), Err(BonesFeedingError::Target));
        }
        for (at, value) in [(0x5e, 25), (6, 2), (24, 0), (0x7a, 3)] {
            let mut bad = b; bad[at] = value; assert!(plan_feed(&a, &bad, rng).is_err());
        }
        for (at, value) in [(0x50, 6), (0x52, 0), (0x57, 2), (0x4f, 1), (0x5d, 1), (0x77, 254), (0x6c, 1)] {
            let mut bad = a; bad[at] = value; assert!(plan_feed(&bad, &b, rng).is_err());
        }
        assert_eq!(plan_feed(&a, &b, BlueprintRng { ring: 32, ..rng }), Err(BonesFeedingError::Rng));
    }
}

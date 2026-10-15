//! Bounded owned-Mule nonlethal self-water damage, source5B9F0.
//! Caller proves6E919..6E944: phase3, water/terrainclass0, physicalheight0,
//! terrain mask53bit1, damage5 and attacker==self. No predicate is inferred here.
//! Source5BB42..5BB50 nulls attacker and marks self;5BC1A calls5CC60 before
//!5C078..5C087 health subtraction. Self/nonflight/nonlethal tails skip retaliation,
//! audio, blood factories and5CDB0 death. This packet consumes no RNG.
//!
//! Static Mule params1045FC+4*45=1046B0: defensebyte+5=6;
//! class bytes104A6C are04,04(nonflight). Mappedobject3 SHA256:
//! aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87.
//! Saved listings: construction-command/instructions-0005b9f0.txt and
//! mission-array-refs/instructions-0005cc60.txt under private/ghidra.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { Identity, State, References, NonlethalDomain, BloodContinuation }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub after: [u8; 124],
    pub effective_damage: i16,
}
fn word(record: &[u8; 124], offset: usize) -> i16 {
    i16::from_le_bytes([record[offset], record[offset + 1]])
}

/// Apply only the admitted postphysics/pre-AI5B9F0 self-damage call. Stale
/// reference indices with null kinds remain opaque, as do intrusive headers.
/// Health>5 deliberately excludes all lethal outcomes even before defense.
/// This does not create the preceding water particle, reveal or choose AI.
pub fn plan(before: &[u8; 124]) -> Result<Plan, Error> {
    if before[6] != 1 || before[24] >= 4 || before[122] != 4
        || !(0..300).contains(&word(before, 15))
    {
        return Err(Error::Identity);
    }
    if before[0x5e] != 8 || !matches!(before[0x60], 8 | 0x88)
        || before[0x61] != 0 || before[0x78] != 0 || before[0x76] > 90
    {
        return Err(Error::State);
    }
    if before[0x4f] != 0 || before[0x52] != 0 || before[0x57] != 0 || before[0x5d] != 0 {
        return Err(Error::References);
    }
    let health = word(before, 0x6a);
    if health <= 5 { return Err(Error::NonlethalDomain); }
    //5BB98..5BBD3:50+trunc(50*min(skill76,100)/100), then defense6*percent/100.
    let skill_percent = 50 + 50 * i32::from(before[0x76]) / 100;
    let defense = 6 * skill_percent / 100;
    let effective_damage = (5 - defense).max(1) as i16;
    if effective_damage > 4 { return Err(Error::BloodContinuation); }
    let mut after = *before;
    //5C920 is RET with attachmentbit2clear. State8 dispatches directly5CCA0.
    after[0x15] &= 0xdf;
    after[0x33..0x37].fill(0);
    let flags = (word(before, 0x60) as u16) & 0xd71a;
    after[0x60..0x62].copy_from_slice(&flags.to_le_bytes());
    after[0x5d] = 0;
    after[0x57] = 0;
    after[0x6c..0x6e].fill(0);
    after[0x6a..0x6c].copy_from_slice(&(health - effective_damage).to_le_bytes());
    //5C098 selfFlag skipsstate20;5C3E2 skipsaudio/retaliation;5C51A skipsblood;
    //5C6C7 positivehealth skipsdeath,5C721 returnsfalse. No further writes.
    Ok(Plan { after, effective_damage })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> [u8; 124] {
        let mut a = [0xa5; 124];
        a[6] = 1; a[24] = 0; a[122] = 4;
        a[15..17].copy_from_slice(&1i16.to_le_bytes());
        a[0x5e] = 8; a[0x60] = 0x88; a[0x61] = 0;
        a[0x4f] = 0; a[0x52] = 0; a[0x57] = 0; a[0x5d] = 0;
        a[0x78] = 0; a[0x76] = 37;
        a[0x6a..0x6c].copy_from_slice(&39i16.to_le_bytes());
        a[0x6c..0x6e].copy_from_slice(&32738i16.to_le_bytes());
        a
    }
    #[test]
    fn defense_boundaries_reset_only_source_fields_without_self_retaliation() {
        for (skill, expected_damage) in [(0, 2i16), (37, 1), (90, 1)] {
            for low in [8, 0x88] {
                let mut before = fixture(); before[0x76] = skill; before[0x60] = low;
                let original = before;
                let p = plan(&before).unwrap();
                assert_eq!(p.effective_damage, expected_damage);
                assert_eq!(word(&p.after, 0x6a), 39 - expected_damage);
                assert_eq!(word(&p.after, 0x6c), 0);
                assert_eq!(&p.after[0x60..0x62], &[8, 0]);
                assert_eq!(p.after[0x5e], 8);
                assert_eq!(before, original);
                for i in 0..124 {
                    if !matches!(i, 0x15 | 0x33..=0x36 | 0x57 | 0x5d | 0x60 | 0x61 | 0x6a..=0x6d) {
                        assert_eq!(p.after[i], before[i], "opaque{i:x}");
                    }
                }
                // Includes native pointer gap, list headers and stale null indices.
                assert_eq!(&p.after[0..6], &before[0..6]);
                assert_eq!(&p.after[31..39], &before[31..39]);
                assert_eq!(&p.after[0x4d..0x4f], &[0xa5; 2]);
                assert_eq!(&p.after[0x50..0x52], &[0xa5; 2]);
            }
        }
    }
    #[test]
    fn lethal_and_other_states_references_attachment_and_commands_stay_unsupported() {
        let base = fixture();
        for health in [i16::MIN, 0, 1, 5] {
            let mut a = base; a[0x6a..0x6c].copy_from_slice(&health.to_le_bytes());
            let original = a;
            assert_eq!(plan(&a), Err(Error::NonlethalDomain)); assert_eq!(a, original);
        }
        for (offset, value) in [(6, 3), (24, 4), (122, 3), (0x5e, 15), (0x5e, 18),
            (0x60, 0x8a), (0x61, 0x40), (0x4f, 4), (0x52, 1), (0x57, 1), (0x5d, 5),
            (0x78, 1), (0x76, 91)] {
            let mut a = base; a[offset] = value; let original = a;
            assert!(plan(&a).is_err(), "offset{offset:x}"); assert_eq!(a, original);
        }
        let mut minimum = base; minimum[0x76] = 0;
        minimum[0x6a..0x6c].copy_from_slice(&6i16.to_le_bytes());
        assert_eq!(word(&plan(&minimum).unwrap().after, 0x6a), 4);
    }
}

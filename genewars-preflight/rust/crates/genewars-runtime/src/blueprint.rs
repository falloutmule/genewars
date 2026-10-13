//! CD candidate 0x7F580/0x7FAD0 blueprint generation with explicit live inputs.
//!
//! Names, prior metadata, reusable stack bytes and RNG state are caller inputs.
//! The two uninitialized scratch bytes remain opaque. No initial mission seed,
//! native resource pointer, or original startup timeline is inferred here.

pub const RECORD_BYTES: usize = 36;
pub const RECORD_COUNT: usize = 32;
pub type BlueprintRecords = [[u8; RECORD_BYTES]; RECORD_COUNT];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlueprintError {
    NameCount(usize),
    InvalidName(usize),
    DuplicateName { first: usize, second: usize },
    InvalidRing(u8),
    NamePoolExhausted(u8),
}
impl core::fmt::Display for BlueprintError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for BlueprintError {}

/// Four contiguous style pools of 32 names, admitted separately from TEXT.
#[derive(Debug, Clone)]
pub struct BlueprintNames { names: [[u8; 20]; 128] }
impl BlueprintNames {
    /// Names exclude their NUL. The safe port requires unique, nonempty names
    /// of at most19 bytes; each admitted name is padded to the original20 bytes.
    pub fn new(names: &[&[u8]]) -> Result<Self, BlueprintError> {
        if names.len() != 128 { return Err(BlueprintError::NameCount(names.len())); }
        let mut result = Self { names: [[0; 20]; 128] };
        for (index, name) in names.iter().enumerate() {
            if name.is_empty() || name.len() > 19 || name.contains(&0) {
                return Err(BlueprintError::InvalidName(index));
            }
            result.names[index][..name.len()].copy_from_slice(name);
            if let Some(first) = result.names[..index].iter().position(|n| n == &result.names[index]) {
                return Err(BlueprintError::DuplicateName { first, second: index });
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlueprintRng {
    pub seed: u16,
    pub ring: u8,
    pub draw_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawPurpose {
    CategoryStyleStart { category: u8 },
    Stat19 { slot: u8, style: u8 },
    Stat17 { slot: u8 },
    NameRank { slot: u8 },
}

/// The original also records source site/time metadata in a32-entry debug ring.
/// This trace exposes every draw and ring slot; it does not fabricate the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlueprintDraw {
    pub purpose: DrawPurpose,
    pub ring_slot: u8,
    pub seed_before: u16,
    pub seed_after: u16,
    pub high_byte: u8,
    pub draw_count_after: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintGeneration {
    pub records: BlueprintRecords,
    pub scratch_after: [u8; RECORD_BYTES],
    pub metadata_count: u8,
    pub rng_after: BlueprintRng,
    pub draws: Vec<BlueprintDraw>,
}

fn draw(rng: &mut BlueprintRng, purpose: DrawPurpose, log: &mut Vec<BlueprintDraw>) -> u8 {
    let seed_before = rng.seed;
    let ring_slot = rng.ring;
    rng.seed = rng.seed.wrapping_mul(0xE62D);
    rng.ring = (rng.ring + 1) & 31;
    rng.draw_count = rng.draw_count.wrapping_add(1);
    let high_byte = (rng.seed >> 8) as u8;
    log.push(BlueprintDraw { purpose, ring_slot, seed_before, seed_after: rng.seed,
        high_byte, draw_count_after: rng.draw_count });
    high_byte
}

/// Generate20 records in category order while preserving unused-record bytes.
/// Errors return no partial after-image and do not mutate any caller input.
/// The caller decides whether the mission-entry regeneration gate applies.
pub fn generate_blueprints(
    prior: &BlueprintRecords, scratch: [u8; RECORD_BYTES], names: &BlueprintNames,
    live_rng: BlueprintRng, campaign: bool,
) -> Result<BlueprintGeneration, BlueprintError> {
    if live_rng.ring >= 32 { return Err(BlueprintError::InvalidRing(live_rng.ring)); }
    let mut records = *prior;
    for record in &mut records { record[0] = 0; }
    let mut scratch = scratch;
    let mut rng = live_rng;
    let mut draws = Vec::with_capacity(64);
    let mut count = 0_u8;
    for category in 0..4 {
        let start = draw(&mut rng, DrawPurpose::CategoryStyleStart { category }, &mut draws) % 100;
        for ordinal in 0..5_u8 {
            let style = (start + ordinal) % 4;
            let difficulty = if campaign { 0 } else { ordinal };
            let slot = count;
            let stat19 = 5 * difficulty + [15,40,10,30][style as usize]
                + draw(&mut rng, DrawPurpose::Stat19 { slot, style }, &mut draws) % [25,10,15,20][style as usize];
            let stat17 = 10 * difficulty + 25
                + draw(&mut rng, DrawPurpose::Stat17 { slot }, &mut draws) % 10;
            scratch[0x14] = 0;
            scratch[0x15] = style;
            scratch[0x16] = category;
            scratch[0x17] = stat17.min([100,75,75,75][style as usize]);
            scratch[0x19] = stat19.min([70,100,65,90][style as usize]);
            scratch[0x1A] = scratch[0x19];
            scratch[0x1C..0x24].fill(0);
            // 0x18 and0x1B are deliberately not initialized by the source.
            let used = records.iter().filter(|r| r[0] != 0 && r[0x15] == style).count();
            let remaining = 32_usize.checked_sub(used).filter(|&n| n != 0)
                .ok_or(BlueprintError::NamePoolExhausted(style))?;
            let mut rank = usize::from(draw(&mut rng, DrawPurpose::NameRank { slot }, &mut draws)) % remaining;
            let pool = &names.names[style as usize * 32..(style as usize + 1) * 32];
            let mut selected = None;
            for name in pool {
                if records.iter().any(|record| record[0] != 0 && &record[..20] == name) { continue; }
                if rank == 0 { selected = Some(name); break; }
                rank -= 1;
            }
            let name = selected.ok_or(BlueprintError::NamePoolExhausted(style))?;
            scratch[..20].copy_from_slice(name);
            records[usize::from(count)] = scratch;
            count += 1;
        }
    }
    Ok(BlueprintGeneration { records, scratch_after: scratch, metadata_count: count, rng_after: rng, draws })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn names() -> BlueprintNames {
        let names: Vec<_> = (0..128).map(|i| format!("test{i:03}").into_bytes()).collect();
        BlueprintNames::new(&names.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap()
    }
    #[test]
    fn generation_preserves_opaque_bytes_and_unused_record_payloads() {
        let prior = core::array::from_fn(|i| [i as u8 + 1;36]);
        let scratch = [0xA5;36];
        let result = generate_blueprints(&prior,scratch,&names(),BlueprintRng{seed:0,ring:31,draw_count:u32::MAX},true).unwrap();
        assert_eq!(result.metadata_count,20);
        assert_eq!(result.draws.len(),64);
        assert_eq!(result.rng_after,BlueprintRng{seed:0,ring:31,draw_count:63});
        for (i,record) in result.records.iter().enumerate() {
            if i < 20 {
                assert_eq!((record[0x18],record[0x1B]),(0xA5,0xA5));
                assert_eq!(record[0x16],(i/5) as u8);
                assert_eq!(record[0x15],(i%5%4) as u8);
                assert_eq!(&record[0x1C..0x24], &[0;8]);
                assert_eq!(record[0x17],25);
                assert_eq!(record[0x19],[15,40,10,30][i%5%4]);
                assert_eq!(record[0x1A],record[0x19]);
            } else { assert_eq!(record[0],0); assert_eq!(&record[1..],&prior[i][1..]); }
        }
        for i in 0..20 { for j in i+1..20 { assert_ne!(&result.records[i][..20], &result.records[j][..20]); } }
    }
    #[test]
    fn difficulty_is_campaign_gated_and_rng_count_is_fixed() {
        let rng=BlueprintRng{seed:5428,ring:7,draw_count:123};
        let a=generate_blueprints(&[[0;36];32],[0;36],&names(),rng,true).unwrap();
        let b=generate_blueprints(&[[0;36];32],[0;36],&names(),rng,false).unwrap();
        assert_eq!(a.rng_after,b.rng_after);
        assert_eq!(a.draws,b.draws);
        for i in 0..20 {
            assert_eq!(&a.records[i][..0x17],&b.records[i][..0x17]);
            assert_eq!(b.records[i][0x17],a.records[i][0x17]+10*(i as u8%5));
            assert_eq!(b.records[i][0x19],a.records[i][0x19]+5*(i as u8%5));
        }
    }
    #[test]
    fn malformed_name_banks_and_rng_are_rejected() {
        assert!(matches!(BlueprintNames::new(&[]),Err(BlueprintError::NameCount(0))));
        assert!(matches!(BlueprintNames::new(&[&b""[..];128]),Err(BlueprintError::InvalidName(0))));
        assert!(matches!(BlueprintNames::new(&[&b"a"[..];128]),Err(BlueprintError::DuplicateName{first:0,second:1})));
        assert!(matches!(generate_blueprints(&[[0;36];32],[0;36],&names(),BlueprintRng{seed:1,ring:32,draw_count:0},true),Err(BlueprintError::InvalidRing(32))));
    }
}

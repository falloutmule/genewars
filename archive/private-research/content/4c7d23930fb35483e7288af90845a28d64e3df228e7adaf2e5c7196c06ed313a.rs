//! Borrowed, source-only access to the admitted CD `CONQ001.DAT` layout.
//!
//! Admission is deliberately narrow: exact 343,958-byte files with the
//! `0x55336` fast-path tag and the proved `0x6BE` extension tag. Known
//! 375,830-byte same-name resources belong to another source variant and are
//! reported as unsupported variants, not as corrupt files. Records remain
//! borrowed byte slices; this module does not create runtime objects or
//! rebase pointers.
use core::fmt;

pub const CONQ_MISSION_LAYOUT_VERSION: &str = "cd-conq-fast-0x55336-v1";
pub const CONQ_MISSION_BYTES: usize = 343_958;
pub const CONQ_MISSION_FAST_TAG: u32 = 0x55336;
pub const CONQ_MISSION_PREFIX_BYTES: usize = 0x3B55E;
pub const CONQ_TARGET_TABLE_OFFSET: usize = 0x1C7E;
pub const CONQ_TARGET_COUNT: usize = 4;
pub const CONQ_TARGET_STRIDE: usize = 4;
pub const CONQ_FAMILY1_FILE_OFFSET: usize = 277_838;
pub const CONQ_FAMILY1_COUNT: usize = 512;
pub const CONQ_FAMILY1_PACKED_STRIDE: usize = 41;
pub const CONQ_TAIL_GLOBALS_OFFSET: usize = 336_998;
pub const CONQ_TAIL_GLOBALS_BYTES: usize = 52;
pub const CONQ_VIEWPORT_SNAPSHOT_OFFSET: usize = 337_050;
pub const CONQ_VIEWPORT_SNAPSHOT_BYTES: usize = 4;
pub const CONQ_EXTENSION_TAG_OFFSET: usize = 337_054;
pub const CONQ_EXTENSION_TAG: u32 = 0x6BE;
pub const CONQ_EXTENSION_PAYLOAD_OFFSET: usize = 337_058;
pub const CONQ_EXTENSION_PAYLOAD_BYTES: usize = 6_900;

const GROUP_LAYOUTS: [MissionRecordGroupLayout; 8] = [
    MissionRecordGroupLayout { family: 0, file_offset: 243_038, count: 300, packed_stride: 116 },
    MissionRecordGroupLayout { family: 1, file_offset: 277_838, count: 512, packed_stride: 41 },
    MissionRecordGroupLayout { family: 2, file_offset: 298_830, count: 20, packed_stride: 140 },
    MissionRecordGroupLayout { family: 3, file_offset: 301_630, count: 256, packed_stride: 59 },
    MissionRecordGroupLayout { family: 4, file_offset: 316_734, count: 200, packed_stride: 73 },
    MissionRecordGroupLayout { family: 5, file_offset: 331_334, count: 32, packed_stride: 41 },
    MissionRecordGroupLayout { family: 6, file_offset: 332_646, count: 32, packed_stride: 89 },
    MissionRecordGroupLayout { family: 7, file_offset: 335_494, count: 16, packed_stride: 94 },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissionSourceError {
    UnsupportedSourceVariantLength { expected: usize, actual: usize },
    UnsupportedFastTag { actual: u32 },
    UnsupportedExtensionTag { actual: u32 },
    TargetIndexOutOfBounds { index: usize, count: usize },
    GroupIndexOutOfBounds { family: usize, count: usize },
    RecordIndexOutOfBounds { family: usize, index: usize, count: usize },
    InvalidFixedBounds,
}

impl fmt::Display for MissionSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for MissionSourceError {}

/// Fixed-size source extent for one serialized record group. Numeric family
/// values preserve the instruction/receipt ordering without assigning labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionRecordGroupLayout {
    pub family: u8,
    pub file_offset: usize,
    pub count: usize,
    pub packed_stride: usize,
}

/// Borrowed view of one packed group; its records have no claimed live-state
/// interpretation. Byte `+6`, when requested, is exposed only as a raw byte.
#[derive(Debug, Clone, Copy)]
pub struct MissionRecordGroup<'a> {
    layout: MissionRecordGroupLayout,
    bytes: &'a [u8],
}

/// Raw source fields read by the traced family-1 sprite consumer. These
/// accessors expose source bytes/words only: they do not apply initialization,
/// world projection, visibility, animation timing, drawing, or runtime state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MissionFamily1SpriteFields {
    pub record_index: usize,
    pub signed_position_words: [i16; 2],
    /// Source byte at family-1 record offset +0x15.
    pub flags_at_15: u8,
    /// Unsigned source byte at family-1 record offset +0x17.
    pub shade_at_17: u8,
    pub animation_selector: u8,
    pub frame_selector: u8,
    /// Raw source byte at +0x23, retained for compatibility with earlier callers.
    pub resource_slot: u8,
    /// The same source byte at +0x23 interpreted as the signed selector read by
    /// the traced initializer.
    pub signed_resource_selector: i8,
    /// Source byte at +0x26, which the native copy places at +0x2E.
    pub native_flags_at_2e: u8,
}

impl<'a> MissionRecordGroup<'a> {
    pub fn layout(&self) -> MissionRecordGroupLayout { self.layout }
    pub fn bytes(&self) -> &'a [u8] { self.bytes }

    pub fn record(&self, index: usize) -> Result<&'a [u8], MissionSourceError> {
        if index >= self.layout.count {
            return Err(MissionSourceError::RecordIndexOutOfBounds {
                family: usize::from(self.layout.family), index, count: self.layout.count,
            });
        }
        let start = index.checked_mul(self.layout.packed_stride)
            .ok_or(MissionSourceError::InvalidFixedBounds)?;
        let end = start.checked_add(self.layout.packed_stride)
            .ok_or(MissionSourceError::InvalidFixedBounds)?;
        self.bytes.get(start..end).ok_or(MissionSourceError::InvalidFixedBounds)
    }

    pub fn raw_byte_6(&self, index: usize) -> Result<u8, MissionSourceError> {
        self.record(index)?.get(6).copied().ok_or(MissionSourceError::InvalidFixedBounds)
    }
}

/// Exact-size borrowed view of the admitted CD serialized mission file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConqMissionSource<'a> {
    bytes: &'a [u8],
}

impl<'a> ConqMissionSource<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, MissionSourceError> {
        if bytes.len() != CONQ_MISSION_BYTES {
            return Err(MissionSourceError::UnsupportedSourceVariantLength {
                expected: CONQ_MISSION_BYTES, actual: bytes.len(),
            });
        }
        let fast_tag = read_u32(bytes, 0);
        if fast_tag != CONQ_MISSION_FAST_TAG {
            return Err(MissionSourceError::UnsupportedFastTag { actual: fast_tag });
        }
        let extension_tag = read_u32(bytes, CONQ_EXTENSION_TAG_OFFSET);
        if extension_tag != CONQ_EXTENSION_TAG {
            return Err(MissionSourceError::UnsupportedExtensionTag { actual: extension_tag });
        }
        if CONQ_TARGET_TABLE_OFFSET + CONQ_TARGET_COUNT * CONQ_TARGET_STRIDE > CONQ_MISSION_PREFIX_BYTES
            || !fixed_groups_cover_tail()
            || CONQ_TAIL_GLOBALS_OFFSET + CONQ_TAIL_GLOBALS_BYTES != CONQ_VIEWPORT_SNAPSHOT_OFFSET
            || CONQ_VIEWPORT_SNAPSHOT_OFFSET + CONQ_VIEWPORT_SNAPSHOT_BYTES != CONQ_EXTENSION_TAG_OFFSET
            || CONQ_EXTENSION_PAYLOAD_OFFSET + CONQ_EXTENSION_PAYLOAD_BYTES != CONQ_MISSION_BYTES
        {
            return Err(MissionSourceError::InvalidFixedBounds);
        }
        Ok(Self { bytes })
    }

    pub fn layout_version(&self) -> &'static str { CONQ_MISSION_LAYOUT_VERSION }
    pub fn bytes(&self) -> &'a [u8] { self.bytes }
    pub fn copied_prefix(&self) -> &'a [u8] { &self.bytes[..CONQ_MISSION_PREFIX_BYTES] }

    /// Raw 16-bit coordinate words in the four-entry start-target table.
    /// Signed-coordinate interpretation is left to the caller that applies
    /// the separate camera helper contract.
    pub fn target_words(&self, index: usize) -> Result<[u16; 2], MissionSourceError> {
        if index >= CONQ_TARGET_COUNT {
            return Err(MissionSourceError::TargetIndexOutOfBounds { index, count: CONQ_TARGET_COUNT });
        }
        let offset = CONQ_TARGET_TABLE_OFFSET + index * CONQ_TARGET_STRIDE;
        Ok([read_u16(self.bytes, offset), read_u16(self.bytes, offset + 2)])
    }

    pub fn groups(&self) -> [MissionRecordGroup<'a>; 8] {
        GROUP_LAYOUTS.map(|layout| {
            let length = layout.count * layout.packed_stride;
            MissionRecordGroup { layout, bytes: &self.bytes[layout.file_offset..layout.file_offset + length] }
        })
    }

    /// Read only family-1 source offsets pinned by the traced consumer. This
    /// returns serialized source values and does not construct or initialize
    /// the wider native runtime record.
    pub fn family1_sprite_fields(&self, index: usize) -> Result<MissionFamily1SpriteFields, MissionSourceError> {
        let record = self.group(1)?.record(index)?;
        Ok(MissionFamily1SpriteFields {
            record_index: index,
            signed_position_words: [
                i16::from_le_bytes([record[7], record[8]]),
                i16::from_le_bytes([record[9], record[10]]),
            ],
            flags_at_15: record[0x15],
            shade_at_17: record[0x17],
            animation_selector: record[0x19],
            frame_selector: record[0x1B],
            resource_slot: record[0x23],
            signed_resource_selector: record[0x23] as i8,
            native_flags_at_2e: record[0x26],
        })
    }

    pub fn group(&self, family: usize) -> Result<MissionRecordGroup<'a>, MissionSourceError> {
        let layout = *GROUP_LAYOUTS.get(family).ok_or(MissionSourceError::GroupIndexOutOfBounds {
            family, count: GROUP_LAYOUTS.len(),
        })?;
        let length = layout.count.checked_mul(layout.packed_stride).ok_or(MissionSourceError::InvalidFixedBounds)?;
        let end = layout.file_offset.checked_add(length).ok_or(MissionSourceError::InvalidFixedBounds)?;
        let bytes = self.bytes.get(layout.file_offset..end).ok_or(MissionSourceError::InvalidFixedBounds)?;
        Ok(MissionRecordGroup { layout, bytes })
    }

    pub fn tail_globals(&self) -> &'a [u8; CONQ_TAIL_GLOBALS_BYTES] {
        self.bytes[CONQ_TAIL_GLOBALS_OFFSET..CONQ_VIEWPORT_SNAPSHOT_OFFSET]
            .try_into().expect("exact admitted globals range")
    }

    pub fn flags_byte(&self) -> u8 { self.tail_globals()[0] }
    pub fn active_count_byte(&self) -> u8 { self.tail_globals()[28] }

    pub fn viewport_snapshot(&self) -> &'a [u8; CONQ_VIEWPORT_SNAPSHOT_BYTES] {
        self.bytes[CONQ_VIEWPORT_SNAPSHOT_OFFSET..CONQ_EXTENSION_TAG_OFFSET]
            .try_into().expect("exact admitted viewport range")
    }

    pub fn extension_tag(&self) -> u32 { read_u32(self.bytes, CONQ_EXTENSION_TAG_OFFSET) }
    pub fn extension_payload(&self) -> &'a [u8] {
        &self.bytes[CONQ_EXTENSION_PAYLOAD_OFFSET..CONQ_EXTENSION_PAYLOAD_OFFSET + CONQ_EXTENSION_PAYLOAD_BYTES]
    }
}

fn fixed_groups_cover_tail() -> bool {
    let mut expected_offset = CONQ_MISSION_PREFIX_BYTES;
    for (family, layout) in GROUP_LAYOUTS.iter().enumerate() {
        if usize::from(layout.family) != family || layout.file_offset != expected_offset { return false; }
        let Some(length) = layout.count.checked_mul(layout.packed_stride) else { return false; };
        let Some(end) = layout.file_offset.checked_add(length) else { return false; };
        expected_offset = end;
    }
    expected_offset == CONQ_TAIL_GLOBALS_OFFSET
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic() -> Vec<u8> {
        let mut bytes = vec![0u8; CONQ_MISSION_BYTES];
        bytes[..4].copy_from_slice(&CONQ_MISSION_FAST_TAG.to_le_bytes());
        bytes[CONQ_TARGET_TABLE_OFFSET..CONQ_TARGET_TABLE_OFFSET + 4]
            .copy_from_slice(&[0x00, 0x51, 0x00, 0x35]);
        bytes[CONQ_TAIL_GLOBALS_OFFSET] = 56;
        bytes[CONQ_TAIL_GLOBALS_OFFSET + 28] = 1;
        bytes[CONQ_EXTENSION_TAG_OFFSET..CONQ_EXTENSION_TAG_OFFSET + 4]
            .copy_from_slice(&CONQ_EXTENSION_TAG.to_le_bytes());
        let record = CONQ_FAMILY1_FILE_OFFSET;
        bytes[record + 7..record + 9].copy_from_slice(&(-123i16).to_le_bytes());
        bytes[record + 9..record + 11].copy_from_slice(&(456i16).to_le_bytes());
        bytes[record + 0x15] = 0x92;
        bytes[record + 0x17] = 0xff;
        bytes[record + 0x19] = 8;
        bytes[record + 0x1B] = 9;
        bytes[record + 0x23] = 0x80;
        bytes[record + 0x26] = 0xa5;
        bytes
    }

    #[test]
    fn admitted_layout_exposes_only_borrowed_bounded_slices() {
        let bytes = synthetic();
        let source = ConqMissionSource::parse(&bytes).unwrap();
        assert_eq!(source.layout_version(), CONQ_MISSION_LAYOUT_VERSION);
        assert_eq!(source.copied_prefix().len(), 243_038);
        assert_eq!(source.target_words(0), Ok([0x5100, 0x3500]));
        assert_eq!(source.flags_byte(), 56);
        assert_eq!(source.active_count_byte(), 1);
        assert_eq!(source.extension_tag(), 0x6BE);
        assert_eq!(source.extension_payload().len(), 6_900);
        let groups = source.groups();
        assert_eq!(groups.len(), 8);
        assert_eq!(groups[0].layout(), GROUP_LAYOUTS[0]);
        assert_eq!(groups[0].record(299).unwrap().len(), 116);
        assert_eq!(groups[0].raw_byte_6(299), Ok(0));
        assert_eq!(groups[7].bytes().len(), 16 * 94);
        assert_eq!(groups[7].record(16), Err(MissionSourceError::RecordIndexOutOfBounds { family: 7, index: 16, count: 16 }));
        assert_eq!(source.family1_sprite_fields(0), Ok(MissionFamily1SpriteFields {
            record_index: 0,
            signed_position_words: [-123, 456],
            flags_at_15: 0x92,
            shade_at_17: 0xff,
            animation_selector: 8,
            frame_selector: 9,
            resource_slot: 0x80,
            signed_resource_selector: i8::MIN,
            native_flags_at_2e: 0xa5,
        }));
        assert_eq!(source.family1_sprite_fields(512), Err(MissionSourceError::RecordIndexOutOfBounds {
            family: 1, index: 512, count: 512,
        }));
    }

    #[test]
    fn family1_source_fields_preserve_extreme_signed_selectors_and_raw_bytes() {
        let mut bytes = synthetic();
        let record = CONQ_FAMILY1_FILE_OFFSET + 7 * CONQ_FAMILY1_PACKED_STRIDE;
        bytes[record + 0x15] = 0x10;
        bytes[record + 0x17] = 30;
        bytes[record + 0x19] = 0xff;
        bytes[record + 0x1b] = 0x80;
        bytes[record + 0x23] = 0xff;
        bytes[record + 0x26] = 0x02;
        let source = ConqMissionSource::parse(&bytes).unwrap();
        let fields = source.family1_sprite_fields(7).unwrap();
        assert_eq!(fields.flags_at_15, 0x10);
        assert_eq!(fields.shade_at_17, 30);
        assert_eq!(fields.animation_selector, 0xff);
        assert_eq!(fields.frame_selector, 0x80);
        assert_eq!(fields.resource_slot, 0xff);
        assert_eq!(fields.signed_resource_selector, -1);
        assert_eq!(fields.native_flags_at_2e, 0x02);
        assert_eq!(source.group(1).unwrap().record(7).unwrap()[0x23], 0xff);
    }

    #[test]
    fn admitted_group_extents_are_contiguous_and_reach_the_tail() {
        assert!(fixed_groups_cover_tail());
        assert_eq!(GROUP_LAYOUTS.first().unwrap().file_offset, CONQ_MISSION_PREFIX_BYTES);
        let last = GROUP_LAYOUTS.last().unwrap();
        assert_eq!(last.file_offset + last.count * last.packed_stride, CONQ_TAIL_GLOBALS_OFFSET);
        assert_eq!(CONQ_EXTENSION_PAYLOAD_OFFSET + CONQ_EXTENSION_PAYLOAD_BYTES, CONQ_MISSION_BYTES);
    }

    #[test]
    fn unsupported_source_variants_are_not_called_corrupt() {
        let short = vec![0; CONQ_MISSION_BYTES - 1];
        assert_eq!(ConqMissionSource::parse(&short), Err(MissionSourceError::UnsupportedSourceVariantLength {
            expected: CONQ_MISSION_BYTES, actual: CONQ_MISSION_BYTES - 1,
        }));
        let other = vec![0; 375_830];
        assert_eq!(ConqMissionSource::parse(&other), Err(MissionSourceError::UnsupportedSourceVariantLength {
            expected: CONQ_MISSION_BYTES, actual: 375_830,
        }));
    }

    #[test]
    fn tag_and_nested_extent_errors_are_explicit() {
        let mut wrong_fast = synthetic();
        wrong_fast[..4].copy_from_slice(&0x1234u32.to_le_bytes());
        assert_eq!(ConqMissionSource::parse(&wrong_fast), Err(MissionSourceError::UnsupportedFastTag { actual: 0x1234 }));
        let mut wrong_extension = synthetic();
        wrong_extension[CONQ_EXTENSION_TAG_OFFSET..CONQ_EXTENSION_TAG_OFFSET + 4].copy_from_slice(&0x6bfu32.to_le_bytes());
        assert_eq!(ConqMissionSource::parse(&wrong_extension), Err(MissionSourceError::UnsupportedExtensionTag { actual: 0x6bf }));
        let bytes = synthetic();
        let source = ConqMissionSource::parse(&bytes).unwrap();
        assert_eq!(source.target_words(4), Err(MissionSourceError::TargetIndexOutOfBounds { index: 4, count: 4 }));
        assert_eq!(source.group(8).unwrap_err(), MissionSourceError::GroupIndexOutOfBounds { family: 8, count: 8 });
    }
}

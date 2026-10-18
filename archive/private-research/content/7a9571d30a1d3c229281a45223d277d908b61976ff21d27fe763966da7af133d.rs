//! Safe, bounded readers for GeneWars resource files.
//!
//! The RNC ProPack header and method 1 codec are implemented independently from
//! the published format descriptions listed in `research/formats/rnc.md`.

use std::fmt;
pub mod map_atlas;
pub use map_atlas::{B32Atlas, ConqMapCells, ConqMapCell, Error as MapAtlasError};
pub mod mission_source;
pub use mission_source::{ConqMissionSource, MissionFamily1SpriteFields, MissionRecordGroup, MissionRecordGroupLayout, MissionSourceError,
    CONQ_MISSION_BYTES, CONQ_MISSION_LAYOUT_VERSION};

const RNC_HEADER_LEN: usize = 18;
const PALETTE256_BYTES: usize = 256 * 3;
const SPRITE_TAB_RECORD_BYTES: usize = 6;
const PLANET_ATLAS_WIDTH: usize = 256;
const PLANET_ATLAS_HEIGHT: usize = 640;
const PLANET_ATLAS_BYTES: usize = PLANET_ATLAS_WIDTH * PLANET_ATLAS_HEIGHT;
const PLANET_TEXTURE_ROW_STRIDE: usize = PLANET_ATLAS_WIDTH;
const FONT_TAB_HEADER_BYTES: usize = 6;
const FONT_TAB_RECORD_BYTES: usize = 6;
const CAMPAIGN_HEADER_BYTES: usize = 699;
const CAMPAIGN_HEADER_TITLE_OFFSET: usize = 32;
const CONQ_FAST_TAG: u32 = 0x55336;
const CONQ_FAST_PREFIX_BYTES: usize = 0x3B55E;
const CONQ_INITIAL_PLAYER_OFFSET: usize = 0x14;
const CONQ_INITIAL_PLAYER_RECORD_BYTES: usize = 0x70D;
const CONQ_INITIAL_PLAYER_COUNT: usize = 4;
const CONQ_OPAQUE_REGION_OFFSET: usize = 0x355E;
/// Number of NUL-terminated entries built by the traced English text indexer.
pub const TEXT_STRING_COUNT: usize = 604;

/// File offset where the observed 18-byte ANB header ends and sprite data begins.
pub const ANB_SPRITE_DATA_BASE: usize = 0x12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    TruncatedHeader,
    BadMagic,
    UnsupportedMethod(u8),
    UnsupportedFlags { locked: bool, encrypted: bool },
    OutputLimit { declared: usize, limit: usize },
    AllocationFailed,
    LengthOverflow,
    PackedLengthMismatch { declared: usize, actual: usize },
    PackedCrcMismatch { expected: u16, actual: u16 },
    UnpackedCrcMismatch { expected: u16, actual: u16 },
    TruncatedBitstream,
    InvalidHuffmanTable,
    InvalidCode,
    InvalidBackReference { distance: usize, produced: usize },
    OutputLengthMismatch { declared: usize, actual: usize },
    PaletteLength { expected: usize, actual: usize },
    SpriteTableLength { actual: usize },
    SpriteRecordOutOfBounds { record: usize, records: usize },
    SpriteOffsetOutOfBounds { record: usize, offset: usize, database_len: usize },
    SpriteStreamTruncated { record: usize, offset: usize },
    SpriteAllocationFailed,
    SpriteDatabaseTruncated { required: usize, actual: usize },
    AnbHeaderTruncated { actual: usize },
    AnbSectionOffsetsInvalid { offsets: [u32; 4], file_len: usize },
    AnbRecordSectionLength { section: &'static str, actual: usize, record_size: usize },
    AnbAnimationOutOfBounds { animation: usize, count: usize },
    AnbFrameOffsetInvalid { animation: usize, offset: u32, frame_bytes: usize },
    AnbFrameOutOfBounds { frame: usize, count: usize },
    AnbComponentRangeInvalid { frame: usize, offset: u32, count: usize, component_bytes: usize },
    AnbComponentOutOfBounds { component: usize, count: usize },
    AnbSpriteReferenceInvalid { component: usize, offset: u32, sprite_records: usize },
    PlanetAtlasLength { expected: usize, actual: usize },
    PlanetTileOutOfRange { tile_type: u8 },
    PlanetTileOutOfBounds { tile_type: u8, offset: usize, width: usize, height: usize },
    PlanetTileAllocationFailed,
    FontTableLength { actual: usize },
    FontDataOffsetOutOfBounds { record: usize, offset: usize, data_len: usize },
    FontGlyphOutOfBounds { record: usize, records: usize },
    FontUnsupportedTextByte { byte: u8 },
    FontGlyphStreamTruncated { record: usize, offset: usize },
    FontGlyphAllocationFailed,
    TextTableAllocationFailed,
    TextTableTruncated { expected: usize, parsed: usize, offset: usize },
    CampaignHeaderLength { expected: usize, actual: usize },
    CampaignHeaderTitleUnterminated { offset: usize, limit: usize },
    ConqFastPrefixTruncated { required: usize, actual: usize },
    ConqUnsupportedFastTag { actual: u32 },
    ConqInitialPlayerOutOfBounds { index: usize, count: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

/// Bounded view of a six-byte opaque header followed by six-byte font records.
/// Record offsets address the supplied DAT byte slice directly; they do not
/// include the ANB sprite-data bias used by other resource types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontResource<'a> {
    data: &'a [u8],
    records: &'a [u8],
}

impl<'a> FontResource<'a> {
    pub fn parse(data: &'a [u8], table: &'a [u8]) -> Result<Self, Error> {
        if table.len() < FONT_TAB_HEADER_BYTES
            || (table.len() - FONT_TAB_HEADER_BYTES) % FONT_TAB_RECORD_BYTES != 0
        {
            return Err(Error::FontTableLength { actual: table.len() });
        }
        Ok(Self { data, records: &table[FONT_TAB_HEADER_BYTES..] })
    }

    pub fn record_count(&self) -> usize {
        self.records.len() / FONT_TAB_RECORD_BYTES
    }

    /// The character table begins after the opaque six-byte TAB prefix.
    /// Byte 32 (space) is record 0; each subsequent byte selects the next row.
    pub fn record_for_byte(&self, byte: u8) -> Result<FontGlyphRecord, Error> {
        if byte < 32 {
            return Err(Error::FontUnsupportedTextByte { byte });
        }
        self.record(usize::from(byte - 32))
    }

    pub fn record(&self, record: usize) -> Result<FontGlyphRecord, Error> {
        if record >= self.record_count() {
            return Err(Error::FontGlyphOutOfBounds { record, records: self.record_count() });
        }
        let start = record.checked_mul(FONT_TAB_RECORD_BYTES)
            .ok_or(Error::FontGlyphOutOfBounds { record, records: self.record_count() })?;
        let end = start.checked_add(FONT_TAB_RECORD_BYTES)
            .ok_or(Error::FontGlyphOutOfBounds { record, records: self.record_count() })?;
        let bytes = self.records.get(start..end)
            .ok_or(Error::FontGlyphOutOfBounds { record, records: self.record_count() })?;
        let data_offset = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let width = bytes[4];
        let height = bytes[5];
        if width != 0 && height != 0 && data_offset >= self.data.len() {
            return Err(Error::FontDataOffsetOutOfBounds { record, offset: data_offset, data_len: self.data.len() });
        }
        Ok(FontGlyphRecord { record, data_offset, width, height })
    }

    /// Decode the row-control stream for one record. Positive signed controls
    /// copy literal index bytes (including zero as opaque), negative controls
    /// skip transparent pixels, and zero ends a row.
    pub fn decode_glyph(&self, record: usize) -> Result<IndexedSprite, Error> {
        let glyph = self.record(record)?;
        let width = usize::from(glyph.width);
        let height = usize::from(glyph.height);
        let pixel_count = width.checked_mul(height).ok_or(Error::FontGlyphAllocationFailed)?;
        let mut indices = Vec::new();
        let mut opaque = Vec::new();
        indices.try_reserve_exact(pixel_count).map_err(|_| Error::FontGlyphAllocationFailed)?;
        opaque.try_reserve_exact(pixel_count).map_err(|_| Error::FontGlyphAllocationFailed)?;
        indices.resize(pixel_count, 0);
        opaque.resize(pixel_count, false);
        if width == 0 || height == 0 {
            return Ok(IndexedSprite { width: glyph.width, height: glyph.height, indices, opaque, consumed_bytes: 0 });
        }

        let mut cursor = glyph.data_offset;
        for row in 0..height {
            let mut x = 0usize;
            loop {
                let control_byte = *self.data.get(cursor)
                    .ok_or(Error::FontGlyphStreamTruncated { record, offset: cursor })?;
                cursor += 1;
                let control = control_byte as i8;
                if control == 0 { break; }
                let run = usize::from(control.unsigned_abs());
                if control > 0 {
                    let end = cursor.checked_add(run).ok_or(Error::FontGlyphStreamTruncated { record, offset: cursor })?;
                    let literal = self.data.get(cursor..end)
                        .ok_or(Error::FontGlyphStreamTruncated { record, offset: cursor })?;
                    let visible = run.min(width.saturating_sub(x));
                    let output_start = row * width + x.min(width);
                    let output_end = output_start + visible;
                    indices[output_start..output_end].copy_from_slice(&literal[..visible]);
                    opaque[output_start..output_end].fill(true);
                    cursor = end;
                    x = x.saturating_add(run).min(width);
                } else {
                    x = x.saturating_add(run).min(width);
                }
            }
        }
        Ok(IndexedSprite { width: glyph.width, height: glyph.height, indices, opaque, consumed_bytes: cursor - glyph.data_offset })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontGlyphRecord {
    pub record: usize,
    pub data_offset: usize,
    pub width: u8,
    pub height: u8,
}

/// The 604 strings indexed by the traced startup loop, plus the byte extent
/// that loop did not index. Bytes are preserved without charset interpretation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextStringTable<'a> {
    strings: Vec<&'a [u8]>,
    consumed_len: usize,
    trailing: &'a [u8],
}

impl<'a> TextStringTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        let mut strings = Vec::new();
        strings.try_reserve_exact(TEXT_STRING_COUNT).map_err(|_| Error::TextTableAllocationFailed)?;
        let mut start = 0usize;
        for index in 0..TEXT_STRING_COUNT {
            let relative_end = bytes.get(start..).and_then(|rest| rest.iter().position(|byte| *byte == 0))
                .ok_or(Error::TextTableTruncated { expected: TEXT_STRING_COUNT, parsed: index, offset: start })?;
            let end = start + relative_end;
            strings.push(&bytes[start..end]);
            start = end + 1;
        }
        Ok(Self { strings, consumed_len: start, trailing: &bytes[start..] })
    }

    pub fn strings(&self) -> &[&'a [u8]] { &self.strings }
    pub fn consumed_len(&self) -> usize { self.consumed_len }
    pub fn trailing(&self) -> &'a [u8] { self.trailing }
}

/// Borrowed view of the exact 699-byte first-campaign header contract.
/// Only the NUL-terminated byte slice at offset 32 is interpreted; all other
/// bytes remain opaque. The title location is evidenced for HDRCO1 and should
/// not be assumed for other filename families without their consumer trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CampaignHeader<'a> {
    bytes: &'a [u8],
    title: &'a [u8],
}

impl<'a> CampaignHeader<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() != CAMPAIGN_HEADER_BYTES {
            return Err(Error::CampaignHeaderLength { expected: CAMPAIGN_HEADER_BYTES, actual: bytes.len() });
        }
        let tail = &bytes[CAMPAIGN_HEADER_TITLE_OFFSET..];
        let terminator = tail.iter().position(|byte| *byte == 0)
            .ok_or(Error::CampaignHeaderTitleUnterminated {
                offset: CAMPAIGN_HEADER_TITLE_OFFSET,
                limit: CAMPAIGN_HEADER_BYTES,
            })?;
        Ok(Self { bytes, title: &tail[..terminator] })
    }

    /// The original 699-byte allocation, including all uninterpreted fields.
    pub fn bytes(&self) -> &'a [u8] { self.bytes }

    /// The bounded NUL string passed by the traced campaign briefing consumer.
    pub fn title_bytes(&self) -> &'a [u8] { self.title }
}

/// Borrowed decomposition of the loader's proven CONQ fast-path byte copy.
/// It does not interpret the copied bytes as a complete live mission state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConqFastPrefix<'a> {
    file: &'a [u8],
}

impl<'a> ConqFastPrefix<'a> {
    /// Accept the `0x55336` fast-path tag and require the exact prefix copied by
    /// the loader. Any bytes after that prefix remain available as opaque tail.
    pub fn parse(file: &'a [u8]) -> Result<Self, Error> {
        if file.len() < CONQ_FAST_PREFIX_BYTES {
            return Err(Error::ConqFastPrefixTruncated {
                required: CONQ_FAST_PREFIX_BYTES,
                actual: file.len(),
            });
        }
        let tag = u32::from_le_bytes([file[0], file[1], file[2], file[3]]);
        if tag != CONQ_FAST_TAG {
            return Err(Error::ConqUnsupportedFastTag { actual: tag });
        }
        Ok(Self { file })
    }

    pub fn tag(&self) -> u32 {
        CONQ_FAST_TAG
    }

    /// Bytes copied by the fast path, at matching source and destination
    /// offsets. The first dword is the observed branch tag.
    pub fn copied_prefix(&self) -> &'a [u8] {
        &self.file[..CONQ_FAST_PREFIX_BYTES]
    }

    /// Opaque bytes after the bounded prefix; no tail format is asserted.
    pub fn trailing_bytes(&self) -> &'a [u8] {
        &self.file[CONQ_FAST_PREFIX_BYTES..]
    }

    /// One of the four initial player-record byte ranges from the copied file
    /// prefix. Later runtime mutations are not modeled by this byte accessor.
    pub fn initial_player_record(&self, index: usize) -> Result<&'a [u8], Error> {
        if index >= CONQ_INITIAL_PLAYER_COUNT {
            return Err(Error::ConqInitialPlayerOutOfBounds {
                index,
                count: CONQ_INITIAL_PLAYER_COUNT,
            });
        }
        let start = CONQ_INITIAL_PLAYER_OFFSET + index * CONQ_INITIAL_PLAYER_RECORD_BYTES;
        Ok(&self.copied_prefix()[start..start + CONQ_INITIAL_PLAYER_RECORD_BYTES])
    }

    /// The opaque copied byte range beginning at file offset `0x355E`.
    /// The loader's legacy path also traverses this region in 14-byte steps;
    /// its fields and semantics are not interpreted here.
    pub fn opaque_14_byte_region(&self) -> &'a [u8] {
        &self.copied_prefix()[CONQ_OPAQUE_REGION_OFFSET..]
    }
}

/// Borrowed, bounds-checked view of the ANB header and its four absolute section offsets.
/// The first 16-bit word is opaque here; [`AnbUiMetadata`] applies its proven animation-count
/// interpretation when reading the final metadata section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnbSections<'a> {
    file: &'a [u8],
    header_word: u16,
    offsets: [u32; 4],
}

impl<'a> AnbSections<'a> {
    /// Parse the 18-byte header (`u16` plus four absolute `u32` offsets).
    pub fn parse(file: &'a [u8]) -> Result<Self, Error> {
        if file.len() < ANB_SPRITE_DATA_BASE {
            return Err(Error::AnbHeaderTruncated { actual: file.len() });
        }
        let header_word = u16::from_le_bytes([file[0], file[1]]);
        let mut offsets = [0u32; 4];
        for (index, offset) in offsets.iter_mut().enumerate() {
            let start = 2 + index * 4;
            *offset = u32::from_le_bytes([file[start], file[start + 1], file[start + 2], file[start + 3]]);
        }
        let len32 = u32::try_from(file.len()).unwrap_or(u32::MAX);
        if offsets[0] < ANB_SPRITE_DATA_BASE as u32
            || offsets.windows(2).any(|pair| pair[0] > pair[1])
            || offsets[3] > len32
        {
            return Err(Error::AnbSectionOffsetsInvalid { offsets, file_len: file.len() });
        }
        Ok(Self { file, header_word, offsets })
    }

    pub fn header_word(&self) -> u16 {
        self.header_word
    }

    /// Absolute byte offsets for the four observed ANB sections.
    pub fn offsets(&self) -> [u32; 4] {
        self.offsets
    }

    /// The first section, interpreted by the observed loader as six-byte sprite records.
    pub fn sprite_table_bytes(&self) -> &'a [u8] {
        &self.file[self.offsets[0] as usize..self.offsets[1] as usize]
    }

    /// File prefix containing the header and bytes before the embedded table section.
    pub fn sprite_database_prefix(&self) -> &'a [u8] {
        &self.file[..self.offsets[0] as usize]
    }

    /// Parse the UI metadata layout validated for the CD IFCUSTOM resource and confirmed by
    /// the loader's section rebasing: 10-byte components, 14-byte frames, then 10-byte
    /// animation entries. This accessor validates all section lengths before returning.
    pub fn ui_metadata(&self) -> Result<AnbUiMetadata<'a>, Error> {
        AnbUiMetadata::parse(*self)
    }
}

const ANB_COMPONENT_BYTES: usize = 10;
const ANB_FRAME_BYTES: usize = 14;
const ANB_ANIMATION_BYTES: usize = 10;

/// Checked borrowed view of the observed ANB UI metadata sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnbUiMetadata<'a> {
    sections: AnbSections<'a>,
    component_bytes: &'a [u8],
    frame_bytes: &'a [u8],
    animation_bytes: &'a [u8],
    component_count: usize,
    frame_count: usize,
    animation_count: usize,
}

impl<'a> AnbUiMetadata<'a> {
    fn parse(sections: AnbSections<'a>) -> Result<Self, Error> {
        let [_, component_start, frame_start, animation_start] = sections.offsets.map(|v| v as usize);
        let component_bytes = &sections.file[component_start..frame_start];
        let frame_bytes = &sections.file[frame_start..animation_start];
        let animation_bytes = &sections.file[animation_start..];
        if component_bytes.len() % ANB_COMPONENT_BYTES != 0 {
            return Err(Error::AnbRecordSectionLength { section: "components", actual: component_bytes.len(), record_size: ANB_COMPONENT_BYTES });
        }
        if frame_bytes.len() % ANB_FRAME_BYTES != 0 {
            return Err(Error::AnbRecordSectionLength { section: "frames", actual: frame_bytes.len(), record_size: ANB_FRAME_BYTES });
        }
        let animation_count = usize::from(sections.header_word);
        let expected_animation_bytes = animation_count.checked_mul(ANB_ANIMATION_BYTES).ok_or(Error::LengthOverflow)?;
        if animation_bytes.len() != expected_animation_bytes {
            return Err(Error::AnbRecordSectionLength { section: "animations", actual: animation_bytes.len(), record_size: ANB_ANIMATION_BYTES });
        }
        Ok(Self {
            sections,
            component_bytes,
            frame_bytes,
            animation_bytes,
            component_count: component_bytes.len() / ANB_COMPONENT_BYTES,
            frame_count: frame_bytes.len() / ANB_FRAME_BYTES,
            animation_count,
        })
    }

    pub fn component_count(&self) -> usize { self.component_count }
    pub fn frame_count(&self) -> usize { self.frame_count }
    pub fn animation_count(&self) -> usize { self.animation_count }

    pub fn animation(&self, index: usize) -> Result<AnbAnimationRecord, Error> {
        if index >= self.animation_count {
            return Err(Error::AnbAnimationOutOfBounds { animation: index, count: self.animation_count });
        }
        let bytes = &self.animation_bytes[index * ANB_ANIMATION_BYTES..][..ANB_ANIMATION_BYTES];
        let first_frame_offset = read_u32(bytes, 0);
        let last_frame_offset = read_u32(bytes, 4);
        let trailing_word = u16::from_le_bytes([bytes[8], bytes[9]]);
        for offset in [first_frame_offset, last_frame_offset] {
            if offset as usize % ANB_FRAME_BYTES != 0 || offset as usize >= self.frame_bytes.len() {
                return Err(Error::AnbFrameOffsetInvalid { animation: index, offset, frame_bytes: self.frame_bytes.len() });
            }
        }
        Ok(AnbAnimationRecord { first_frame_offset, last_frame_offset, trailing_word })
    }

    /// Read one frame by its byte offset relative to the frame section, as stored in an
    /// animation record. The component start is relative to the component section.
    pub fn frame_at(&self, frame_offset: u32) -> Result<AnbFrameRecord, Error> {
        let offset = frame_offset as usize;
        if offset % ANB_FRAME_BYTES != 0 || offset >= self.frame_bytes.len() {
            return Err(Error::AnbFrameOutOfBounds { frame: offset / ANB_FRAME_BYTES, count: self.frame_count });
        }
        let index = offset / ANB_FRAME_BYTES;
        let bytes = &self.frame_bytes[offset..][..ANB_FRAME_BYTES];
        let component_offset = read_u32(bytes, 0);
        let width_raw = read_i16(bytes, 4);
        let height_raw = read_i16(bytes, 6);
        let component_count = bytes[11] as usize;
        let start = component_offset as usize;
        let byte_count = component_count.checked_mul(ANB_COMPONENT_BYTES).ok_or(Error::LengthOverflow)?;
        if start % ANB_COMPONENT_BYTES != 0 || start.checked_add(byte_count).filter(|end| *end <= self.component_bytes.len()).is_none() {
            return Err(Error::AnbComponentRangeInvalid { frame: index, offset: component_offset, count: component_count, component_bytes: self.component_bytes.len() });
        }
        Ok(AnbFrameRecord {
            frame_offset,
            component_offset,
            component_count: component_count as u8,
            width_raw,
            height_raw,
            byte_10: bytes[10],
            trailing_bytes: [bytes[12], bytes[13]],
        })
    }

    pub fn component(&self, index: usize) -> Result<AnbComponentRecord, Error> {
        if index >= self.component_count {
            return Err(Error::AnbComponentOutOfBounds { component: index, count: self.component_count });
        }
        let bytes = &self.component_bytes[index * ANB_COMPONENT_BYTES..][..ANB_COMPONENT_BYTES];
        let sprite_offset = read_u32(bytes, 0);
        if sprite_offset as usize % SPRITE_TAB_RECORD_BYTES != 0
            || (sprite_offset as usize).checked_add(SPRITE_TAB_RECORD_BYTES)
                .filter(|end| *end <= self.sections.sprite_table_bytes().len()).is_none()
        {
            return Err(Error::AnbSpriteReferenceInvalid { component: index, offset: sprite_offset, sprite_records: self.sections.sprite_table_bytes().len() / SPRITE_TAB_RECORD_BYTES });
        }
        Ok(AnbComponentRecord {
            sprite_offset,
            x_raw: read_i16(bytes, 4),
            y_raw: read_i16(bytes, 6),
            flags: bytes[8],
            extra: bytes[9],
        })
    }

    /// Resolve a component by its zero-based position in a frame's contiguous component list.
    pub fn frame_component(&self, frame: AnbFrameRecord, local_index: usize) -> Result<AnbComponentRecord, Error> {
        if local_index >= usize::from(frame.component_count) {
            return Err(Error::AnbComponentRangeInvalid {
                frame: (frame.frame_offset as usize) / ANB_FRAME_BYTES,
                offset: frame.component_offset,
                count: local_index.saturating_add(1),
                component_bytes: self.component_bytes.len(),
            });
        }
        let start = frame.component_offset as usize / ANB_COMPONENT_BYTES;
        let index = start.checked_add(local_index).ok_or(Error::LengthOverflow)?;
        self.component(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnbAnimationRecord {
    /// First and last byte offsets relative to the frame section.
    pub first_frame_offset: u32,
    pub last_frame_offset: u32,
    /// Raw final little-endian word, retained without assigning timing semantics.
    pub trailing_word: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnbFrameRecord {
    pub frame_offset: u32,
    /// Byte offset relative to the component section.
    pub component_offset: u32,
    pub component_count: u8,
    pub width_raw: i16,
    pub height_raw: i16,
    /// Uninterpreted byte at frame offset +10.
    pub byte_10: u8,
    /// Uninterpreted bytes at frame offsets +12 and +13.
    pub trailing_bytes: [u8; 2],
}

impl AnbFrameRecord {
    /// The loader uses signed arithmetic right shift by one for both dimensions.
    pub fn display_dimensions(&self) -> (i16, i16) { (self.width_raw >> 1, self.height_raw >> 1) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnbComponentRecord {
    /// Byte offset relative to the six-byte sprite table.
    pub sprite_offset: u32,
    pub x_raw: i16,
    pub y_raw: i16,
    pub flags: u8,
    pub extra: u8,
}

impl AnbComponentRecord {
    /// The loader uses signed arithmetic right shift by one for component positions.
    pub fn local_position(&self) -> (i16, i16) { (self.x_raw >> 1, self.y_raw >> 1) }
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

/// Caller-supplied interpretation of the three raw bytes in each palette entry.
/// Parsing preserves byte order and values; it does not swap channels or scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteChannelMode {
    Rgb,
    Bgr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Palette256 {
    channel_mode: PaletteChannelMode,
    entries: [[u8; 3]; 256],
}

impl Palette256 {
    pub fn parse(data: &[u8], channel_mode: PaletteChannelMode) -> Result<Self, Error> {
        if data.len() != PALETTE256_BYTES {
            return Err(Error::PaletteLength { expected: PALETTE256_BYTES, actual: data.len() });
        }
        let mut entries = [[0; 3]; 256];
        for (entry, bytes) in entries.iter_mut().zip(data.chunks_exact(3)) {
            entry.copy_from_slice(bytes);
        }
        Ok(Self { channel_mode, entries })
    }

    pub fn channel_mode(&self) -> PaletteChannelMode {
        self.channel_mode
    }

    pub fn entries(&self) -> &[[u8; 3]; 256] {
        &self.entries
    }
}

/// A bounded view of the fixed-stride planet texture atlas used by the CD
/// planet-selection sampler. This is only the source texture data; it does
/// not assign a palette or a projected screen position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanetTextureAtlas<'a> {
    bytes: &'a [u8],
}

impl<'a> PlanetTextureAtlas<'a> {
    /// Parse an exact 256×640 byte atlas. The 256-byte row stride and
    /// 163,840-byte extent are established by the texture sampler and the
    /// decoded PL1–PL4 resources.
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() != PLANET_ATLAS_BYTES {
            return Err(Error::PlanetAtlasLength { expected: PLANET_ATLAS_BYTES, actual: bytes.len() });
        }
        Ok(Self { bytes })
    }

    /// Extract one explicitly selected atlas tile. Types 0–7 are 128×128;
    /// types 8–13 are 64×64. No tile type is inferred from the source data.
    pub fn tile(&self, tile_type: u8) -> Result<PlanetTextureTile, Error> {
        let (offset, width, height) = match tile_type {
            0..=7 => {
                let index = usize::from(tile_type);
                let offset = (index & 1) * 128 + (index >> 1) * 32_768;
                (offset, 128, 128)
            }
            8..=13 => {
                let index = usize::from(tile_type - 8);
                let offset = 131_072 + (index / 4) * 16_384 + (index % 4) * 64;
                (offset, 64, 64)
            }
            _ => return Err(Error::PlanetTileOutOfRange { tile_type }),
        };

        let pixel_count = width * height;
        let mut indices = Vec::new();
        indices.try_reserve_exact(pixel_count).map_err(|_| Error::PlanetTileAllocationFailed)?;
        for row in 0..height {
            let row_start = offset
                .checked_add(row * PLANET_TEXTURE_ROW_STRIDE)
                .ok_or(Error::LengthOverflow)?;
            let row_end = row_start.checked_add(width).ok_or(Error::LengthOverflow)?;
            let source = self.bytes.get(row_start..row_end).ok_or(Error::PlanetTileOutOfBounds {
                tile_type,
                offset: row_start,
                width,
                height,
            })?;
            indices.extend_from_slice(source);
        }
        Ok(PlanetTextureTile { tile_type, width, height, indices })
    }
}

/// Contiguous raw palette indices for one explicitly selected planet tile.
/// Index zero is transparent in the evidenced sampler; all other values are
/// copied unchanged. Palette selection and color conversion are separate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanetTextureTile {
    pub tile_type: u8,
    pub width: usize,
    pub height: usize,
    pub indices: Vec<u8>,
}

impl PlanetTextureTile {
    pub fn is_opaque(&self, x: usize, y: usize) -> Option<bool> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = y.checked_mul(self.width)?.checked_add(x)?;
        self.indices.get(index).map(|value| *value != 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpriteDescriptor {
    /// Offset from the ANB sprite-data base at file offset 0x12.
    pub data_offset: u32,
    pub width: u8,
    pub height: u8,
}

impl SpriteDescriptor {
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteTable {
    records: Vec<SpriteDescriptor>,
    database_limit: Option<usize>,
}

impl SpriteTable {
    /// Read six-byte little-endian offset/width/height records and validate
    /// non-empty data offsets against the caller-provided ANB byte range.
    pub fn parse(tab: &[u8], database: &[u8]) -> Result<Self, Error> {
        if tab.len() % SPRITE_TAB_RECORD_BYTES != 0 {
            return Err(Error::SpriteTableLength { actual: tab.len() });
        }
        let count = tab.len() / SPRITE_TAB_RECORD_BYTES;
        let mut records = Vec::new();
        records.try_reserve_exact(count).map_err(|_| Error::SpriteAllocationFailed)?;
        for (record, bytes) in tab.chunks_exact(SPRITE_TAB_RECORD_BYTES).enumerate() {
            let descriptor = SpriteDescriptor {
                data_offset: u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
                width: bytes[4],
                height: bytes[5],
            };
            if !descriptor.is_empty() {
                let offset = ANB_SPRITE_DATA_BASE.checked_add(descriptor.data_offset as usize)
                    .ok_or(Error::SpriteOffsetOutOfBounds {
                        record,
                        offset: descriptor.data_offset as usize,
                        database_len: database.len(),
                    })?;
                if offset >= database.len() {
                    return Err(Error::SpriteOffsetOutOfBounds { record, offset, database_len: database.len() });
                }
            }
            records.push(descriptor);
        }
        Ok(Self { records, database_limit: None })
    }

    /// Parse the embedded first section of an ANB database as six-byte sprite records.
    pub fn from_anb(anb: &[u8]) -> Result<Self, Error> {
        let sections = AnbSections::parse(anb)?;
        let database_limit = sections.offsets()[0] as usize;
        let mut table = Self::parse(sections.sprite_table_bytes(), sections.sprite_database_prefix())?;
        table.database_limit = Some(database_limit);
        Ok(table)
    }

    pub fn records(&self) -> &[SpriteDescriptor] {
        &self.records
    }

    pub fn decode(&self, record: usize, database: &[u8]) -> Result<IndexedSprite, Error> {
        let descriptor = *self.records.get(record).ok_or(Error::SpriteRecordOutOfBounds {
            record,
            records: self.records.len(),
        })?;
        let bounded = if let Some(limit) = self.database_limit {
            let prefix = database.get(..limit).ok_or(Error::SpriteDatabaseTruncated {
                required: limit,
                actual: database.len(),
            })?;
            prefix
        } else {
            database
        };
        decode_sprite_rle(record, descriptor, bounded)
    }

    /// Decode a caller-selected packed four-bit sprite variant. Each positive run is a
    /// pixel count, with high nibble first; palette values are `base.wrapping_add(nibble)`.
    /// No automatic format selection or translation-table lookup is performed.
    pub fn decode_packed4(&self, record: usize, database: &[u8], base: u8) -> Result<IndexedSprite, Error> {
        let descriptor = *self.records.get(record).ok_or(Error::SpriteRecordOutOfBounds {
            record,
            records: self.records.len(),
        })?;
        let bounded = if let Some(limit) = self.database_limit {
            database.get(..limit).ok_or(Error::SpriteDatabaseTruncated { required: limit, actual: database.len() })?
        } else {
            database
        };
        decode_sprite_rle_packed4(record, descriptor, bounded, base)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedSprite {
    pub width: u8,
    pub height: u8,
    /// Row-major source palette indices. Transparent skip runs leave zero here.
    pub indices: Vec<u8>,
    /// Explicit run-derived opacity; a literal palette index zero is still opaque.
    pub opaque: Vec<bool>,
    /// Number of bytes consumed from the ANB database beginning at the record offset.
    pub consumed_bytes: usize,
}

fn decode_sprite_rle(record: usize, descriptor: SpriteDescriptor, database: &[u8]) -> Result<IndexedSprite, Error> {
    let pixel_count = usize::from(descriptor.width) * usize::from(descriptor.height);
    let mut indices = Vec::new();
    let mut opaque = Vec::new();
    indices.try_reserve_exact(pixel_count).map_err(|_| Error::SpriteAllocationFailed)?;
    opaque.try_reserve_exact(pixel_count).map_err(|_| Error::SpriteAllocationFailed)?;
    indices.resize(pixel_count, 0);
    opaque.resize(pixel_count, false);

    if descriptor.is_empty() {
        return Ok(IndexedSprite {
            width: descriptor.width,
            height: descriptor.height,
            indices,
            opaque,
            consumed_bytes: 0,
        });
    }

    let start = ANB_SPRITE_DATA_BASE.checked_add(descriptor.data_offset as usize)
        .ok_or(Error::SpriteOffsetOutOfBounds {
            record,
            offset: descriptor.data_offset as usize,
            database_len: database.len(),
        })?;
    if start >= database.len() {
        return Err(Error::SpriteOffsetOutOfBounds { record, offset: start, database_len: database.len() });
    }
    let mut cursor = start;
    for y in 0..usize::from(descriptor.height) {
        let mut x = 0usize;
        loop {
            let control = *database.get(cursor).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })? as i8;
            cursor += 1;
            if control == 0 {
                break;
            }
            let run = usize::from(control.unsigned_abs());
            if control > 0 {
                let end = cursor.checked_add(run).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })?;
                let literal = database.get(cursor..end).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })?;
                let visible = run.min(usize::from(descriptor.width).saturating_sub(x));
                let output_start = y * usize::from(descriptor.width) + x.min(usize::from(descriptor.width));
                let output_end = output_start + visible;
                indices[output_start..output_end].copy_from_slice(&literal[..visible]);
                opaque[output_start..output_end].fill(true);
                cursor = end;
                x = x.saturating_add(run).min(usize::from(descriptor.width));
            } else {
                x = x.saturating_add(run).min(usize::from(descriptor.width));
            }
        }
    }
    Ok(IndexedSprite {
        width: descriptor.width,
        height: descriptor.height,
        indices,
        opaque,
        consumed_bytes: cursor - start,
    })
}

fn decode_sprite_rle_packed4(
    record: usize,
    descriptor: SpriteDescriptor,
    database: &[u8],
    base: u8,
) -> Result<IndexedSprite, Error> {
    let pixel_count = usize::from(descriptor.width) * usize::from(descriptor.height);
    let mut indices = Vec::new();
    let mut opaque = Vec::new();
    indices.try_reserve_exact(pixel_count).map_err(|_| Error::SpriteAllocationFailed)?;
    opaque.try_reserve_exact(pixel_count).map_err(|_| Error::SpriteAllocationFailed)?;
    indices.resize(pixel_count, 0);
    opaque.resize(pixel_count, false);
    if descriptor.is_empty() {
        return Ok(IndexedSprite { width: descriptor.width, height: descriptor.height, indices, opaque, consumed_bytes: 0 });
    }
    let start = ANB_SPRITE_DATA_BASE.checked_add(descriptor.data_offset as usize)
        .ok_or(Error::SpriteOffsetOutOfBounds { record, offset: descriptor.data_offset as usize, database_len: database.len() })?;
    if start >= database.len() {
        return Err(Error::SpriteOffsetOutOfBounds { record, offset: start, database_len: database.len() });
    }
    let mut cursor = start;
    for y in 0..usize::from(descriptor.height) {
        let mut x = 0usize;
        loop {
            let control = *database.get(cursor).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })? as i8;
            cursor += 1;
            if control == 0 { break; }
            let run = usize::from(control.unsigned_abs());
            if control < 0 {
                x = x.saturating_add(run).min(usize::from(descriptor.width));
                continue;
            }
            let packed_bytes = run.checked_add(1).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })? / 2;
            let end = cursor.checked_add(packed_bytes).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })?;
            let packed = database.get(cursor..end).ok_or(Error::SpriteStreamTruncated { record, offset: cursor })?;
            let visible = run.min(usize::from(descriptor.width).saturating_sub(x));
            for pixel in 0..visible {
                let byte = packed[pixel / 2];
                let nibble = if pixel % 2 == 0 { byte >> 4 } else { byte & 0x0f };
                let output = y * usize::from(descriptor.width) + x + pixel;
                indices[output] = base.wrapping_add(nibble);
                opaque[output] = true;
            }
            cursor = end;
            x = x.saturating_add(run).min(usize::from(descriptor.width));
        }
    }
    Ok(IndexedSprite { width: descriptor.width, height: descriptor.height, indices, opaque, consumed_bytes: cursor - start })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub method: u8,
    pub unpacked_size: u32,
    pub packed_size: u32,
    pub unpacked_crc16: u16,
    pub packed_crc16: u16,
    /// Encoder bookkeeping bytes. Their semantics are not needed for decoding.
    pub leeway: u8,
    pub chunk_count_hint: u8,
}

impl Header {
    pub fn parse(data: &[u8]) -> Result<Self, Error> {
        if data.len() < RNC_HEADER_LEN {
            return Err(Error::TruncatedHeader);
        }
        if &data[..3] != b"RNC" {
            return Err(Error::BadMagic);
        }
        Ok(Self {
            method: data[3],
            unpacked_size: be_u32(data, 4),
            packed_size: be_u32(data, 8),
            unpacked_crc16: be_u16(data, 12),
            packed_crc16: be_u16(data, 14),
            leeway: data[16],
            chunk_count_hint: data[17],
        })
    }

    pub fn packed_payload<'a>(&self, data: &'a [u8]) -> Result<&'a [u8], Error> {
        let packed_len = usize::try_from(self.packed_size).map_err(|_| Error::LengthOverflow)?;
        let expected_total = RNC_HEADER_LEN.checked_add(packed_len).ok_or(Error::LengthOverflow)?;
        let actual_total = data.len().checked_sub(RNC_HEADER_LEN).ok_or(Error::TruncatedHeader)?;
        if actual_total != packed_len {
            return Err(Error::PackedLengthMismatch { declared: packed_len, actual: actual_total });
        }
        Ok(&data[RNC_HEADER_LEN..expected_total])
    }
}

/// Decode a complete RNC file, validating both CRCs and enforcing `output_limit`.
///
/// The input must contain exactly one complete 18-byte-header RNC stream. Method
/// 1 is supported; method 2 is rejected explicitly rather than guessed.
pub fn decode(data: &[u8], output_limit: usize) -> Result<Vec<u8>, Error> {
    let header = Header::parse(data)?;
    if header.method != 1 {
        return Err(Error::UnsupportedMethod(header.method));
    }
    let declared = usize::try_from(header.unpacked_size).map_err(|_| Error::LengthOverflow)?;
    if declared > output_limit {
        return Err(Error::OutputLimit { declared, limit: output_limit });
    }
    let packed = header.packed_payload(data)?;
    let actual_packed_crc = crc16_rnc(packed);
    if actual_packed_crc != header.packed_crc16 {
        return Err(Error::PackedCrcMismatch { expected: header.packed_crc16, actual: actual_packed_crc });
    }
    let output = decode_method1(packed, declared, header.chunk_count_hint)?;
    let actual_unpacked_crc = crc16_rnc(&output);
    if actual_unpacked_crc != header.unpacked_crc16 {
        return Err(Error::UnpackedCrcMismatch { expected: header.unpacked_crc16, actual: actual_unpacked_crc });
    }
    Ok(output)
}

/// Decode the first complete RNC file in `data`, returning its byte length and
/// leaving any following bytes untouched. The header's packed length bounds
/// the CRC and decoder input; this does not search for or infer a second file.
pub fn decode_prefix(data: &[u8], output_limit: usize) -> Result<(Vec<u8>, usize), Error> {
    let header = Header::parse(data)?;
    if header.method != 1 {
        return Err(Error::UnsupportedMethod(header.method));
    }
    let declared = usize::try_from(header.unpacked_size).map_err(|_| Error::LengthOverflow)?;
    if declared > output_limit {
        return Err(Error::OutputLimit { declared, limit: output_limit });
    }
    let packed_len = usize::try_from(header.packed_size).map_err(|_| Error::LengthOverflow)?;
    let consumed = RNC_HEADER_LEN.checked_add(packed_len).ok_or(Error::LengthOverflow)?;
    let packed = data.get(RNC_HEADER_LEN..consumed).ok_or(Error::PackedLengthMismatch {
        declared: packed_len,
        actual: data.len().saturating_sub(RNC_HEADER_LEN),
    })?;
    let actual_packed_crc = crc16_rnc(packed);
    if actual_packed_crc != header.packed_crc16 {
        return Err(Error::PackedCrcMismatch { expected: header.packed_crc16, actual: actual_packed_crc });
    }
    let output = decode_method1(packed, declared, header.chunk_count_hint)?;
    let actual_unpacked_crc = crc16_rnc(&output);
    if actual_unpacked_crc != header.unpacked_crc16 {
        return Err(Error::UnpackedCrcMismatch { expected: header.unpacked_crc16, actual: actual_unpacked_crc });
    }
    Ok((output, consumed))
}

fn decode_method1(packed: &[u8], expected: usize, block_count_hint: u8) -> Result<Vec<u8>, Error> {
    let mut bits = BitReader::new(packed);
    let locked = bits.read(1)? != 0;
    let encrypted = bits.read(1)? != 0;
    if locked || encrypted {
        return Err(Error::UnsupportedFlags { locked, encrypted });
    }

    let mut output = Vec::new();
    output.try_reserve_exact(expected).map_err(|_| Error::AllocationFailed)?;
    let mut blocks = 0u16;
    let mut raw = HuffmanTable::default();
    let mut distance = HuffmanTable::default();
    let mut length = HuffmanTable::default();
    while output.len() < expected {
        if block_count_hint != 0 && blocks >= u16::from(block_count_hint) {
            return Err(Error::OutputLengthMismatch { declared: expected, actual: output.len() });
        }
        blocks += 1;
        raw.read(&mut bits)?;
        distance.read(&mut bits)?;
        length.read(&mut bits)?;
        let chunks = bits.read(16)? as usize;
        if chunks == 0 {
            return Err(Error::InvalidHuffmanTable);
        }

        for chunk in 0..chunks {
            let literals = raw.symbol(&mut bits)?;
            if literals > expected - output.len() {
                return Err(Error::OutputLengthMismatch { declared: expected, actual: output.len() + literals });
            }
            let literal_bytes = bits.read_literals(literals)?;
            output.extend_from_slice(literal_bytes);
            if literals != 0 {
                bits.rebase_after_literals();
            }

            if output.len() == expected {
                if chunk + 1 != chunks {
                    return Err(Error::OutputLengthMismatch { declared: expected, actual: output.len() });
                }
                break;
            }
            if chunk + 1 < chunks {
                let back = distance.symbol(&mut bits)?.checked_add(1).ok_or(Error::LengthOverflow)?;
                let count = length.symbol(&mut bits)?.checked_add(2).ok_or(Error::LengthOverflow)?;
                copy_backref(&mut output, back, count, expected)?;
            }
        }
    }
    if output.len() != expected {
        return Err(Error::OutputLengthMismatch { declared: expected, actual: output.len() });
    }
    if block_count_hint != 0 && blocks != u16::from(block_count_hint) {
        return Err(Error::OutputLengthMismatch { declared: expected, actual: output.len() });
    }
    Ok(output)
}

fn copy_backref(output: &mut Vec<u8>, distance: usize, count: usize, limit: usize) -> Result<(), Error> {
    if distance == 0 || distance > output.len() {
        return Err(Error::InvalidBackReference { distance, produced: output.len() });
    }
    if count > limit.saturating_sub(output.len()) {
        return Err(Error::OutputLengthMismatch { declared: limit, actual: output.len().saturating_add(count) });
    }
    for _ in 0..count {
        let value = output[output.len() - distance];
        output.push(value);
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Default)]
struct HuffmanEntry {
    depth: u8,
    code: u16,
}

#[derive(Default)]
struct HuffmanTable {
    entries: [HuffmanEntry; 16],
}

impl HuffmanTable {
    fn read(&mut self, bits: &mut BitReader<'_>) -> Result<(), Error> {
        let count = bits.read(5)? as usize;
        if count > 16 {
            return Err(Error::InvalidHuffmanTable);
        }
        // Reference decoders retain the previous table when count is zero.
        if count == 0 {
            return Ok(());
        }
        self.entries = [HuffmanEntry::default(); 16];
        for entry in self.entries.iter_mut().take(count) {
            entry.depth = bits.read(4)? as u8;
        }

        let mut slots = 1u32;
        for depth in 1..=16 {
            slots = slots.checked_mul(2).ok_or(Error::InvalidHuffmanTable)?;
            let used = self.entries.iter().filter(|entry| entry.depth as u32 == depth).count() as u32;
            slots = slots.checked_sub(used).ok_or(Error::InvalidHuffmanTable)?;
        }

        // Assign canonical codes in symbol order and reverse each code because
        // method 1 consumes the bitstream least-significant bit first.
        let mut value = 0u64;
        let mut divisor = 0x8000_0000u64;
        for depth in 1..=16u8 {
            for entry in self.entries.iter_mut().take(count) {
                if entry.depth == depth {
                    let canonical = (value / divisor) as u16;
                    entry.code = reverse_bits(canonical, depth);
                    value += divisor;
                }
            }
            divisor >>= 1;
        }
        Ok(())
    }

    fn symbol(&self, bits: &mut BitReader<'_>) -> Result<usize, Error> {
        for (index, entry) in self.entries.iter().enumerate() {
            if entry.depth == 0 {
                continue;
            }
            if bits.peek(entry.depth as usize)? == entry.code as u64 {
                bits.read(entry.depth as usize)?;
                let value = if index < 2 { index } else {
                    bits.read(index - 1)? as usize + (1usize << (index - 1))
                };
                return Ok(value);
            }
        }
        Err(Error::InvalidCode)
    }
}

fn reverse_bits(mut value: u16, count: u8) -> u16 {
    let mut reversed = 0;
    for _ in 0..count {
        reversed = (reversed << 1) | (value & 1);
        value >>= 1;
    }
    reversed
}

struct BitReader<'a> {
    data: &'a [u8],
    cursor: usize,
    buffer: u64,
    available: usize,
}

impl<'a> BitReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: 0, buffer: 0, available: 0 }
    }

    fn read(&mut self, count: usize) -> Result<u64, Error> {
        if count > 32 {
            return Err(Error::InvalidCode);
        }
        let mut value = 0;
        for bit in 0..count {
            if self.available == 0 {
                self.refill_word()?;
            }
            value |= (self.buffer & 1) << bit;
            self.buffer >>= 1;
            self.available -= 1;
        }
        Ok(value)
    }

    fn peek(&mut self, count: usize) -> Result<u64, Error> {
        // Huffman candidates may cross a 16-bit word boundary. Probe using a
        // copy so that failed/longer candidates never commit speculative words
        // past interleaved literal bytes.
        let mut probe = Self { data: self.data, cursor: self.cursor, buffer: self.buffer, available: self.available };
        probe.read(count)
    }

    fn read_literals(&mut self, count: usize) -> Result<&'a [u8], Error> {
        if count == 0 {
            return Ok(&self.data[0..0]);
        }
        let end = self.cursor.checked_add(count).ok_or(Error::LengthOverflow)?;
        let bytes = self.data.get(self.cursor..end).ok_or(Error::TruncatedBitstream)?;
        self.cursor = end;
        Ok(bytes)
    }

    fn refill_word(&mut self) -> Result<(), Error> {
        if self.cursor >= self.data.len().saturating_add(2) {
            return Err(Error::TruncatedBitstream);
        }
        let first = self.data.get(self.cursor).copied().unwrap_or(0);
        let second = self.data.get(self.cursor + 1).copied().unwrap_or(0);
        let third = self.data.get(self.cursor + 2).copied().unwrap_or(0);
        let fourth = self.data.get(self.cursor + 3).copied().unwrap_or(0);
        self.buffer = u64::from_le_bytes([first, second, third, fourth, 0, 0, 0, 0]);
        self.cursor += 2;
        self.available = 16;
        Ok(())
    }

    fn rebase_after_literals(&mut self) {
        let pending_mask = if self.available == 0 { 0 } else { (1u64 << self.available) - 1 };
        let first = self.data.get(self.cursor).copied().unwrap_or(0);
        let second = self.data.get(self.cursor + 1).copied().unwrap_or(0);
        let next_word = u16::from_le_bytes([first, second]);
        self.buffer = (self.buffer & pending_mask) | (u64::from(next_word) << self.available);
    }
}

fn be_u16(data: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([data[at], data[at + 1]])
}

fn be_u32(data: &[u8], at: usize) -> u32 {
    u32::from_be_bytes([data[at], data[at + 1], data[at + 2], data[at + 3]])
}

/// RNC's CRC-16: reflected polynomial 0xA001, initial value zero, no final xor.
pub fn crc16_rnc(bytes: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in bytes {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rnc_crc_matches_published_header_sample() {
        assert_eq!(crc16_rnc(b"123456789"), 0xBB3D);
    }

    #[test]
    fn parses_big_endian_header_fields() {
        let data = [b'R', b'N', b'C', 1, 0, 0, 0, 9, 0, 0, 0, 3, 0x12, 0x34, 0x56, 0x78, 9, 10];
        let h = Header::parse(&data).unwrap();
        assert_eq!(h, Header { method: 1, unpacked_size: 9, packed_size: 3,
            unpacked_crc16: 0x1234, packed_crc16: 0x5678, leeway: 9, chunk_count_hint: 10 });
    }

    #[test]
    fn rejects_short_or_malformed_headers() {
        assert_eq!(Header::parse(b"RNC"), Err(Error::TruncatedHeader));
        let mut data = [0; RNC_HEADER_LEN];
        assert_eq!(Header::parse(&data), Err(Error::BadMagic));
        data[..3].copy_from_slice(b"RNC");
        data[3] = 2;
        assert!(matches!(decode(&data, 32), Err(Error::UnsupportedMethod(2))));
    }

    #[test]
    fn rejects_declared_output_over_limit_before_allocating() {
        let mut data = [0; RNC_HEADER_LEN];
        data[..4].copy_from_slice(b"RNC\x01");
        data[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        assert_eq!(
            decode(&data, 16),
            Err(Error::OutputLimit { declared: u32::MAX as usize, limit: 16 })
        );
    }

    #[test]
    fn validates_exact_packed_length() {
        let h = Header { method: 1, unpacked_size: 0, packed_size: 1,
            unpacked_crc16: 0, packed_crc16: 0, leeway: 0, chunk_count_hint: 0 };
        let bytes = [b'R', b'N', b'C', 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0];
        assert!(matches!(h.packed_payload(&bytes), Err(Error::PackedLengthMismatch { declared: 1, actual: 0 })));
    }

    #[test]
    fn decodes_synthetic_method1_literal_stream() {
        let packed = one_literal_packed(b'Z');
        let file = one_literal_file(&packed, b"Z", 1);
        assert_eq!(decode(&file, 1).unwrap(), b"Z");
    }

    #[test]
    fn lookahead_crosses_word_boundary_without_committing_cursor() {
        let mut bits = BitReader::new(&[0b0000_0001, 0b0000_0000, 0b1010_0101, 0x5a]);
        assert_eq!(bits.read(15).unwrap(), 1);
        let cursor_before = bits.cursor;
        assert_eq!(bits.peek(8).unwrap(), 0b0100_1010);
        assert_eq!(bits.cursor, cursor_before);
        assert_eq!(bits.read(8).unwrap(), 0b0100_1010);
    }

    #[test]
    fn rejects_oversubscribed_huffman_table() {
        // Three one-bit codes cannot fit in a binary prefix tree.
        let packed = bits_to_bytes(&[(3, 5), (1, 4), (1, 4), (1, 4)]);
        let mut reader = BitReader::new(&packed);
        assert!(matches!(HuffmanTable::default().read(&mut reader), Err(Error::InvalidHuffmanTable)));
    }

    #[test]
    fn zero_count_table_reuses_previous_table() {
        let mut table = HuffmanTable::default();
        table.entries[0] = HuffmanEntry { depth: 1, code: 0 };
        let packed = bits_to_bytes(&[(0, 5), (0, 1)]);
        let mut reader = BitReader::new(&packed);
        table.read(&mut reader).unwrap();
        assert_eq!(table.symbol(&mut reader).unwrap(), 0);
    }

    #[test]
    fn deterministic_mutations_never_panic_or_exceed_output_limit() {
        let original = one_literal_packed(b'Z');
        for bit in 0..original.len() * 8 {
            let mut packed = original.clone();
            packed[bit / 8] ^= 1 << (bit % 8);
            let file = one_literal_file(&packed, b"Z", 1);
            let result = std::panic::catch_unwind(|| decode(&file, 1));
            let decoded = result.expect("mutated synthetic stream must not panic");
            if let Ok(output) = decoded {
                assert!(output.len() <= 1);
            }
        }
        let valid = one_literal_file(&original, b"Z", 1);
        for len in 0..valid.len() {
            let result = std::panic::catch_unwind(|| decode(&valid[..len], 1));
            assert!(result.is_ok(), "truncated synthetic stream panicked at length {len}");
        }
    }

    #[test]
    fn copies_overlapping_backreference_matches() {
        let mut output = b"A".to_vec();
        copy_backref(&mut output, 1, 5, 6).unwrap();
        assert_eq!(output, b"AAAAAA");
    }

    #[test]
    fn parses_raw_palette_triplets_without_reordering_or_scaling() {
        let mut bytes = vec![0; PALETTE256_BYTES];
        bytes[..6].copy_from_slice(&[1, 2, 3, 63, 32, 1]);
        let palette = Palette256::parse(&bytes, PaletteChannelMode::Bgr).unwrap();
        assert_eq!(palette.channel_mode(), PaletteChannelMode::Bgr);
        assert_eq!(&palette.entries()[..2], &[[1, 2, 3], [63, 32, 1]]);
    }

    #[test]
    fn rejects_palette_lengths_other_than_256_triplets() {
        assert_eq!(
            Palette256::parse(&[0; PALETTE256_BYTES - 1], PaletteChannelMode::Rgb),
            Err(Error::PaletteLength { expected: PALETTE256_BYTES, actual: PALETTE256_BYTES - 1 })
        );
        assert_eq!(
            Palette256::parse(&[0; PALETTE256_BYTES + 1], PaletteChannelMode::Rgb),
            Err(Error::PaletteLength { expected: PALETTE256_BYTES, actual: PALETTE256_BYTES + 1 })
        );
    }

    #[test]
    fn extracts_explicit_planet_tiles_with_the_proven_256_byte_stride() {
        let mut bytes = vec![0u8; PLANET_ATLAS_BYTES];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = ((index.wrapping_mul(17).wrapping_add(3)) & 0xff) as u8;
        }
        // Type zero's top-left texel is the transparent index. Neighboring
        // rows are separated by the source atlas stride, not tile width.
        bytes[0] = 0;
        let atlas = PlanetTextureAtlas::parse(&bytes).unwrap();
        for tile_type in 0..=13u8 {
            let tile = atlas.tile(tile_type).unwrap();
            let (offset, side) = if tile_type < 8 {
                (usize::from(tile_type & 1) * 128 + usize::from(tile_type >> 1) * 32_768, 128)
            } else {
                let index = usize::from(tile_type - 8);
                (131_072 + (index / 4) * 16_384 + (index % 4) * 64, 64)
            };
            assert_eq!((tile.tile_type, tile.width, tile.height), (tile_type, side, side));
            assert_eq!(tile.indices.len(), side * side);
            assert_eq!(tile.indices[0], bytes[offset]);
            assert_eq!(tile.indices[side + 1], bytes[offset + PLANET_TEXTURE_ROW_STRIDE + 1]);
            assert_eq!(tile.indices[side * side - 1], bytes[offset + (side - 1) * PLANET_TEXTURE_ROW_STRIDE + side - 1]);
            assert_eq!(tile.is_opaque(0, 0), Some(tile.indices[0] != 0));
            assert_eq!(tile.is_opaque(side, 0), None);
            assert_eq!(tile.is_opaque(0, side), None);
        }
        assert_eq!(atlas.tile(14), Err(Error::PlanetTileOutOfRange { tile_type: 14 }));
        assert_eq!(atlas.tile(255), Err(Error::PlanetTileOutOfRange { tile_type: 255 }));
    }

    #[test]
    fn validates_planet_atlas_extent_before_tile_reads() {
        assert_eq!(
            PlanetTextureAtlas::parse(&[0; PLANET_ATLAS_BYTES - 1]),
            Err(Error::PlanetAtlasLength { expected: PLANET_ATLAS_BYTES, actual: PLANET_ATLAS_BYTES - 1 })
        );
        assert_eq!(
            PlanetTextureAtlas::parse(&[0; PLANET_ATLAS_BYTES + 1]),
            Err(Error::PlanetAtlasLength { expected: PLANET_ATLAS_BYTES, actual: PLANET_ATLAS_BYTES + 1 })
        );
    }

    #[test]
    fn planet_tile_transparency_is_explicit_and_only_index_zero_is_clear() {
        let mut bytes = vec![1u8; PLANET_ATLAS_BYTES];
        bytes[0] = 0;
        bytes[1] = 1;
        let tile = PlanetTextureAtlas::parse(&bytes).unwrap().tile(0).unwrap();
        assert_eq!(tile.is_opaque(0, 0), Some(false));
        assert_eq!(tile.is_opaque(1, 0), Some(true));
    }

    #[test]
    fn parses_six_byte_sprite_record_and_preserves_rle_opacity() {
        let mut table_bytes = Vec::new();
        table_bytes.extend_from_slice(&0u32.to_le_bytes());
        table_bytes.extend_from_slice(&[4, 2]);
        let mut database = vec![0; ANB_SPRITE_DATA_BASE];
        database.extend_from_slice(&[2, 1, 2, 0xff, 0, 3, 0, 7, 8, 0]);

        let table = SpriteTable::parse(&table_bytes, &database).unwrap();
        assert_eq!(table.records(), &[SpriteDescriptor { data_offset: 0, width: 4, height: 2 }]);
        let sprite = table.decode(0, &database).unwrap();
        assert_eq!(sprite.indices, [1, 2, 0, 0, 0, 7, 8, 0]);
        assert_eq!(sprite.opaque, [true, true, false, false, true, true, true, false]);
        assert_eq!(sprite.consumed_bytes, 10);
    }

    #[test]
    fn parses_borrowed_anb_sections_with_absolute_offsets_and_opaque_word() {
        let anb = synthetic_anb();
        let sections = AnbSections::parse(&anb).unwrap();
        assert_eq!(sections.header_word(), 0xbeef);
        assert_eq!(sections.offsets(), [24, 30, 40, 48]);
        assert_eq!(sections.sprite_table_bytes(), &anb[24..30]);

        let table = SpriteTable::from_anb(&anb).unwrap();
        assert_eq!(table.records(), &[SpriteDescriptor { data_offset: 0, width: 1, height: 1 }]);
        assert_eq!(table.decode(0, &anb).unwrap().opaque, [false]);
    }

    #[test]
    fn embedded_sprite_decode_cannot_cross_into_later_anb_sections() {
        let mut anb = vec![0; 64];
        anb[..2].copy_from_slice(&1u16.to_le_bytes());
        for (index, offset) in [26u32, 32, 40, 48].into_iter().enumerate() {
            let start = 2 + index * 4;
            anb[start..start + 4].copy_from_slice(&offset.to_le_bytes());
        }
        anb[24..26].copy_from_slice(&[1, 7]);
        anb[26..30].copy_from_slice(&6u32.to_le_bytes());
        anb[30..32].copy_from_slice(&[1, 1]);

        let embedded = SpriteTable::from_anb(&anb).unwrap();
        assert_eq!(
            embedded.decode(0, &anb),
            Err(Error::SpriteStreamTruncated { record: 0, offset: 26 })
        );

        // Explicit generic/external-table parsing retains its caller-selected full slice.
        let external = SpriteTable::parse(&anb[26..32], &anb).unwrap();
        let diagnostic = external.decode(0, &anb).unwrap();
        assert_eq!(diagnostic.indices, [7]);
        assert_eq!(diagnostic.consumed_bytes, 10);
        assert_eq!(
            embedded.decode(0, &anb[..20]),
            Err(Error::SpriteDatabaseTruncated { required: 26, actual: 20 })
        );
    }

    #[test]
    fn rejects_truncated_anb_header() {
        for len in 0..ANB_SPRITE_DATA_BASE {
            assert_eq!(AnbSections::parse(&vec![0; len]), Err(Error::AnbHeaderTruncated { actual: len }));
        }
    }

    fn synthetic_metadata_anb() -> Vec<u8> {
        let mut file = vec![0; 58];
        file[..2].copy_from_slice(&1u16.to_le_bytes());
        for (index, offset) in [18u32, 24, 34, 48].into_iter().enumerate() {
            file[2 + index * 4..6 + index * 4].copy_from_slice(&offset.to_le_bytes());
        }
        file[18..24].copy_from_slice(&[0, 0, 0, 0, 1, 1]); // sprite table
        file[24..34].copy_from_slice(&[
            0, 0, 0, 0, // sprite offset in table
            2, 0,       // x raw
            0xff, 0xff, // y raw (-1)
            0, 0,       // flags and opaque byte
        ]);
        file[34..48].copy_from_slice(&[
            0, 0, 0, 0, // component section offset
            38, 0,      // raw width
            62, 0,      // raw height
            0, 0,       // opaque u16 at +8
            0, 1,       // opaque/count bytes; count is at +11
            0, 0,       // trailing bytes
        ]);
        file[48..58].copy_from_slice(&[
            0, 0, 0, 0, // first frame offset
            0, 0, 0, 0, // last frame offset
            1, 0,       // raw count and flags
        ]);
        file
    }

    #[test]
    fn reads_bounded_anb_animation_frame_and_component_records() {
        let file = synthetic_metadata_anb();
        let metadata = AnbSections::parse(&file).unwrap().ui_metadata().unwrap();
        assert_eq!((metadata.animation_count(), metadata.frame_count(), metadata.component_count()), (1, 1, 1));
        let animation = metadata.animation(0).unwrap();
        assert_eq!(animation, AnbAnimationRecord { first_frame_offset: 0, last_frame_offset: 0, trailing_word: 1 });
        let frame = metadata.frame_at(animation.first_frame_offset).unwrap();
        assert_eq!(frame.component_count, 1);
        assert_eq!(frame.display_dimensions(), (19, 31));
        let component = metadata.frame_component(frame, 0).unwrap();
        assert_eq!(component.sprite_offset, 0);
        assert_eq!(component.local_position(), (1, -1));
    }

    #[test]
    fn rejects_bad_anb_metadata_section_lengths_and_references() {
        let mut bad_frame_stride = synthetic_metadata_anb();
        bad_frame_stride[6..10].copy_from_slice(&25u32.to_le_bytes());
        bad_frame_stride[10..14].copy_from_slice(&36u32.to_le_bytes());
        bad_frame_stride[14..18].copy_from_slice(&50u32.to_le_bytes());
        bad_frame_stride.extend_from_slice(&[0, 0]);
        assert_eq!(
            AnbSections::parse(&bad_frame_stride).unwrap().ui_metadata().unwrap_err(),
            Error::AnbRecordSectionLength { section: "components", actual: 11, record_size: 10 }
        );

        let file = synthetic_metadata_anb();
        let metadata = AnbSections::parse(&file).unwrap().ui_metadata().unwrap();
        assert_eq!(metadata.animation(1), Err(Error::AnbAnimationOutOfBounds { animation: 1, count: 1 }));
        assert_eq!(metadata.frame_at(1), Err(Error::AnbFrameOutOfBounds { frame: 0, count: 1 }));
        assert_eq!(metadata.frame_component(metadata.frame_at(0).unwrap(), 1), Err(Error::AnbComponentRangeInvalid {
            frame: 0, offset: 0, count: 2, component_bytes: 10,
        }));
        assert_eq!(metadata.component(1), Err(Error::AnbComponentOutOfBounds { component: 1, count: 1 }));

        let mut invalid_component = synthetic_metadata_anb();
        invalid_component[24..28].copy_from_slice(&6u32.to_le_bytes());
        assert_eq!(
            AnbSections::parse(&invalid_component).unwrap().ui_metadata().unwrap().component(0),
            Err(Error::AnbSpriteReferenceInvalid { component: 0, offset: 6, sprite_records: 1 })
        );

        let mut invalid_animation = synthetic_metadata_anb();
        invalid_animation[48..52].copy_from_slice(&1u32.to_le_bytes());
        assert_eq!(
            AnbSections::parse(&invalid_animation).unwrap().ui_metadata().unwrap().animation(0),
            Err(Error::AnbFrameOffsetInvalid { animation: 0, offset: 1, frame_bytes: 14 })
        );

        let mut invalid_frame_components = synthetic_metadata_anb();
        invalid_frame_components[34..38].copy_from_slice(&10u32.to_le_bytes());
        assert_eq!(
            AnbSections::parse(&invalid_frame_components).unwrap().ui_metadata().unwrap().frame_at(0),
            Err(Error::AnbComponentRangeInvalid { frame: 0, offset: 10, count: 1, component_bytes: 10 })
        );
    }

    #[test]
    fn rejects_unordered_and_out_of_bounds_anb_offsets() {
        let mut unordered = synthetic_anb();
        unordered[6..10].copy_from_slice(&17u32.to_le_bytes());
        assert_eq!(
            AnbSections::parse(&unordered),
            Err(Error::AnbSectionOffsetsInvalid { offsets: [24, 17, 40, 48], file_len: unordered.len() })
        );

        let mut out_of_bounds = synthetic_anb();
        out_of_bounds[14..18].copy_from_slice(&u32::MAX.to_le_bytes());
        assert_eq!(
            AnbSections::parse(&out_of_bounds),
            Err(Error::AnbSectionOffsetsInvalid { offsets: [24, 30, 40, u32::MAX], file_len: out_of_bounds.len() })
        );
    }

    #[test]
    fn clips_runs_to_declared_width_but_consumes_full_literals() {
        let descriptor = SpriteDescriptor { data_offset: 0, width: 2, height: 1 };
        let mut database = vec![0; ANB_SPRITE_DATA_BASE];
        database.extend_from_slice(&[4, 9, 8, 7, 6, 0]);
        let sprite = decode_sprite_rle(0, descriptor, &database).unwrap();
        assert_eq!(sprite.indices, [9, 8]);
        assert_eq!(sprite.opaque, [true, true]);
        assert_eq!(sprite.consumed_bytes, 6);
    }

    #[test]
    fn decodes_explicit_packed4_high_first_odd_rows_and_transparent_skips() {
        let descriptor = SpriteDescriptor { data_offset: 0, width: 5, height: 2 };
        let mut database = vec![0; ANB_SPRITE_DATA_BASE];
        database.extend_from_slice(&[5, 0x12, 0x34, 0x59, 0, 0xfe, 3, 0x67, 0x80, 0]);
        let mut table_bytes = Vec::new();
        table_bytes.extend_from_slice(&descriptor.data_offset.to_le_bytes());
        table_bytes.extend_from_slice(&[descriptor.width, descriptor.height]);
        let table = SpriteTable::parse(&table_bytes, &database).unwrap();
        let sprite = table.decode_packed4(0, &database, 250).unwrap();
        assert_eq!(sprite.indices, [251, 252, 253, 254, 255, 0, 0, 0, 1, 2]);
        assert_eq!(sprite.opaque, [true, true, true, true, true, false, false, true, true, true]);
        assert_eq!(sprite.consumed_bytes, 10);
    }

    #[test]
    fn packed4_consumes_full_clipped_run_and_rejects_truncated_payload() {
        let descriptor = SpriteDescriptor { data_offset: 0, width: 3, height: 1 };
        let mut table_bytes = Vec::new();
        table_bytes.extend_from_slice(&0u32.to_le_bytes());
        table_bytes.extend_from_slice(&[descriptor.width, descriptor.height]);
        let mut database = vec![0; ANB_SPRITE_DATA_BASE];
        database.extend_from_slice(&[5, 0x12, 0x34, 0x56, 0]);
        let table = SpriteTable::parse(&table_bytes, &database).unwrap();
        let clipped = table.decode_packed4(0, &database, 0).unwrap();
        assert_eq!(clipped.indices, [1, 2, 3]);
        assert_eq!(clipped.opaque, [true; 3]);
        assert_eq!(clipped.consumed_bytes, 5);

        let truncated = [0u8; 18].into_iter().chain([3, 0x12]).collect::<Vec<_>>();
        let truncated_table = SpriteTable::parse(&table_bytes, &truncated).unwrap();
        assert_eq!(
            truncated_table.decode_packed4(0, &truncated, 0),
            Err(Error::SpriteStreamTruncated { record: 0, offset: 19 })
        );
    }

    #[test]
    fn rejects_malformed_sprite_tables_offsets_and_streams() {
        assert_eq!(
            SpriteTable::parse(&[0; 5], &[0; 32]),
            Err(Error::SpriteTableLength { actual: 5 })
        );
        let mut out_of_bounds = Vec::new();
        out_of_bounds.extend_from_slice(&100u32.to_le_bytes());
        out_of_bounds.extend_from_slice(&[1, 1]);
        assert_eq!(
            SpriteTable::parse(&out_of_bounds, &[0; 32]),
            Err(Error::SpriteOffsetOutOfBounds { record: 0, offset: 0x12 + 100, database_len: 32 })
        );
        let descriptor = SpriteDescriptor { data_offset: 0, width: 2, height: 1 };
        let mut truncated = vec![0; ANB_SPRITE_DATA_BASE];
        truncated.push(2);
        truncated.push(7);
        assert_eq!(
            decode_sprite_rle(0, descriptor, &truncated),
            Err(Error::SpriteStreamTruncated { record: 0, offset: ANB_SPRITE_DATA_BASE + 1 })
        );
    }

    #[test]
    fn rejects_output_completed_before_declared_chunks_finish() {
        let packed = one_literal_packed_with_chunks(b'Z', 2);
        let mut file = b"RNC\x01".to_vec();
        file.extend_from_slice(&1u32.to_be_bytes());
        file.extend_from_slice(&(packed.len() as u32).to_be_bytes());
        file.extend_from_slice(&crc16_rnc(b"Z").to_be_bytes());
        file.extend_from_slice(&crc16_rnc(&packed).to_be_bytes());
        file.extend_from_slice(&[0, 1]);
        file.extend_from_slice(&packed);
        assert!(matches!(decode(&file, 1), Err(Error::OutputLengthMismatch { declared: 1, actual: 1 })));
    }

    #[test]
    fn prefix_decode_reports_consumed_length_and_preserves_tail() {
        let packed = one_literal_packed(b'Z');
        let mut file = b"RNC\x01".to_vec();
        file.extend_from_slice(&1u32.to_be_bytes());
        file.extend_from_slice(&(packed.len() as u32).to_be_bytes());
        file.extend_from_slice(&crc16_rnc(b"Z").to_be_bytes());
        file.extend_from_slice(&crc16_rnc(&packed).to_be_bytes());
        file.extend_from_slice(&[0, 1]);
        file.extend_from_slice(&packed);
        let consumed = file.len();
        file.extend_from_slice(b"TAIL");
        assert_eq!(decode_prefix(&file, 1).unwrap(), (b"Z".to_vec(), consumed));
        assert!(matches!(decode(&file, 1), Err(Error::PackedLengthMismatch { .. })));
    }

    fn one_literal_packed(literal: u8) -> Vec<u8> {
        one_literal_packed_with_chunks(literal, 1)
    }

    fn one_literal_packed_with_chunks(literal: u8, chunks: usize) -> Vec<u8> {
        let mut values = vec![(0, 2), (2, 5), (1, 4), (1, 4),
            (1, 5), (1, 4), (1, 5), (1, 4), (chunks, 16), (1, 1)];
        let mut words = bits_to_bytes(&values);
        values.clear();
        while words.len() % 2 != 0 { words.push(0); }
        words.push(literal);
        words
    }

    fn one_literal_file(packed: &[u8], output: &[u8], block_count_hint: u8) -> Vec<u8> {
        let mut file = b"RNC\x01".to_vec();
        file.extend_from_slice(&(output.len() as u32).to_be_bytes());
        file.extend_from_slice(&(packed.len() as u32).to_be_bytes());
        file.extend_from_slice(&crc16_rnc(output).to_be_bytes());
        file.extend_from_slice(&crc16_rnc(packed).to_be_bytes());
        file.extend_from_slice(&[0, block_count_hint]);
        file.extend_from_slice(packed);
        file
    }

    fn synthetic_anb() -> Vec<u8> {
        let mut file = vec![0; 64];
        file[..2].copy_from_slice(&0xbeefu16.to_le_bytes());
        for (index, offset) in [24u32, 30, 40, 48].into_iter().enumerate() {
            let start = 2 + index * 4;
            file[start..start + 4].copy_from_slice(&offset.to_le_bytes());
        }
        file[24..28].copy_from_slice(&0u32.to_le_bytes());
        file[28..30].copy_from_slice(&[1, 1]);
        file[18] = 0;
        file
    }

    fn bits_to_bytes(fields: &[(usize, usize)]) -> Vec<u8> {
        let mut words = Vec::new();
        let mut bit = 0usize;
        for &(value, width) in fields {
            for i in 0..width {
                let absolute = bit + i;
                let byte = absolute / 8;
                if words.len() <= byte { words.push(0); }
                if (value >> i) & 1 != 0 { words[byte] |= 1 << (absolute % 8); }
            }
            bit += width;
        }
        words
    }
}

#[cfg(test)]
mod font_text_tests {
    use super::*;

    fn tab(records: &[(u32, u8, u8)]) -> Vec<u8> {
        let mut bytes = vec![0xA5; FONT_TAB_HEADER_BYTES];
        for &(offset, width, height) in records {
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&[width, height]);
        }
        bytes
    }

    #[test]
    fn font_records_start_after_opaque_header_and_codepoint_maps_once() {
        let data = [1, 9, 8, 0];
        let table = tab(&[(0, 2, 1), (3, 4, 1)]);
        let font = FontResource::parse(&data, &table).unwrap();
        assert_eq!(font.record_for_byte(32).unwrap(), FontGlyphRecord { record: 0, data_offset: 0, width: 2, height: 1 });
        assert_eq!(font.record_for_byte(33).unwrap(), FontGlyphRecord { record: 1, data_offset: 3, width: 4, height: 1 });
        assert_eq!(font.record_count(), 2);
        assert_eq!(font.record_for_byte(31), Err(Error::FontUnsupportedTextByte { byte: 31 }));
        assert!(matches!(font.record_for_byte(34), Err(Error::FontGlyphOutOfBounds { .. })));
        assert_eq!(font.record(usize::MAX), Err(Error::FontGlyphOutOfBounds { record: usize::MAX, records: 2 }));
    }

    #[test]
    fn font_rle_literal_zero_is_opaque_and_negative_run_is_transparent() {
        let data = [2, 0, 5, 0, 0xFD, 0];
        let table = tab(&[(0, 3, 2)]);
        let glyph = FontResource::parse(&data, &table).unwrap().decode_glyph(0).unwrap();
        assert_eq!(glyph.indices, [0, 5, 0, 0, 0, 0]);
        assert_eq!(glyph.opaque, [true, true, false, false, false, false]);
        assert_eq!(glyph.consumed_bytes, data.len());
    }

    #[test]
    fn font_rle_clips_writes_but_consumes_full_overwide_literal_run() {
        let data = [3, 7, 8, 9, 0];
        let table = tab(&[(0, 2, 1)]);
        let glyph = FontResource::parse(&data, &table).unwrap().decode_glyph(0).unwrap();
        assert_eq!(glyph.indices, [7, 8]);
        assert_eq!(glyph.opaque, [true, true]);
        assert_eq!(glyph.consumed_bytes, data.len());
    }

    #[test]
    fn font_parser_rejects_partial_tables_bad_offsets_and_truncated_streams() {
        assert_eq!(FontResource::parse(&[], &[0; 11]).unwrap_err(), Error::FontTableLength { actual: 11 });
        let bad_offset = tab(&[(7, 1, 1)]);
        assert_eq!(FontResource::parse(&[0], &bad_offset).unwrap().record(0), Err(Error::FontDataOffsetOutOfBounds { record: 0, offset: 7, data_len: 1 }));
        let truncated_table = tab(&[(0, 2, 1)]);
        let truncated = FontResource::parse(&[2, 9], &truncated_table).unwrap();
        assert_eq!(truncated.decode_glyph(0), Err(Error::FontGlyphStreamTruncated { record: 0, offset: 1 }));
    }

    #[test]
    fn text_table_consumes_only_the_proven_604_entries() {
        let bytes = [vec![0; TEXT_STRING_COUNT], vec![b'X', 0]].concat();
        let table = TextStringTable::parse(&bytes).unwrap();
        assert_eq!(table.strings().len(), 604);
        assert_eq!(table.consumed_len(), TEXT_STRING_COUNT);
        assert_eq!(table.trailing(), &[b'X', 0]);
        assert_eq!(TextStringTable::parse(&bytes[..TEXT_STRING_COUNT - 1]), Err(Error::TextTableTruncated { expected: 604, parsed: 603, offset: 603 }));
    }
}

#[cfg(test)]
mod campaign_header_tests {
    use super::*;

    #[test]
    fn campaign_header_exposes_only_the_bounded_string_and_keeps_all_bytes_opaque() {
        let mut bytes = vec![0xA5; CAMPAIGN_HEADER_BYTES];
        bytes[CAMPAIGN_HEADER_TITLE_OFFSET..CAMPAIGN_HEADER_TITLE_OFFSET + 3].copy_from_slice(b"ABC");
        bytes[CAMPAIGN_HEADER_TITLE_OFFSET + 3] = 0;
        let header = CampaignHeader::parse(&bytes).unwrap();
        assert_eq!(header.bytes(), bytes);
        assert_eq!(header.title_bytes(), b"ABC");
    }

    #[test]
    fn campaign_header_allows_empty_and_maximum_bounded_strings() {
        let mut empty = vec![0xFF; CAMPAIGN_HEADER_BYTES];
        empty[CAMPAIGN_HEADER_TITLE_OFFSET] = 0;
        assert_eq!(CampaignHeader::parse(&empty).unwrap().title_bytes(), b"");

        let mut maximum = vec![0x41; CAMPAIGN_HEADER_BYTES];
        maximum[CAMPAIGN_HEADER_TITLE_OFFSET + 666] = 0;
        assert_eq!(CampaignHeader::parse(&maximum).unwrap().title_bytes().len(), 666);
    }

    #[test]
    fn campaign_header_rejects_wrong_extents_and_missing_terminator() {
        assert_eq!(CampaignHeader::parse(&[0; CAMPAIGN_HEADER_BYTES - 1]), Err(Error::CampaignHeaderLength { expected: 699, actual: 698 }));
        assert_eq!(CampaignHeader::parse(&[0; CAMPAIGN_HEADER_BYTES + 1]), Err(Error::CampaignHeaderLength { expected: 699, actual: 700 }));
        assert_eq!(CampaignHeader::parse(&[0x41; CAMPAIGN_HEADER_BYTES]), Err(Error::CampaignHeaderTitleUnterminated { offset: 32, limit: 699 }));
    }
}

#[cfg(test)]
mod conq_fast_prefix_tests {
    use super::*;

    fn stream() -> Vec<u8> {
        let mut bytes = vec![0xA5; CONQ_FAST_PREFIX_BYTES + 7];
        bytes[..4].copy_from_slice(&CONQ_FAST_TAG.to_le_bytes());
        bytes
    }

    #[test]
    fn conq_fast_prefix_exposes_only_the_proven_copy_and_opaque_tail() {
        let bytes = stream();
        let prefix = ConqFastPrefix::parse(&bytes).unwrap();
        assert_eq!(prefix.tag(), CONQ_FAST_TAG);
        assert_eq!(prefix.copied_prefix(), &bytes[..CONQ_FAST_PREFIX_BYTES]);
        assert_eq!(prefix.trailing_bytes(), &[0xA5; 7]);
        assert_eq!(prefix.opaque_14_byte_region().len(), 0x38000);
    }

    #[test]
    fn conq_fast_prefix_exposes_four_bounded_initial_player_byte_ranges() {
        let bytes = stream();
        let prefix = ConqFastPrefix::parse(&bytes).unwrap();
        for index in 0..CONQ_INITIAL_PLAYER_COUNT {
            assert_eq!(prefix.initial_player_record(index).unwrap().len(), CONQ_INITIAL_PLAYER_RECORD_BYTES);
        }
        assert_eq!(prefix.initial_player_record(4), Err(Error::ConqInitialPlayerOutOfBounds { index: 4, count: 4 }));
        assert_eq!(prefix.initial_player_record(usize::MAX), Err(Error::ConqInitialPlayerOutOfBounds { index: usize::MAX, count: 4 }));
    }

    #[test]
    fn conq_fast_prefix_rejects_short_inputs_and_other_loader_tags() {
        let mut short = stream();
        short.truncate(CONQ_FAST_PREFIX_BYTES - 1);
        assert_eq!(ConqFastPrefix::parse(&short), Err(Error::ConqFastPrefixTruncated { required: CONQ_FAST_PREFIX_BYTES, actual: CONQ_FAST_PREFIX_BYTES - 1 }));

        let mut unsupported = stream();
        unsupported[..4].copy_from_slice(&0x12345678u32.to_le_bytes());
        assert_eq!(ConqFastPrefix::parse(&unsupported), Err(Error::ConqUnsupportedFastTag { actual: 0x12345678 }));
    }
}

//! Bounded views of the traced CONQ map cell region and B32 terrain atlas.
//!
//! Cell fields other than height (+2) and texture selector (+4) remain opaque.
//! The tile API returns indexed bytes only. Palette and rendering policy are
//! caller supplied; zero is not assigned transparency semantics here.

use core::fmt;

pub const MAP_SIDE: usize = 128;
pub const MAP_CELL_BYTES: usize = 14;
pub const MAP_CELLS_BYTES: usize = MAP_SIDE * MAP_SIDE * MAP_CELL_BYTES;
pub const B32_TILE_SIDE: usize = 32;
pub const B32_TILE_BYTES: usize = B32_TILE_SIDE * B32_TILE_SIDE;
pub const B32_TILE_COUNT: usize = 160;
pub const B32_SOURCE_STRIDE: usize = 256;
pub const B32_TILES_PER_ROW: usize = 8;
pub const B32_ATLAS_BYTES: usize = B32_TILE_COUNT * B32_TILE_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MapRegionLength { expected: usize, actual: usize },
    CellCoordinateOutOfRange { x: usize, y: usize },
    AtlasLength { expected: usize, actual: usize },
    TileOutOfRange { index: usize },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for Error {}

/// Borrowed view of exactly 16,384 consecutive 14-byte map cells.
#[derive(Debug, Clone, Copy)]
pub struct ConqMapCells<'a> {
    bytes: &'a [u8],
}

impl<'a> ConqMapCells<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() != MAP_CELLS_BYTES {
            return Err(Error::MapRegionLength { expected: MAP_CELLS_BYTES, actual: bytes.len() });
        }
        Ok(Self { bytes })
    }

    pub fn bytes(&self) -> &'a [u8] { self.bytes }

    pub fn cell(&self, x: usize, y: usize) -> Result<ConqMapCell<'a>, Error> {
        if x >= MAP_SIDE || y >= MAP_SIDE {
            return Err(Error::CellCoordinateOutOfRange { x, y });
        }
        let offset = (y * MAP_SIDE + x) * MAP_CELL_BYTES;
        // Coordinates and exact region length prove this complete record exists.
        let bytes = &self.bytes[offset..offset + MAP_CELL_BYTES];
        Ok(ConqMapCell { bytes })
    }
}

/// Borrowed cell record, with only instruction-confirmed fields named.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConqMapCell<'a> {
    bytes: &'a [u8],
}

impl<'a> ConqMapCell<'a> {
    pub fn raw(&self) -> &'a [u8; MAP_CELL_BYTES] {
        self.bytes.try_into().expect("validated fixed-size map cell")
    }

    pub fn height(&self) -> i16 {
        i16::from_le_bytes([self.bytes[2], self.bytes[3]])
    }

    pub fn texture_selector(&self) -> u8 { self.bytes[4] }

    /// Return the traced B32 orientation bits from cell byte 5 (mask `0x70`).
    pub fn orientation_flags(&self) -> u8 { self.bytes[5] & 0x70 }
}

/// Borrowed B32 atlas using the traced 256-byte source row pitch.
#[derive(Debug, Clone, Copy)]
pub struct B32Atlas<'a> {
    bytes: &'a [u8],
}

impl<'a> B32Atlas<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.len() != B32_ATLAS_BYTES {
            return Err(Error::AtlasLength { expected: B32_ATLAS_BYTES, actual: bytes.len() });
        }
        Ok(Self { bytes })
    }

    pub fn tile(&self, index: usize) -> Result<[u8; B32_TILE_BYTES], Error> {
        if index >= B32_TILE_COUNT { return Err(Error::TileOutOfRange { index }); }
        let source_x = (index % B32_TILES_PER_ROW) * B32_TILE_SIDE;
        let source_y = (index / B32_TILES_PER_ROW) * B32_TILE_SIDE;
        let mut out = [0u8; B32_TILE_BYTES];
        for row in 0..B32_TILE_SIDE {
            let start = (source_y + row) * B32_SOURCE_STRIDE + source_x;
            let end = start + B32_TILE_SIDE;
            out[row * B32_TILE_SIDE..(row + 1) * B32_TILE_SIDE]
                .copy_from_slice(&self.bytes[start..end]);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_fields_are_little_endian_and_other_bytes_remain_raw() {
        let mut bytes = vec![0xA5; MAP_CELLS_BYTES];
        let offset = (127 * MAP_SIDE + 126) * MAP_CELL_BYTES;
        bytes[offset + 2..offset + 4].copy_from_slice(&(-12345i16).to_le_bytes());
        bytes[offset + 4] = 119;
        bytes[offset + 5] = 0xD7;
        let map = ConqMapCells::parse(&bytes).unwrap();
        let cell = map.cell(126, 127).unwrap();
        assert_eq!(cell.height(), -12345);
        assert_eq!(cell.texture_selector(), 119);
        assert_eq!(cell.orientation_flags(), 0x50);
        assert_eq!(cell.raw()[..2], [0xA5, 0xA5]);
        assert_eq!(cell.raw()[5..], [0xD7, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5, 0xA5]);
        assert_eq!(map.cell(128, 0), Err(Error::CellCoordinateOutOfRange { x: 128, y: 0 }));
    }

    #[test]
    fn map_region_requires_exact_traced_extent() {
        assert!(matches!(ConqMapCells::parse(&vec![0; MAP_CELLS_BYTES - 1]), Err(Error::MapRegionLength { .. })));
        let mut extra = vec![0; MAP_CELLS_BYTES + 1];
        assert!(matches!(ConqMapCells::parse(&extra), Err(Error::MapRegionLength { .. })));
        extra.pop();
        assert!(ConqMapCells::parse(&extra).is_ok());
    }

    #[test]
    fn b32_tiles_use_eight_columns_and_skip_source_row_padding() {
        let mut bytes = vec![0xEE; B32_ATLAS_BYTES];
        for tile in 0..B32_TILE_COUNT {
            let x = (tile % B32_TILES_PER_ROW) * B32_TILE_SIDE;
            let y = (tile / B32_TILES_PER_ROW) * B32_TILE_SIDE;
            for row in 0..B32_TILE_SIDE {
                for col in 0..B32_TILE_SIDE {
                    bytes[(y + row) * B32_SOURCE_STRIDE + x + col] = tile as u8;
                }
            }
        }
        let atlas = B32Atlas::parse(&bytes).unwrap();
        for tile in [0, 7, 8, 159] {
            assert!(atlas.tile(tile).unwrap().iter().all(|&v| v == tile as u8));
        }
        assert_eq!(atlas.tile(160), Err(Error::TileOutOfRange { index: 160 }));
        assert!(matches!(B32Atlas::parse(&bytes[..bytes.len() - 1]), Err(Error::AtlasLength { .. })));
    }
}

//! Explicit-input terrain vertex and B32 UV primitives from the static receipt.
//!
//! These helpers do not choose a camera, build a scene, clip, shade, or
//! rasterize terrain. The vertex primitive accepts explicit unwrapped map and
//! camera coordinates plus a caller placement. UV pairs are the four entries
//! of the verified table in `[00, 10, 11, 01]` order; this module does not
//! interpolate between them.

use core::fmt;

pub const TERRAIN_HORIZONTAL_STEP: i32 = 32;
pub const TERRAIN_DIAGONAL_VERTICAL_STEP: i32 = 16;
pub const B32_UV_MAX_16_16: u32 = 0x001f_ffff;
pub const B32_UV_ORIENTATION_COUNT: usize = 8;
pub const CELL_TRIANGLES: [[usize; 3]; 2] = [[0, 2, 3], [0, 1, 2]];
pub const TERRAIN_FADE_SHADE_COUNT: usize = 64;
pub const TERRAIN_FADE_ROW_BYTES: usize = 256;
pub const TERRAIN_FADE_TABLE_BYTES: usize = TERRAIN_FADE_SHADE_COUNT * TERRAIN_FADE_ROW_BYTES;
pub const MODE5_TEXTURE_ROW_STRIDE: usize = 256;

const B32_UV_CORNERS: [[(u32, u32); 4]; B32_UV_ORIENTATION_COUNT] = [
    [(0, 0), (B32_UV_MAX_16_16, 0), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (0, B32_UV_MAX_16_16)],
    [(B32_UV_MAX_16_16, 0), (0, 0), (0, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, B32_UV_MAX_16_16)],
    [(0, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, 0), (0, 0)],
    [(B32_UV_MAX_16_16, B32_UV_MAX_16_16), (0, B32_UV_MAX_16_16), (0, 0), (B32_UV_MAX_16_16, 0)],
    [(0, 0), (0, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, 0)],
    [(0, B32_UV_MAX_16_16), (0, 0), (B32_UV_MAX_16_16, 0), (B32_UV_MAX_16_16, B32_UV_MAX_16_16)],
    [(B32_UV_MAX_16_16, 0), (B32_UV_MAX_16_16, B32_UV_MAX_16_16), (0, B32_UV_MAX_16_16), (0, 0)],
    [(B32_UV_MAX_16_16, B32_UV_MAX_16_16), (B32_UV_MAX_16_16, 0), (0, 0), (0, B32_UV_MAX_16_16)],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainPrimitiveError {
    UvOrientationOutOfRange { orientation: u8 },
    CoordinateOverflow,
    FadeTableLength { expected: usize, actual: usize },
    FadeShadeIndexOutOfRange { shade_index: u8 },
    TextureSampleOutOfBounds { offset: usize, available: usize },
}

impl fmt::Display for TerrainPrimitiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for TerrainPrimitiveError {}

/// Return four 16.16 UV pairs for one explicitly selected table row.
pub fn b32_uv_corners(orientation: u8) -> Result<[(u32, u32); 4], TerrainPrimitiveError> {
    B32_UV_CORNERS.get(usize::from(orientation)).copied()
        .ok_or(TerrainPrimitiveError::UvOrientationOutOfRange { orientation })
}

/// Select the UV row using bits 4–6 of a map-cell flag byte.
pub fn b32_uv_corners_for_cell_flags(flags: u8) -> [(u32, u32); 4] {
    B32_UV_CORNERS[usize::from((flags & 0x70) >> 4)]
}

/// Convert a signed map-cell height to the traced vertex height displacement.
/// This is `2 * (height >> 6)`; shifting first is observably different from
/// `(2 * height) >> 6` for nonmultiples of 64 and negative values.
pub fn terrain_vertex_height_displacement(height: i16) -> i32 {
    2 * (i32::from(height) >> 6)
}

/// Compute the traced signed 16.16 shade from explicit raw operands.
/// The source factor's producer and gameplay meaning remain unresolved.
pub fn terrain_vertex_shade_16_16(light_operand: i16, factor_operand: u8) -> i32 {
    4 * i32::from(light_operand) * i32::from(factor_operand) + 0x8000
}

/// Borrowed view of the 64×256 FADE.DAT lookup table.
#[derive(Debug, Clone, Copy)]
pub struct TerrainFadeTable<'a> {
    bytes: &'a [u8],
}

impl<'a> TerrainFadeTable<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, TerrainPrimitiveError> {
        if bytes.len() != TERRAIN_FADE_TABLE_BYTES {
            return Err(TerrainPrimitiveError::FadeTableLength {
                expected: TERRAIN_FADE_TABLE_BYTES,
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes })
    }

    /// Look up one caller-selected shade row and palette index. This does not
    /// infer a row from the 16.16 shade calculation or choose lighting inputs.
    pub fn map_palette_index(
        &self,
        shade_index: u8,
        source_palette_index: u8,
    ) -> Result<u8, TerrainPrimitiveError> {
        if usize::from(shade_index) >= TERRAIN_FADE_SHADE_COUNT {
            return Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index });
        }
        let offset = usize::from(shade_index) * TERRAIN_FADE_ROW_BYTES + usize::from(source_palette_index);
        Ok(self.bytes[offset])
    }
}

/// Perform one explicit mode-5 texture/fade sample. `u` and `v` are the
/// integer-byte coordinates supplied to the sampler; shade row selection is
/// also explicit. Every source index, including zero, is passed through FADE.
pub fn mode5_sample_index(
    texture: &[u8],
    fade: &TerrainFadeTable<'_>,
    u_integer_byte: u8,
    v_integer_byte: u8,
    shade_index: u8,
) -> Result<u8, TerrainPrimitiveError> {
    let offset = usize::from(v_integer_byte) * MODE5_TEXTURE_ROW_STRIDE
        + usize::from(u_integer_byte);
    let source_index = *texture.get(offset).ok_or(TerrainPrimitiveError::TextureSampleOutOfBounds {
        offset,
        available: texture.len(),
    })?;
    fade.map_palette_index(shade_index, source_index)
}

/// Convert an explicitly supplied unwrapped map cell into buffer coordinates.
///
/// `map_cell` and `camera_cell` are integer cell coordinates; they are not
/// masked to 0..127. Callers select the neighboring unwrapped cell while map
/// data lookup separately wraps its indices. The returned position is
/// `screen_anchor + ((dx-dy)*32, (dx+dy)*16 - 2*(height>>6) - vertical_top)`.
/// `screen_anchor` and `vertical_top` are explicit placement inputs, not a
/// claim about the original startup camera. Unlike the source's 32-bit wrap,
/// overflow is rejected for this safe helper.
pub fn terrain_vertex_buffer_position(
    map_cell: [i32; 2],
    camera_cell: [i32; 2],
    height: i16,
    vertical_top: i32,
    screen_anchor: [i32; 2],
) -> Result<[i32; 2], TerrainPrimitiveError> {
    let dx = map_cell[0].checked_sub(camera_cell[0]).ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    let dy = map_cell[1].checked_sub(camera_cell[1]).ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    let x = dx.checked_sub(dy)
        .and_then(|v| v.checked_mul(TERRAIN_HORIZONTAL_STEP))
        .and_then(|v| v.checked_add(screen_anchor[0]))
        .ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    let y = dx.checked_add(dy)
        .and_then(|v| v.checked_mul(TERRAIN_DIAGONAL_VERTICAL_STEP))
        .and_then(|v| v.checked_sub(terrain_vertex_height_displacement(height)))
        .and_then(|v| v.checked_sub(vertical_top))
        .and_then(|v| v.checked_add(screen_anchor[1]))
        .ok_or(TerrainPrimitiveError::CoordinateOverflow)?;
    Ok([x, y])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uv_table_matches_all_eight_corner_permutations() {
        let m = B32_UV_MAX_16_16;
        let expected = [
            [(0, 0), (m, 0), (m, m), (0, m)],
            [(m, 0), (0, 0), (0, m), (m, m)],
            [(0, m), (m, m), (m, 0), (0, 0)],
            [(m, m), (0, m), (0, 0), (m, 0)],
            [(0, 0), (0, m), (m, m), (m, 0)],
            [(0, m), (0, 0), (m, 0), (m, m)],
            [(m, 0), (m, m), (0, m), (0, 0)],
            [(m, m), (m, 0), (0, 0), (0, m)],
        ];
        for orientation in 0..8 {
            assert_eq!(b32_uv_corners(orientation).unwrap(), expected[usize::from(orientation)]);
            assert_eq!(b32_uv_corners_for_cell_flags(orientation << 4), expected[usize::from(orientation)]);
            assert_eq!(b32_uv_corners_for_cell_flags((orientation << 4) | 0x8f), expected[usize::from(orientation)]);
        }
        assert_eq!(b32_uv_corners(8), Err(TerrainPrimitiveError::UvOrientationOutOfRange { orientation: 8 }));
    }

    #[test]
    fn vertex_height_shifts_signed_value_before_doubling() {
        assert_eq!(terrain_vertex_height_displacement(0), 0);
        assert_eq!(terrain_vertex_height_displacement(63), 0);
        assert_eq!(terrain_vertex_height_displacement(64), 2);
        assert_eq!(terrain_vertex_height_displacement(-1), -2);
        assert_eq!(terrain_vertex_height_displacement(-63), -2);
        assert_eq!(terrain_vertex_height_displacement(-64), -2);
        assert_eq!(terrain_vertex_height_displacement(-65), -4);
        assert_eq!(terrain_vertex_height_displacement(i16::MIN), -1024);
        assert_eq!(terrain_vertex_height_displacement(i16::MAX), 1022);
    }

    #[test]
    fn shade_calculation_matches_signed_16_16_vectors_and_extrema() {
        for (light, factor, expected) in [
            (0, 0, 32768),
            (8192, 0, 32768),
            (8192, 64, 2129920),
            (3840, 64, 1015808),
            (8192, 255, 8388608),
            (-1, 64, 32512),
            (i16::MIN, 255, -33390592),
            (i16::MAX, 255, 33455108),
        ] {
            assert_eq!(terrain_vertex_shade_16_16(light, factor), expected);
        }
    }

    #[test]
    fn fade_lookup_uses_explicit_bounded_row_and_palette_index() {
        let mut bytes = vec![0u8; TERRAIN_FADE_TABLE_BYTES];
        for shade in 0..TERRAIN_FADE_SHADE_COUNT {
            for palette in 0..TERRAIN_FADE_ROW_BYTES {
                bytes[shade * TERRAIN_FADE_ROW_BYTES + palette] = (shade as u8) ^ (palette as u8);
            }
        }
        let table = TerrainFadeTable::parse(&bytes).unwrap();
        assert_eq!(table.map_palette_index(0, 0), Ok(0));
        assert_eq!(table.map_palette_index(32, 1), Ok(33));
        assert_eq!(table.map_palette_index(63, 255), Ok(192));
        assert_eq!(table.map_palette_index(64, 0), Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 }));
        assert!(matches!(TerrainFadeTable::parse(&bytes[..bytes.len() - 1]), Err(TerrainPrimitiveError::FadeTableLength { .. })));
        let mut extra = bytes;
        extra.push(0);
        assert!(matches!(TerrainFadeTable::parse(&extra), Err(TerrainPrimitiveError::FadeTableLength { .. })));
    }

    #[test]
    fn mode5_sample_uses_vu_address_then_fade_even_for_zero_source() {
        let mut texture = vec![0xA5; 65_536];
        let offset = 7 * MODE5_TEXTURE_ROW_STRIDE + 19;
        texture[offset] = 0;
        let mut fade_bytes = vec![0; TERRAIN_FADE_TABLE_BYTES];
        fade_bytes[11 * TERRAIN_FADE_ROW_BYTES] = 0xD3;
        let fade = TerrainFadeTable::parse(&fade_bytes).unwrap();
        assert_eq!(mode5_sample_index(&texture, &fade, 19, 7, 11), Ok(0xD3));
        assert_eq!(mode5_sample_index(&texture[..offset], &fade, 19, 7, 11), Err(TerrainPrimitiveError::TextureSampleOutOfBounds { offset, available: offset }));
        assert_eq!(mode5_sample_index(&texture, &fade, 19, 7, 64), Err(TerrainPrimitiveError::FadeShadeIndexOutOfRange { shade_index: 64 }));
    }

    #[test]
    fn vertex_position_uses_unwrapped_deltas_and_explicit_placement() {
        // dx=3,dy=2; height65 displaces 2 pixels and top contributes 7.
        assert_eq!(terrain_vertex_buffer_position([4, 3], [1, 1], 65, 7, [100, 200]), Ok([132, 271]));
        // A seam neighbor is represented as cell128, not wrapped to zero.
        assert_eq!(terrain_vertex_buffer_position([128, 0], [127, 0], 0, 0, [0, 0]), Ok([32, 16]));
        assert_eq!(terrain_vertex_buffer_position([0, 0], [1, 0], 0, 0, [640, 480]), Ok([608, 464]));
    }

    #[test]
    fn vertex_position_rejects_i32_overflow() {
        assert_eq!(terrain_vertex_buffer_position([i32::MAX, 0], [i32::MIN, 0], 0, 0, [0, 0]), Err(TerrainPrimitiveError::CoordinateOverflow));
        assert_eq!(terrain_vertex_buffer_position([i32::MAX, 0], [0, 0], 0, 0, [i32::MAX, 0]), Err(TerrainPrimitiveError::CoordinateOverflow));
    }
}

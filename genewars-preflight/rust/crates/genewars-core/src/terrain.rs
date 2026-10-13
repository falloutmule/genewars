//! Fixed-point height interpolation for the proven 128×128 CONQ cell region.

use core::fmt;

pub const MAP_SIDE: usize = 128;
pub const CELL_BYTES: usize = 14;
pub const HEIGHT_OFFSET: usize = 2;
pub const MAP_REGION_BYTES: usize = MAP_SIDE * MAP_SIDE * CELL_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainError {
    pub expected: usize,
    pub actual: usize,
}

impl fmt::Display for TerrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "terrain map requires {} bytes, received {}", self.expected, self.actual)
    }
}
impl std::error::Error for TerrainError {}

/// Interpolate and return the doubled signed height at a signed 8.8 coordinate.
///
/// The input must be exactly the traced 16,384-cell region. Coordinates wrap
/// both cell axes modulo 128. Arithmetic and the final shift are signed integer
/// operations; `>> 7` therefore preserves arithmetic-floor behavior.
pub fn interpolate_height(map: &[u8], x: i16, y: i16) -> Result<i32, TerrainError> {
    if map.len() != MAP_REGION_BYTES {
        return Err(TerrainError { expected: MAP_REGION_BYTES, actual: map.len() });
    }

    let ux = x as u16;
    let uy = y as u16;
    let fx = i32::from((ux & 0x00ff) as u8);
    let fy = i32::from((uy & 0x00ff) as u8);
    let cx = usize::from((ux >> 8) & 127);
    let cy = usize::from((uy >> 8) & 127);
    let nx = (cx + 1) & 127;
    let ny = (cy + 1) & 127;

    let h00 = height(map, cx, cy);
    let h10 = height(map, nx, cy);
    let h01 = height(map, cx, ny);
    let h11 = height(map, nx, ny);

    let numerator = if fx < fy {
        (h11 - h01) * fx + (h01 - h00) * fy
    } else {
        (h10 - h00) * fx + (h11 - h10) * fy
    };
    Ok(2 * h00 + (numerator >> 7))
}

#[inline]
fn height(map: &[u8], x: usize, y: usize) -> i32 {
    let offset = (y * MAP_SIDE + x) * CELL_BYTES + HEIGHT_OFFSET;
    i32::from(i16::from_le_bytes([map[offset], map[offset + 1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map_with_heights(entries: &[(usize, usize, i16)]) -> Vec<u8> {
        let mut map = vec![0; MAP_REGION_BYTES];
        for &(x, y, value) in entries {
            let offset = (y * MAP_SIDE + x) * CELL_BYTES + HEIGHT_OFFSET;
            map[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        map
    }

    #[test]
    fn samples_each_piecewise_triangle_and_vertices() {
        // h00=0, h10=100, h01=200, h11=400.
        let map = map_with_heights(&[(1, 1, 0), (2, 1, 100), (1, 2, 200), (2, 2, 400)]);
        assert_eq!(interpolate_height(&map, 0x0100, 0x0100), Ok(0));
        assert_eq!(interpolate_height(&map, 0x0200, 0x0100), Ok(200));
        assert_eq!(interpolate_height(&map, 0x0100, 0x0200), Ok(400));
        assert_eq!(interpolate_height(&map, 0x0180, 0x01c0), Ok(500)); // fx < fy
        assert_eq!(interpolate_height(&map, 0x01c0, 0x0180), Ok(450)); // fx >= fy
    }

    #[test]
    fn negative_fraction_uses_arithmetic_floor_shift() {
        let map = map_with_heights(&[(0, 0, 0), (1, 0, -1), (0, 1, 0), (1, 1, 0)]);
        // Numerator=-1; arithmetic shift is -1, not truncation toward zero.
        assert_eq!(interpolate_height(&map, 0x0001, 0x0000), Ok(-1));
    }

    #[test]
    fn signed_coordinates_wrap_both_cell_axes_and_neighbor_edges() {
        let map = map_with_heights(&[(127, 127, 100), (0, 127, 200), (127, 0, 300), (0, 0, 400)]);
        assert_eq!(interpolate_height(&map, i16::MIN, i16::MIN), Ok(800));
        assert_eq!(interpolate_height(&map, -1, -1), Ok(797));
        assert_eq!(interpolate_height(&map, i16::MAX, i16::MAX), Ok(797));
    }

    #[test]
    fn extrema_fit_and_wrong_map_extent_is_rejected() {
        let map = map_with_heights(&[(0, 0, i16::MIN), (1, 0, i16::MAX), (0, 1, i16::MAX), (1, 1, i16::MIN)]);
        assert!(interpolate_height(&map, 255, 255).is_ok());
        assert_eq!(interpolate_height(&map[..MAP_REGION_BYTES - 1], 0, 0), Err(TerrainError { expected: MAP_REGION_BYTES, actual: MAP_REGION_BYTES - 1 }));
        let mut extra = map;
        extra.push(0);
        assert_eq!(interpolate_height(&extra, 0, 0), Err(TerrainError { expected: MAP_REGION_BYTES, actual: MAP_REGION_BYTES + 1 }));
    }
}

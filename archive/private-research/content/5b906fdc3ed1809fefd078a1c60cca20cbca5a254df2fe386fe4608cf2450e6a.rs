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

/// Compute only the two camera-cell words written by CD helper 0x82E20.
///
/// The target is explicit. This does not select a mission's initial target or
/// reproduce that helper's viewport, minimap, sound, and redraw side effects.
/// First arithmetic-shift the interpolated doubled height by six, then divide
/// by 32 with truncation toward zero. Coordinate subtraction wraps as a byte;
/// the stored words zero-extend those bytes, so their domain is 0..=255.
pub fn camera_cells_for_target(map: &[u8], x: i16, y: i16) -> Result<[u16; 2], TerrainError> {
    let doubled_height = interpolate_height(map, x, y)?;
    let correction = (doubled_height >> 6) / 32;
    let cx = ((i32::from(x) >> 8) - correction - 12) as u8;
    let cy = ((i32::from(y) >> 8) - correction - 2) as u8;
    Ok([u16::from(cx), u16::from(cy)])
}

/// Project a world position into the local scene coordinates of CD0x7AF00.
///
/// Relative coordinates wrap into signed15-bit deltas on the128-cell torus.
/// Height is interpolated before its arithmetic shift, unlike the terrain
/// vertex's doubled already-shifted height. The panel margin is an unsigned
/// word; results are full signed32-bit values, including off-screen points.
/// Downstream object storage, visibility, sorting and drawing are separate.
pub fn entity_view_position(
    map: &[u8], position: [i16; 2], camera: [u16; 2], panel_margin: u16,
) -> Result<[i32; 2], TerrainError> {
    let doubled_height = interpolate_height(map, position[0], position[1])?;
    let delta = |value: i16, origin: u16| {
        let word = (value as u16).wrapping_sub(origin.wrapping_shl(8));
        i32::from((word.wrapping_shl(1) as i16) >> 1)
    };
    let dx = delta(position[0], camera[0]);
    let dy = delta(position[1], camera[1]);
    Ok([(((dx - dy) * 32) >> 8) - i32::from(panel_margin),
        (((dx + dy) * 16) >> 8) - (doubled_height >> 6)])
}

/// Return the bounded entity-shade result of CD helper 0x7B240.
///
/// Multiply signed cell+0 by unsigned cell+12 and arithmetic-shift each
/// vertex by six before triangular interpolation. The final bias and clamp
/// are specific to this helper; this is not a terrain FADE table row, whose
/// upper bound is 63. No entity or map state is changed.
pub fn interpolate_entity_shade(map: &[u8], x: i16, y: i16) -> Result<u8, TerrainError> {
    if map.len() != MAP_REGION_BYTES {
        return Err(TerrainError { expected: MAP_REGION_BYTES, actual: map.len() });
    }
    let (ux, uy) = (x as u16, y as u16);
    let (fx, fy) = (i32::from(ux & 255), i32::from(uy & 255));
    let (cx, cy) = (usize::from((ux >> 8) & 127), usize::from((uy >> 8) & 127));
    let vertex = |dx: usize, dy: usize| {
        let offset = (((cy + dy) & 127) * MAP_SIDE + ((cx + dx) & 127)) * CELL_BYTES;
        let light = i32::from(i16::from_le_bytes([map[offset], map[offset + 1]]));
        (light * i32::from(map[offset + 12])) >> 6
    };
    let (a, b, c, d) = (vertex(0, 0), vertex(1, 0), vertex(0, 1), vertex(1, 1));
    let numerator = if fx < fy { (d - c) * fx + (c - a) * fy }
        else { (b - a) * fx + (d - b) * fy };
    let doubled = 2 * a + (numerator >> 7);
    Ok((((doubled - 0x1e00) >> 8) + 16).clamp(0, 64) as u8)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerrainFactorChange {
    /// The original byte-grid position passed to the change callback.
    pub grid_position: [i16; 2],
    /// Wrapped source-map indices, separate from the callback coordinates.
    pub source_cell: [u8; 2],
    pub previous: u8,
    pub next: u8,
}

/// Plan the local cell+12 increases in CD 0x75B00 for radius 0x1000 only.
///
/// This pure helper does not modify the map or execute the original callback's
/// dirty-region, minimap, entity-shade, or virtual-predicate side effects.
/// Consequently it is not a complete visibility update or an initial mission
/// state. An explicit immutable map snapshot supplies all heights and factors.
/// Notification positions retain the original row-major byte-grid scan order.
pub fn plan_terrain_factor_raise_4096(map: &[u8], target: [i16; 2]) -> Result<Vec<TerrainFactorChange>, TerrainError> {
    let center_height = interpolate_height(map, target[0], target[1])?;
    let first_x = ((target[0] as u16).wrapping_sub(4096) >> 8) as u8;
    let first_y = ((target[1] as u16).wrapping_sub(4096) >> 8) as u8;
    let mut changes = Vec::new();
    for row in 0..33u8 {
        let gy = first_y.wrapping_add(row);
        for column in 0..33u8 {
            let gx = first_x.wrapping_add(column);
            let sx = gx & 127;
            let sy = gy & 127;
            let offset = (usize::from(sy) * MAP_SIDE + usize::from(sx)) * CELL_BYTES;
            let previous = map[offset + 12];
            if previous >= 63 { continue; }
            let point = [(u16::from(gx) << 8) as i16, (u16::from(gy) << 8) as i16];
            // At an aligned vertex, the height sampler returns twice its i16.
            let vertical = (2 * height(map, usize::from(sx), usize::from(sy)) - center_height) >> 3;
            let distance = crate::cd_candidate::distance_squared_wrapping_15(target, point)
                + (vertical * vertical) as u32;
            let next = if distance < 3840 * 3840 {
                63
            } else if distance < 4096 * 4096 {
                ((4096 - crate::cd_candidate::integer_sqrt(distance) - 1) / 4) as u8
            } else {
                continue;
            };
            if next > previous {
                changes.push(TerrainFactorChange { grid_position: point, source_cell: [sx, sy], previous, next });
            }
        }
    }
    Ok(changes)
}

#[inline]
fn height(map: &[u8], x: usize, y: usize) -> i32 {
    let offset = (y * MAP_SIDE + x) * CELL_BYTES + HEIGHT_OFFSET;
    i32::from(i16::from_le_bytes([map[offset], map[offset + 1]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_shade_multiplies_before_interpolation_and_keeps_64_endpoint() {
        let mut map = vec![0u8; MAP_REGION_BYTES];
        let set = |map: &mut [u8], x: usize, y: usize, light: i16, factor: u8| {
            let offset = (y * MAP_SIDE + x) * CELL_BYTES;
            map[offset..offset + 2].copy_from_slice(&light.to_le_bytes());
            map[offset + 12] = factor;
        };
        set(&mut map, 0, 0, 3840, 64);
        set(&mut map, 1, 0, 8192, 63);
        set(&mut map, 0, 1, 4097, 63);
        set(&mut map, 1, 1, 8192, 63);
        assert_eq!(interpolate_entity_shade(&map, 0, 0), Ok(16));
        assert_eq!(interpolate_entity_shade(&map, 256, 0), Ok(49));
        assert_eq!(interpolate_entity_shade(&map, 128, 192), Ok(32));
        assert_eq!(interpolate_entity_shade(&map, 192, 128), Ok(40));
        set(&mut map, 1, 1, 16384, 63);
        assert_eq!(interpolate_entity_shade(&map, 256, 256), Ok(64));
        set(&mut map, 0, 0, i16::MIN, 255);
        assert_eq!(interpolate_entity_shade(&map, i16::MIN, i16::MIN), Ok(0));
        assert_eq!(interpolate_entity_shade(&map, -1, -1), Ok(0));
        assert!(interpolate_entity_shade(&map[..map.len() - 1], 0, 0).is_err());
    }

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

    #[test]
    fn target_camera_zero_extends_wrapped_bytes_and_truncates_negative_correction() {
        let map = map_with_heights(&[]);
        assert_eq!(camera_cells_for_target(&map, 0, 0), Ok([244, 254]));
        assert_eq!(camera_cells_for_target(&map, -256, i16::MIN), Ok([243, 126]));
        // A negative sub-unit height correction truncates to zero, not -1.
        let low = map_with_heights(&[(0, 0, -1)]);
        assert_eq!(camera_cells_for_target(&low, 0, 0), Ok([244, 254]));
        let min = map_with_heights(&[(0, 0, i16::MIN)]);
        assert_eq!(camera_cells_for_target(&min, 0, 0), Ok([20, 30]));
        let max = map_with_heights(&[(0, 0, i16::MAX)]);
        assert_eq!(camera_cells_for_target(&max, 0, 0), Ok([213, 223]));
        assert!(camera_cells_for_target(&map[..map.len() - 1], 0, 0).is_err());
    }

    #[test]
    fn entity_projection_wraps_signed15_deltas_and_keeps_full_width_outputs() {
        let map = map_with_heights(&[]);
        assert_eq!(entity_view_position(&map, [-1, -1], [0, 0], 128), Ok([-128, -1]));
        assert_eq!(entity_view_position(&map, [16384, 0], [0, 0], 128), Ok([-2176, -1024]));
        assert_eq!(entity_view_position(&map, [i16::MIN, i16::MIN], [0, 0], 128), Ok([-128, 0]));
        assert_eq!(entity_view_position(&map, [0, 0], [65535, 65535], 65535), Ok([-65535, 32]));
        assert_eq!(entity_view_position(&map, [0, 0], [32768, 32768], 0), Ok([0, 0]));
        assert!(entity_view_position(&map[..map.len() - 1], [0, 0], [0, 0], 128).is_err());
    }

    #[test]
    fn entity_projection_interpolates_height_before_shifting() {
        let map = map_with_heights(&[(1, 0, 100)]);
        assert_eq!(entity_view_position(&map, [128, 0], [0, 0], 0), Ok([16, 7]));
        let low = map_with_heights(&[(0, 0, -1)]);
        assert_eq!(entity_view_position(&low, [0, 0], [0, 0], 0), Ok([0, 1]));
    }

    #[test]
    fn factor_plan_preserves_snapshot_and_uses_strict_radius_with_inner_ring() {
        let mut map = map_with_heights(&[]);
        // An already larger byte remains untouched, even though it is not 0..63.
        map[12] = 200;
        let before = map.clone();
        let changes = plan_terrain_factor_raise_4096(&map, [0, 0]).unwrap();
        assert_eq!(map, before);
        assert!(!changes.iter().any(|c| c.source_cell == [0, 0]));
        assert!(!changes.iter().any(|c| c.grid_position == [4096, 0]));
        assert_eq!(changes.iter().find(|c| c.grid_position == [3840, 0]).unwrap().next, 63);
        assert_eq!(changes.iter().find(|c| c.grid_position == [-3840, 0]).unwrap().source_cell, [113, 0]);
        assert!(changes.iter().all(|c| c.previous == 0 && c.next <= 63 && c.next > 0));
        assert!(plan_terrain_factor_raise_4096(&map[..map.len() - 1], [0, 0]).is_err());
    }

    #[test]
    fn factor_plan_uses_height_difference_and_subcell_target() {
        let flat = map_with_heights(&[]);
        let center = [100, 0];
        let flat_changes = plan_terrain_factor_raise_4096(&flat, center).unwrap();
        let edge = flat_changes.iter().find(|c| c.grid_position == [4096, 0]).unwrap();
        assert_eq!(edge.next, 24); // floor((4096 - 3996 - 1) / 4).
        let high = map_with_heights(&[(16, 0, i16::MAX)]);
        let high_changes = plan_terrain_factor_raise_4096(&high, center).unwrap();
        assert!(!high_changes.iter().any(|c| c.grid_position == [4096, 0]));
    }
}

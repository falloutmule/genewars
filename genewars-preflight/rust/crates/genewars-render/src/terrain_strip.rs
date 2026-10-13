//! Bounded planner for one explicitly selected terrain strip.
//!
//! This reports the ordered requests emitted by the traced strip routine. It
//! does not choose a camera, viewport, initial light-factor values, or outer
//! column interval. Modes 6 and 15 are surfaced as requests because their
//! pixel semantics are not implemented by this module.

use core::fmt;

use genewars_assets::map_atlas::{ConqMapCells, MAP_SIDE};

use crate::terrain::{
    b32_uv_corners_for_cell_flags, terrain_vertex_buffer_position,
    terrain_vertex_shade_16_16, Mode5TriangleVertex, TerrainPrimitiveError,
};

pub const TERRAIN_STRIP_MAX_COLUMN: i32 = 19;
pub const TERRAIN_STRIP_MAX_PAIRS: usize = 48;
pub const TERRAIN_STRIP_MAX_CLIP_HEIGHT: i32 = 480;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainStripError {
    CameraOutOfRange { x: i32, y: i32 },
    ColumnOutOfRange { column: i32 },
    VerticalRangeInvalid { top: i32, bottom: i32 },
    Projection(TerrainPrimitiveError),
    StopBoundExceeded,
}

impl fmt::Display for TerrainStripError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{self:?}") }
}
impl std::error::Error for TerrainStripError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTextureSource {
    /// Selector addresses one of the 160 traced 32×32 B32 tiles.
    B32Tile(u8),
    /// Outside the admitted 0..160 B32 atlas range. No address is inferred.
    OutOfRangeSelector(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainTriangleRequest {
    pub pair_index: usize,
    pub half: u8,
    /// Numeric mode from the renderer dispatch table: 0, 5, 6, or 15.
    pub mode: u8,
    pub owner_world: [i32; 2],
    pub owner_map_index: usize,
    pub texture_selector: u8,
    pub texture_source: TerrainTextureSource,
    pub orientation: u8,
    pub corner_indices: [u8; 3],
    pub vertex_world: [[i32; 2]; 3],
    pub vertices: [Mode5TriangleVertex; 3],
    /// Present only for mode 6; resolving the pointer table is caller work.
    pub overlay_table_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TerrainStripEvent {
    /// Request a bytewise AND on owner cell byte +11 before that half's cull.
    /// The planner does not mutate map input.
    DirtyAnd { world: [i32; 2], map_index: usize, mask: u8 },
    Triangle(TerrainTriangleRequest),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainStripPlan {
    pub pairs: usize,
    pub events: Vec<TerrainStripEvent>,
}

#[derive(Clone, Copy)]
struct CellSample {
    world: [i32; 2],
    map_index: usize,
    height: i16,
    light: i16,
    texture: u8,
    orientation_flags: u8,
    flags11: u8,
    factor: u8,
    overlay: u8,
}

impl CellSample {
    fn shade(self) -> i32 { terrain_vertex_shade_16_16(self.light, self.factor) }
}

/// Plan one traced `0x7B7F0` strip for explicit camera and vertical inputs.
///
/// Camera coordinates must be map-cell coordinates 0..127; columns are 0..19
/// and `0 <= top < bottom <= 480`. The exact 128×128 map region is borrowed
/// from `ConqMapCells`. Results preserve dirty requests and draw-call order;
/// callers must not treat an omitted mode 6/15 implementation as a complete
/// frame.
pub fn plan_terrain_strip(
    map: &ConqMapCells<'_>,
    camera: [i32; 2],
    column: i32,
    top: i32,
    bottom: i32,
    context_flags: u8,
) -> Result<TerrainStripPlan, TerrainStripError> {
    if !(0..MAP_SIDE as i32).contains(&camera[0]) || !(0..MAP_SIDE as i32).contains(&camera[1]) {
        return Err(TerrainStripError::CameraOutOfRange { x: camera[0], y: camera[1] });
    }
    if !(0..=TERRAIN_STRIP_MAX_COLUMN).contains(&column) {
        return Err(TerrainStripError::ColumnOutOfRange { column });
    }
    if top < 0 || top >= bottom || bottom > TERRAIN_STRIP_MAX_CLIP_HEIGHT {
        return Err(TerrainStripError::VerticalRangeInvalid { top, bottom });
    }

    let n = column >> 1;
    let parity = column & 1;
    let x0 = camera[0] + n;
    let y0 = camera[1] - n - 1 - parity;
    let mut events = Vec::with_capacity(TERRAIN_STRIP_MAX_PAIRS * 8);

    for pair_index in 0..TERRAIN_STRIP_MAX_PAIRS {
        let k = pair_index as i32;
        let x = x0 + k;
        let y = y0 + k;
        let a = sample_cell(map, [x, y]);
        let b = sample_cell(map, [x, y + 1]);
        let c = sample_cell(map, [x + 1, y + 1]);
        let d = sample_cell(map, [x + 1, y + 2]);

        if context_flags & 0x04 != 0 {
            events.push(TerrainStripEvent::DirtyAnd {
                world: a.world, map_index: a.map_index, mask: 0xFB,
            });
        }
        plan_half(&mut events, map, camera, top, pair_index, 0, a, b,
            [a, c, b], [0, 2, 3], context_flags)?;

        if context_flags & 0x04 != 0 {
            events.push(TerrainStripEvent::DirtyAnd {
                world: b.world, map_index: b.map_index, mask: 0xF7,
            });
        }
        plan_half(&mut events, map, camera, top, pair_index, 1, b, b,
            [b, c, d], [0, 1, 2], context_flags)?;

        let stop_y = bottom - top;
        if project_y(d, camera, top)? >= stop_y && project_y(c, camera, top)? >= stop_y {
            return Ok(TerrainStripPlan { pairs: pair_index + 1, events });
        }
    }
    Err(TerrainStripError::StopBoundExceeded)
}

fn sample_cell(map: &ConqMapCells<'_>, world: [i32; 2]) -> CellSample {
    let x = world[0].rem_euclid(MAP_SIDE as i32) as usize;
    let y = world[1].rem_euclid(MAP_SIDE as i32) as usize;
    let cell = map.cell(x, y).expect("wrapped coordinates are in map bounds");
    let index = y * MAP_SIDE + x;
    CellSample {
        world,
        map_index: index,
        height: cell.height(),
        light: cell.raw_light_operand(),
        texture: cell.texture_selector(),
        orientation_flags: cell.orientation_flags(),
        flags11: cell.raw()[11],
        factor: cell.raw_factor_operand(),
        overlay: cell.raw()[13],
    }
}

fn project_y(cell: CellSample, camera: [i32; 2], top: i32) -> Result<i32, TerrainStripError> {
    terrain_vertex_buffer_position(cell.world, camera, cell.height, top, [0, 0])
        .map(|position| position[1])
        .map_err(TerrainStripError::Projection)
}

fn plan_half(
    events: &mut Vec<TerrainStripEvent>,
    _map: &ConqMapCells<'_>,
    camera: [i32; 2],
    top: i32,
    pair_index: usize,
    half: u8,
    owner: CellSample,
    other: CellSample,
    points: [CellSample; 3],
    corner_indices: [u8; 3],
    context_flags: u8,
) -> Result<(), TerrainStripError> {
    let mut positions = [[0i32; 2]; 3];
    let mut shades = [0i32; 3];
    for i in 0..3 {
        positions[i] = terrain_vertex_buffer_position(points[i].world, camera, points[i].height, top, [0, 0])
            .map_err(TerrainStripError::Projection)?;
        shades[i] = points[i].shade();
    }
    if positions.iter().all(|point| point[1] < 0) { return Ok(()); }

    let corners = b32_uv_corners_for_cell_flags(owner.orientation_flags);
    let vertices = core::array::from_fn(|i| {
        let (u, v) = corners[usize::from(corner_indices[i])];
        Mode5TriangleVertex { x: positions[i][0], y: positions[i][1], u: u as i32, v: v as i32, shade: shades[i] }
    });
    let source = if owner.texture < 160 {
        TerrainTextureSource::B32Tile(owner.texture)
    } else {
        TerrainTextureSource::OutOfRangeSelector(owner.texture)
    };
    let base = TerrainTriangleRequest {
        pair_index,
        half,
        mode: if shades.iter().all(|shade| *shade == 0) { 0 } else { 5 },
        owner_world: owner.world,
        owner_map_index: owner.map_index,
        texture_selector: owner.texture,
        texture_source: source,
        orientation: (owner.orientation_flags & 0x70) >> 4,
        corner_indices,
        vertex_world: [points[0].world, points[1].world, points[2].world],
        vertices,
        overlay_table_index: None,
    };
    events.push(TerrainStripEvent::Triangle(base.clone()));

    if owner.overlay > 1 {
        let index = if owner.overlay >= 97 { 2 } else { usize::from(owner.overlay - 1) / 32 };
        let mut overlay = base.clone();
        overlay.mode = 6;
        overlay.overlay_table_index = Some(index);
        events.push(TerrainStripEvent::Triangle(overlay));
    }
    if context_flags & 0x40 != 0 && owner.flags11 & 1 != 0 && other.flags11 & 0x20 == 0 {
        let mut overlay = base;
        overlay.mode = 15;
        events.push(TerrainStripEvent::Triangle(overlay));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use genewars_assets::map_atlas::{MAP_CELL_BYTES, MAP_CELLS_BYTES};

    fn synthetic_map() -> Vec<u8> {
        let mut bytes = vec![0u8; MAP_CELLS_BYTES];
        for index in 0..MAP_SIDE * MAP_SIDE {
            let start = index * MAP_CELL_BYTES;
            let light = ((index % 9) as i16 - 4) * 512;
            let height = ((index % 17) as i16 - 8) * 64;
            bytes[start..start + 2].copy_from_slice(&light.to_le_bytes());
            bytes[start + 2..start + 4].copy_from_slice(&height.to_le_bytes());
            bytes[start + 4] = (index % 160) as u8;
            bytes[start + 5] = ((index % 8) as u8) << 4;
            bytes[start + 11] = [0, 1, 0x21, 0x0c][index % 4];
            bytes[start + 12] = (index % 64) as u8;
            bytes[start + 13] = [1, 2, 33, 65, 97][index % 5];
        }
        bytes
    }

    fn planned_triangles(plan: &TerrainStripPlan) -> impl Iterator<Item = &TerrainTriangleRequest> {
        plan.events.iter().filter_map(|event| match event {
            TerrainStripEvent::Triangle(request) => Some(request),
            TerrainStripEvent::DirtyAnd { .. } => None,
        })
    }

    #[test]
    fn synthetic_strip_preserves_pair_call_modes_order_and_vertices() {
        let bytes = synthetic_map();
        let map = ConqMapCells::parse(&bytes).unwrap();
        let plan = plan_terrain_strip(&map, [0, 0], 0, 0, 128, 0x44).unwrap();
        assert_eq!(plan.pairs, 5);
        let calls: Vec<_> = planned_triangles(&plan).collect();
        assert_eq!(calls.len(), 20);
        assert_eq!((calls[0].pair_index, calls[0].half, calls[0].mode), (0, 0, 5));
        assert_eq!(calls[0].owner_map_index, 16256);
        assert_eq!(calls[0].texture_selector, 96);
        assert_eq!(calls[0].orientation, 0);
        assert_eq!(calls[0].corner_indices, [0, 2, 3]);
        assert_eq!(calls[0].vertex_world, [[0, -1], [1, 0], [0, 0]]);
        assert_eq!(calls[0].vertices.map(|vertex| [vertex.x, vertex.y]), [[32, -8], [32, 30], [0, 16]]);
        assert_eq!(calls[0].vertices.map(|vertex| vertex.shade), [32768, 26624, 32768]);
        assert_eq!((calls[1].pair_index, calls[1].half, calls[1].mode), (0, 0, 6));
        assert_eq!(calls[1].overlay_table_index, Some(0));
        assert_eq!((calls[2].pair_index, calls[2].half, calls[2].mode), (0, 1, 5));
        assert_eq!(calls[2].corner_indices, [0, 1, 2]);
        assert_eq!((calls[3].pair_index, calls[3].half, calls[3].mode), (1, 0, 5));
        assert_eq!((calls[4].pair_index, calls[4].half, calls[4].mode), (1, 0, 6));
        assert_eq!((calls[5].pair_index, calls[5].half, calls[5].mode), (1, 0, 15));

        let dirty: Vec<_> = plan.events.iter().filter_map(|event| match event {
            TerrainStripEvent::DirtyAnd { world, map_index, mask } => Some((*world, *map_index, *mask)),
            _ => None,
        }).collect();
        assert_eq!(&dirty[..4], &[([0, -1], 16256, 0xFB), ([0, 0], 0, 0xF7), ([1, 0], 1, 0xFB), ([1, 1], 129, 0xF7)]);
        assert!(matches!(plan.events.first(), Some(TerrainStripEvent::DirtyAnd { mask: 0xFB, .. })));
    }

    #[test]
    fn strip_validation_wraps_map_coordinates_and_caps_pairs() {
        let bytes = synthetic_map();
        let map = ConqMapCells::parse(&bytes).unwrap();
        let seam = plan_terrain_strip(&map, [127, 127], 19, 64, 320, 0).unwrap();
        assert!(seam.pairs <= TERRAIN_STRIP_MAX_PAIRS);
        assert_eq!(seam.pairs, 11);
        assert!(matches!(plan_terrain_strip(&map, [-1, 0], 0, 0, 480, 0), Err(TerrainStripError::CameraOutOfRange { .. })));
        assert!(matches!(plan_terrain_strip(&map, [0, 0], 20, 0, 480, 0), Err(TerrainStripError::ColumnOutOfRange { .. })));
        assert!(matches!(plan_terrain_strip(&map, [0, 0], 0, 0, 481, 0), Err(TerrainStripError::VerticalRangeInvalid { .. })));
    }

    #[test]
    fn dirty_masks_are_emitted_before_culling_and_unknown_selectors_stay_visible() {
        let mut bytes = vec![0u8; MAP_CELLS_BYTES];
        // A completely offscreen pair still emits its pre-cull dirty request.
        // The owner selector remains visible as out-of-range instead of
        // silently disappearing from the plan.
        for index in 0..MAP_SIDE * MAP_SIDE {
            let start = index * MAP_CELL_BYTES;
            bytes[start + 2..start + 4].copy_from_slice(&(-30000i16).to_le_bytes());
            bytes[start + 4] = 200;
        }
        let map = ConqMapCells::parse(&bytes).unwrap();
        let plan = plan_terrain_strip(&map, [0, 0], 0, 0, 480, 0x04).unwrap();
        assert!(plan.events.iter().any(|event| matches!(event,
            TerrainStripEvent::DirtyAnd { mask: 0xFB, .. })));
        assert!(planned_triangles(&plan).all(|call| matches!(call.texture_source, TerrainTextureSource::OutOfRangeSelector(200))));
    }
}

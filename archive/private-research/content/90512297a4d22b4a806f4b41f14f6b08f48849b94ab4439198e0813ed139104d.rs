//! Terrain backing buffers assembled from verified strip requests.
//! The initial layout preserves the proved 128-pixel left margin. A full
//! twenty-column diagnostic remains available as an explicit caller policy.
use genewars_assets::{B32Atlas, ConqMapCells};
use crate::{terrain, terrain_modes as modes, terrain_strip::{self, TerrainStripEvent, TerrainTextureSource}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerrainView {
    pub indices: Vec<u8>,
    pub pairs: usize,
    pub requested_modes: [usize; 16],
    pub prepared_triangles: usize,
    pub pixels_written: usize,
    /// Ordered map-index/AND-mask requests; input map is never mutated.
    pub dirty_requests: Vec<(usize, u8)>,
}

/// Draw all requested 0/5/6/15 modes into a new indexed framebuffer. A missing
/// MAIN.GLA is an error only when mode15 is requested. No partial frame escapes
/// an error. Source zero is opaque in5 and transparent in6.
pub fn render_full_terrain_view(
    map: &ConqMapCells<'_>, camera: [i32; 2], atlas: &[u8],
    fade: &terrain::TerrainFadeTable<'_>, gla: Option<&modes::MainGlaTable<'_>>,
    context_flags: u8,
) -> Result<TerrainView, String> {
    render_columns(map, camera, atlas, fade, gla, context_flags, 0)
}

/// Initial dirty-column traversal from context0x16E0A8: columns4..19,
/// top0/bottom480, full640-pixel clip/pitch, and temporary context bit4.
/// Other context flags are explicit caller state. This does not initialize
/// live entities, execute dirty callbacks, or select a panel background.
pub fn render_initial_terrain_view(
    map: &ConqMapCells<'_>, camera: [i32; 2], atlas: &[u8],
    fade: &terrain::TerrainFadeTable<'_>, gla: Option<&modes::MainGlaTable<'_>>,
    context_flags: u8,
) -> Result<TerrainView, String> {
    render_columns(map, camera, atlas, fade, gla, context_flags | 4, 4)
}

/// Copy a caller-selected 128x480 indexed background into the left panel.
/// This is the proved copy geometry; background selection and widgets are
/// separate. Invalid inputs leave the frame unchanged.
pub fn compose_left_panel(view: &mut TerrainView, panel: &[u8]) -> Result<(), String> {
    if panel.len() != 128 * 480 || view.indices.len() != 640 * 480 {
        return Err("panel composition requires a128x480 panel and640x480 frame".into());
    }
    for (source, row) in panel.chunks_exact(128).zip(view.indices.chunks_exact_mut(640)) {
        row[..128].copy_from_slice(source);
    }
    Ok(())
}

fn render_columns(
    map: &ConqMapCells<'_>, camera: [i32; 2], atlas: &[u8],
    fade: &terrain::TerrainFadeTable<'_>, gla: Option<&modes::MainGlaTable<'_>>,
    context_flags: u8, first_column: i32,
) -> Result<TerrainView, String> {
    B32Atlas::parse(atlas).map_err(|e| e.to_string())?;
    let mut view = TerrainView { indices: vec![0; 640 * 480], pairs: 0, requested_modes: [0; 16],
        prepared_triangles: 0, pixels_written: 0, dirty_requests: Vec::new() };
    for column in first_column..20 {
        let plan = terrain_strip::plan_terrain_strip(map, camera, column, 0, 480, context_flags)
            .map_err(|e| e.to_string())?;
        view.pairs += plan.pairs;
        for event in plan.events {
            let request = match event {
                TerrainStripEvent::DirtyAnd { map_index, mask, .. } => {
                    view.dirty_requests.push((map_index, mask)); continue;
                }
                TerrainStripEvent::Triangle(request) => request,
            };
            view.requested_modes[usize::from(request.mode)] += 1;
            let receipt = match request.mode {
                0 | 15 => {
                    let table = if request.mode == 15 {
                        Some(gla.ok_or("mode15 requires exact MAIN.GLA")?)
                    } else { None };
                    let points = request.vertices.map(|v| modes::TerrainGeometryVertex { x: v.x, y: v.y });
                    let Some(prepared) = modes::prepare_mode0_15_triangle(points, 480).map_err(|e| e.to_string())?
                        else { continue; };
                    if let Some(table) = table {
                        modes::draw_mode15_triangle(&prepared, table, 640, &mut view.indices, 640, 0)
                    } else {
                        modes::draw_mode0_triangle(&prepared, 640, &mut view.indices, 640, 0)
                    }.map_err(|e| e.to_string())?
                }
                5 | 6 => {
                    let tile = if request.mode == 6 {
                        let index = request.overlay_table_index.ok_or("mode6 request lacks overlay index")?;
                        if index > 2 { return Err("overlay index outside proved range".into()); }
                        157 + index
                    } else {
                        match request.texture_source {
                            TerrainTextureSource::B32Tile(tile) => usize::from(tile),
                            TerrainTextureSource::OutOfRangeSelector(_) => return Err("unsupported base texture selector".into()),
                        }
                    };
                    let texture = atlas.get((tile / 8) * 8192 + (tile % 8) * 32..).ok_or("texture base outside atlas")?;
                    let Some(prepared) = terrain::prepare_mode5_triangle(request.vertices, 480).map_err(|e| e.to_string())?
                        else { continue; };
                    if request.mode == 5 {
                        terrain::draw_mode5_triangle(texture, fade, &prepared, 640, &mut view.indices, 640)
                    } else {
                        modes::draw_mode6_triangle(texture, fade, &prepared, 640, &mut view.indices, 640)
                    }.map_err(|e| e.to_string())?
                }
                mode => return Err(format!("unsupported terrain mode {mode}")),
            };
            view.prepared_triangles += 1;
            view.pixels_written += receipt.pixels_written;
        }
    }
    Ok(view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_view_preserves_mode_order_and_reports_dirty_requests_without_mutation() {
        let mut bytes = vec![0u8; 128 * 128 * 14];
        for cell in bytes.chunks_exact_mut(14) {
            cell[..2].copy_from_slice(&(-8192i16).to_le_bytes());
            cell[12] = 1; // S=0 selects flat mode0.
            cell[11] = 0x0d; // dirtybits and the mode15 enable bit.
            cell[13] = 2; // first overlay tile, mode6.
        }
        let before = bytes.clone();
        let map = ConqMapCells::parse(&bytes).unwrap();
        let atlas = vec![7u8; 163840];
        let fade_bytes: Vec<u8> = (0..16384).map(|i| i as u8).collect();
        let fade = terrain::TerrainFadeTable::parse(&fade_bytes).unwrap();
        let table: Vec<u8> = (0..65536).map(|i| ((i / 256) as u8).wrapping_add(3)).collect();
        let gla = modes::MainGlaTable::parse(&table).unwrap();
        assert!(render_full_terrain_view(&map, [0, 0], &atlas, &fade, None, 0x44).is_err());
        let result = render_full_terrain_view(&map, [0, 0], &atlas, &fade, Some(&gla), 0x44).unwrap();
        // Each half fills0, then overlays7, then maps existing7 to10.
        assert!(result.indices.iter().all(|&v| v == 10));
        assert_eq!(result.pixels_written, 3 * 640 * 480);
        assert_eq!(result.requested_modes[0], result.requested_modes[6]);
        assert_eq!(result.requested_modes[6], result.requested_modes[15]);
        assert_eq!(result.requested_modes[5], 0);
        assert_eq!(result.dirty_requests.len(), result.pairs * 2);
        for pair in result.dirty_requests.chunks_exact(2) {
            assert_eq!(pair[0].1, 0xfb);
            assert_eq!(pair[1].1, 0xf7);
        }
        assert_eq!(bytes, before);
    }
}

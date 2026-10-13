//! Explicitly placed real-resource diagnostics. These commands do not infer
//! the original initial camera or live visibility state.
use genewars_assets::{B32Atlas, ConqFastPrefix, ConqMapCells, Palette256, PaletteChannelMode};
use genewars_render::{RgbImage, terrain::{self, Mode5TriangleVertex, TerrainFadeTable}};
use std::path::Path;

fn resource(directory: &str, name: &str) -> Result<Vec<u8>, String> {
    let path = Path::new(directory).join(name);
    super::read_input(path.to_str().ok_or("resource path is not UTF-8")?)
}

pub fn cell(args: &[String]) -> Result<(), String> {
    let mission_bytes = resource(&args[0], "CONQ001.DAT")?;
    let atlas_bytes = resource(&args[0], "B32A.DAT")?;
    let fade_bytes = resource(&args[0], "FADE.DAT")?;
    let palette_bytes = resource(&args[0], "MAIN.PAL")?;
    let mission = ConqFastPrefix::parse(&mission_bytes).map_err(|e| e.to_string())?;
    let map = ConqMapCells::parse(mission.opaque_14_byte_region()).map_err(|e| e.to_string())?;
    B32Atlas::parse(&atlas_bytes).map_err(|e| e.to_string())?;
    let fade = TerrainFadeTable::parse(&fade_bytes).map_err(|e| e.to_string())?;
    let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).map_err(|e| e.to_string())?;
    if palette.entries().iter().flatten().any(|&v| v > 63) {
        return Err("MAIN.PAL contains a channel outside the verified RGB6 range".into());
    }
    let x: i16 = args[1].parse().map_err(|_| "cell x must fit signed 16-bit")?;
    let y: i16 = args[2].parse().map_err(|_| "cell y must fit signed 16-bit")?;
    let factor: u8 = args[3].parse().map_err(|_| "explicit factor must fit unsigned byte")?;
    let x = i32::from(x);
    let y = i32::from(y);
    let origin = map.cell(x.rem_euclid(128) as usize, y.rem_euclid(128) as usize).map_err(|e| e.to_string())?;
    let tile = usize::from(origin.texture_selector());
    if tile >= 160 { return Err("cell texture selector exceeds B32A's 160 tiles".into()); }
    let uv = terrain::b32_uv_corners_for_cell_flags(origin.orientation_flags());
    let mut vertices = [Mode5TriangleVertex { x: 0, y: 0, u: 0, v: 0, shade: 0 }; 4];
    for (i, [dx, dy]) in [[0, 0], [1, 0], [1, 1], [0, 1]].into_iter().enumerate() {
        let sample = map.cell((x + dx).rem_euclid(128) as usize, (y + dy).rem_euclid(128) as usize)
            .map_err(|e| e.to_string())?;
        let p = terrain::terrain_vertex_buffer_position([x + dx, y + dy], [x, y], sample.height(), 0, [320, 240])
            .map_err(|e| e.to_string())?;
        vertices[i] = Mode5TriangleVertex { x: p[0], y: p[1], u: uv[i].0 as i32, v: uv[i].1 as i32,
            shade: terrain::terrain_vertex_shade_16_16(sample.raw_light_operand(), factor) };
    }
    // Retain the original 256-byte pitch. Compact 32x32 tile bytes would give
    // a different sampler address for every row after the first.
    let texture = &atlas_bytes[(tile / 8) * 8192 + (tile % 8) * 32..];
    let mut indices = vec![0u8; 640 * 480];
    let (mut drawn, mut pixels) = (0, 0);
    for corners in terrain::CELL_TRIANGLES {
        if let Some(prepared) = terrain::prepare_mode5_triangle(corners.map(|i| vertices[i]), 480)
            .map_err(|e| e.to_string())? {
            let receipt = terrain::draw_mode5_triangle(texture, &fade, &prepared, 640, &mut indices, 640)
                .map_err(|e| e.to_string())?;
            drawn += 1;
            pixels += receipt.pixels_written;
        }
    }
    let rgb = indices.iter().flat_map(|&index| palette.entries()[usize::from(index)]
        .map(|component| (u16::from(component) * 255 / 63) as u8)).collect();
    let image = RgbImage::from_rgb24(640, 480, rgb).map_err(|e| e.to_string())?;
    super::write_output(&args[4], &image.to_bmp_bytes().map_err(|e| e.to_string())?)?;
    println!("{{\"cell\":[{x},{y}],\"texture\":{tile},\"factor_override\":{factor},\"anchor\":[320,240],\"triangles_drawn\":{drawn},\"pixel_writes\":{pixels},\"index_crc16\":{},\"rgb_crc16\":{},\"scene_rendered\":false,\"original_camera_inferred\":false}}",
        genewars_assets::crc16_rnc(&indices), genewars_assets::crc16_rnc(image.rgb_bytes()));
    Ok(())
}

/// Render a caller-selected target on the actual map with an explicitly chosen
/// map snapshot policy. The full 640x480 viewport and columns 0..20 are a tool
/// policy; the mission's initial context/player state remains unestablished.
pub fn view(args: &[String]) -> Result<(), String> {
    use genewars_core::cd_candidate::{camera_cells_for_target, plan_terrain_factor_raise_4096};
    let target: [i16; 2] = [args[1].parse().map_err(|_| "target x must fit signed 16-bit")?,
                           args[2].parse().map_err(|_| "target y must fit signed 16-bit")?];
    if !matches!(args[3].as_str(), "raw" | "raise4096") {
        return Err("terrain-view policy must be raw or raise4096".into());
    }
    let mission_bytes = resource(&args[0], "CONQ001.DAT")?;
    let atlas = resource(&args[0], "B32A.DAT")?;
    let fade_bytes = resource(&args[0], "FADE.DAT")?;
    let palette_bytes = resource(&args[0], "MAIN.PAL")?;
    let mission = ConqFastPrefix::parse(&mission_bytes).map_err(|e| e.to_string())?;
    B32Atlas::parse(&atlas).map_err(|e| e.to_string())?;
    let fade = TerrainFadeTable::parse(&fade_bytes).map_err(|e| e.to_string())?;
    let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).map_err(|e| e.to_string())?;
    if palette.entries().iter().flatten().any(|&v| v > 63) {
        return Err("MAIN.PAL contains a channel outside the verified RGB6 range".into());
    }
    let mut map_bytes = mission.opaque_14_byte_region().to_vec();
    let camera_words = camera_cells_for_target(&map_bytes, target[0], target[1]).map_err(|e| e.to_string())?;
    // Translating camera and every generated world coordinate by a multiple
    // of128 preserves relative projection and modulo128 map samples.
    let camera = camera_words.map(|v| i32::from(v & 127));
    let mut changed = 0;
    if args[3] == "raise4096" {
        let changes = plan_terrain_factor_raise_4096(&map_bytes, target).map_err(|e| e.to_string())?;
        changed = changes.len();
        for change in changes {
            let [x, y] = change.source_cell;
            map_bytes[(usize::from(y) * 128 + usize::from(x)) * 14 + 12] = change.next;
        }
    }
    let map = ConqMapCells::parse(&map_bytes).map_err(|e| e.to_string())?;
    let view = genewars_render::terrain_view::render_full_terrain_view(&map, camera, &atlas, &fade, None, 0)?;
    let (indices, requested, pairs, prepared_count, pixels) = (view.indices, view.requested_modes,
        view.pairs, view.prepared_triangles, view.pixels_written);
    let rgb = indices.iter().flat_map(|&index| palette.entries()[usize::from(index)]
        .map(|component| (u16::from(component) * 255 / 63) as u8)).collect();
    let image = RgbImage::from_rgb24(640, 480, rgb).map_err(|e| e.to_string())?;
    super::write_output(&args[4], &image.to_bmp_bytes().map_err(|e| e.to_string())?)?;
    println!("{{\"target\":{:?},\"camera_words\":{:?},\"wrapped_camera\":{:?},\"map_policy\":\"{}\",\"factor_changes\":{changed},\"columns\":20,\"pairs\":{pairs},\"requested_modes\":{:?},\"prepared_triangles\":{prepared_count},\"pixel_writes\":{pixels},\"index_crc16\":{},\"rgb_crc16\":{},\"entity_callbacks_applied\":false,\"gameplay\":false,\"initial_mission_state_inferred\":false}}",
        target, camera_words, camera, args[3], requested, genewars_assets::crc16_rnc(&indices), genewars_assets::crc16_rnc(image.rgb_bytes()));
    Ok(())
}

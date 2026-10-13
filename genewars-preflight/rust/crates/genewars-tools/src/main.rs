//! Native research utilities, not a playable GeneWars host.
use std::{env, fs, io::{self, Read}, path::Path, process};
use genewars_assets::{B32Atlas, CampaignHeader, ConqFastPrefix, ConqMapCells, Header, Palette256, PaletteChannelMode, SpriteTable, crc16_rnc, decode, decode_prefix};
mod terrain_render;

const INPUT_LIMIT: u64 = 64 * 1024 * 1024;
const DEFAULT_OUTPUT_LIMIT: usize = 64 * 1024 * 1024;
const SPRITE_RECORD_LIMIT: usize = 65_536;

fn read_input(name: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut source: Box<dyn Read> = if name == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(fs::File::open(name).map_err(|e| format!("input: {e}"))?)
    };
    source.by_ref().take(INPUT_LIMIT + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > INPUT_LIMIT {
        return Err("input exceeds 64 MiB limit".into());
    }
    Ok(bytes)
}

fn limit(arg: Option<&String>) -> Result<usize, String> {
    match arg {
        Some(s) => s.parse::<usize>().map_err(|_| "invalid output limit".into()),
        None => Ok(DEFAULT_OUTPUT_LIMIT),
    }
}

fn parse_sprite_table(tab: &[u8], database: &[u8]) -> Result<SpriteTable, String> {
    if tab.len() / 6 > SPRITE_RECORD_LIMIT {
        return Err("sprite table exceeds diagnostic record limit".into());
    }
    SpriteTable::parse(tab, database).map_err(|e| e.to_string())
}

fn write_output(path: &str, bytes: &[u8]) -> Result<(), String> {
    let target = Path::new(path);
    if target.exists() {
        if fs::metadata(target).map_err(|e| e.to_string())?.len() != bytes.len() as u64 {
            return Err("output exists with different content; refusing to overwrite".into());
        }
        if fs::read(target).map_err(|e| e.to_string())? != bytes {
            return Err("output exists with different content; refusing to overwrite".into());
        }
        return Ok(());
    }
    // create_new prevents clobbering a file created after the exists check.
    use std::io::Write;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(target).map_err(|e| e.to_string())?;
    file.write_all(bytes).map_err(|e| e.to_string())
}

fn run(args: &[String]) -> Result<(), String> {
    let usage = "usage: genewars-tools rnc-info INPUT | rnc-verify INPUT [MAX_BYTES] | rnc-decode INPUT OUTPUT [MAX_BYTES] | rnc-prefix INPUT OUTPUT [MAX_BYTES] | panel-render RNC_INPUT MAIN_PAL OUTPUT_BMP | sprite-verify ANB TAB | sprite4-verify ANB TAB BASE | sprite-decode ANB TAB RECORD INDEX_OUTPUT MASK_OUTPUT | sprite4-decode ANB TAB RECORD BASE INDEX_OUTPUT MASK_OUTPUT | distance SX SY TX TY | campaign-plan PROGRESS PRIOR_SYSTEM | campaign-inspect HEADER MISSION | terrain-info MISSION | terrain-height MISSION X Y | terrain-camera MISSION X Y | terrain-tile DECODED_B32A TILE OUTPUT | terrain-cell MISSION X Y CAMERA_X CAMERA_Y TOP ANCHOR_X ANCHOR_Y (INPUT '-' reads stdin)";
    let usage = format!("{usage} | terrain-shade MISSION X Y | terrain-cell-render INPUT_DIR X Y FACTOR OUTPUT_BMP | terrain-view INPUT_DIR TARGET_X TARGET_Y raw|raise4096 OUTPUT_BMP");
    let command = args.first().ok_or(usage.as_str())?;
    match command.as_str() {
        "terrain-cell-render" if args.len() == 6 => terrain_render::cell(&args[1..]),
        "terrain-view" if args.len() == 6 => terrain_render::view(&args[1..]),
        "terrain-shade" if args.len() == 4 => {
            let bytes = read_input(&args[1])?;
            let mission = ConqFastPrefix::parse(&bytes).map_err(|e| e.to_string())?;
            let x: i16 = args[2].parse().map_err(|_| "x must fit signed 16-bit")?;
            let y: i16 = args[3].parse().map_err(|_| "y must fit signed 16-bit")?;
            let shade = genewars_core::cd_candidate::interpolate_entity_shade(mission.opaque_14_byte_region(), x, y)
                .map_err(|e| e.to_string())?;
            println!("{{\"target\":[{x},{y}],\"entity_shade\":{shade},\"function_va\":\"0x7B240\",\"state_modified\":false}}");
            Ok(())
        }
        "terrain-camera" if args.len() == 4 => {
            let bytes = read_input(&args[1])?;
            let mission = ConqFastPrefix::parse(&bytes).map_err(|e| e.to_string())?;
            let x: i16 = args[2].parse().map_err(|_| "x must fit signed 16-bit")?;
            let y: i16 = args[3].parse().map_err(|_| "y must fit signed 16-bit")?;
            let cells = genewars_core::cd_candidate::camera_cells_for_target(mission.opaque_14_byte_region(), x, y)
                .map_err(|e| e.to_string())?;
            println!("{{\"target\":[{x},{y}],\"camera_cells\":[{},{}],\"function_va\":\"0x82E20\",\"initial_mission_target_inferred\":false}}", cells[0], cells[1]);
            Ok(())
        }
        "terrain-cell" if args.len() == 9 => {
            use genewars_render::terrain::{b32_uv_corners_for_cell_flags, terrain_vertex_buffer_position, CELL_TRIANGLES};
            let bytes = read_input(&args[1])?;
            let mission = ConqFastPrefix::parse(&bytes).map_err(|e| e.to_string())?;
            let cells = ConqMapCells::parse(mission.opaque_14_byte_region()).map_err(|e| e.to_string())?;
            let numbers = args[2..].iter().map(|s| s.parse::<i32>().map_err(|_| "terrain-cell coordinates must fit signed 32-bit"))
                .collect::<Result<Vec<_>, _>>()?;
            let [x, y, camera_x, camera_y, top, anchor_x, anchor_y] = <[i32; 7]>::try_from(numbers)
                .map_err(|_| "terrain-cell expects seven placement integers")?;
            let origin = cells.cell(x.rem_euclid(128) as usize, y.rem_euclid(128) as usize)
                .map_err(|e| e.to_string())?;
            let uv = b32_uv_corners_for_cell_flags(origin.orientation_flags());
            let mut vertices = Vec::with_capacity(4);
            for (i, [ox, oy]) in [[0i32, 0i32], [1, 0], [1, 1], [0, 1]].into_iter().enumerate() {
                let mx = x.checked_add(ox).ok_or("terrain-cell neighbor coordinate overflow")?;
                let my = y.checked_add(oy).ok_or("terrain-cell neighbor coordinate overflow")?;
                let sx = mx.rem_euclid(128) as usize;
                let sy = my.rem_euclid(128) as usize;
                let cell = cells.cell(sx, sy).map_err(|e| e.to_string())?;
                let point = terrain_vertex_buffer_position([mx, my], [camera_x, camera_y], cell.height(), top, [anchor_x, anchor_y])
                    .map_err(|e| e.to_string())?;
                vertices.push(format!("{{\"map\":[{mx},{my}],\"source_cell\":[{sx},{sy}],\"height\":{},\"buffer\":[{},{}],\"uv_16_16\":[{},{}]}}",
                    cell.height(), point[0], point[1], uv[i].0, uv[i].1));
            }
            println!("{{\"source_cell\":[{},{}],\"texture\":{},\"orientation\":{},\"camera\":[{camera_x},{camera_y}],\"vertical_top\":{top},\"anchor\":[{anchor_x},{anchor_y}],\"vertices\":[{}],\"triangles\":{:?},\"scene_rendered\":false}}",
                x.rem_euclid(128), y.rem_euclid(128), origin.texture_selector(), origin.orientation_flags() >> 4,
                vertices.join(","), CELL_TRIANGLES);
            Ok(())
        }
        "terrain-height" if args.len() == 4 => {
            let bytes = read_input(&args[1])?;
            let mission = ConqFastPrefix::parse(&bytes).map_err(|e| e.to_string())?;
            let x: i16 = args[2].parse().map_err(|_| "x must fit signed 16-bit")?;
            let y: i16 = args[3].parse().map_err(|_| "y must fit signed 16-bit")?;
            let height = genewars_core::cd_candidate::interpolate_height(mission.opaque_14_byte_region(), x, y)
                .map_err(|e| e.to_string())?;
            println!("{{\"x\":{x},\"y\":{y},\"doubled_height\":{height},\"function_va\":\"0x7B140\"}}");
            Ok(())
        }
        "terrain-info" if args.len() == 2 => {
            let bytes = read_input(&args[1])?;
            let mission = ConqFastPrefix::parse(&bytes).map_err(|e| e.to_string())?;
            let cells = ConqMapCells::parse(mission.opaque_14_byte_region()).map_err(|e| e.to_string())?;
            let (mut low, mut high, mut min_texture, mut max_texture) = (i16::MAX, i16::MIN, u8::MAX, 0u8);
            for y in 0..128 {
                for x in 0..128 {
                    let cell = cells.cell(x, y).map_err(|e| e.to_string())?;
                    low = low.min(cell.height()); high = high.max(cell.height());
                    min_texture = min_texture.min(cell.texture_selector()); max_texture = max_texture.max(cell.texture_selector());
                }
            }
            println!("{{\"width_cells\":128,\"height_cells\":128,\"record_bytes\":14,\"min_height\":{low},\"max_height\":{high},\"min_texture\":{min_texture},\"max_texture\":{max_texture},\"gameplay\":false}}");
            Ok(())
        }
        "terrain-tile" if args.len() == 4 => {
            let bytes = read_input(&args[1])?;
            let atlas = B32Atlas::parse(&bytes).map_err(|e| e.to_string())?;
            let index: usize = args[2].parse().map_err(|_| "invalid tile index")?;
            let tile = atlas.tile(index).map_err(|e| e.to_string())?;
            write_output(&args[3], &tile)?;
            println!("{{\"tile\":{index},\"width\":32,\"height\":32,\"bytes\":1024,\"crc16\":{}}}", crc16_rnc(&tile));
            Ok(())
        }
        "campaign-inspect" if args.len() == 3 => {
            if args[1] == "-" || args[2] == "-" {
                return Err("campaign-inspect requires two explicit file paths".into());
            }
            let header_data = read_input(&args[1])?;
            let mission_data = read_input(&args[2])?;
            let header = CampaignHeader::parse(&header_data).map_err(|e| e.to_string())?;
            let mission = ConqFastPrefix::parse(&mission_data).map_err(|e| e.to_string())?;
            let players = (0..4).map(|i| {
                mission.initial_player_record(i).map(|record| crc16_rnc(record).to_string())
            }).collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?;
            println!("{{\"header_bytes\":{},\"title_bytes\":{},\"title_crc16\":{},\"mission_bytes\":{},\"tag\":{},\"prefix_bytes\":{},\"prefix_crc16\":{},\"tail_bytes\":{},\"tail_crc16\":{},\"opaque_region_bytes\":{},\"opaque_region_crc16\":{},\"initial_player_crc16\":[{}],\"gameplay\":false}}",
                header.bytes().len(), header.title_bytes().len(), crc16_rnc(header.title_bytes()),
                mission_data.len(), mission.tag(), mission.copied_prefix().len(), crc16_rnc(mission.copied_prefix()),
                mission.trailing_bytes().len(), crc16_rnc(mission.trailing_bytes()),
                mission.opaque_14_byte_region().len(), crc16_rnc(mission.opaque_14_byte_region()), players.join(","));
            Ok(())
        }
        "campaign-plan" if args.len() == 3 => {
            let progress: i8 = args[1].parse().map_err(|_| "progress must fit a signed byte")?;
            let prior: u8 = args[2].parse().map_err(|_| "prior system must fit an unsigned byte")?;
            let p = genewars_core::cd_candidate::CampaignResourcePlan::new(progress, prior)
                .map_err(|e| format!("unsupported campaign state: {e:?}"))?;
            // All path strings are generated from bounded integers and fixed
            // ASCII names; none contains user-provided text needing JSON escape.
            let movie = p.system_movie.map_or("null".to_string(), |s| format!("\"{s}\""));
            println!("{{\"progress\":{},\"local_planet\":{},\"effective_system\":{},\"header\":\"{}\",\"mission\":\"{}\",\"texture_atlas\":\"{}\",\"campaign_background\":\"{}\",\"campaign_palette\":\"{}\",\"system_movie\":{},\"transition_background\":\"{}\",\"transition_palette\":\"{}\",\"gameplay\":false}}",
                p.progress, p.local_planet, p.effective_system, p.header, p.mission,
                p.texture_atlas, p.campaign_background, p.campaign_palette, movie,
                p.transition_background, p.transition_palette);
            Ok(())
        }
        "rnc-info" if args.len() == 2 => {
            let bytes = read_input(&args[1])?;
            let header = Header::parse(&bytes).map_err(|e| e.to_string())?;
            let packed = header.packed_payload(&bytes).map_err(|e| e.to_string())?;
            let valid_crc = crc16_rnc(packed) == header.packed_crc16;
            println!("{{\"method\":{},\"packed_bytes\":{},\"unpacked_bytes\":{},\"packed_crc16\":{},\"unpacked_crc16\":{},\"packed_crc_matches\":{}}}",
                     header.method, header.packed_size, header.unpacked_size, header.packed_crc16, header.unpacked_crc16, valid_crc);
            Ok(())
        }
        "rnc-verify" if (2..=3).contains(&args.len()) => {
            let bytes = read_input(&args[1])?;
            let output = decode(&bytes, limit(args.get(2))?).map_err(|e| e.to_string())?;
            println!("{{\"decoded_bytes\":{},\"decoded_crc16\":{},\"status\":\"verified\"}}", output.len(), crc16_rnc(&output));
            Ok(())
        }
        "rnc-decode" if (3..=4).contains(&args.len()) => {
            let bytes = read_input(&args[1])?;
            let output = decode(&bytes, limit(args.get(3))?).map_err(|e| e.to_string())?;
            write_output(&args[2], &output)?;
            println!("{{\"decoded_bytes\":{},\"decoded_crc16\":{},\"status\":\"written\"}}", output.len(), crc16_rnc(&output));
            Ok(())
        }
        "rnc-prefix" if (3..=4).contains(&args.len()) => {
            let bytes = read_input(&args[1])?;
            let (output, consumed) = decode_prefix(&bytes, limit(args.get(3))?).map_err(|e| e.to_string())?;
            write_output(&args[2], &output)?;
            println!("{{\"decoded_bytes\":{},\"decoded_crc16\":{},\"consumed_bytes\":{},\"tail_bytes\":{},\"status\":\"written\"}}",
                     output.len(), crc16_rnc(&output), consumed, bytes.len() - consumed);
            Ok(())
        }
        "distance" if args.len() == 5 => {
            let mut coords = [0i16; 4];
            for (i, value) in args[1..].iter().enumerate() {
                coords[i] = value.parse().map_err(|_| "coordinates must fit signed 16-bit")?;
            }
            let value = genewars_core::cd_candidate::distance_squared_wrapping_15([coords[0], coords[1]], [coords[2], coords[3]]);
            println!("{{\"squared_distance\":{value},\"variant\":\"cd-candidate\",\"function_va\":\"0x73690\"}}");
            Ok(())
        }
        "panel-render" if args.len() == 4 => {
            let bytes = read_input(&args[1])?;
            let indices = decode(&bytes, genewars_render::GENE_PANEL_INDEX_BYTES).map_err(|e| e.to_string())?;
            let palette_bytes = read_input(&args[2])?;
            let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).map_err(|e| e.to_string())?;
            let rendered = genewars_render::render_gene_panel(&indices, &palette).map_err(|e| e.to_string())?;
            let bmp = rendered.to_bmp_bytes().map_err(|e| e.to_string())?;
            write_output(&args[3], &bmp)?;
            println!("{{\"width\":{},\"height\":{},\"bmp_bytes\":{},\"palette\":\"RGB6\",\"transfer\":\"floor(v*255/63)\",\"status\":\"written\"}}",
                     rendered.width(), rendered.height(), bmp.len());
            Ok(())
        }
        "sprite-verify" | "sprite4-verify" if (command == "sprite-verify" && args.len() == 3) || (command == "sprite4-verify" && args.len() == 4) => {
            let database = read_input(&args[1])?;
            let table_data = read_input(&args[2])?;
            let table = parse_sprite_table(&table_data, &database)?;
            let packed_base = if command == "sprite4-verify" {
                Some(args[3].parse::<u8>().map_err(|_| "palette base must fit an unsigned byte")?)
            } else { None };
            let mut records = Vec::new();
            let mut pixels = 0usize;
            let mut consumed = 0usize;
            for i in 0..table.records().len() {
                let sprite = match packed_base {
                    Some(base) => table.decode_packed4(i, &database, base),
                    None => table.decode(i, &database),
                }.map_err(|e| e.to_string())?;
                pixels = pixels.checked_add(sprite.indices.len()).ok_or("pixel count overflow")?;
                consumed = consumed.checked_add(sprite.consumed_bytes).ok_or("consumed-byte count overflow")?;
                if pixels > DEFAULT_OUTPUT_LIMIT || consumed > DEFAULT_OUTPUT_LIMIT {
                    return Err("sprite verification exceeds diagnostic work budget".into());
                }
                records.push(format!("{{\"record\":{},\"width\":{},\"height\":{},\"consumed_bytes\":{},\"opaque_pixels\":{}}}",
                    i, sprite.width, sprite.height, sprite.consumed_bytes, sprite.opaque.iter().filter(|&&v| v).count()));
            }
            println!("{{\"records\":[{}],\"pixel_slots\":{},\"status\":\"verified\",\"association\":\"caller_supplied\",\"encoding\":\"{}\",\"palette_base\":{}}}", records.join(","), pixels,
                     if packed_base.is_some() { "packed4-high-first" } else { "indexed8" },
                     packed_base.map_or("null".to_string(), |base| base.to_string()));
            Ok(())
        }
        "sprite-decode" | "sprite4-decode" if (command == "sprite-decode" && args.len() == 6) || (command == "sprite4-decode" && args.len() == 7) => {
            let database = read_input(&args[1])?;
            let table_data = read_input(&args[2])?;
            let record: usize = args[3].parse().map_err(|_| "invalid sprite record")?;
            let table = parse_sprite_table(&table_data, &database)?;
            let packed_base = if command == "sprite4-decode" {
                Some(args[4].parse::<u8>().map_err(|_| "palette base must fit an unsigned byte")?)
            } else { None };
            let sprite = match packed_base {
                Some(base) => table.decode_packed4(record, &database, base),
                None => table.decode(record, &database),
            }.map_err(|e| e.to_string())?;
            let output_index = if packed_base.is_some() { 5 } else { 4 };
            write_output(&args[output_index], &sprite.indices)?;
            let mask: Vec<u8> = sprite.opaque.iter().map(|&v| u8::from(v)).collect();
            write_output(&args[output_index + 1], &mask)?;
            println!("{{\"record\":{},\"width\":{},\"height\":{},\"consumed_bytes\":{},\"opaque_pixels\":{},\"status\":\"written\",\"association\":\"caller_supplied\",\"encoding\":\"{}\",\"palette_base\":{}}}",
                record, sprite.width, sprite.height, sprite.consumed_bytes, mask.iter().filter(|&&v| v == 1).count(),
                if packed_base.is_some() { "packed4-high-first" } else { "indexed8" },
                packed_base.map_or("null".to_string(), |base| base.to_string()));
            Ok(())
        }
        _ => Err(usage.into()),
    }
}

fn main() {
    if let Err(message) = run(&env::args().skip(1).collect::<Vec<_>>()) {
        eprintln!("{message}");
        process::exit(2);
    }
}

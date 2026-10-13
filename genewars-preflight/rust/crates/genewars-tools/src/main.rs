//! Native research utilities, not a playable GeneWars host.
use std::{env, fs, io::{self, Read}, path::Path, process};
use genewars_assets::{Header, Palette256, PaletteChannelMode, SpriteTable, crc16_rnc, decode, decode_prefix};

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
    let usage = "usage: genewars-tools rnc-info INPUT | rnc-verify INPUT [MAX_BYTES] | rnc-decode INPUT OUTPUT [MAX_BYTES] | rnc-prefix INPUT OUTPUT [MAX_BYTES] | panel-render RNC_INPUT MAIN_PAL OUTPUT_BMP | sprite-verify ANB TAB | sprite4-verify ANB TAB BASE | sprite-decode ANB TAB RECORD INDEX_OUTPUT MASK_OUTPUT | sprite4-decode ANB TAB RECORD BASE INDEX_OUTPUT MASK_OUTPUT | distance SX SY TX TY (INPUT '-' reads stdin)";
    let command = args.first().ok_or(usage)?;
    match command.as_str() {
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

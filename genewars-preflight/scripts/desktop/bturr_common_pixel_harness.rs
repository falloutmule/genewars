use genewars_render::{terrain::TerrainFadeTable, terrain_modes::MainGlaTable};
use genewars_runtime::{building_resources::{BuildingCommonDrawContext, BuildingResourceDatabase}, identity::sha256_hex};
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 4 { return Err("usage: bturr_pixels ANB FADE GLA CASES.txt".into()); }
    let resource = BuildingResourceDatabase::load_selector1_bturr(fs::read(&args[0])?)?;
    let fade_bytes = fs::read(&args[1])?;
    let _fade_table = TerrainFadeTable::parse(&fade_bytes)?;
    let gla_bytes = fs::read(&args[2])?;
    let cases = fs::read_to_string(&args[3])?;
    let _gla = MainGlaTable::parse(&gla_bytes)?;
    let mut reports = Vec::new();
    for line in cases.lines() {
        let fields = line.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 9 { return Err(format!("malformed case line: {line}").into()); }
        let id: usize = fields[0].parse()?;
        let frame: u8 = fields[1].parse()?;
        let owner: u8 = fields[2].parse()?;
        let shade: u8 = fields[3].parse()?;
        let actor_flags: u8 = fields[4].parse()?;
        let global_flags: u8 = fields[5].parse()?;
        let x: i16 = fields[6].parse()?;
        let y: i16 = fields[7].parse()?;
        let phase: u8 = fields[8].parse()?;
        let mut record = [0u8; 67];
        record[6] = 4;
        record[0x0f..0x11].copy_from_slice(&((id % 256) as i16).to_le_bytes());
        record[0x15] = actor_flags;
        record[0x17] = shade;
        record[0x18] = owner;
        record[0x19] = 0;
        record[0x1b] = frame;
        record[0x2d] = 1;
        record[0x30] = phase;
        let mut pixels = vec![0x35; (480 - 1) * 648 + 640];
        let use_fade = actor_flags & 2 == 0 && shade < 30;
        let fade_row = if use_fade {
            Some(&fade_bytes[usize::from(shade) * 256..usize::from(shade + 1) * 256])
        } else { None };
        let blend = if global_flags & 0x0c != 0 { Some(gla_bytes.as_slice()) } else { None };
        let receipt = resource.draw_incomplete_selector1_common(
            &record, BuildingCommonDrawContext { origin: [x, y], global_flags },
            fade_row, blend, &mut pixels, 640, 480, 648,
        )?;
        reports.push(format!("{id} sha={} written={} clipped={} opaque={} x={} y={}",
            sha256_hex(&pixels), receipt.color.pixels_written, receipt.color.pixels_clipped,
            receipt.color.opaque_pixels, receipt.draw_origin[0], receipt.draw_origin[1]));
    }
    for line in reports { println!("{line}"); }

    // Source-unsupported branches and missing tables must reject before any
    // caller-owned framebuffer byte changes.
    let mut rejected = [0u8; 67];
    rejected[6] = 4; rejected[0x18] = 2; rejected[0x2d] = 1;
    rejected[0x17] = 30;
    let original = [0x5a; 52];
    for (label, flags31, actor15, global, frame, shade, pass_fade, pass_blend, expected) in [
        ("destruction", 0x0800u16, 0u8, 0u8, 0u8, 30u8, true, true, "UnsupportedBuildingRecord"),
        ("blink", 0u16, 0x10u8, 0u8, 0u8, 30u8, true, true, "UnsupportedBuildingRecord"),
        ("global", 0u16, 0u8, 1u8, 0u8, 30u8, true, true, "UnsupportedDrawFlags"),
        ("frame", 0u16, 0u8, 0u8, 3u8, 30u8, true, true, "FrameOutOfBounds"),
        ("fade", 0u16, 0u8, 0u8, 0u8, 12u8, false, true, "FadeRowRequired"),
        ("blend", 0u16, 0u8, 4u8, 0u8, 30u8, true, false, "BlendTableRequired"),
    ] {
        rejected[0x31..0x33].copy_from_slice(&flags31.to_le_bytes());
        rejected[0x15] = actor15; rejected[0x17] = shade; rejected[0x1b] = frame;
        let mut destination = original;
        let needs_fade = rejected[0x15] & 2 == 0 && rejected[0x17] < 30;
        let row = if pass_fade && needs_fade { Some(&fade_bytes[..256]) } else { None };
        let blend = if pass_blend && global & 0x0c != 0 { Some(gla_bytes.as_slice()) } else { None };
        let result = resource.draw_incomplete_selector1_common(
            &rejected, BuildingCommonDrawContext { origin: [40, 30], global_flags: global },
            row, blend, &mut destination, 8, 5, 11,
        );
        if result.is_ok() || destination != original { return Err(format!("{label} did not reject atomically").into()); }
        let error = format!("{:?}", result.unwrap_err());
        if !error.contains(expected) { return Err(format!("{label} rejected as {error}, expected {expected}").into()); }
        println!("reject_{label}=atomic");
    }
    let _ = Path::new(&args[0]);
    Ok(())
}

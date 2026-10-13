#[path = "../../rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"]
mod family8_scene;

use family8_scene::{draw_family8_selector0_state4, draw_family8_selector0_state5, draw_family8_selector1_state0, EtherealAnimationDatabase};
use genewars_render::{terrain::TerrainFadeTable, terrain_modes::MainGlaTable};
use genewars_runtime::identity::sha256_hex;
use std::{env, fs, fs::OpenOptions, io::Write, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 4 { return Err("usage: family8_pixel_harness ETHEREAL FADE GLA OUTPUT_DIR".into()); }
    let anb = fs::read(&args[0])?;
    let fade_bytes = fs::read(&args[1])?;
    let gla_bytes = fs::read(&args[2])?;
    let resource = EtherealAnimationDatabase::from_bytes(&anb)?;
    let fade = TerrainFadeTable::parse(&fade_bytes)?;
    let gla = MainGlaTable::parse(&gla_bytes)?;
    let out = Path::new(&args[3]);
    fs::create_dir(out)?;
    for (name, selector, state, frame, origin, raw_shade, virtual_height) in [
        ("fade0", 0u8, 4u8, 0u8, [320i16, 240i16], 0u8, 0i32),
        ("fade7_clip", 0, 4, 4, [1, 1], 7, 0),
        ("direct_clip", 0, 4, 10, [637, 475], 200, 16_000),
        ("factory_height", 0, 4, 0, [320, 380], 0, 16_000),
        ("selector1_state0", 1, 0, 0, [72, 90], 0, 4_800),
        ("selector1_state0_direction4", 1, 0, 0, [120, 115], 0, 4_800),
        ("selector0_state5_frame6", 0, 5, 6, [280, 170], 0, 22_302),
    ] {
        let mut record = [0u8; 102];
        record[6] = 8;
        record[0x0f..0x11].copy_from_slice(&3u16.to_le_bytes());
        record[0x11..0x15].copy_from_slice(&1000i32.to_le_bytes());
        let flags = if state == 5 { 0x0020u16 } else { 0x4020u16 };
        record[0x15..0x17].copy_from_slice(&flags.to_le_bytes());
        record[0x17] = raw_shade;
        record[0x19] = if name == "selector1_state0_direction4" { 4 } else { 0 };
        record[0x1b] = frame;
        record[0x55] = selector;
        record[0x56] = state;
        record[0x27..0x2b].copy_from_slice(&(1000i32.wrapping_add(virtual_height)).to_le_bytes());
        let width = 640usize;
        let height = 480usize;
        let pitch = 648usize;
        let required = (height - 1) * pitch + width;
        let mut pixels = vec![0x35u8; required];
        let receipt = if selector == 0 && state == 4 {
            draw_family8_selector0_state4(&resource, &record, origin, &fade, &gla, &mut pixels, [width, height], pitch)?
        } else if selector == 0 && state == 5 {
            draw_family8_selector0_state5(&resource, &record, origin, &fade, &gla, &mut pixels, [width, height], pitch)?
        } else {
            draw_family8_selector1_state0(&resource, &record, origin, &fade, &gla, &mut pixels, [width, height], pitch)?
        };
        let mut file = OpenOptions::new().write(true).create_new(true).open(out.join(format!("{name}.raw")))?;
        file.write_all(&pixels)?;
        println!("{name} sha256={} shadow={} shadow_clipped={} color={} color_clipped={} virtual_height={} effective_shade={} color_y={} global_animation={}",
            sha256_hex(&pixels), receipt.shadow_pixels, receipt.shadow_clipped,
            receipt.color_pixels, receipt.color_clipped, receipt.virtual_height,
            receipt.effective_shade, receipt.color_origin[1], receipt.global_animation);
    }
    Ok(())
}

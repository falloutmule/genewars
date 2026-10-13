#[path = "../../rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"]
mod family8_scene;

use family8_scene::{draw_family8_selector1_state0, draw_family8_selector1_state1, EtherealAnimationDatabase};
use genewars_render::{terrain::TerrainFadeTable, terrain_modes::MainGlaTable};
use genewars_runtime::identity::sha256_hex;
use std::{env, fs, fs::OpenOptions, io::Write, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 5 { return Err("usage: family8_selector1_direction4_harness ETHEREAL FADE GLA RECORD OUTPUT_DIR".into()); }
    let anb = fs::read(&args[0])?;
    let fade_bytes = fs::read(&args[1])?;
    let gla_bytes = fs::read(&args[2])?;
    let record_bytes = fs::read(&args[3])?;
    let record: [u8; 102] = record_bytes.try_into().map_err(|_| "selector1 record must be 102 bytes")?;
    let resource = EtherealAnimationDatabase::from_bytes(&anb)?;
    let fade = TerrainFadeTable::parse(&fade_bytes)?;
    let gla = MainGlaTable::parse(&gla_bytes)?;
    let out = Path::new(&args[4]);
    fs::create_dir(out)?;
    for (name, origin) in [("direction4", [120, 115]), ("direction4_visible", [120, 350])] {
        let mut pixels = vec![0x35u8; (480 - 1) * 648 + 640];
        let receipt = match record[0x56] {
            0 => draw_family8_selector1_state0(&resource, &record, origin, &fade, &gla,
                &mut pixels, [640, 480], 648)?,
            1 => draw_family8_selector1_state1(&resource, &record, origin, &fade, &gla,
                &mut pixels, [640, 480], 648)?,
            state => return Err(format!("unsupported family8 test state {state}").into()),
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

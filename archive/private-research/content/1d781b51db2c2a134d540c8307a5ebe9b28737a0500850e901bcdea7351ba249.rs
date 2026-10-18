#[path = "../../rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"]
mod family8_scene;

use family8_scene::{draw_family8_selector0_state8, draw_family8_selector0_state9, EtherealAnimationDatabase};
use genewars_render::{terrain::TerrainFadeTable, terrain_modes::MainGlaTable};
use genewars_runtime::identity::sha256_hex;
use std::{env, fs, fs::OpenOptions, io::Write, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 6 {
        return Err("usage: family8_parent_pixel_harness ETHEREAL FADE GLA STATE8 STATE9 OUTPUT_DIR".into());
    }
    let resource = EtherealAnimationDatabase::from_bytes(&fs::read(&args[0])?)?;
    let fade_bytes = fs::read(&args[1])?;
    let gla_bytes = fs::read(&args[2])?;
    let fade = TerrainFadeTable::parse(&fade_bytes)?;
    let gla = MainGlaTable::parse(&gla_bytes)?;
    let out = Path::new(&args[5]);
    fs::create_dir(out)?;
    for (name, path) in [("state8", &args[3]), ("state9", &args[4])] {
        let record: [u8; 102] = fs::read(path)?.try_into().map_err(|_| "family-8 record must be 102 bytes")?;
        let mut pixels = vec![0x35u8; (480 - 1) * 648 + 640];
        let receipt = if name == "state8" {
            draw_family8_selector0_state8(&resource, &record, [280, 430], &fade, &gla,
                &mut pixels, [640, 480], 648)?
        } else {
            draw_family8_selector0_state9(&resource, &record, [280, 430], &fade, &gla,
                &mut pixels, [640, 480], 648)?
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

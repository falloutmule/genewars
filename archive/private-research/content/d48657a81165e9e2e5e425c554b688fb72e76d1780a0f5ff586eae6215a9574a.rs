#[path = "../../rust/crates/genewars-desktop/src/bin/support/building_scene.rs"]
mod building_scene;

use building_scene::draw_bturr_incomplete_common;
use genewars_runtime::{building_resources::BuildingResourceDatabase, identity::sha256_hex};
use std::{env, fs, fs::OpenOptions, io::Write, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 2 { return Err("usage: building_scene_pixel_harness BTURR.ANB OUTPUT_DIR".into()); }
    let resource = BuildingResourceDatabase::load_selector1_bturr(fs::read(&args[0])?)?;
    let out = Path::new(&args[1]);
    fs::create_dir(out)?;
    for frame in 0..3u8 {
        let mut record = [0u8; 67];
        record[6] = 4;
        record[0x0f..0x11].copy_from_slice(&17i16.to_le_bytes());
        record[0x17] = 30;
        record[0x18] = 2;
        record[0x1b] = frame;
        record[0x2d] = 1;
        let mut pixels = vec![0x35u8; (180 - 1) * 192 + 180];
        let receipt = draw_bturr_incomplete_common(&resource, &record, [2, 3], 0, None, None,
            &mut pixels, 180, 180, 192)?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(out.join(format!("frame{frame}.raw")))?;
        file.write_all(&pixels)?;
        println!("{frame} sha={} written={} clipped={} opaque={} x={} y={} shadow={} post_overlays={}",
            sha256_hex(&pixels), receipt.main_image.color.pixels_written,
            receipt.main_image.color.pixels_clipped, receipt.main_image.color.opaque_pixels,
            receipt.main_image.draw_origin[0], receipt.main_image.draw_origin[1],
            receipt.omissions.shadow_pass_included, receipt.omissions.post_image_overlays_included);
    }
    Ok(())
}

use std::{env, fs, path::PathBuf};

use genewars_assets::{FontResource, Palette256, PaletteChannelMode, TextStringTable};
use genewars_render::RgbImage;
use genewars_runtime::{blueprint::{BlueprintNames, BlueprintRng}, player_state::PlayerState};

#[path = "../../../../rust/crates/genewars-desktop/src/bin/support/actor_panel_text.rs"]
mod actor_panel_text;

fn read(path: &PathBuf) -> Vec<u8> { fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())) }

fn main() {
    let mut args = env::args_os().skip(1).map(PathBuf::from);
    let root = args.next().expect("project root");
    let output = args.next().expect("fresh output directory");
    fs::create_dir(&output).expect("output must be fresh");
    let font_dat = read(&root.join("private/review/font-menu/font0.dat"));
    let font_tab = read(&root.join("private/review/font-menu/font0.tab"));
    let font = FontResource::parse(&font_dat, &font_tab).unwrap();
    let text_bytes = read(&root.join("private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT"));
    let text = TextStringTable::parse(&text_bytes).unwrap();
    let names = BlueprintNames::new(&text.strings()[..128]).unwrap();
    let palette_bytes = read(&root.join("private/reference-run/run-005/drive-c/GAME/DATA/MAIN.PAL"));
    let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).unwrap();
    let mut player = PlayerState::fresh_campaign(0, [0, 0], b"Pixel oracle\0").unwrap();
    player.regenerate_blueprints([0; 36], &names, BlueprintRng { seed: 5428, ring: 0, draw_count: 0 }, true).unwrap();
    for category in 0..4u8 {
        let (slot, record) = (0..20usize).map(|slot| (slot, player.blueprint(slot).unwrap()))
            .find(|(_, record)| record[0x16] == category).expect("generated category record");
        let mut background = Vec::with_capacity(640 * 480 * 3);
        for pixel in 0..640 * 480 {
            background.extend_from_slice(&[(pixel % 251) as u8, ((pixel / 640) % 239) as u8, 37]);
        }
        let mut image = RgbImage::from_rgb24(640, 480, background).unwrap();
        let receipt = actor_panel_text::draw_blueprint_details(&mut image, &font, &text, &palette, slot,
            genewars_runtime::actor_initialization::BlueprintRecordHandle { player_id: 0, blueprint_index: slot as u8 }, record).unwrap();
        fs::write(output.join(format!("category-{category}.rgb")), image.rgb_bytes()).unwrap();
        fs::write(output.join(format!("category-{category}.record")), record).unwrap();
        fs::write(output.join(format!("category-{category}.receipt")), format!("{receipt:?}\n")).unwrap();
    }
}

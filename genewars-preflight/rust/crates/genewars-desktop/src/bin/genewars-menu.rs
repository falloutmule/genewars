//! The traced CD three-button menu, including bounded capture/hit behavior.
//! Requests are recorded; scene dispatch and mission simulation are not implemented.
use genewars_assets::{FontResource, Palette256, PaletteChannelMode, TextStringTable};
use genewars_render::{blit_text_line, measure_text_line, CdTitleButton, RgbImage};
use std::{env, fs, io::{Read, Write}, path::{Path, PathBuf}, process};
#[path = "support/menu_window.rs"]
mod menu_window;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const LIMIT: u64 = 64 * 1024 * 1024;

fn read(path: &Path) -> Result<Vec<u8>, String> {
    let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut data = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut data).map_err(|e| e.to_string())?;
    if data.len() as u64 > LIMIT { return Err("input exceeds 64 MiB".into()); }
    Ok(data)
}

fn save_new(path: &Path, data: &[u8]) -> Result<(), String> {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|e| format!("refusing to overwrite or unable to create {}: {e}", path.display()))?;
    file.write_all(data).map_err(|e| e.to_string())
}

fn compose(dir: &Path, palette: &Palette256) -> Result<(Vec<RgbImage>, CdTitleButton, usize, usize), String> {
    let background = read(&dir.join("titlescr.dat"))?;
    if background.len() != WIDTH * HEIGHT {
        return Err("TITLESCR must contain exactly 640x480 decoded indices".into());
    }
    if palette.entries().iter().flatten().any(|&v| v > 63) {
        return Err("menu palette must contain RGB6 channels".into());
    }
    let rgb = background.iter().flat_map(|&index| {
        palette.entries()[usize::from(index)].map(|v| (u16::from(v) * 255 / 63) as u8)
    }).collect();
    let background = RgbImage::from_rgb24(WIDTH as u32, HEIGHT as u32, rgb).map_err(|e| e.to_string())?;
    let anb = read(&dir.join("iftitle.anb"))?;
    let button = CdTitleButton::parse(&anb).map_err(|e| e.to_string())?;
    let dat = read(&dir.join("font2.dat"))?;
    let tab = read(&dir.join("font2.tab"))?;
    let bytes = read(&dir.join("text.dat"))?;
    let font = FontResource::parse(&dat, &tab).map_err(|e| e.to_string())?;
    let text = TextStringTable::parse(&bytes).map_err(|e| e.to_string())?;
    // The initial non-network branch disables record one and repositions the
    // remaining controls. Text labels are loaded from the user's own resource.
    let mut images = Vec::with_capacity(4);
    for selected in 0..4 {
        let mut image = background.clone();
        for (row, y) in [115, 186, 257].into_iter().enumerate() {
            button.draw(&mut image, palette, 140, y, selected == row + 1).map_err(|e| e.to_string())?;
        }
        for (index, y) in [(255, 140), (258, 211), (259, 282)] {
            let label = text.strings()[index];
            let width = measure_text_line(&font, label).map_err(|e| e.to_string())?;
            blit_text_line(&mut image, &font, label, palette, 312 - (width >> 1), y)
                .map_err(|e| e.to_string())?;
        }
        images.push(image);
    }
    Ok((images, button, text.consumed_len(), text.trailing().len()))
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() < 3 {
        return Err("usage: genewars-menu DECODED_ASSET_DIR PALETTE NEW_BMP [--window] [--frames N] [--report NEW_JSON]".into());
    }
    let mut show = false;
    let mut frames = None;
    let mut report: Option<PathBuf> = None;
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--window" if !show => { show = true; i += 1; }
            "--frames" if frames.is_none() && i + 1 < args.len() => {
                let n: u64 = args[i + 1].parse().map_err(|_| "invalid frame count")?;
                if n == 0 { return Err("frame count must be positive".into()); }
                frames = Some(n); show = true; i += 2;
            }
            "--report" if report.is_none() && i + 1 < args.len() => {
                report = Some(PathBuf::from(&args[i + 1])); i += 2;
            }
            value => return Err(format!("unknown, duplicate or incomplete option: {value}")),
        }
    }
    let palette = Palette256::parse(&read(Path::new(&args[1]))?, PaletteChannelMode::Rgb)
        .map_err(|e| e.to_string())?;
    let (images, button, text_consumed, text_trailing) = compose(Path::new(&args[0]), &palette)?;
    save_new(Path::new(&args[2]), &images[0].to_bmp_bytes().map_err(|e| e.to_string())?)?;
    let (presented, reason, activations, updates) = if show {
        menu_window::show(&images, &button, frames)?
    } else { (0, "bmp_written", Vec::new(), 0) };
    let requests = activations.iter().map(i16::to_string).collect::<Vec<_>>().join(",");
    let json = format!("{{\"width\":640,\"height\":480,\"layout\":\"three_button_normal\",\"text_entries\":604,\"text_consumed_bytes\":{text_consumed},\"text_trailing_bytes\":{text_trailing},\"presented_frames\":{presented},\"exit_reason\":\"{reason}\",\"activation_requests\":[{requests}],\"input_updates\":{updates},\"scene_dispatch\":false,\"gameplay\":false}}\n");
    if let Some(path) = report { save_new(&path, json.as_bytes())?; }
    print!("{json}");
    Ok(())
}

fn main() {
    if let Err(error) = run() { eprintln!("{error}"); process::exit(1); }
}

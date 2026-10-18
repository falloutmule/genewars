//! The traced CD three-button menu, including bounded capture/hit behavior.
//! Requests are recorded; scene dispatch and mission simulation are not implemented.
use genewars_assets::{Palette256, PaletteChannelMode};
#[path = "support/menu_art.rs"]
mod menu_art;
use menu_art::compose;
use std::{env, fs, io::{Read, Write}, path::{Path, PathBuf}, process};
#[path = "support/menu_window.rs"]
mod menu_window;

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

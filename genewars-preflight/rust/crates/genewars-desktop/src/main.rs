//! Native asset inspector. This is a bounded viewer, not a playable game.

use genewars_assets::{decode, Palette256, PaletteChannelMode, SpriteTable};
use genewars_render::{render_cd_ifcustom_arrows, render_gene_panel, GENE_PANEL_INDEX_BYTES};
use minifb::{Key, Window, WindowOptions};
use std::{
    env, fs,
    io::Read,
    process,
};

const INPUT_LIMIT: u64 = 64 * 1024 * 1024;
const PAL_BYTES: usize = 768;
const WINDOW_TITLE: &str = "GeneWars asset inspector";

#[derive(Debug)]
struct Options {
    view: View,
    frames: Option<u64>,
    report_path: Option<String>,
}

#[derive(Debug)]
enum View {
    Panel { rnc_path: String, palette_path: String },
    Sprite { anb_path: String, palette_path: String, record: usize },
    Sprite4 { anb_path: String, palette_path: String, record: usize, base: u8 },
    IfCustomArrows { anb_path: String, palette_path: String, selected: bool, bmp_path: String },
}

impl View {
    fn title(&self) -> String {
        match self {
            Self::Panel { .. } => WINDOW_TITLE.into(),
            Self::Sprite { record, .. } => format!("{WINDOW_TITLE} - sprite record {record}"),
            Self::Sprite4 { record, base, .. } => format!("{WINDOW_TITLE} - packed4 sprite {record}, base {base} (diagnostic)"),
            Self::IfCustomArrows { selected, .. } => format!("{WINDOW_TITLE} - IFCUSTOM arrows ({})", if *selected { "selected" } else { "normal" }),
        }
    }
}

#[derive(Debug)]
struct Report {
    presented_frames: u64,
    exit_reason: String,
    error: Option<String>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (report, report_path) = match parse_args(&args) {
        Ok(options) => {
            let report_path = options.report_path.clone();
            let report = run(options).unwrap_or_else(|error| Report {
                presented_frames: 0,
                exit_reason: "error".into(),
                error: Some(error),
            });
            (report, report_path)
        }
        Err(error) => (Report { presented_frames: 0, exit_reason: "error".into(), error: Some(error) }, None),
    };
    let json = report.to_json();
    println!("{json}");
    if let Some(path) = report_path {
        if let Err(error) = write_report(&path, &json) {
            eprintln!("could not write report: {error}");
            process::exit(1);
        }
    }
    if report.error.is_some() {
        process::exit(1);
    }
}

impl Report {
    fn to_json(&self) -> String {
        let error = self.error.as_ref().map(|value| format!(",\"error\":\"{}\"", json_escape(value))).unwrap_or_default();
        format!("{{\"presented_frames\":{},\"exit_reason\":\"{}\"{error}}}",
            self.presented_frames, json_escape(&self.exit_reason))
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mode = args.first().map(String::as_str).ok_or("usage: genewars-desktop panel PANEL.RNC MAIN.PAL [--frames N] [--report RESULT.json] | sprite ANB MAIN.PAL RECORD [--frames N] [--report RESULT.json] | sprite4 DECODED_ANB MAIN.PAL RECORD BASE [--frames N] [--report RESULT.json] | render-ifcustom ANB MAIN.PAL normal|selected OUTPUT.BMP")?;
    let (view, mut index) = match mode {
        "panel" if args.len() >= 3 => (View::Panel { rnc_path: args[1].clone(), palette_path: args[2].clone() }, 3),
        "sprite" if args.len() >= 4 => {
            let record = args[3].parse::<usize>().map_err(|_| "sprite RECORD must be a nonnegative integer")?;
            (View::Sprite { anb_path: args[1].clone(), palette_path: args[2].clone(), record }, 4)
        }
        "sprite4" if args.len() >= 5 => {
            let record = args[3].parse::<usize>().map_err(|_| "sprite4 RECORD must be a nonnegative integer")?;
            let base = args[4].parse::<u8>().map_err(|_| "sprite4 BASE must be an integer from 0 through 255")?;
            (View::Sprite4 { anb_path: args[1].clone(), palette_path: args[2].clone(), record, base }, 5)
        }
        "render-ifcustom" if args.len() == 5 => {
            let selected = match args[3].as_str() {
                "normal" => false,
                "selected" => true,
                _ => return Err("render-ifcustom state must be normal or selected".into()),
            };
            (View::IfCustomArrows { anb_path: args[1].clone(), palette_path: args[2].clone(), selected, bmp_path: args[4].clone() }, 5)
        }
        "panel" => return Err("usage: genewars-desktop panel PANEL.RNC MAIN.PAL [--frames N] [--report RESULT.json]".into()),
        "sprite" => return Err("usage: genewars-desktop sprite ANB MAIN.PAL RECORD [--frames N] [--report RESULT.json]".into()),
        "sprite4" => return Err("usage: genewars-desktop sprite4 DECODED_ANB MAIN.PAL RECORD BASE [--frames N] [--report RESULT.json]".into()),
        "render-ifcustom" => return Err("usage: genewars-desktop render-ifcustom ANB MAIN.PAL normal|selected OUTPUT.BMP".into()),
        _ => return Err("mode must be panel, sprite, sprite4, or render-ifcustom".into()),
    };
    let mut frames = None;
    let mut report_path = None;
    while index < args.len() {
        match args[index].as_str() {
            "--frames" if index + 1 < args.len() => {
                if frames.is_some() {
                    return Err("--frames may only be specified once".into());
                }
                let count = args[index + 1].parse::<u64>().map_err(|_| "--frames requires a positive integer")?;
                if count == 0 {
                    return Err("--frames requires a positive integer".into());
                }
                frames = Some(count);
                index += 2;
            }
            "--report" if index + 1 < args.len() => {
                if report_path.is_some() {
                    return Err("--report may only be specified once".into());
                }
                report_path = Some(args[index + 1].clone());
                index += 2;
            }
            option => return Err(format!("unknown or incomplete option: {option}")),
        }
    }
    Ok(Options { view, frames, report_path })
}

fn run(options: Options) -> Result<Report, String> {
    if let View::IfCustomArrows { anb_path, palette_path, selected, bmp_path } = &options.view {
        let anb = read_bounded(anb_path)?;
        let palette = read_palette(palette_path)?;
        let image = render_cd_ifcustom_arrows(&anb, &palette, *selected).map_err(|error| error.to_string())?;
        write_bmp(bmp_path, &image.to_bmp_bytes().map_err(|error| error.to_string())?)?;
        return Ok(Report { presented_frames: 0, exit_reason: "bmp_written".into(), error: None });
    }
    let (image, title) = match &options.view {
        View::Panel { rnc_path, palette_path } => {
            let packed = read_bounded(rnc_path)?;
            let indices = decode(&packed, GENE_PANEL_INDEX_BYTES).map_err(|error| format!("panel RNC: {error}"))?;
            let palette = read_palette(palette_path)?;
            (render_gene_panel(&indices, &palette).map_err(|error| error.to_string())?, options.view.title())
        }
        View::Sprite { anb_path, palette_path, record } => {
            let anb = read_bounded(anb_path)?;
            let table = SpriteTable::from_anb(&anb).map_err(|error| format!("ANB header/table: {error}"))?;
            let sprite = table.decode(*record, &anb).map_err(|error| format!("sprite record {record}: {error}"))?;
            if sprite.width == 0 || sprite.height == 0 {
                return Err(format!("sprite record {record} is empty"));
            }
            let palette = read_palette(palette_path)?;
            let pixels = usize::from(sprite.width) * usize::from(sprite.height);
            let mut image = genewars_render::RgbImage::from_rgb24(u32::from(sprite.width), u32::from(sprite.height), vec![0; pixels * 3])
                .map_err(|error| error.to_string())?;
            image.blit_indexed_sprite(&sprite, &palette, 0, 0).map_err(|error| error.to_string())?;
            (image, options.view.title())
        }
        View::Sprite4 { anb_path, palette_path, record, base } => {
            let anb = read_bounded(anb_path)?;
            let table = SpriteTable::from_anb(&anb).map_err(|error| format!("ANB header/table: {error}"))?;
            let sprite = table.decode_packed4(*record, &anb, *base)
                .map_err(|error| format!("packed4 sprite record {record}, base {base}: {error}"))?;
            if sprite.width == 0 || sprite.height == 0 {
                return Err(format!("packed4 sprite record {record} is empty"));
            }
            let palette = read_palette(palette_path)?;
            let pixels = usize::from(sprite.width) * usize::from(sprite.height);
            let mut image = genewars_render::RgbImage::from_rgb24(u32::from(sprite.width), u32::from(sprite.height), vec![0; pixels * 3])
                .map_err(|error| error.to_string())?;
            image.blit_indexed_sprite(&sprite, &palette, 0, 0).map_err(|error| error.to_string())?;
            (image, options.view.title())
        }
        View::IfCustomArrows { .. } => unreachable!("handled before interactive presentation"),
    };
    present(image, &title, options.frames)
}

fn write_bmp(path: &str, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("could not create BMP without overwriting an existing file: {error}"))?;
    file.write_all(bytes).map_err(|error| error.to_string())
}

fn read_palette(path: &str) -> Result<Palette256, String> {
    let palette_bytes = read_bounded(path)?;
    if palette_bytes.len() != PAL_BYTES {
        return Err(format!("palette must contain exactly {PAL_BYTES} bytes, got {}", palette_bytes.len()));
    }
    Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).map_err(|error| error.to_string())
}

fn present(image: genewars_render::RgbImage, title: &str, frame_limit: Option<u64>) -> Result<Report, String> {
    let mut buffer = Vec::new();
    buffer.try_reserve_exact((image.width() * image.height()) as usize).map_err(|_| "framebuffer allocation failed")?;
    for rgb in image.rgb_bytes().chunks_exact(3) {
        buffer.push((u32::from(rgb[0]) << 16) | (u32::from(rgb[1]) << 8) | u32::from(rgb[2]));
    }

    let mut window = match Window::new(title, image.width() as usize, image.height() as usize, WindowOptions::default()) {
        Ok(window) => window,
        Err(error) => return Ok(Report { presented_frames: 0, exit_reason: "error".into(), error: Some(format!("window creation failed: {error}")) }),
    };
    window.set_target_fps(60);
    let mut presented_frames = 0u64;
    let exit_reason = loop {
        if !window.is_open() {
            break "window_closed";
        }
        if window.is_key_down(Key::Escape) {
            break "escape";
        }
        if let Err(error) = window.update_with_buffer(&buffer, image.width() as usize, image.height() as usize) {
            return Ok(Report { presented_frames, exit_reason: "error".into(), error: Some(format!("frame presentation failed: {error}")) });
        }
        presented_frames = presented_frames.saturating_add(1);
        if frame_limit.is_some_and(|limit| presented_frames >= limit) {
            break "frame_limit";
        }
    };
    Ok(Report { presented_frames, exit_reason: exit_reason.into(), error: None })
}

fn read_bounded(path: &str) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(path).map_err(|error| format!("could not open {path}: {error}"))?;
    let mut bytes = Vec::new();
    file.by_ref().take(INPUT_LIMIT + 1).read_to_end(&mut bytes).map_err(|error| format!("could not read {path}: {error}"))?;
    if bytes.len() as u64 > INPUT_LIMIT {
        return Err(format!("input {path} exceeds the 64 MiB limit"));
    }
    Ok(bytes)
}

fn write_report(path: &str, json: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("could not create report without overwriting an existing file: {error}"))?;
    file.write_all(format!("{json}\n").as_bytes()).map_err(|error| error.to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push('?'),
            ch => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_required_paths_and_bounded_frame_option() {
        let args = vec!["panel".into(), "panel.rnc".into(), "main.pal".into(), "--frames".into(), "3".into()];
        let options = parse_args(&args).unwrap();
        assert!(matches!(options.view, View::Panel { rnc_path, palette_path } if rnc_path == "panel.rnc" && palette_path == "main.pal"));
        assert_eq!(options.frames, Some(3));
    }

    #[test]
    fn parses_explicit_sprite_database_palette_and_record() {
        let args = vec!["sprite".into(), "creature.anb".into(), "main.pal".into(), "7".into()];
        let options = parse_args(&args).unwrap();
        assert!(matches!(options.view, View::Sprite { anb_path, palette_path, record: 7 } if anb_path == "creature.anb" && palette_path == "main.pal"));
    }

    #[test]
    fn parses_explicit_packed4_base_and_diagnostic_view() {
        let args = vec!["sprite4".into(), "creature.anb".into(), "main.pal".into(), "5".into(), "255".into(), "--frames".into(), "1".into()];
        let options = parse_args(&args).unwrap();
        assert!(matches!(options.view, View::Sprite4 { anb_path, palette_path, record: 5, base: 255 }
            if anb_path == "creature.anb" && palette_path == "main.pal"));
        assert_eq!(options.frames, Some(1));

        for invalid_base in ["-1", "256", "x"] {
            let bad = vec!["sprite4".into(), "a".into(), "p".into(), "0".into(), invalid_base.into()];
            assert!(parse_args(&bad).unwrap_err().contains("BASE"));
        }
        let missing_base = vec!["sprite4".into(), "a".into(), "p".into(), "0".into()];
        assert!(parse_args(&missing_base).is_err());
    }

    #[test]
    fn parses_named_ifcustom_static_render_state_and_output() {
        let args = vec!["render-ifcustom".into(), "ifcustom.anb".into(), "main.pal".into(), "selected".into(), "arrows.bmp".into()];
        let options = parse_args(&args).unwrap();
        assert!(matches!(options.view, View::IfCustomArrows { anb_path, palette_path, selected: true, bmp_path }
            if anb_path == "ifcustom.anb" && palette_path == "main.pal" && bmp_path == "arrows.bmp"));
        let invalid = vec!["render-ifcustom".into(), "a".into(), "p".into(), "pressed".into(), "out.bmp".into()];
        assert!(parse_args(&invalid).is_err());
    }

    #[test]
    fn positional_input_named_report_is_not_an_output_path() {
        let args = vec!["panel".into(), "--report".into(), "main.pal".into()];
        let options = parse_args(&args).unwrap();
        assert!(matches!(options.view, View::Panel { rnc_path, .. } if rnc_path == "--report"));
        assert_eq!(options.report_path, None);
    }

    #[test]
    fn rejects_zero_or_missing_frame_counts() {
        let zero = vec!["panel".into(), "p".into(), "c".into(), "--frames".into(), "0".into()];
        assert!(parse_args(&zero).is_err());
        let missing = vec!["panel".into(), "p".into(), "c".into(), "--frames".into()];
        assert!(parse_args(&missing).is_err());
        let invalid_record = vec!["sprite".into(), "a".into(), "p".into(), "-1".into()];
        assert!(parse_args(&invalid_record).is_err());
    }

    #[test]
    fn rejects_duplicate_options() {
        let duplicate_frames = vec!["panel".into(), "p".into(), "c".into(), "--frames".into(), "1".into(), "--frames".into(), "2".into()];
        assert!(parse_args(&duplicate_frames).is_err());
        let duplicate_report = vec!["panel".into(), "p".into(), "c".into(), "--report".into(), "r1".into(), "--report".into(), "r2".into()];
        assert!(parse_args(&duplicate_report).is_err());
    }

    #[test]
    fn writes_machine_readable_report_with_escaped_error() {
        let report = Report { presented_frames: 2, exit_reason: "error".into(), error: Some("bad \"path\"".into()) };
        assert_eq!(report.to_json(), "{\"presented_frames\":2,\"exit_reason\":\"error\",\"error\":\"bad \\\"path\\\"\"}");
    }

    #[test]
    fn refuses_to_overwrite_report_targets() {
        let path = std::env::temp_dir().join(format!("genewars-desktop-report-{}.json", std::process::id()));
        std::fs::write(&path, "preserve me").unwrap();
        assert!(write_report(path.to_str().unwrap(), "{}").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "preserve me");
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn refuses_to_overwrite_bmp_targets() {
        let path = std::env::temp_dir().join(format!("genewars-desktop-bmp-{}.bmp", std::process::id()));
        std::fs::write(&path, "preserve me").unwrap();
        assert!(write_bmp(path.to_str().unwrap(), b"BM").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "preserve me");
        std::fs::remove_file(path).unwrap();
    }
}

//! Native diagnostic viewer for an explicitly selected CONQ terrain snapshot.
//! Camera target, factor snapshot, full viewport, and key policy are caller
//! inputs; this is not an original mission startup or input-cadence claim.
use genewars_assets::{B32Atlas, ConqFastPrefix, ConqMapCells, ConqMissionSource, Palette256, PaletteChannelMode};
use genewars_core::cd_candidate::{camera_cells_for_target, plan_terrain_factor_raise_4096};
use genewars_render::{terrain::TerrainFadeTable, terrain_view::{render_full_terrain_view, render_initial_terrain_view, compose_left_panel}, RgbImage};
use genewars_render::{PlantFrame, PlantColorInputs, draw_normal_plant, terrain_modes::MainGlaTable};
use std::{env, fs, io::{Read, Write}, num::NonZeroU32, path::{Path, PathBuf}, process, sync::Arc};
use softbuffer::{Context, Surface};
use winit::{application::ApplicationHandler, dpi::PhysicalSize,
    event::{ElementState, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey}, window::{Window, WindowId}};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const INPUT_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Policy { Raw, Raise4096 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layout { Full, InitialGenePanel }

impl Layout {
    fn name(self) -> &'static str { match self { Self::Full => "full", Self::InitialGenePanel => "initial-genepanl" } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TargetInput { Explicit([i16; 2]), SourcePlayer(usize) }

impl Policy {
    fn name(self) -> &'static str { match self { Self::Raw => "raw", Self::Raise4096 => "raise4096" } }
    fn toggle(self) -> Self { match self { Self::Raw => Self::Raise4096, Self::Raise4096 => Self::Raw } }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeyAction { Exit, StepX(i16), StepY(i16), TogglePolicy }

fn key_action(key: &Key) -> Option<KeyAction> {
    match key {
        Key::Named(NamedKey::Escape) => Some(KeyAction::Exit),
        Key::Named(NamedKey::ArrowLeft) => Some(KeyAction::StepX(-256)),
        Key::Named(NamedKey::ArrowRight) => Some(KeyAction::StepX(256)),
        Key::Named(NamedKey::ArrowUp) => Some(KeyAction::StepY(-256)),
        Key::Named(NamedKey::ArrowDown) => Some(KeyAction::StepY(256)),
        Key::Character(value) if value.as_str().eq_ignore_ascii_case("r") => Some(KeyAction::TogglePolicy),
        _ => None,
    }
}

struct Resources {
    mission: Vec<u8>,
    atlas: Vec<u8>,
    fade: Vec<u8>,
    palette: Palette256,
    panel: Option<Vec<u8>>,
    plant: Option<SelectedPlant>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PlantShade { Source, Sampled }
impl PlantShade {
    fn name(self) -> &'static str { match self { Self::Source => "source", Self::Sampled => "sampled" } }
}

struct PlantSelection { directory: PathBuf, record: usize, shade: PlantShade }
struct SelectedPlant {
    frame: PlantFrame,
    fields: genewars_assets::MissionFamily1SpriteFields,
    shade: PlantShade,
    gla: Vec<u8>,
}

impl Resources {
    fn read(path: &Path) -> Result<Vec<u8>, String> {
        let file = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut bytes = Vec::new();
        file.take(INPUT_LIMIT + 1).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        if bytes.len() as u64 > INPUT_LIMIT { return Err(format!("{} exceeds 64 MiB", path.display())); }
        Ok(bytes)
    }

    fn open(dir: &Path, layout: Layout, selection: Option<&PlantSelection>) -> Result<Self, String> {
        let mission = Self::read(&dir.join("CONQ001.DAT"))?;
        let atlas = Self::read(&dir.join("B32A.DAT"))?;
        let fade = Self::read(&dir.join("FADE.DAT"))?;
        let palette_bytes = Self::read(&dir.join("MAIN.PAL"))?;
        ConqFastPrefix::parse(&mission).map_err(|e| format!("CONQ001.DAT: {e}"))?;
        B32Atlas::parse(&atlas).map_err(|e| format!("B32A.DAT: {e}"))?;
        TerrainFadeTable::parse(&fade).map_err(|e| format!("FADE.DAT: {e}"))?;
        let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb)
            .map_err(|e| format!("MAIN.PAL: {e}"))?;
        if palette.entries().iter().flatten().any(|&v| v > 63) {
            return Err("MAIN.PAL contains a channel outside the traced RGB6 range 0..63".into());
        }
        let panel = if layout == Layout::InitialGenePanel {
            let data = Self::read(&dir.join("GENEPANL.DAT"))?;
            if data.len() != 128 * 480 { return Err("GENEPANL.DAT must contain61440 decoded indices".into()); }
            Some(data)
        } else { None };
        let plant = if let Some(selection) = selection {
            if layout != Layout::InitialGenePanel { return Err("selected plant requires initial-genepanl layout".into()); }
            let source = ConqMissionSource::parse(&mission).map_err(|e| e.to_string())?;
            let fields = source.family1_sprite_fields(selection.record).map_err(|e| e.to_string())?;
            if source.group(1).map_err(|e| e.to_string())?.raw_byte_6(selection.record).map_err(|e| e.to_string())? == 0 {
                return Err("selected plant source record has zero byte6".into());
            }
            let slot = fields.signed_resource_selector;
            if !(0..12).contains(&slot) { return Err("plant signed resource selector must be0..11".into()); }
            let data = Self::read(&selection.directory.join(format!("slot-{slot:02}.anb")))?;
            let frame = PlantFrame::parse_indexed8(&data, fields.animation_selector, fields.frame_selector)
                .map_err(|e| e.to_string())?;
            let gla = Self::read(&dir.join("MAIN.GLA"))?;
            MainGlaTable::parse(&gla).map_err(|e| e.to_string())?;
            Some(SelectedPlant { frame, fields, shade: selection.shade, gla })
        } else { None };
        Ok(Self { mission, atlas, fade, palette, panel, plant })
    }

    fn snapshot(&self, target: [i16; 2], policy: Policy) -> Result<(RgbImage, [u16; 2], [u16; 2], usize, usize), String> {
        let prefix = ConqFastPrefix::parse(&self.mission).map_err(|e| e.to_string())?;
        let mut map = prefix.opaque_14_byte_region().to_vec();
        if policy == Policy::Raise4096 {
            for change in plan_terrain_factor_raise_4096(&map, target).map_err(|e| e.to_string())? {
                let offset = (usize::from(change.source_cell[1]) * 128 + usize::from(change.source_cell[0])) * 14 + 12;
                let field = map.get_mut(offset).ok_or("factor plan addressed outside map snapshot")?;
                if *field != change.previous { return Err("factor plan/source snapshot mismatch".into()); }
                *field = change.next;
            }
        }
        let raw_camera = camera_cells_for_target(&map, target[0], target[1]).map_err(|e| e.to_string())?;
        // The planner's camera words are byte-domain values. The strip view
        // wraps map-space coordinates to its proven 128×128 region.
        let camera = [raw_camera[0] & 127, raw_camera[1] & 127];
        let cells = ConqMapCells::parse(&map).map_err(|e| e.to_string())?;
        let fade = TerrainFadeTable::parse(&self.fade).map_err(|e| e.to_string())?;
        let mut rendered = if self.panel.is_some() {
            render_initial_terrain_view(&cells, camera.map(i32::from), &self.atlas, &fade, None, 0)?
        } else {
            render_full_terrain_view(&cells, camera.map(i32::from), &self.atlas, &fade, None, 0)?
        };
        if let Some(panel) = &self.panel { compose_left_panel(&mut rendered, panel)?; }
        if let Some(plant) = &self.plant {
            let fields = plant.fields;
            let position = fields.signed_position_words;
            let origin = genewars_core::cd_candidate::entity_view_position(&map, position, raw_camera, 128)
                .map_err(|e| e.to_string())?;
            let shade = match plant.shade {
                PlantShade::Source => fields.shade_at_17,
                PlantShade::Sampled => genewars_core::cd_candidate::interpolate_entity_shade(&map, position[0], position[1])
                    .map_err(|e| e.to_string())?,
            };
            let gla = MainGlaTable::parse(&plant.gla).map_err(|e| e.to_string())?;
            draw_normal_plant(&plant.frame, &mut rendered.indices[128..], [512, 480], 640, origin,
                PlantColorInputs { context_flags_at_16e0c4: 0, entity_flags_at_15: fields.flags_at_15,
                    native_flags_at_2e: fields.native_flags_at_2e, shade_at_17: shade }, Some(&fade), &gla)
                .map_err(|e| e.to_string())?;
        }
        let mut rgb = Vec::with_capacity(WIDTH * HEIGHT * 3);
        for &index in &rendered.indices {
            let [r, g, b] = self.palette.entries()[usize::from(index)];
            rgb.extend_from_slice(&[(u16::from(r) * 255 / 63) as u8,
                (u16::from(g) * 255 / 63) as u8, (u16::from(b) * 255 / 63) as u8]);
        }
        let image = RgbImage::from_rgb24(WIDTH as u32, HEIGHT as u32, rgb).map_err(|e| e.to_string())?;
        Ok((image, raw_camera, camera, rendered.pairs, rendered.pixels_written))
    }
}

#[derive(Clone, Copy)]
struct State { target: [i16; 2], policy: Policy }

struct TerrainWindow {
    resources: Resources,
    state: State,
    image: RgbImage,
    camera: [u16; 2],
    raw_camera: [u16; 2],
    pairs: usize,
    pixels_written: usize,
    history: Vec<State>,
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    frames: u64,
    frame_limit: Option<u64>,
    reason: &'static str,
    error: Option<String>,
    report_path: Option<PathBuf>,
    layout: Layout,
    source_player: Option<usize>,
}

impl TerrainWindow {
    fn rebuild(&mut self, events: &ActiveEventLoop) {
        match self.resources.snapshot(self.state.target, self.state.policy) {
            Ok((image, raw_camera, camera, pairs, pixels)) => {
                self.image = image; self.raw_camera = raw_camera; self.camera = camera; self.pairs = pairs; self.pixels_written = pixels;
                if self.history.len() < 1024 { self.history.push(self.state); }
                self.window.as_ref().unwrap().set_title(&format!("Terrain preview — {} — target {},{} — {}", self.layout.name(), self.state.target[0], self.state.target[1], self.state.policy.name()));
                self.window.as_ref().unwrap().request_redraw();
            }
            Err(e) => { self.error = Some(e); events.exit(); }
        }
    }

    fn present(&mut self) -> Result<bool, String> {
        let window = self.window.as_ref().ok_or("window unavailable")?;
        let size = window.inner_size();
        let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return Ok(false); };
        let surface = self.surface.as_mut().ok_or("surface unavailable")?;
        surface.resize(width, height).map_err(|e| e.to_string())?;
        let mut buffer = surface.buffer_mut().map_err(|e| e.to_string())?;
        let (w, h) = (width.get() as usize, height.get() as usize);
        if buffer.len() != w.checked_mul(h).ok_or("window dimensions overflow")? { return Err("unexpected display buffer length".into()); }
        let source = self.image.rgb_bytes();
        for y in 0..h {
            for x in 0..w {
                let offset = ((y * HEIGHT / h) * WIDTH + x * WIDTH / w) * 3;
                buffer[y * w + x] = (u32::from(source[offset]) << 16) | (u32::from(source[offset + 1]) << 8) | u32::from(source[offset + 2]);
            }
        }
        buffer.present().map_err(|e| e.to_string())?;
        Ok(true)
    }

    fn finish_report(&self) -> Result<(), String> {
        let Some(path) = &self.report_path else { return Ok(()); };
        let states = self.history.iter().map(|s| format!("{{\"target\":[{},{}],\"policy\":\"{}\"}}", s.target[0], s.target[1], s.policy.name())).collect::<Vec<_>>().join(",");
        let player = self.source_player.map(|v| v.to_string()).unwrap_or_else(|| "null".to_owned());
        let plant = self.resources.plant.as_ref().map(|p| format!("{{\"record\":{},\"slot\":{},\"shade_policy\":\"{}\",\"selection_policy\":\"explicit-single-source-record\",\"live_visibility_inferred\":false,\"live_order_inferred\":false}}",
            p.fields.record_index, p.fields.signed_resource_selector, p.shade.name())).unwrap_or_else(|| "null".to_owned());
        let rgb_crc16 = genewars_assets::crc16_rnc(self.image.rgb_bytes());
        let report = format!("{{\"width\":640,\"height\":480,\"presented_frames\":{},\"exit_reason\":\"{}\",\"final_target\":[{},{}],\"camera_words\":[{},{}],\"view_camera_cells\":[{},{}],\"final_policy\":\"{}\",\"pairs\":{},\"pixels_written\":{},\"state_history\":[{}],\"history_truncated\":{},\"layout\":\"{}\",\"initial_source_player\":{player},\"plant\":{plant},\"rgb_crc16\":{rgb_crc16},\"original_initial_panel_inferred\":false,\"live_callbacks_applied\":false}}\n",
            self.frames, self.reason, self.state.target[0], self.state.target[1], self.raw_camera[0], self.raw_camera[1], self.camera[0], self.camera[1], self.state.policy.name(), self.pairs, self.pixels_written, states, self.history.len() >= 1024, self.layout.name());
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
            .map_err(|e| format!("refusing to overwrite or unable to create {}: {e}", path.display()))?;
        file.write_all(report.as_bytes()).map_err(|e| e.to_string())
    }
}

impl ApplicationHandler for TerrainWindow {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = Window::default_attributes().with_title(format!("Terrain preview — {} — target {},{} — {}", self.layout.name(), self.state.target[0], self.state.target[1], self.state.policy.name()))
            .with_inner_size(PhysicalSize::new(WIDTH as u32, HEIGHT as u32)).with_resizable(false);
        let window = match events.create_window(attrs) { Ok(w) => Arc::new(w), Err(e) => { self.error = Some(e.to_string()); events.exit(); return; } };
        let context = match Context::new(window.clone()) { Ok(c) => c, Err(e) => { self.error = Some(e.to_string()); events.exit(); return; } };
        let surface = match Surface::new(&context, window.clone()) { Ok(s) => s, Err(e) => { self.error = Some(e.to_string()); events.exit(); return; } };
        window.request_redraw();
        self.window = Some(window); self.context = Some(context); self.surface = Some(surface);
        events.set_control_flow(ControlFlow::Wait);
    }

    fn window_event(&mut self, events: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|w| w.id() != id) { return; }
        match event {
            WindowEvent::CloseRequested => { self.reason = "window_closed"; events.exit(); }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                let changed = match key_action(&event.logical_key) {
                    Some(KeyAction::Exit) => { self.reason = "escape"; events.exit(); false }
                    Some(KeyAction::StepX(delta)) => { self.state.target[0] = self.state.target[0].wrapping_add(delta); true }
                    Some(KeyAction::StepY(delta)) => { self.state.target[1] = self.state.target[1].wrapping_add(delta); true }
                    Some(KeyAction::TogglePolicy) => { self.state.policy = self.state.policy.toggle(); true }
                    None => false,
                };
                if changed { self.rebuild(events); }
            }
            WindowEvent::Resized(_) => self.window.as_ref().unwrap().request_redraw(),
            WindowEvent::RedrawRequested => match self.present() {
                Ok(true) => {
                    self.frames = self.frames.saturating_add(1);
                    if self.frame_limit.is_some_and(|limit| self.frames >= limit) { self.reason = "frame_limit"; events.exit(); }
                    else if self.frame_limit.is_some() { self.window.as_ref().unwrap().request_redraw(); }
                }
                Ok(false) => {},
                Err(e) => { self.error = Some(e); events.exit(); }
            },
            _ => {}
        }
    }
}

struct Options {
    dir: PathBuf, target: TargetInput, policy: Policy, frames: Option<u64>,
    report: Option<PathBuf>, layout: Layout,
    plant: Option<PlantSelection>,
}

fn parse_args_from(args: &[String]) -> Result<Options, String> {
    if args.len() < 5 { return Err("usage: genewars-terrain DECODED_RESOURCE_DIR (--target X Y | --source-player0..3) --policy raw|raise4096 [--layout full|initial-genepanl] [--plant-record INDEX --plant-resources DIR --plant-shade source|sampled] [--frames N] [--report NEW_JSON]".into()); }
    let dir = PathBuf::from(&args[0]);
    let (mut target, mut policy, mut frames, mut report, mut layout) = (None, None, None, None, None);
    let (mut plant_record, mut plant_dir, mut plant_shade) = (None, None, None);
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--plant-record" if plant_record.is_none() && i + 1 < args.len() => {
                let record: usize = args[i + 1].parse().map_err(|_| "plant record must be0..511")?;
                if record >= 512 { return Err("plant record must be0..511".into()); }
                plant_record = Some(record); i += 2;
            }
            "--plant-resources" if plant_dir.is_none() && i + 1 < args.len() => {
                plant_dir = Some(PathBuf::from(&args[i + 1])); i += 2;
            }
            "--plant-shade" if plant_shade.is_none() && i + 1 < args.len() => {
                plant_shade = Some(match args[i + 1].as_str() {
                    "source" => PlantShade::Source, "sampled" => PlantShade::Sampled,
                    _ => return Err("plant shade policy must be source or sampled".into()),
                }); i += 2;
            }
            "--target" if target.is_none() && i + 2 < args.len() => {
                target = Some(TargetInput::Explicit([args[i + 1].parse().map_err(|_| "target X must be signed16")?, args[i + 2].parse().map_err(|_| "target Y must be signed16")?])); i += 3;
            }
            "--source-player" if target.is_none() && i + 1 < args.len() => {
                let player: usize = args[i + 1].parse().map_err(|_| "source player must be0..3")?;
                if player >= 4 { return Err("source player must be0..3".into()); }
                target = Some(TargetInput::SourcePlayer(player)); i += 2;
            }
            "--layout" if layout.is_none() && i + 1 < args.len() => {
                layout = Some(match args[i + 1].as_str() { "full" => Layout::Full, "initial-genepanl" => Layout::InitialGenePanel, _ => return Err("layout must be full or initial-genepanl".into()) }); i += 2;
            }
            "--policy" if policy.is_none() && i + 1 < args.len() => {
                policy = Some(match args[i + 1].as_str() { "raw" => Policy::Raw, "raise4096" => Policy::Raise4096, _ => return Err("policy must be raw or raise4096".into()) }); i += 2;
            }
            "--frames" if frames.is_none() && i + 1 < args.len() => {
                let count: u64 = args[i + 1].parse().map_err(|_| "invalid frame count")?;
                if count == 0 { return Err("frame count must be positive".into()); }
                frames = Some(count); i += 2;
            }
            "--report" if report.is_none() && i + 1 < args.len() => { report = Some(PathBuf::from(&args[i + 1])); i += 2; }
            option => return Err(format!("unknown, duplicate or incomplete option: {option}")),
        }
    }
    let layout = layout.unwrap_or(Layout::Full);
    let plant = match (plant_record, plant_dir, plant_shade) {
        (None, None, None) => None,
        (Some(record), Some(directory), Some(shade)) if layout == Layout::InitialGenePanel =>
            Some(PlantSelection { directory, record, shade }),
        _ => return Err("plant selection requires --plant-record, --plant-resources, --plant-shade and --layout initial-genepanl".into()),
    };
    Ok(Options { dir, target: target.ok_or("--target or --source-player is required")?,
        policy: policy.ok_or("--policy is required")?, frames, report, layout, plant })
}

fn parse_args() -> Result<Options, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    parse_args_from(&args)
}

fn run() -> Result<(), String> {
    let options = parse_args()?;
    let resources = Resources::open(&options.dir, options.layout, options.plant.as_ref())?;
    let (target, source_player) = match options.target {
        TargetInput::Explicit(target) => (target, None),
        TargetInput::SourcePlayer(player) => {
            let source = ConqMissionSource::parse(&resources.mission).map_err(|e| e.to_string())?;
            (source.target_words(player).map_err(|e| e.to_string())?.map(|v| v as i16), Some(player))
        }
    };
    let policy = options.policy;
    let (image, raw_camera, camera, pairs, pixels_written) = resources.snapshot(target, policy)?;
    let mut app = TerrainWindow { resources, state: State { target, policy }, image, raw_camera, camera, pairs, pixels_written,
        history: vec![State { target, policy }], window: None, context: None, surface: None, frames: 0,
        frame_limit: options.frames, reason: "window_closed", error: None, report_path: options.report,
        layout: options.layout, source_player };
    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;
    event_loop.run_app(&mut app).map_err(|e| e.to_string())?;
    if let Some(error) = app.error { return Err(error); }
    app.finish_report()?;
    println!("{{\"width\":640,\"height\":480,\"presented_frames\":{},\"exit_reason\":\"{}\",\"final_target\":[{},{}],\"camera_words\":[{},{}],\"view_camera_cells\":[{},{}],\"final_policy\":\"{}\",\"pairs\":{},\"pixels_written\":{}}}", app.frames, app.reason, app.state.target[0], app.state.target[1], app.raw_camera[0], app.raw_camera[1], app.camera[0], app.camera[1], app.state.policy.name(), app.pairs, app.pixels_written);
    Ok(())
}

fn main() { if let Err(error) = run() { eprintln!("{error}"); process::exit(1); } }

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> { values.iter().map(|s| (*s).to_owned()).collect() }

    #[test]
    fn cli_requires_explicit_signed_target_and_snapshot_policy() {
        assert!(parse_args_from(&args(&["assets", "--target", "0", "0"])).is_err());
        assert!(parse_args_from(&args(&["assets", "--target", "0", "0", "--policy", "guess"])).is_err());
        assert!(parse_args_from(&args(&["assets", "--target", "32768", "0", "--policy", "raw"])).is_err());
        assert!(parse_args_from(&args(&["assets", "--target", "0", "0", "--policy", "raw", "--policy", "raise4096"])).is_err());
    }

    #[test]
    fn cli_accepts_bounded_frame_limit_and_fresh_report_path() {
        let parsed = parse_args_from(&args(&["assets", "--target", "-256", "512", "--policy", "raise4096", "--frames", "3", "--report", "fresh.json"])).unwrap();
        assert_eq!(parsed.dir, PathBuf::from("assets"));
        assert_eq!(parsed.target, TargetInput::Explicit([-256, 512]));
        assert_eq!(parsed.policy, Policy::Raise4096);
        assert_eq!(parsed.frames, Some(3));
        assert_eq!(parsed.report, Some(PathBuf::from("fresh.json")));
        assert!(parse_args_from(&args(&["assets", "--target", "0", "0", "--policy", "raw", "--frames", "0"])).is_err());
    }

    #[test]
    fn source_player_and_layout_are_explicit_and_conflicting_targets_rejected() {
        let parsed = parse_args_from(&args(&["assets", "--source-player", "0", "--policy", "raw", "--layout", "initial-genepanl"])).unwrap();
        assert_eq!(parsed.target, TargetInput::SourcePlayer(0));
        assert_eq!(parsed.layout, Layout::InitialGenePanel);
        for tail in [["--source-player", "4"], ["--source-player", "-1"], ["--layout", "guessed"]] {
            let mut values = args(&["assets", "--source-player", "0", "--policy", "raw"]);
            values.extend(args(&tail));
            assert!(parse_args_from(&values).is_err());
        }
        assert!(parse_args_from(&args(&["assets", "--target", "0", "0", "--source-player", "0", "--policy", "raw"])).is_err());
    }

    #[test]
    fn plant_selection_requires_all_inputs_and_the_offset_viewport() {
        let base = args(&["assets", "--source-player", "0", "--policy", "raw", "--layout", "initial-genepanl"]);
        let mut valid = base.clone();
        valid.extend(args(&["--plant-record", "29", "--plant-resources", "plants", "--plant-shade", "sampled"]));
        let selection = parse_args_from(&valid).unwrap().plant.unwrap();
        assert_eq!((selection.record, selection.shade), (29, PlantShade::Sampled));
        assert_eq!(selection.directory, PathBuf::from("plants"));
        for tail in [vec!["--plant-record", "29"], vec!["--plant-record", "512"],
                     vec!["--plant-record", "-1"], vec!["--plant-shade", "guessed"]] {
            let mut invalid = base.clone(); invalid.extend(args(&tail));
            assert!(parse_args_from(&invalid).is_err());
        }
        let mut full = valid; full[6] = "full".to_owned();
        assert!(parse_args_from(&full).is_err());
    }

    #[test]
    fn logical_key_mapping_covers_navigation_exit_and_toggle() {
        assert_eq!(key_action(&Key::Named(NamedKey::ArrowRight)), Some(KeyAction::StepX(256)));
        assert_eq!(key_action(&Key::Named(NamedKey::ArrowLeft)), Some(KeyAction::StepX(-256)));
        assert_eq!(key_action(&Key::Named(NamedKey::ArrowDown)), Some(KeyAction::StepY(256)));
        assert_eq!(key_action(&Key::Named(NamedKey::ArrowUp)), Some(KeyAction::StepY(-256)));
        assert_eq!(key_action(&Key::Named(NamedKey::Escape)), Some(KeyAction::Exit));
        assert_eq!(key_action(&Key::Character("R".into())), Some(KeyAction::TogglePolicy));
        assert_eq!(key_action(&Key::Character("x".into())), None);
    }
}

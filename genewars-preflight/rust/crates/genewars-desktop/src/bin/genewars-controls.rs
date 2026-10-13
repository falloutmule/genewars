//! Event-driven, bounded inspector for the two statically verified CD IFCUSTOM arrows.
//! This is a diagnostic host, not a playable game or an original timing emulator.

use genewars_assets::{Palette256, PaletteChannelMode};
use genewars_core::cd_candidate::{mode2_button_packet, CustomArrow, CustomArrowCapture};
use genewars_render::{hit_test_cd_ifcustom_arrows, render_cd_ifcustom_controls, CdArrow, RgbImage, CD_SCREEN_HEIGHT, CD_SCREEN_WIDTH};
use softbuffer::{Context, Surface};
use std::{
    env, fs,
    io::{Read, Write},
    num::NonZeroU32,
    path::PathBuf,
    process,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const INPUT_LIMIT: u64 = 64 * 1024 * 1024;
const PALETTE_BYTES: usize = 768;
const HOST_UPDATE_HZ: u64 = 60;
const HOST_TICK: Duration = Duration::from_nanos(1_000_000_000 / HOST_UPDATE_HZ);
const TRACE_LIMIT: usize = 4096;
const WINDOW_TITLE: &str = "GeneWars IFCUSTOM arrow inspector (diagnostic)";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Options {
    anb_path: PathBuf,
    palette_path: PathBuf,
    frame_limit: Option<u64>,
    report_path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CursorPoint {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PressLatch {
    position: Option<CursorPoint>,
}

#[derive(Clone, Copy, Debug, Default)]
struct LeftButtonLatch {
    cursor: Option<CursorPoint>,
    physical_held: bool,
    press: Option<PressLatch>,
}

impl LeftButtonLatch {
    fn cursor_moved(&mut self, position: CursorPoint) {
        self.cursor = Some(position);
    }

    fn button_transition(&mut self, pressed: bool) {
        if pressed {
            if self.press.is_none() {
                self.press = Some(PressLatch { position: self.cursor });
            }
            self.physical_held = true;
        } else {
            self.physical_held = false;
        }
    }

    fn consume_press(&mut self) -> Option<PressLatch> {
        self.press.take()
    }
}

#[derive(Debug)]
struct Report {
    presented_frames: u64,
    packet_updates: u64,
    exit_reason: &'static str,
    activated_control_ids: Vec<i16>,
    omitted_activation_ids: u64,
    mouse_transitions: Vec<MouseTransitionRecord>,
    packet_trace: Vec<PacketRecord>,
    omitted_mouse_transitions: u64,
    omitted_packet_records: u64,
    error: Option<String>,
}

#[derive(Clone, Copy, Debug)]
struct MouseTransitionRecord {
    pressed: bool,
    cursor: Option<CursorPoint>,
}

#[derive(Clone, Copy, Debug)]
struct PacketRecord {
    tick: u64,
    pressed: bool,
    released: bool,
    held: bool,
    cursor: Option<(i32, i32)>,
    event_id: i16,
}

impl Report {
    fn json(&self) -> String {
        let activated = self.activated_control_ids.iter().map(i16::to_string).collect::<Vec<_>>().join(",");
        let error = self.error.as_ref().map(|message| format!(",\"error\":\"{}\"", json_escape(message))).unwrap_or_default();
        let mouse_transitions = self.mouse_transitions.iter().map(|event| {
            let (x, y) = event.cursor.map(|point| (format_float(point.x), format_float(point.y)))
                .unwrap_or_else(|| ("null".into(), "null".into()));
            format!("{{\"state\":\"{}\",\"x_physical\":{},\"y_physical\":{}}}",
                if event.pressed { "down" } else { "up" }, x, y)
        }).collect::<Vec<_>>().join(",");
        let packet_trace = self.packet_trace.iter().map(|packet| {
            let (x, y) = packet.cursor.map(|(x, y)| (x.to_string(), y.to_string()))
                .unwrap_or_else(|| ("null".into(), "null".into()));
            format!("{{\"tick\":{},\"pressed\":{},\"released\":{},\"held\":{},\"x_virtual\":{},\"y_virtual\":{},\"control_result\":{}}}",
                packet.tick, packet.pressed, packet.released, packet.held, x, y, packet.event_id)
        }).collect::<Vec<_>>().join(",");
        format!(
            "{{\"presented_frames\":{},\"packet_updates\":{},\"host_update_hz\":{},\"exit_reason\":\"{}\",\"activated_control_ids\":[{}],\"mouse_transitions\":[{}],\"packet_trace\":[{}],\"omitted_mouse_transitions\":{},\"omitted_packet_records\":{},\"omitted_activation_ids\":{}{}}}",
            self.presented_frames, self.packet_updates, HOST_UPDATE_HZ, self.exit_reason, activated,
            mouse_transitions, packet_trace, self.omitted_mouse_transitions, self.omitted_packet_records,
            self.omitted_activation_ids, error
        )
    }
}

struct ControlsApp {
    anb: Vec<u8>,
    palette: Palette256,
    frame_limit: Option<u64>,
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    image: RgbImage,
    input: LeftButtonLatch,
    capture: CustomArrowCapture,
    previous_packet_held: bool,
    next_tick: Instant,
    update_due: bool,
    presented_frames: u64,
    packet_updates: u64,
    activated_control_ids: Vec<i16>,
    omitted_activation_ids: u64,
    mouse_transitions: Vec<MouseTransitionRecord>,
    packet_trace: Vec<PacketRecord>,
    omitted_mouse_transitions: u64,
    omitted_packet_records: u64,
    exit_reason: &'static str,
    error: Option<String>,
}

impl ControlsApp {
    fn new(anb: Vec<u8>, palette: Palette256, frame_limit: Option<u64>) -> Result<Self, String> {
        let image = render_cd_ifcustom_controls(&anb, &palette, None).map_err(|error| error.to_string())?;
        Ok(Self {
            anb,
            palette,
            frame_limit,
            window: None,
            context: None,
            surface: None,
            image,
            input: LeftButtonLatch::default(),
            capture: CustomArrowCapture::default(),
            previous_packet_held: false,
            next_tick: Instant::now(),
            update_due: false,
            presented_frames: 0,
            packet_updates: 0,
            activated_control_ids: Vec::new(),
            omitted_activation_ids: 0,
            mouse_transitions: Vec::new(),
            packet_trace: Vec::new(),
            omitted_mouse_transitions: 0,
            omitted_packet_records: 0,
            exit_reason: "window_closed",
            error: None,
        })
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, message: impl Into<String>) {
        self.exit_reason = "error";
        self.error = Some(message.into());
        event_loop.exit();
    }

    fn packet_update(&mut self) -> Result<(), String> {
        let size = self.window.as_ref().ok_or("window is not initialized")?.inner_size();
        let (packet, cursor, event_id) = control_packet_tick(
            &self.anb,
            &mut self.input,
            &mut self.capture,
            self.previous_packet_held,
            size,
        )?;
        self.previous_packet_held = packet.held;
        if event_id > 0 {
            if self.activated_control_ids.len() < TRACE_LIMIT {
                self.activated_control_ids.push(event_id);
            } else {
                self.omitted_activation_ids = self.omitted_activation_ids.saturating_add(1);
            }
        }
        if packet_is_trace_worthy(packet, event_id) {
            if self.packet_trace.len() < TRACE_LIMIT {
                self.packet_trace.push(PacketRecord {
                    tick: self.packet_updates.saturating_add(1),
                    pressed: packet.pressed,
                    released: packet.released,
                    held: packet.held,
                    cursor,
                    event_id,
                });
            } else {
                self.omitted_packet_records = self.omitted_packet_records.saturating_add(1);
            }
        }
        self.packet_updates = self.packet_updates.saturating_add(1);
        let selected = if self.capture.selected(CustomArrow::Up) {
            Some(CdArrow::Up)
        } else if self.capture.selected(CustomArrow::Down) {
            Some(CdArrow::Down)
        } else {
            None
        };
        match render_cd_ifcustom_controls(&self.anb, &self.palette, selected) {
            Ok(image) => self.image = image,
            Err(error) => {
                self.error = Some(error.to_string());
                self.exit_reason = "error";
            }
        }
        Ok(())
    }

    fn present(&mut self) -> Result<(), String> {
        let window = self.window.as_ref().ok_or("window is not initialized")?;
        let size = window.inner_size();
        let width = NonZeroU32::new(size.width.max(1)).ok_or("invalid window width")?;
        let height = NonZeroU32::new(size.height.max(1)).ok_or("invalid window height")?;
        let surface = self.surface.as_mut().ok_or("surface is not initialized")?;
        surface.resize(width, height).map_err(|error| format!("surface resize failed: {error}"))?;
        let mut buffer = surface.buffer_mut().map_err(|error| format!("surface buffer failed: {error}"))?;
        let out_width = buffer.width().get() as usize;
        let out_height = buffer.height().get() as usize;
        let src_width = self.image.width() as usize;
        let src_height = self.image.height() as usize;
        let src = self.image.rgb_bytes();
        if buffer.len() != out_width.saturating_mul(out_height) {
            return Err("softbuffer returned an unexpected pixel count".into());
        }
        for y in 0..out_height {
            let source_y = y * src_height / out_height;
            for x in 0..out_width {
                let source_x = x * src_width / out_width;
                let offset = (source_y * src_width + source_x) * 3;
                buffer[y * out_width + x] = (u32::from(src[offset]) << 16)
                    | (u32::from(src[offset + 1]) << 8)
                    | u32::from(src[offset + 2]);
            }
        }
        buffer.present().map_err(|error| format!("frame presentation failed: {error}"))
    }

    fn into_report(self) -> Report {
        Report {
            presented_frames: self.presented_frames,
            packet_updates: self.packet_updates,
            exit_reason: self.exit_reason,
            activated_control_ids: self.activated_control_ids,
            omitted_activation_ids: self.omitted_activation_ids,
            mouse_transitions: self.mouse_transitions,
            packet_trace: self.packet_trace,
            omitted_mouse_transitions: self.omitted_mouse_transitions,
            omitted_packet_records: self.omitted_packet_records,
            error: self.error,
        }
    }
}

impl ApplicationHandler for ControlsApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(PhysicalSize::new(CD_SCREEN_WIDTH, CD_SCREEN_HEIGHT))
            .with_resizable(false);
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => return self.fail(event_loop, format!("window creation failed: {error}")),
        };
        let context = match Context::new(window.clone()) {
            Ok(context) => context,
            Err(error) => return self.fail(event_loop, format!("software display context failed: {error}")),
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(surface) => surface,
            Err(error) => return self.fail(event_loop, format!("software display surface failed: {error}")),
        };
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        self.next_tick = Instant::now();
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: StartCause) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|window| window.id() != window_id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.exit_reason = "window_closed";
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => self.input.cursor_moved(CursorPoint { x: position.x, y: position.y }),
            WindowEvent::MouseInput { state, button: winit::event::MouseButton::Left, .. } => {
                let pressed = state == ElementState::Pressed;
                let cursor = self.input.cursor;
                if self.mouse_transitions.len() < TRACE_LIMIT {
                    self.mouse_transitions.push(MouseTransitionRecord { pressed, cursor });
                } else {
                    self.omitted_mouse_transitions = self.omitted_mouse_transitions.saturating_add(1);
                }
                self.input.button_transition(pressed);
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && event.logical_key == Key::Named(NamedKey::Escape) => {
                self.exit_reason = "escape";
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.update_due {
                    self.update_due = false;
                    if let Err(error) = self.packet_update() {
                        return self.fail(event_loop, error);
                    }
                }
                if self.error.is_none() {
                    if let Err(error) = self.present() {
                        return self.fail(event_loop, error);
                    }
                    self.presented_frames = self.presented_frames.saturating_add(1);
                    if self.update_due == false && self.frame_limit.is_some_and(|limit| self.packet_updates >= limit) {
                        self.exit_reason = "frame_limit";
                        event_loop.exit();
                    }
                } else {
                    event_loop.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.error.is_some() {
            event_loop.exit();
            return;
        }
        let now = Instant::now();
        if !self.update_due && now >= self.next_tick {
            self.update_due = true;
            self.next_tick = now + HOST_TICK;
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_tick));
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let options = match parse_args(&args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            process::exit(2);
        }
    };
    let report_path = options.report_path.clone();
    let report = run(options).unwrap_or_else(|error| Report {
        presented_frames: 0,
        packet_updates: 0,
        exit_reason: "error",
        activated_control_ids: Vec::new(),
        omitted_activation_ids: 0,
        mouse_transitions: Vec::new(),
        packet_trace: Vec::new(),
        omitted_mouse_transitions: 0,
        omitted_packet_records: 0,
        error: Some(error),
    });
    let json = report.json();
    println!("{json}");
    if let Some(path) = report_path {
        if let Err(error) = write_report(&path, &json) {
            eprintln!("report write failed: {error}");
            process::exit(1);
        }
    }
    if report.error.is_some() {
        process::exit(1);
    }
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    if args.len() < 2 {
        return Err("usage: genewars-controls DECODED_IFCUSTOM.ANB MAIN.PAL [--frames N] [--report NEW.json]".into());
    }
    let mut frame_limit = None;
    let mut report_path = None;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--frames" if index + 1 < args.len() => {
                if frame_limit.is_some() {
                    return Err("--frames may only be specified once".into());
                }
                let value = args[index + 1].parse::<u64>().map_err(|_| "--frames requires a positive integer")?;
                if value == 0 {
                    return Err("--frames requires a positive integer".into());
                }
                frame_limit = Some(value);
                index += 2;
            }
            "--report" if index + 1 < args.len() => {
                if report_path.is_some() {
                    return Err("--report may only be specified once".into());
                }
                report_path = Some(PathBuf::from(&args[index + 1]));
                index += 2;
            }
            option => return Err(format!("unknown or incomplete option: {option}")),
        }
    }
    Ok(Options {
        anb_path: PathBuf::from(&args[0]),
        palette_path: PathBuf::from(&args[1]),
        frame_limit,
        report_path,
    })
}

fn run(options: Options) -> Result<Report, String> {
    let anb = read_bounded(&options.anb_path)?;
    let palette_bytes = read_bounded(&options.palette_path)?;
    if palette_bytes.len() != PALETTE_BYTES {
        return Err(format!("MAIN.PAL must contain exactly {PALETTE_BYTES} bytes, got {}", palette_bytes.len()));
    }
    let palette = Palette256::parse(&palette_bytes, PaletteChannelMode::Rgb).map_err(|error| error.to_string())?;
    // Rendering validates every six-bit RGB component before opening a window.
    render_cd_ifcustom_controls(&anb, &palette, None).map_err(|error| error.to_string())?;
    let mut app = ControlsApp::new(anb, palette, options.frame_limit)?;
    let event_loop = EventLoop::new().map_err(|error| format!("event loop creation failed: {error}"))?;
    event_loop.run_app(&mut app).map_err(|error| format!("event loop failed: {error}"))?;
    Ok(app.into_report())
}

fn client_point(point: CursorPoint, size: PhysicalSize<u32>) -> Option<(i32, i32)> {
    if !point.x.is_finite() || !point.y.is_finite() || point.x < 0.0 || point.y < 0.0 || size.width == 0 || size.height == 0 {
        return None;
    }
    let x = (point.x * f64::from(CD_SCREEN_WIDTH) / f64::from(size.width)).floor();
    let y = (point.y * f64::from(CD_SCREEN_HEIGHT) / f64::from(size.height)).floor();
    if x < 0.0 || y < 0.0 || x >= f64::from(CD_SCREEN_WIDTH) || y >= f64::from(CD_SCREEN_HEIGHT) {
        return None;
    }
    Some((x as i32, y as i32))
}

fn control_packet_tick(
    anb: &[u8],
    input: &mut LeftButtonLatch,
    capture: &mut CustomArrowCapture,
    previous_packet_held: bool,
    client_size: PhysicalSize<u32>,
) -> Result<(genewars_core::cd_candidate::ButtonPacket, Option<(i32, i32)>, i16), String> {
    let press_latched = input.consume_press().is_some();
    let packet = mode2_button_packet(previous_packet_held, press_latched, input.physical_held);
    let cursor = input.cursor.and_then(|point| client_point(point, client_size));
    let selected = selected_arrow(capture);
    let hit = cursor
        .map(|(x, y)| hit_test_cd_ifcustom_arrows(anb, selected, x, y).map_err(|error| error.to_string()))
        .transpose()?
        .flatten()
        .map(|arrow| match arrow { CdArrow::Up => CustomArrow::Up, CdArrow::Down => CustomArrow::Down });
    let packet_byte8 = u8::from(packet.pressed)
        | (u8::from(packet.released) << 1)
        | (u8::from(packet.held) << 2);
    let event_id = capture.update(packet_byte8, hit);
    Ok((packet, cursor, event_id))
}

fn selected_arrow(capture: &CustomArrowCapture) -> Option<CdArrow> {
    if capture.selected(CustomArrow::Up) {
        Some(CdArrow::Up)
    } else if capture.selected(CustomArrow::Down) {
        Some(CdArrow::Down)
    } else {
        None
    }
}

fn packet_is_trace_worthy(packet: genewars_core::cd_candidate::ButtonPacket, event_id: i16) -> bool {
    packet.pressed || packet.released || packet.held || event_id != -1
}

fn read_bounded(path: &PathBuf) -> Result<Vec<u8>, String> {
    let mut file = fs::File::open(path).map_err(|error| format!("could not open {}: {error}", path.display()))?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file).take(INPUT_LIMIT + 1).read_to_end(&mut bytes).map_err(|error| format!("could not read {}: {error}", path.display()))?;
    if bytes.len() as u64 > INPUT_LIMIT {
        return Err(format!("input {} exceeds the 64 MiB limit", path.display()));
    }
    Ok(bytes)
}

fn write_report(path: &PathBuf, json: &str) -> Result<(), String> {
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)
        .map_err(|error| format!("could not create report without overwriting an existing file: {error}"))?;
    file.write_all(format!("{json}\n").as_bytes()).map_err(|error| error.to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => escaped.push('?'),
            character => escaped.push(character),
        }
    }
    escaped
}

fn format_float(value: f64) -> String {
    if value.is_finite() { value.to_string() } else { "null".into() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(x: f64, y: f64) -> CursorPoint {
        CursorPoint { x, y }
    }

    #[test]
    fn quick_press_release_before_tick_preserves_press_and_position() {
        let mut input = LeftButtonLatch::default();
        input.cursor_moved(point(455.0, 229.0));
        input.button_transition(true);
        input.button_transition(false);
        let press = input.consume_press().unwrap();
        assert_eq!(press.position, Some(point(455.0, 229.0)));
        assert!(!input.physical_held);
        assert!(input.consume_press().is_none());
    }

    #[test]
    fn first_press_latch_survives_release_and_repress_until_consumed() {
        let mut input = LeftButtonLatch::default();
        input.cursor_moved(point(10.0, 20.0));
        input.button_transition(true);
        input.button_transition(false);
        input.cursor_moved(point(30.0, 40.0));
        input.button_transition(true);
        assert_eq!(input.consume_press().unwrap().position, Some(point(10.0, 20.0)));
        assert!(input.physical_held);
    }

    #[test]
    fn missing_cursor_still_latches_press_without_fabricating_coordinates() {
        let mut input = LeftButtonLatch::default();
        input.button_transition(true);
        assert_eq!(input.consume_press(), Some(PressLatch { position: None }));
    }

    #[test]
    fn duplicate_down_does_not_overwrite_first_position() {
        let mut input = LeftButtonLatch::default();
        input.cursor_moved(point(3.0, 4.0));
        input.button_transition(true);
        input.cursor_moved(point(5.0, 6.0));
        input.button_transition(true);
        assert_eq!(input.consume_press().unwrap().position, Some(point(3.0, 4.0)));
    }

    #[test]
    fn press_notification_after_prior_latch_consumption_is_not_deduplicated_by_held_state() {
        let mut input = LeftButtonLatch::default();
        input.button_transition(true);
        assert!(input.consume_press().is_some());
        // A second OS down notification while the held bit remains set is still
        // a new press latch, matching the separate latch/held packet inputs.
        input.button_transition(true);
        assert!(input.physical_held);
        assert!(input.consume_press().is_some());
    }

    #[test]
    fn host_packet_updates_preserve_press_then_release_for_quick_click() {
        let mut input = LeftButtonLatch::default();
        input.button_transition(true);
        input.button_transition(false);
        let press = mode2_button_packet(false, input.consume_press().is_some(), input.physical_held);
        assert_eq!((press.pressed, press.released, press.held), (true, false, true));
        let release = mode2_button_packet(press.held, input.consume_press().is_some(), input.physical_held);
        assert_eq!((release.pressed, release.released, release.held), (false, true, false));
    }

    #[test]
    fn idle_packet_does_not_consume_action_trace_capacity() {
        let idle = mode2_button_packet(false, false, false);
        assert!(!packet_is_trace_worthy(idle, -1));
        assert!(packet_is_trace_worthy(mode2_button_packet(false, true, false), -1));
        assert!(packet_is_trace_worthy(mode2_button_packet(false, false, true), -1));
        assert!(packet_is_trace_worthy(idle, 27));
    }

    #[test]
    fn maps_physical_cursor_to_virtual_640_by_480_client_coordinates() {
        let size = PhysicalSize::new(1280, 960);
        assert_eq!(client_point(point(910.0, 458.0), size), Some((455, 229)));
        assert_eq!(client_point(point(1280.0, 2.0), size), None);
        assert_eq!(client_point(point(f64::NAN, 1.0), size), None);
    }

    #[test]
    fn parses_only_bounded_control_inspector_options() {
        let args = vec!["ifcustom.anb".into(), "main.pal".into(), "--frames".into(), "2".into(), "--report".into(), "fresh.json".into()];
        let options = parse_args(&args).unwrap();
        assert_eq!(options.frame_limit, Some(2));
        assert_eq!(options.report_path, Some(PathBuf::from("fresh.json")));
        for args in [
            vec!["a".into(), "p".into(), "--frames".into(), "0".into()],
            vec!["a".into(), "p".into(), "--frames".into(), "1".into(), "--frames".into(), "2".into()],
            vec!["a".into(), "p".into(), "--report".into(), "x".into(), "--report".into(), "y".into()],
            vec!["a".into(), "p".into(), "--other".into()],
        ] {
            assert!(parse_args(&args).is_err());
        }
    }

    #[test]
    fn report_has_explicit_host_cadence_and_no_game_claim() {
        let report = Report {
            presented_frames: 1,
            packet_updates: 1,
            exit_reason: "frame_limit",
            activated_control_ids: vec![27],
            omitted_activation_ids: 0,
            mouse_transitions: Vec::new(),
            packet_trace: Vec::new(),
            omitted_mouse_transitions: 0,
            omitted_packet_records: 0,
            error: None,
        };
        assert_eq!(report.json(), "{\"presented_frames\":1,\"packet_updates\":1,\"host_update_hz\":60,\"exit_reason\":\"frame_limit\",\"activated_control_ids\":[27],\"mouse_transitions\":[],\"packet_trace\":[],\"omitted_mouse_transitions\":0,\"omitted_packet_records\":0,\"omitted_activation_ids\":0}");
    }

    #[test]
    fn report_writes_exclusively() {
        let path = std::env::temp_dir().join(format!("genewars-controls-report-{}.json", process::id()));
        fs::write(&path, "preserve me").unwrap();
        assert!(write_report(&path, "{}").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "preserve me");
        fs::remove_file(path).unwrap();
    }
}

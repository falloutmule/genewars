//! Event-driven menu inspector. Ordered host events are explicit adapter policy,
//! not a claim of the original game's packet sampling cadence.
use genewars_render::{CdTitleButton, RgbImage};
use genewars_core::cd_candidate::{MenuCallback0State, MenuControl, MenuPacket};
use softbuffer::{Context, Surface};
use std::{num::NonZeroU32, sync::Arc};
use winit::{application::ApplicationHandler, dpi::PhysicalSize,
    event::{ElementState, MouseButton, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey}, window::{Window, WindowId}};

struct MenuWindow<'a> {
    images: &'a [RgbImage],
    artwork: &'a CdTitleButton,
    controls: MenuCallback0State,
    cursor: Option<(i32, i32)>,
    left_held: bool,
    right_held: bool,
    input_mask: u8,
    activations: Vec<i16>,
    updates: u64,
    window: Option<Arc<Window>>,
    context: Option<Context<Arc<Window>>>,
    surface: Option<Surface<Arc<Window>, Arc<Window>>>,
    frames: u64,
    limit: Option<u64>,
    reason: &'static str,
    error: Option<String>,
}

impl MenuWindow<'_> {
    fn image(&self) -> &RgbImage {
        let index = match self.controls.selected() {
            Some(MenuControl::Id0) => 1,
            Some(MenuControl::Id3) => 2,
            Some(MenuControl::Id2) => 3,
            _ => 0,
        };
        &self.images[index]
    }

    fn hit(&self, control: MenuControl) -> bool {
        let Some((x, y)) = self.cursor else { return false; };
        let top = match control {
            MenuControl::Id0 => 115, MenuControl::Id3 => 186,
            MenuControl::Id2 => 257, MenuControl::Id1 => return false,
        };
        self.artwork.hit_test(x - 140, y - top)
    }

    fn input(&mut self, events: &ActiveEventLoop, press: bool, release: bool) {
        let first = MenuControl::hit_order().into_iter().find(|&c| self.hit(c));
        let captured_hit = self.controls.captured().is_some_and(|c| self.hit(c));
        let before = self.controls.selected();
        let result = self.controls.update(MenuPacket {
            byte8: u8::from(press) | (u8::from(release) << 1) | (u8::from(self.left_held) << 2),
            byte9: u8::from(self.right_held),
        }, first, captured_hit);
        self.updates = self.updates.saturating_add(1);
        if let Some(mask) = result.input_mask_write { self.input_mask = mask; }
        if result.event_id >= 0 && self.input_mask & 8 != 0 {
            if self.activations.len() >= 4096 { return self.fail(events, "menu activation trace limit reached"); }
            self.activations.push(result.event_id);
            // This inspector records the scene request. It does not pretend to
            // initialize the absent campaign or options controllers.
            self.window.as_ref().unwrap().set_title(&format!("GeneWars menu inspector — request {} recorded", result.event_id));
        }
        if before != self.controls.selected() { self.window.as_ref().unwrap().request_redraw(); }
    }

    fn fail(&mut self, events: &ActiveEventLoop, error: impl ToString) {
        self.error = Some(error.to_string());
        events.exit();
    }

    fn present(&mut self) -> Result<bool, String> {
        let window = self.window.as_ref().ok_or("window unavailable")?;
        let size = window.inner_size();
        let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return Ok(false); // A minimized zero-size window is not a presented frame.
        };
        let index = match self.controls.selected() {
            Some(MenuControl::Id0) => 1, Some(MenuControl::Id3) => 2,
            Some(MenuControl::Id2) => 3, _ => 0,
        };
        let image = &self.images[index];
        let surface = self.surface.as_mut().ok_or("surface unavailable")?;
        surface.resize(width, height).map_err(|e| e.to_string())?;
        let mut buffer = surface.buffer_mut().map_err(|e| e.to_string())?;
        let (w, h) = (width.get() as usize, height.get() as usize);
        if buffer.len() != w.checked_mul(h).ok_or("window dimensions overflow")? {
            return Err("unexpected display buffer length".into());
        }
        let (source_w, source_h) = (image.width() as usize, image.height() as usize);
        for y in 0..h {
            for x in 0..w {
                let source = ((y * source_h / h) * source_w + x * source_w / w) * 3;
                let p = &image.rgb_bytes()[source..source + 3];
                buffer[y * w + x] = (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2]);
            }
        }
        buffer.present().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

impl ApplicationHandler for MenuWindow<'_> {
    fn resumed(&mut self, events: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let attrs = Window::default_attributes().with_title("GeneWars menu inspector — scene requests recorded")
            .with_inner_size(PhysicalSize::new(self.image().width(), self.image().height())).with_resizable(false);
        let window = match events.create_window(attrs) {
            Ok(w) => Arc::new(w), Err(e) => return self.fail(events, e),
        };
        let context = match Context::new(window.clone()) {
            Ok(c) => c, Err(e) => return self.fail(events, e),
        };
        let surface = match Surface::new(&context, window.clone()) {
            Ok(s) => s, Err(e) => return self.fail(events, e),
        };
        window.request_redraw();
        self.window = Some(window);
        self.context = Some(context);
        self.surface = Some(surface);
        events.set_control_flow(ControlFlow::Wait);
    }

    fn window_event(&mut self, events: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|w| w.id() != id) { return; }
        match event {
            WindowEvent::CloseRequested => { self.reason = "window_closed"; events.exit(); }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && event.logical_key == Key::Named(NamedKey::Escape) => {
                self.reason = "escape"; events.exit();
            }
            WindowEvent::Resized(_) => { self.window.as_ref().unwrap().request_redraw(); }
            WindowEvent::CursorMoved { position, .. } => {
                let size = self.window.as_ref().unwrap().inner_size();
                self.cursor = if size.width > 0 && size.height > 0
                    && position.x >= 0.0 && position.y >= 0.0
                    && position.x < f64::from(size.width) && position.y < f64::from(size.height) {
                    Some(((position.x * 640.0 / f64::from(size.width)).floor() as i32,
                          (position.y * 480.0 / f64::from(size.height)).floor() as i32))
                } else { None };
                self.input(events, false, false);
            }
            WindowEvent::CursorLeft { .. } => { self.cursor = None; self.input(events, false, false); }
            WindowEvent::MouseInput { state, button, .. } => {
                let down = state == ElementState::Pressed;
                match button {
                    MouseButton::Left => {
                        let changed = self.left_held != down;
                        self.left_held = down;
                        if changed { self.input(events, down, !down); }
                    }
                    MouseButton::Right => { self.right_held = down; self.input(events, false, false); }
                    _ => {}
                }
            }
            WindowEvent::Focused(false) => {
                // Port policy: cancel capture on host focus loss; never activate
                // a stale button when the user returns to this diagnostic.
                self.cursor = None; self.left_held = false; self.right_held = false;
                self.input(events, false, true);
            }
            WindowEvent::RedrawRequested => {
                match self.present() {
                    Ok(true) => self.frames = self.frames.saturating_add(1),
                    Ok(false) => return,
                    Err(e) => return self.fail(events, e),
                }
                if let Some(limit) = self.limit {
                    if self.frames >= limit { self.reason = "frame_limit"; events.exit(); }
                    else { self.window.as_ref().unwrap().request_redraw(); }
                }
            }
            _ => {}
        }
    }
}

pub fn show(images: &[RgbImage], artwork: &CdTitleButton, limit: Option<u64>) -> Result<(u64, &'static str, Vec<i16>, u64), String> {
    if images.len() != 4 || images.iter().any(|im| im.width() != 640 || im.height() != 480) {
        return Err("menu requires four 640x480 states".into());
    }
    let events = EventLoop::new().map_err(|e| e.to_string())?;
    let mut app = MenuWindow { images, artwork, controls: MenuCallback0State::default(),
        cursor: None, left_held: false, right_held: false, input_mask: 0,
        activations: Vec::new(), updates: 0, window: None, context: None, surface: None,
        frames: 0, limit, reason: "window_closed", error: None };
    events.run_app(&mut app).map_err(|e| e.to_string())?;
    if let Some(error) = app.error { return Err(error); }
    Ok((app.frames, app.reason, app.activations, app.updates))
}

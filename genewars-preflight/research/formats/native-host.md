# Native asset inspector host

`genewars-desktop` is a small native window host for inspecting evidence-bounded GeneWars assets. It is an asset inspector, not a playable game. Panel mode requires explicit paths to a panel RNC file and `MAIN.PAL`, decodes the RNC stream under the exact 61,440-byte panel output limit, parses exactly 768 palette bytes as RGB triplets, and renders the fixed 128×480 panel through `genewars-render`.

Sprite mode requires an ANB path, a palette path, and an explicit record index: `genewars-desktop sprite CREATURE.ANB MAIN.PAL RECORD`. It parses the borrowed 18-byte ANB header view, uses the embedded first-section table, and bounds sprite reads to bytes before that table section. It displays just the selected record at its indexed dimensions over a black diagnostic background. It does not provide a menu, animation, or left/right navigation; choose another record by launching with a different index. The canvas and origin are for inspection only and do not assert game placement or background behavior.

The window shows the same raster continuously. Escape or closing the window exits interactive mode. `--frames N` ends after N successful framebuffer updates, and the program emits a JSON report with `presented_frames` and `exit_reason` (`frame_limit`, `escape`, or `window_closed`). Errors return a nonzero process status and a JSON `error`; physical keyboard input is not claimed by the automated smoke run.

Example automated launch, using already staged inputs:

```text
python scripts/desktop/smoke.py private/assets/GENEPANL.DAT private/assets/MAIN.PAL private/assets/desktop-smoke.json --frames 3
```

An individual sprite can be opened directly with the same bounded frame/report flags:

```text
genewars-desktop sprite private/assets/CREATURE.ANB private/assets/MAIN.PAL 7 --frames 3 --report private/assets/sprite-smoke.json
```

For assets with positive evidence for the packed-four-bit run payload, `sprite4` is a separate explicit diagnostic mode. The caller must supply a numeric base from 0 through 255; it does not infer the codec or base:

```text
genewars-desktop sprite4 DECODED_ANB MAIN.PAL RECORD BASE --frames 1 --report fresh.json
```

The smoke wrapper resolves the input paths before Cargo changes directories, uses `--offline --locked`, and enforces a 30-second process timeout. It does not extract source media or alter global Rust configuration. Inputs are individually capped at 64 MiB. Provenance validation and exact hash-bound staging are handled by the preflight staging workflow before this viewer is launched.

The window loop is capped at 60 updates per second. A requested report path is accepted only after command-line validation, and the report uses exclusive file creation so it cannot replace an existing input or report. Parse failures do not write to a path inferred from malformed arguments.

On 2026-10-12, a Windows native launch used the 16,204-byte panel RNC (`8116b2b883320b8573c50496a570f0f3fa978e4c20b9c85a284339ff1f920e24`) and 768-byte `MAIN.PAL` (`8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05`). The private report records `{"presented_frames":3,"exit_reason":"frame_limit"}`. This confirms three successful framebuffer update calls; it does not claim physical keyboard or window-close testing. The report is at `private/integration/panel/desktop-smoke-20261012-2.json`.

Sprite mode also completed a one-frame Windows launch for record 1 of the 31,413-byte decoded `IFEDITOR.ANB` (decoded SHA-256 `ff244244435c5dfe009d3a88932db9735f0fd2f9783f2399d02b3a855e1b24ed`) with the same verified `MAIN.PAL` using this embedded-section bound. Its private JSON report records `{"presented_frames":1,"exit_reason":"frame_limit"}` at `private/integration/panel/desktop-sprite-smoke-2.json`. This checks one selected embedded record through the bounded host, without claiming menu selection or in-game placement behavior.

Packed-four-bit mode completed a one-frame diagnostic launch for record 1 of decoded `DINO.ANB` (48,939 bytes, SHA-256 `0c9d9dbbcb29e39ab29aac17eca97da9a404791febdb79123b6c0eb44af78767`) with caller-supplied base `0` and the same 768-byte palette. Its report at `private/integration/panel/dino4-base0-smoke.json` records `{"presented_frames":1,"exit_reason":"frame_limit"}`. Base zero was a diagnostic choice for this view, not a claim about the game's runtime palette base or visual behavior.

## Window backend dependency

The native window and input loop use [`minifb` 0.29.0](https://docs.rs/minifb/0.29.0/minifb/), pinned exactly in `Cargo.toml`, with default features disabled and its `x11` backend selected for future Linux builds. The reviewed upstream project is [`emoon/rust_minifb`](https://github.com/emoon/rust_minifb); version 0.29.0 is offered under MIT OR Apache-2.0. The dependency is used only for creating the window, polling Escape/window state, and presenting the CPU-rendered framebuffer. Core parsing and rendering stay independent of the window backend.

For the Windows GNU target selected by this project's local toolchain, `cargo tree --offline --locked -p genewars-desktop --target x86_64-pc-windows-gnu -e normal` resolves these registry dependencies:

| Crate | Version | Manifest license | SHA-256 (Cargo.lock checksum; verified against local `.crate`) |
|---|---:|---|---|
| `minifb` | 0.29.0 | MIT OR Apache-2.0 | `d18dab18b966c390844db9f634d41c231dc2320594b84aecfd739b26f5e34bd4` |
| `raw-window-handle` | 0.6.2 | MIT OR Apache-2.0 OR Zlib | `20675572f6f24e9e76ef639bc5552774ed45f1c30e2951e1e99c59888861c539` |
| `web-time` | 1.1.0 | MIT OR Apache-2.0 | `5a6580f308b1fad9207618087a65c04e7a10bc77e02c8e84e9b00dd4b12fa0bb` |
| `winapi` | 0.3.9 | MIT/Apache-2.0 | `5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419` |
| `winapi-x86_64-pc-windows-gnu` | 0.4.0 | MIT/Apache-2.0 | `712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f` |

The normal dependency tree omits three build dependencies used for this Windows build. Their local manifest licenses and archive checksums are:

| Crate | Version | Manifest license | SHA-256 (Cargo.lock checksum; verified against local `.crate`) |
|---|---:|---|---|
| `cc` | 1.8.0 | MIT OR Apache-2.0 | `6651c9ed80effdc7db0ff72512157f901af5e3549e341e24b1dd4887d836d838` |
| `find-msvc-tools` | 0.1.14 | MIT OR Apache-2.0 | `aedcfb3409746eddb02b9e19ebda1c3394f759a152e48ee875a0844d1b955484` |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 | `f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba` |

The remaining lockfile entries are not in the Windows host's normal or build dependency tree; they cover other targets/features and are not included in this host-specific receipt. Exact versions and checksums are pinned in `rust/Cargo.lock`.

## Limits

This host does not simulate gameplay, audio, timing, input mappings beyond Escape, or other game screens. Its display conversion is the renderer's explicit floor(component×255/63) 6-bit-to-8-bit presentation rule; it is not a claim of analog VGA fidelity. A JSON frame count proves calls returned successfully from the framebuffer presentation API, not that a human saw the window or that physical input was tested.

### Mouse packet adapter feasibility

The pinned Windows backend does not expose enough information to reproduce the CD mouse press latches in `research/ghidra/input-packet-bridge.md`. In minifb 0.29.0, `get_mouse_down` returns a held-state boolean. Its Windows window procedure assigns that boolean on `WM_LBUTTONDOWN` and `WM_LBUTTONUP`; `update_with_buffer_stride` drains the complete pending Win32 message queue before returning. If both messages are queued in one drain, the caller observes only the final released state and cannot recover the intervening press. `get_mouse_pos` samples the current cursor position; it does not provide the first-press coordinates. The public `InputCallback` reports text and keyboard state only, with no mouse-button callback or event queue.

This was verified against the locally pinned crate source, not inferred from the event names:

| Source | Relevant code | Local SHA-256 |
|---|---|---|
| `minifb-0.29.0/src/os/windows/mod.rs` | lines 259–263 set held state; 868 reads held state; 957–980 samples current cursor; 982–997 drains messages; 1019–1042 updates and drains | `4d4077de8cd9169457b13d257037674df23a3acb8212d7cd600cd0225e65e4dd` |
| `minifb-0.29.0/src/lib.rs` | lines 200–210 define keyboard/text-only `InputCallback`; 722 reads mouse position; 744–756 expose held-state mouse query | `a55d53751b8cd1a719ca925560d2c26221942cf713e66ecd8e4cfcea90f0e0a8` |

Accordingly, the existing minifb inspector remains a static normal/selected arrow renderer. It does not implement a mouse-driven capture mode or claim parity with the original packet producer. The separate event-driven mode described below adds the transitions that minifb's held-state API does not expose.

### Event-driven backend assessment

The candidate backend for a separate interactive control-inspection mode is `winit` 0.30.13 plus `softbuffer` 0.4.8, each pinned exactly and used only by the native host. The official [`winit 0.30.13 WindowEvent` API](https://docs.rs/winit/0.30.13/winit/event/enum.WindowEvent.html) exposes `MouseInput` transitions with button and state, and `CursorMoved` with a physical position measured from the window's top-left. The event loop processes window events in order. `MouseInput` itself has no coordinates, so the host must retain the latest `CursorMoved` position and latch that position on the first left-button-down event. On each button event it must update physical-held state; it must preserve an outstanding press latch until the host's next packet-production step consumes it. Synthetic host tests can establish the adapter's ordered transition semantics, but native Windows testing is still needed to confirm the platform event stream and cursor ordering in practice.

The host would call the existing core `mode2_button_packet` once per chosen presentation/input-update iteration, pass the latch and current held flag, then clear the host latch when that packet is produced. This reproduces the evidenced press-before-release ordering and queued quick press/release behavior without asserting the original program's wall-clock polling cadence. The UI mode must remain explicitly a diagnostic inspector; a host-selected update interval is not original game timing.

[`softbuffer 0.4.8 Buffer`](https://docs.rs/softbuffer/0.4.8/softbuffer/struct.Buffer.html) provides a writable row-major pixel buffer and a `present` operation for a CPU-rendered framebuffer. The planned Windows configuration is `winit = { version = "=0.30.13", default-features = false, features = ["rwh_06"] }` and `softbuffer = { version = "=0.4.8", default-features = false }`. Winit is Apache-2.0; softbuffer is MIT OR Apache-2.0. The softbuffer manifest uses raw-window-handle 0.6, matching winit's selected `rwh_06` feature. These source-level licenses and API contracts do not replace the later lockfile/build-dependency receipt. With defaults disabled, the proposed selection is Windows-focused; Linux support would require selecting and verifying a Linux display backend feature separately.

This is preferred over SDL2 for the current Windows GNU target. Official [`sdl2 0.38.0 Event`](https://docs.rs/sdl2/0.38.0/sdl2/event/enum.Event.html) button events carry x/y directly, and [`Window::surface`](https://docs.rs/sdl2/0.38.0/sdl2/video/struct.Window.html#method.surface) supports direct window-surface pixels. However, SDL2's bundled feature compiles SDL's native C sources; the official Rust-SDL2 changelog records that its Windows GNU bundled support was not supported at the time of the 0.31 feature introduction. That path would add a C build and target-support risk. SDL2 0.38.0's Rust wrapper is MIT and SDL2 itself is zlib-licensed. No SDL backend, vendored library, or global tool is added by this assessment.

The existing minifb asset-inspector binary remains in place. The separate `genewars-controls` binary now uses the proposed winit/softbuffer backend and core's evidence-bounded packet/capture primitives. It displays only the two actual CD IFCUSTOM arrow controls, uses their selected/normal frames and explicit RLE opacity for pixel hits, and emits only events 27/28. It ignores right-button and focus changes. `--frames N` bounds host packet updates; the host schedules a nominal 60 Hz update using winit's event loop, which is a diagnostic choice and not original cadence.

Example, with private decoded input paths supplied by the operator:

```text
cargo run --offline --locked -p genewars-desktop --bin genewars-controls -- DECODED_IFCUSTOM.ANB MAIN.PAL --frames 3 --report NEW-CONTROLS.json
```

The report path is created exclusively and never overwrites an existing file. JSON includes successfully presented frames, scheduled packet updates, the nominal host rate, activated IDs, and bounded traces (first 4,096 records) of raw left down/up events and eventful produced packets (press, release, held, or non-sentinel control result), with current virtual cursor and result, plus dropped-trace counts. Idle packet updates still contribute to `packet_updates` but do not consume trace space. This makes a real human interaction auditable when a user launches the binary. The winit press event has no coordinate; hit testing uses the most recent cursor-move position at each host packet update, matching the analyzed producer's current-coordinate fields rather than the separate press-coordinate latch. The host still retains first-press coordinates internally for event-latch inspection, but does not feed them to UI hit testing. Its diagnostic display uses a fixed 640×480 source area and nearest-neighbor presentation scaling; pointer physical coordinates are mapped through the current physical client size. We have not claimed physical input or quick-click behavior until a Windows interaction is actually observed in a private report.

The first bounded Windows native launch completed three framebuffer presentations and three scheduled packet updates against the decoded CD IFCUSTOM database (36,660 bytes, SHA-256 `daf41bf104dd5d6436f7af23158a5d23d624c12780e3e66177b3b5aec2fbe006`) and `MAIN.PAL` (SHA-256 `8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05`). The report at `private/integration/panel/genewars-controls-smoke-20261012.json` records `exit_reason: frame_limit`, with no mouse transitions or activated controls. This verifies bounded window/surface presentation only; it is not physical input evidence.

A supervised Windows interaction was recorded in `private/integration/panel/controls-interaction-20261013-01.json`, with operator actions in `controls-ui-actions-20261013-01.jsonl`. Across 5,246 presented frames and packet updates, clicks inside the two arrow hit regions produced control IDs 27 and 28. A click in the black margin produced no control result. A drag from the up-arrow region released outside it; the latest cursor position was already outside at the first packet update, so this produced no activation. This confirms the observed host event stream and in-region clicks, but it does not test a press latched inside followed by a release outside after packet production has begun. Escape exited the window. The report records zero dropped event or packet records. These are host observations, not original-game cadence or behavior parity.

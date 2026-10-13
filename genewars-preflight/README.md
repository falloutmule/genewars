# GeneWars reconstruction workspace

This is an active, evidence-backed Rust reconstruction using the user's local GeneWars archive. It provides a native menu and mutable plant-world development build, resource tools, RNC1/sprite decoding, a CPU renderer and source-backed player/actor startup transactions. It does not yet run a complete playable game. See [RUNTIME.md](docs/RUNTIME.md) for launch controls and current limits.

The original [preflight](research/preflight.md) remains a dated baseline. Current work is tracked in [WORK_STATUS.md](docs/WORK_STATUS.md), [GAME_FEATURE_MATRIX.md](docs/GAME_FEATURE_MATRIX.md), and the [evidence ledger](research/EVIDENCE_LEDGER.md).

## Build and verify

Run from this directory using the existing project-local Rust toolchain:

```text
python -B scripts/cargo.py test --workspace --offline --locked
python -B scripts/cargo.py build -p genewars-tools -p genewars-desktop --offline --locked
python -B scripts/verify.py
```

`scripts/cargo.py` confines Cargo/Rustup homes and build output to `private/`. Core/assets/render use only the standard library. The desktop binaries pin minifb0.29.0 and winit0.30.13/softbuffer0.4.8, with checksums in Cargo.lock and a Windows dependency audit in `research/variants/windows-dependency-review.json`. GNU Rust1.99.0 is used on this host. Its bundled GNU import-library tool lacks an assembler; `scripts/prepare_llvm_dlltool.py` prepares a checksum-pinned official LLVM equivalent privately. See `research/integration/llvm-dlltool.json`. The build wrapper only sets child-process environment variables and never installs tools or changes global PATH.

`scripts/verify.py` checks Rust, all RNC records, ANB accounting, real panel pixels, bounded native-window launches, static arithmetic/LE mapping, private-file Git exclusion, archive SHA-256 and dated preflight preservation. One GPF ANB record retains an explicit unresolved failure; BONES now has a separately traced codec admission. The verifier does not redo broad extraction or overwrite the preflight receipt. Results go to `research/integration/verification.json`.

## Native research utility

The built executable is `private/build/rust-target/debug/genewars-tools.exe`:

```text
genewars-tools.exe rnc-info INPUT
genewars-tools.exe rnc-verify INPUT [MAX_BYTES]
genewars-tools.exe rnc-decode INPUT OUTPUT [MAX_BYTES]
genewars-tools.exe rnc-prefix INPUT OUTPUT [MAX_BYTES]
genewars-tools.exe panel-render PANEL_RNC MAIN_PAL OUTPUT_BMP
genewars-tools.exe sprite-verify SPRITE_DATABASE_PREFIX DECODED_TAB
genewars-tools.exe sprite-decode SPRITE_DATABASE_PREFIX DECODED_TAB RECORD INDEX_OUTPUT MASK_OUTPUT
genewars-tools.exe sprite4-verify SPRITE_DATABASE_PREFIX DECODED_TAB BASE
genewars-tools.exe sprite4-decode SPRITE_DATABASE_PREFIX DECODED_TAB RECORD BASE INDEX_OUTPUT MASK_OUTPUT
genewars-tools.exe distance 0 0 3 4
genewars-tools.exe campaign-plan 0 0
genewars-tools.exe campaign-inspect HEADER_PATH MISSION_PATH
genewars-tools.exe terrain-info MISSION_PATH
genewars-tools.exe terrain-height MISSION_PATH X Y
genewars-tools.exe terrain-tile DECODED_B32A TILE OUTPUT
genewars-tools.exe terrain-cell MISSION X Y CAMERA_X CAMERA_Y TOP ANCHOR_X ANCHOR_Y
```

`INPUT` may be `-` for binary stdin. Inputs are bounded to 64 MiB. The output limit defaults to 64 MiB and can be tightened. `rnc-decode` requires a complete single RNC stream. `rnc-prefix` explicitly reports consumed and remaining bytes; callers must account for the tail. Existing differing output files are never overwritten. The distance command returns squared distance using the CD candidate's wrapped arithmetic, with no assumed gameplay units.

`scripts/source_member.py ENTRY_ID --output private/PATH` copies one manifest-identified member, validates source and output hashes, and rejects ambiguous filename selection. Nested ZIP and existing raw-CD reads are targeted; no bundled programs are executed. Full-corpus validation is `python -B scripts/validate_rnc_corpus.py`.

`python -B scripts/render_panel.py` reconstructs the exact CD GENEPANL/MAIN.PAL pair and writes a private BMP/PNG. RGB6 display conversion is explicit floor(component*255/63), not a measured VGA monitor transfer. `python -B scripts/validate_sprites.py` accounts for every ANB; it intentionally exits nonzero while the documented GPF case remains unresolved.

Sprite CLI commands use an explicitly supplied database extent and table. For an embedded ANB, pass the original prefix ending at the first metadata offset, not the entire file; the corpus script does this automatically. Indexed8 and packed4 are distinct caller-selected codecs. The latter is admitted for 25 exact creature hashes and one separately traced BONES hash by executable caller evidence. BASE is an explicit 0-255 diagnostic palette offset; runtime palette and lighting selection remain separate. Desktop ANB modes enforce the metadata bound internally.

The native asset window can be opened with:

```text
python -B scripts/cargo.py run -p genewars-desktop --offline --locked -- panel ../private/integration/panel/GENEPANL.RNC ../private/integration/panel/MAIN.PAL
```

Paths passed through Cargo resolve from `rust/`. Escape or window close exits; `--frames 3` limits an automated presentation check. Actual three-frame launches passed. Physical keyboard/close input has not yet been independently exercised. The window displays original decoded assets; it is an inspector, not a game menu or simulation.

The separate event-driven control inspector uses the verified CD IFCUSTOM arrows:

```text
python -B scripts/cargo.py run -p genewars-desktop --bin genewars-controls --offline --locked -- ../private/integration/panel/ifcustom-controls-smoke-20261012.anb ../private/integration/panel/MAIN.PAL
```

A supervised Windows test observed events27/28, a nonactivating margin click and Escape exit. See `research/integration/native-controls-observation.json`. It preserves ordered button transitions and uses current cursor coordinates at each diagnostic60Hz packet update. This is not original game cadence or a full interface. The portable core also exposes a bounded construction-stage resource debit; callback, cancellation and later world mutations remain outside that helper.

## Native menu composition

`python -B scripts/menu_inputs.py` stages six exact CD resources privately, reusing the validated RNC outputs. `python -B scripts/render_menu.py --frames 3` exports a fresh private BMP/PNG and presents the native window for three frames. The executable also accepts `--window` for an interactive menu inspection until Escape or window close:

```text
private/build/rust-target/debug/genewars-menu.exe private/integration/menu private/integration/menu/titlescr.pal private/integration/menu/new-menu.bmp --window
```

Choose a new output filename; the executable refuses to overwrite one. The three-button composition uses the original FONT2 glyphs and TEXT.DAT strings, TITLESCR palette/background, and IFTITLE components. All 307,200 pixels match an independent instruction-derived composition. The proven scope assumes a zero UI origin, zero incoming draw flags and full 640×480 clip. The event host now runs the proved capture/release state, selected artwork and opacity hit tests. It records activation requests without dispatching game scenes. Optional `--report NEW_JSON` records requests, updates and presented frames. A supervised test observed Single Player/Options/Quit requests 0/3/2, transparent-corner rejection and Escape exit. Immediate host events and focus cancellation are explicit port policies; original polling cadence remains separate.

`python -B scripts/campaign_inputs.py` stages seven exact resources for the initial campaign/mission. The `campaign-plan` command accepts progress 0–23 and prior system 0–3; these are explicit port bounds. It preserves the original conditional system update. `campaign-inspect` validates the 699-byte header and tagged mission prefix and reports metadata only. It does not deserialize the opaque tail or run a mission. The native plan and first-file slices pass independent comparisons in `scripts/validate_campaign_plan.py`.

Terrain commands expose the proved 128×128 map layout and piecewise triangular height interpolation. Coordinates for `terrain-height` are signed 16-bit words, with cell indices in their high bytes and fractions in their low bytes. Its result is doubled height, not screen height or a gameplay-unit claim. `terrain-tile` exports 1,024 raw palette indices from one of 160 32×32 B32A tiles; zero is an opaque index for the traced mode-five terrain consumer. Every raw cell byte, tile byte and nearly one million real/synthetic height samples passed independent comparison. `terrain-cell` combines actual cell heights, texture/orientation and four UV/vertex positions with explicit unwrapped cell/camera coordinates and placement. Map reads wrap separately at 128. Its two triangle index triples describe cell topology, not the original scene draw order; overflow is rejected. This metadata command is separate from the full viewport renderer described below.

## Terrain viewport and native preview

The terrain renderer now handles the original strip order, clipped triangle preparation and draw modes0/5/6/15, including transparent B32A overlays and MAIN.GLA destination lookup. The separate full viewport uses explicit target/map-snapshot policies; it does not initialize a playable mission. See [TERRAIN_PREVIEW.md](docs/TERRAIN_PREVIEW.md) for commands and limits. Supervised native testing passed all four camera arrows, raw/reveal toggling and Escape after correcting named-key handling.

`terrain-camera` exposes the explicit-target camera words, and `terrain-shade` exposes the separately bounded entity-shade value0..64. `terrain-cell-render INPUT_DIR X Y FACTOR OUTPUT_BMP` renders an explicitly centered cell. `terrain-view INPUT_DIR TARGET_X TARGET_Y raw|raise4096 OUTPUT_BMP` renders all20 diagnostic columns. Full-view integration compares4,300,800 RGB pixels, with an additional independent native composition review covering5,836,800 indexed pixels.

## Evidence and distribution boundaries

The CD candidate is SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Its static analysis does not establish canonical retail behavior. Standalone Ghidra 12.0.1 uses the matched LX/LE extension; REA has not analyzed this executable. No incompatible symbol maps have been applied.

The first DOSBox-X reference attempt stopped at missing `config.ini`. Run 002 reached intro/menu/options artwork. Runs 003 and 004 used the statically proved English runtime copies and displayed readable labels. Guest mouse movement remained unresolved through both warp and raw input diagnostics, preventing a mission test. The sessions ended by verified-PID operator termination after UI close failed; exit 15 is not a game result. No reference process remains running. Window screenshots and logs remain private. The archive, media and executable hashes are unchanged; see [run 003 outcome](research/variants/reference-run-003-outcome.md) and [run 004 outcome](research/variants/reference-run-004-outcome.md). The latest user instruction authorizes necessary reference reruns. No installer or patcher ran.

All original resources, downloaded tools, raw decompiler exports and build caches stay under ignored `private/`. No commercial assets are embedded in Rust or synthetic tests. Third-party research license findings are in [independent-review.md](research/variants/independent-review.md). No product distribution license has been selected, and no remote publication is part of this work.

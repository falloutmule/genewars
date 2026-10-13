# Native terrain preview

The renderer reconstructs terrain from the exact CD candidate's CONQ001, B32A, FADE and MAIN resources. It preserves the original strip traversal, triangle preparation and mode-specific sampling arithmetic. It can also inspect one selected source plant with its original foreground and shadow. It is not a playable mission: simulation commands, objectives, audio, save/reload and original timing are absent.

From the project directory:

```text
python -B scripts/terrain_inputs.py
python -B scripts/cargo.py build -p genewars-tools -p genewars-desktop --offline --locked
private/build/rust-target/debug/genewars-tools.exe terrain-view private/integration/terrain-inputs 20736 13568 raise4096 private/integration/terrain-example.bmp
private/build/rust-target/debug/genewars-terrain.exe private/integration/terrain-inputs --target 20736 13568 --policy raise4096
private/build/rust-target/debug/genewars-tools.exe mission-source-view private/integration/terrain-inputs 0 raise4096 private/integration/mission-source-example.bmp
private/build/rust-target/debug/genewars-terrain.exe private/integration/terrain-inputs --source-player 0 --layout initial-genepanl --policy raise4096
```

Use a new BMP filename; differing existing output is rejected. The native window uses arrow keys to step the explicit target by 256, R to toggle the snapshot policy, and Escape to exit. These are inspector controls, not reconstructed game commands. `--frames 3 --report NEW_JSON` provides a bounded window check. Reports also record state changes.

Targets are two signed 16-bit coordinates. Their high bytes select cells and low bytes select fractions. Camera helper `0x82E20` computes its two byte-valued words from the target and map height. The preview translates those words modulo128 while preserving relative projection and wrapped source reads. The default `full` layout draws twenty columns as an explicit diagnostic policy.

The source view selects a raw target from the bounded mission parser. Player index0 reads `(20736,13568)` from the exact CONQ001 table and derives camera `(66,48)`; selection is explicit, not recovered from a running session. `initial-genepanl` uses the proved initial traversal, columns4–19, full640-pixel clip/pitch and temporary context bit4. It copies the decoded GENEPANL background into x0–127. GENEPANL remains an explicit artwork choice: new static evidence instead selects MAINPANL under particular campaign/progress/empty-child gates, whose live values are not inferred here. The original pitch equivalence assumes successful display locking and inherited pitch640. Widgets and live mission startup are absent.

`raw` renders the map's saved factor bytes unchanged. `raise4096` starts with a fresh map copy and applies only the independently checked local factor-increase plan around the explicit target. It does not perform entity callbacks, dirty-region handling or a complete live visibility update. Each key change starts from the same source snapshot, so it does not accumulate exploration state. Raw CONQ001 factors produce a black viewport in the tested areas; the alternate snapshot makes the terrain visible without pretending that the whole mission has been initialized.

Base terrain uses mode5 with the original coupled carry arithmetic. Mode6 uses separate fixed-point accumulators and treats source index0 as transparent; its overlays come from B32A tiles157–159. Geometry-only mode0 and mode15/MAIN.GLA are implemented separately. The shared full-view API returns dirty requests without mutating its input. A missing requested table or an unsupported source address fails explicitly.

The cell integration check compares84 real-resource renders (25,804,800 RGB pixels). The viewport check compares14 raw/raised snapshots (4,300,800 RGB pixels), including wrapped coordinates, fractional targets and all three overlay test locations. These checks compose independently reviewed static models; they are not comparisons against an original-game framebuffer. See `research/integration/terrain-cell-render.json`, `terrain-view.json`, and the independent span/triangle/strip/modes reviews under `research/variants/`.

The source-view integration compares eight frames (2,457,600 pixels), including all four target entries and both policies, rejects invalid requests, verifies unchanged inputs and opens a bounded three-frame native window. Independent renderer review covers39 cases,33 complete frames,14 initial-layout frames,7 panel copies and165 invalid composition checks. The source parser preserves all serialized bytes and exposes eight numeric families; it creates no runtime pointers or vtables.

Original resources, rendered images, native binaries and window captures remain in ignored private storage. The original archive is unchanged.

## Selected plant inspection

Stage the twelve exact CD plant resources, then select a source record explicitly:

```text
python -B scripts/plant_inputs.py
private/build/rust-target/debug/genewars-tools.exe mission-plant-view private/integration/terrain-inputs private/integration/plant-inputs 0 29 raise4096 sampled private/integration/plant-example.bmp
private/build/rust-target/debug/genewars-terrain.exe private/integration/terrain-inputs --source-player 0 --layout initial-genepanl --policy raise4096 --plant-record 29 --plant-resources private/integration/plant-inputs --plant-shade sampled
```

`source` uses the serialized shade byte; `sampled` explicitly calls the proved shade helper on the chosen map snapshot. Neither policy asserts the original shade-refresh schedule. The selected plant uses its source resource selector, animation and frame, signed projection, odd-row MAIN.GLA shadow, then indexed foreground with the original direct/FADE dispatch. It clips to the right-side512x480 view while preserving the640-byte pitch. Unsupported flags, selectors or extents reject before publishing output.

This single-record selection deliberately leaves other entities out of the inspector. The separate queue helpers reproduce the proved admission rectangle,256-entry limit, special offscreen buckets, reverse equal-row order and flag packets for explicit inputs; they do not infer a complete initial world. Scheduled updates, other-family capacity competition and the dirty prepass still need integration.

The plant integration compares21 frames /6,451,200 pixels with independent models and rejects five invalid requests without clobbering output. A bounded native window matches the CLI RGB checksum. A supervised Right/R/Left/R/Escape sequence produced five presentations and restored the original plant/terrain view; see `research/integration/plant-controls-observed.json`.

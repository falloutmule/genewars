# Native terrain preview

The renderer reconstructs terrain from the exact CD candidate's CONQ001, B32A, FADE and MAIN resources. It preserves the original strip traversal, triangle preparation and mode-specific sampling arithmetic. It is a terrain diagnostic, not a playable mission: there are no entities, simulation commands, objectives, audio, save/reload or original timing.

From the project directory:

```text
python -B scripts/terrain_inputs.py
python -B scripts/cargo.py build -p genewars-tools -p genewars-desktop --offline --locked
private/build/rust-target/debug/genewars-tools.exe terrain-view private/integration/terrain-inputs 20736 13568 raise4096 private/integration/terrain-example.bmp
private/build/rust-target/debug/genewars-terrain.exe private/integration/terrain-inputs --target 20736 13568 --policy raise4096
```

Use a new BMP filename; differing existing output is rejected. The native window uses arrow keys to step the explicit target by 256, R to toggle the snapshot policy, and Escape to exit. These are inspector controls, not reconstructed game commands. `--frames 3 --report NEW_JSON` provides a bounded window check. Reports also record state changes.

Targets are two signed 16-bit coordinates. Their high bytes select cells and low bytes select fractions. Camera helper `0x82E20` computes its two byte-valued words from the explicit target and map height. The preview translates those words modulo128 while preserving relative projection and wrapped source reads. Its full640x480 viewport, twenty columns and zero context flags are explicit diagnostic choices; the original viewport initialization remains separate work.

`raw` renders the map's saved factor bytes unchanged. `raise4096` starts with a fresh map copy and applies only the independently checked local factor-increase plan around the explicit target. It does not perform entity callbacks, dirty-region handling or a complete live visibility update. Each key change starts from the same source snapshot, so it does not accumulate exploration state. Raw CONQ001 factors produce a black viewport in the tested areas; the alternate snapshot makes the terrain visible without pretending that the whole mission has been initialized.

Base terrain uses mode5 with the original coupled carry arithmetic. Mode6 uses separate fixed-point accumulators and treats source index0 as transparent; its overlays come from B32A tiles157–159. Geometry-only mode0 and mode15/MAIN.GLA are implemented separately. The shared full-view API returns dirty requests without mutating its input. A missing requested table or an unsupported source address fails explicitly.

The cell integration check compares84 real-resource renders (25,804,800 RGB pixels). The viewport check compares14 raw/raised snapshots (4,300,800 RGB pixels), including wrapped coordinates, fractional targets and all three overlay test locations. These checks compose independently reviewed static models; they are not comparisons against an original-game framebuffer. See `research/integration/terrain-cell-render.json`, `terrain-view.json`, and the independent span/triangle/strip/modes reviews under `research/variants/`.

Original resources, rendered images, native binaries and window captures remain in ignored private storage. The original archive is unchanged.

# CONQ map cells and B32 terrain atlas

## Proven byte layouts

The CD CONQ001 loader copies the 243,038-byte fast prefix directly into state. The map cell region begins at file offset `0x355E` and contains exactly `128 × 128` row-major records of 14 bytes (229,376 bytes, ending at `0x3B55E`). The traced height sampler reads a signed little-endian 16-bit value at cell offset `+2`; a separate sampler reads the texture selector at `+4`. Other bytes are left unnamed by the asset API.

The traced coordinate words are signed 8.8 values. Cell coordinates are `(signed_coordinate >> 8) & 127`, fractional coordinates are the low byte, and each `+1` neighbor wraps modulo 128. For corner heights `h00`, `h10`, `h01`, `h11` and fractions `fx`, `fy`, the sampler returns a doubled height:

```text
if fx < fy:
    2*h00 + (((h11-h01)*fx + (h01-h00)*fy) >> 7)
else:
    2*h00 + (((h10-h00)*fx + (h11-h10)*fy) >> 7)
```

The shift is signed arithmetic right shift. Intermediate values fit `i32` for every `i16` corner height and byte fraction. `genewars_core::cd_candidate::interpolate_height` accepts the exact map region and returns this integer result; it does not implement camera placement, projection, clipping, or shading.

The renderer trace supplies explicit unwrapped integer cell deltas from a caller-selected camera cell: `dx = map_x - camera_x`, `dy = map_y - camera_y`. The vertex buffer offset is `x = (dx - dy) * 32`, `y = (dx + dy) * 16 - 2 * (height >> 6) - vertical_top`. The height shift occurs before doubling. The input camera cell and `vertical_top` are explicit parameters; the initial camera placement is unknown. Geometry deltas are not masked at the map seam, although the height lookup wraps its cell indices. The safe Rust helper rejects `i32` overflow rather than reproducing source wraparound.

The four cell vertices and their UVs are ordered `[00, 10, 11, 01]`, with triangles `[00, 11, 01]` and `[00, 10, 11]`. The UV row index is `(cell_byte_5 & 0x70) >> 4`; the eight 16.16 corner permutations are returned by the render helper without interpolation. Mode 5 applies the fade table to sampled indices and does not skip index zero. These primitives do not implement projection, camera selection, UV interpolation, visibility, or full terrain rendering.

The vertex shade uses a signed 16-bit operand at cell `+0` and an unsigned byte factor at `+12`: `shade_16_16 = 4 * light_operand * factor_operand + 0x8000`. The factor's producer and meaning are unresolved, so `terrain_vertex_shade_16_16` accepts it explicitly. The mode-5 sampler computes one source byte at `texture_base + (v_byte << 8) + u_byte`, then writes `FADE[shade_row * 256 + source_index]`; zero source indices are still mapped. `mode5_sample_index` implements the single-sample operation. `render_mode5_span` advances an already-prepared horizontal span using the proven packed carry coupling; it does not derive edge gradients or shade rows. CD `FADE.DAT` (`root/2/disc/61`, 16,384 decoded bytes) is a 64×256 lookup; `TerrainFadeTable` requires the exact extent and maps a caller-selected shade row `0..63` and source palette index to one output index. It does not derive the row from the shade value or pick a light factor.

`GAME/DATA/B32A.DAT` at manifest entry `root/2/disc/40` is a 163,840-byte decoded indexed atlas. It contains 160 32×32 tiles with eight tiles per row and a 256-byte source row pitch. Tile `n` begins at `(n / 8) * 8192 + (n % 8) * 32`; tile indices 0–159 are valid. Startup's index 160 is a one-past-end sentinel and is rejected by the safe API. Mode 5 writes sampled indices through its fade table; index zero is not treated as transparent. `B32Atlas::tile` returns raw indices and assigns no palette or presentation semantics.

## Evidence and limits

The instruction-listing hashes, exact resource identities, byte offsets, coordinate and height vectors, and all tile address/hash vectors are recorded in [`terrain-evidence.json`](../ghidra/terrain-evidence.json). Reproduce the static and byte checks with `python -B scripts/ghidra/validate_terrain_bridge.py`; the receipt records 12 instruction contracts, 393,216 independent height comparisons, 42 explicit height vectors, seven signed-coordinate boundary vectors, and hashes for all 160 tiles covering the source allocation. The vertex and UV contracts are recorded in [`terrain-projection-evidence.json`](../ghidra/terrain-projection-evidence.json) and reproduced by `python -B scripts/ghidra/validate_terrain_projection_bridge.py`; it checks four instruction contracts, 307,200 alternating-strip vertex comparisons, eight UV bindings, shade arithmetic, and all 64 fade rows.

The exact source identities are CONQ001 (`root/2/disc/152`, SHA-256 `72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2`) and B32A.DAT (`root/2/disc/40`, input SHA-256 `481677e31b05d9e4ba428932a4eaf9003fae4fe320f90daf22a947cbd3e4ef9c`, decoded SHA-256 `8206e4be65dd7f1fa12df131cf2d1aab5f5b47f7181c8ace86facbb1787bcdb8`).

Live terrain shading, visibility fields, whole-frame rasterization, and the original camera position remain unimplemented. The atlas reader does not associate a palette or infer rendering behavior from the map's texture selector.

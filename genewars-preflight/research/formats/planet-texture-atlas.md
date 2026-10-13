# Numbered planet texture atlases

## Static contract

The CD planet-selection initializer at VA `0x15B90` formats `data/pl%d.dat` from the current system byte and loads that resource into a buffer stored at screen offset `+0x18`. The descriptor at VA `0x102E58` independently names `data/planets.anb`. The file resolver passes the generated name unchanged to the whole-file reader. Static analysis identifies the PL buffer as texture data consumed by VA `0x192D0`, not as a complete 640×480 raster.

In mode 3, the sampler at VA `0xE3A4F` reads one byte at `texture_base + ((low8(v) << 8) | low8(u))`. A zero byte leaves the destination unchanged; nonzero bytes are copied as their original index. The source row pitch is 256 bytes. Tile types 0–7 address 128×128 regions at `(type & 1) * 128 + (type >> 1) * 32768`; types 8–13 address 64×64 regions at `131072 + floor((type-8)/4) * 16384 + ((type-8) % 4) * 64`. These formulas are from the CD consumer and sampler disassembly, with exact instructions and hash-bound input/output calculations recorded in `research/ghidra/planet-texture-evidence.json`.

The four CD resources `PL1.DAT` through `PL4.DAT` each decode to exactly 163,840 bytes, with distinct decoded hashes. The evidence script checks every tile offset and every sampled source index for all four exact-hash inputs: 56 tiles and 622,592 output indices total. Each accessed index is inside its decoded buffer. This supports a 256×640 byte-addressed extent for these decoded samples and the fixed-stride accesses; it does not establish a general file format for unrelated resources.

## Rust API and limits

`genewars-assets` exposes `PlanetTextureAtlas::parse(&[u8])`, which requires exactly 163,840 bytes, and an explicit `tile(kind)` operation for kinds 0–13. It gathers each tile into contiguous raw index bytes using the 256-byte source stride. `PlanetTextureTile::is_opaque(x, y)` reports `false` for index zero and `true` for every nonzero index. Invalid kinds and malformed atlas lengths are rejected. Synthetic tests cover all tile address formulas, row stride, final source index, malformed lengths, out-of-range kinds, and transparency.

The API does not associate a palette, convert indices to RGB, implement UV interpolation or projection, set screen positions, or claim full game rendering. `PLANETS.DAT` remains consumer-unproven and is not parsed by this API. The `PLANETS.ANB` from `root/4/145` is a different build variant and is not paired with the CD `PL1.DAT` evidence.

The evidence receipt is reproducibly checked by `python -B scripts/ghidra/validate_planet_texture_bridge.py`; the reported result is 7 instruction-listing contracts, 56 atlas tiles, and 622,592 indices. Private disassembly and original/decoded bytes remain ignored under `private/`.

# Numbered planet textures; PLANETS.DAT remains unresolved

The exact CD candidate does not establish a record parser for `PLANETS.DAT`. None of its four mapped object images contains that literal filename, and the inspected selection-screen path instead loads numbered `data/pl%d.dat` files and an independent `data/planets.anb` database. This is bounded negative evidence, not proof that PLANETS.DAT is dead or unreachable through every possible computed filename. Its 245,760 decoded bytes and possible 24-way partition do not establish planet records.

## Actual loader and consumer

Initializer `0x15B90` formats the numbered path using the unsigned system byte at `0x11945C + player*0x70D`, incremented by one, through format string `0x10D097`. It calls `0x154F0` at `0x15CF0`, with destination from global `0x115E6C`, then saves that buffer at screen+`0x18`. At `0x15D07` it separately loads the descriptor list beginning at `0x102E58`, whose first filename is `data/planets.anb` and second is `data/levscr01.pal`.

Descriptor loader `0x15410 -> 0x9A09C` passes the resolved filename to the whole-file loader `0x9954C`. Configuration setup assigns resolver `0x150B0`, which joins a selected configured prefix and filename; there is no ANB-to-DAT extension substitution in this path. Loading the palette descriptor alone does not establish the palette upload association, which remains outside this tranche.

Render callback `0x17C50` calls `0x192D0`. That consumer reads the numbered-file buffer from screen+`0x18`, chooses a texture base, sets rendering mode byte `0x102E25 = 3`, and calls triangle routine `0xE1900` twice for a quadrilateral. The mode dispatch at `0xE1F56` uses table `0xE3513`, whose entry 3 is `0xE3A4F`.

The direct byte sampler at `0xE3ADC..0xE3AF9` places the low integer byte of v in BH and that of u in BL, reads one byte at `texture_base + BX`, and writes that byte unchanged only when it is nonzero. This proves a source stride of 256 and index-zero transparency for this rendering mode. It does not prove a modern RGB palette or full triangle interpolation behavior.

## Bounded tile extraction

These rules apply to the four exact CD inputs PL1 through PL4 in `planet-texture-evidence.json`, each independently validated as 163,840 decoded bytes. The API can expose an explicit type selector 0 through 13 and return row-major index bytes with an opacity mask `index != 0`.

| Type | Width and height | Source base | Source row stride |
|---|---|---|---|
| 0 through 7 | 128 × 128 | `(type & 1)*128 + (type >> 1)*32768` | 256 |
| 8 through 13 | 64 × 64 | `131072 + ((type-8)/4)*16384 + ((type-8)%4)*64` | 256 |

For the first group, consumer `0x19694..0x196D9` supplies 16.16 texture coordinates from 0 through 127. For the second, `0x1942F..0x1948B` supplies coordinates from 0 through 63. The extraction contract reads `base + y*256 + x` for each bounded integer coordinate. Types 14 and above must be rejected by this bounded API; remaining allocation slots are not interpreted merely because they fit.

This layout is a set of textures used by projected geometry, not a flat selection-screen image. Tile extraction leaves projection, interpolation, clipping, draw order, input hit testing, palette selection, and RGB conversion to separately proven contracts.

## Reproduction

Run `python -B scripts/ghidra/validate_planet_texture_bridge.py` from the project directory. It checks the CD candidate hash, loader strings and pointers, mode dispatch, seven private instruction-listing contracts, decoded hashes, and all 622,592 indices across 56 admitted tiles. Each tile has a source extent, index hash, opacity hash, and opaque count in the safe JSON receipt. No proprietary pixel arrays are stored in research outputs, and no original executable is run.

Private evidence is in `private/ghidra/planets-refs`, `planets-file`, `planets-consumers`, and `planets-texture`; the mode-three sampler is `planets-texture/mode3-sampler.txt`. PLANETS.DAT entry `root/2/disc/122`, SHA-256 `1e3b70c2b6856c645af01db2a98f8a71f99b7c823324c24f6406bce734c5e23c`, remains byte-only reconnaissance pending an actual consumer trace.

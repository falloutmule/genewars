# First-mission terrain records, height sampling, and texture atlas

This contract applies to CD executable candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba` and the exact first-mission resource selected in `campaign-entry-bridge.md`. It establishes fields and numerical consumers for a bounded native terrain inspection. It does not reproduce a complete original terrain frame.

## Map dimensions and fields

The map region begins at CONQ file offset `0x355E`, corresponding to live state `0x11C2BE`. The proven fast-prefix path copies this region unchanged; its length is 229,376 bytes. Consumers establish the dimensions directly: `0x74CC0` sign-extends the two coordinate words, shifts each right by eight, masks each with `0x7F`, and computes `base + y*0x700 + x*14`. Thus this is a row-major 128×128 wrapping map, with 14-byte records and 1,792-byte rows. Utility `0x8C860` independently splits an index into `index&127` and `index>>7`, and wraps neighboring coordinates with the same mask. The shape is not inferred solely from the record count.

| Record offset | Proven representation and use |
|---|---|
| +2 | Little-endian signed16 height, used for terrain vertical geometry |
| +4 | Unsigned8 texture selector, used to index texture pointer table `0x115BEC` |
| +5 | Raw byte; bits `0x70` select texture-coordinate orientation |
| +0 and +12 | Signed16 and unsigned8 operands in vertex lighting calculations; no gameplay field names assigned |
| Remaining bytes/bits | Opaque in this contract |

The first exact `CONQ001.DAT` input is entry `root/2/disc/152`, SHA-256 `72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2`. Its initial heights range from zero to 4,800, and texture selectors from zero to 118. Those measurements do not constrain all possible live state values. A safe raw-record accessor accepts exactly the region extent and explicit cell coordinates 0 through 127; the separate fixed-point sampler implements the original wrapping semantics.

## Piecewise height interpolation

Function `0x7B140` takes a pointer to signed16 x and y words in EAX and returns a signed32 **doubled height**. Low coordinate bytes are unsigned fractions `fx`, `fy` in 0 through 255. Cell coordinates are `(signed_coordinate >> 8) & 127`; the next x and y wrap independently modulo 128. Read signed16 heights `h00`, `h10`, `h01`, `h11` at the four corners.

- If `fx < fy`, numerator is `(h11-h01)*fx + (h01-h00)*fy`.
- Otherwise, numerator is `(h10-h00)*fx + (h11-h10)*fy`.
- Return `2*h00 + (numerator >> 7)`, using arithmetic right shift.

The diagonal is from corner 00 to corner 11. This is piecewise triangle interpolation, not bilinear interpolation. Equality uses the second branch. At a grid vertex the result is twice its stored height. A numerator of -1 shifts to -1, so division that truncates toward zero is incompatible. Full signed16 heights and unsigned8 fractions fit all intermediates in signed32; the result lies in [-65,536,65,534]. The receipt contains 42 synthetic height vectors and seven signed-coordinate/wrap boundary vectors. Independent barycentric weighting agrees for all 65,536 fraction pairs across six height sets, totaling 393,216 comparisons.

## B32A texture storage and sampler

Startup `0x1D770` loads descriptor `0x1038B8`, filename `data/b32a.dat`, into allocation global `0x115E6C`. At `0x1DB7A..0x1DDFB`, it constructs texture pointers at `0x115BEC` with base

`atlas + (texture/8)*8192 + (texture%8)*32`.

Exact CD entry `root/2/disc/40` has compressed input SHA-256 `481677e31b05d9e4ba428932a4eaf9003fae4fe320f90daf22a947cbd3e4ef9c`. Its independently decoded 163,840 bytes have SHA-256 `8206e4be65dd7f1fa12df131cf2d1aab5f5b47f7181c8ace86facbb1787bcdb8`. There are 160 complete 32×32 textures, indexed 0 through 159, arranged eight per source row group, with a source pitch of 256 bytes. Safe tile extraction reads `base + y*256 + x` for x/y 0 through 31. Across all tiles each atlas byte is covered exactly once. Startup also computes pointer 160 at precisely one past the allocation; that does not authorize reading another texture. Safe access rejects selectors 160 and above.

Terrain strip routine `0x7B7F0` reads each cached vertex's texture selector, chooses its pointer, and selects UV orientation `(record_byte5 & 0x70)>>4`. The table at `0x1060EC` has eight 32-byte records, each containing four pairs of 16.16 coordinates whose endpoints are zero and `0x1FFFFF` (31 plus fractional bits). It supplies corner permutations to the two triangles. The safe receipt contains the exact geometric table values; complete strip traversal and raster coverage remain separate work.

Mode byte five dispatches to `0xE3F93`. The sampler at `0xE40D0..0xE40E9` uses integer y in BH and x in BL, proving the 256-byte pitch. It reads the source palette index, places the interpolated shade in AH, then reads `fade[shade*256 + index]` at `0x188F48` and writes the resulting index. There is **no index-zero transparency test**. A B32 tile API returns raw indices; callers must not apply the earlier PL-atlas zero-transparency rule. The actual mode-five output is shaded, so direct palette conversion of a raw texture is an inspection image, not proof of original terrain colors.

## Geometry and palette association

Constructor `0x8DB90` establishes horizontal grid step 32, diagonal vertical step 16, and height factor two. It loads ECX=2 before `0x402E0`; that routine does not modify ECX, and the remaining called helpers either preserve ECX or do not touch it before the store at `0x8DC29`. The terrain vertex routine sign-extends record+2, shifts it right by six, multiplies by factor two, and subtracts that displacement from its diagonal vertical position. Preserve `2*(height>>6)`; substituting `(2*height)>>6` changes rounding. The arbitrary-point projection routine `0x7AF00` uses interpolated doubled height before its final shift, so it must not silently replace the terrain-vertex rounding rule.

Startup descriptor `0x1043E4` names `data/main.pal` and stores its allocation in `0x115E80`. Startup `0x1DAE7..0x1DAF6` places that pointer at `0x1863A4`, the palette field at active mission view `0x1863A2 + 2`. Generic view presentation reads view+2 and passes it to palette routine `0xA3DA0`. Exact MAIN.PAL provenance is entry `root/2/disc/108`, SHA-256 `8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05`; RGB DAC order and six-bit range are established separately in `renderer-bridge.md`.

Initial camera origin, live visibility/shade state, mode-five interpolation/clipping details, overlay layers, and the complete rasterizer remain unresolved. In particular, cell+12 is zero throughout the initial CONQ001 bytes and is later used in lighting; raw loaded data is not sufficient to claim the original first visible terrain frame. A native view may accurately expose the verified heights and texture resources while naming its camera and shading choices as inspection choices.

## Reproduction and analysis repair

Run `python -B scripts/ghidra/validate_terrain_bridge.py` from the project directory. It verifies 12 private instruction contracts, exact mapped descriptors, all height/index boundary vectors, all 160 atlas hashes and full byte coverage, and the first map's field extents. `terrain-evidence.json` is the safe receipt. Raw instructions and decoded resources remain private.

The constructor export uses the actual entry `0x8DB90`. A previous exploratory range began at interior byte `0x8DBE0` and produced an invalid first instruction; that unused exploratory file is not evidence. Only Ghidra analysis code units in `[0x8DB90,0x8DBE5]` were cleared and correctly disassembled from the function entry with `RepairAnalysisRange.java`. Original executable bytes and reconstructed mapped bytes were not changed. `ExportExistingRange.java` is available for future read-only range inspection without disassembling a possibly interior start address.

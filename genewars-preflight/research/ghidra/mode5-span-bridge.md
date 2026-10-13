# Exact mode-five prepared horizontal spans

This is a bounded static contract for the exact CD candidate and sampler `0xE3F93..0xE42A9`. It consumes an already prepared scanline; it does not yet provide triangle edge setup, vertical coverage, culling, or gradient derivation. The original mode-five pixel loop uses packed arithmetic with coupled carries, so three independent fixed-point accumulators are not an equivalent replacement.

## Inputs and horizontal clipping

A prepared row contains signed32 16.16 left/right x, U, V and shade S. Three signed32 gradients dU/dV/dS describe per-pixel steps. The caller supplies the horizontal clip width, full pitched texture allocation plus selected texture base, and exact 16,384-byte FADE data. All attribute addition and multiplication during clipping uses wrapping32 arithmetic.

Let `left=left_fixed>>16`, `right=right_fixed>>16`, with arithmetic shifts. For positive `clip_width`, covered x is `[max(left,0), min(right,clip_width))`: left inclusive, right exclusive, both using floor rather than fractional ceiling. Empty spans produce no samples. If left is negative, advance each full32 U/V/S by `(-left)*its_gradient` before packing. Right clipping only reduces the sample count. A safe zero-width API returns empty as an explicit safety deviation: the original negative-left path checks right before clamping and can enter its 16-lane unroll with count zero when clip width is zero. Actual viewport width 640 is within the source-equivalent positive domain.

The source reads a 20-byte scanline record at `0x100124` with fields left/right/U/V/S at +0/+4/+8/+0xC/+0x10. The mode-five loop interprets horizontal limits and performs negative-left advancement at `0xE3FD6..0xE406E`. This receipt deliberately leaves the record's row placement and vertical triangle traversal to later evidence.

## Packed state and gradient

Interpret signed32 input bit patterns as unsigned for the following packing. `rol16` rotates the full32 word left by16.

| Register role | Initial state | Gradient |
|---|---|---|
| C, u32 | `(rol16(U)&0xFFFF0000) | ((S>>8)&0xFFFF)` | `(rol16(dU)&0xFFFF0000) | ((dS>>8)&0xFFFF)` |
| D, u32 | `(rol16(V)&0xFFFF0000) | ((U>>16)&255)` | `(rol16(dV)&0xFFFFFF00) | ((dU>>16)&255)` |
| v, u8 | `(V>>16)&255` | `(dV>>16)&255` |

The gradient's middle byte in D is preserved from rotated dV; the initial state's middle byte is zeroed. Shade loses its lowest eight fractional bits during packing. The gradient preparation literally compares dS against zero and uses JNC to skip DEC of the gradient-U register. The exact original bytes at `0xE3FAB` are `83 FD 00 73 01 49`; CMP against zero cannot borrow, so the decrement is skipped for negative dS as well. Preserve this observed behavior rather than adding a conventional negative-gradient correction.

## Sample and advance

For every covered pixel, in this order:

1. Read `u=D&255`, `v=v_byte`, `shade=(C>>8)&255`.
2. Read palette index from `source[texture_base + v*256 + u]`.
3. Read output index from `fade[shade*256 + source_index]` and emit it, including when the source or output index is zero.
4. Add gradient C to C with unsigned32 carry.
5. Add gradient D and C's carry to D, producing another unsigned32 carry.
6. Add gradient v and D's carry to the v byte with modulo256 wrapping.

The assembly interleaves the reads with the additions, but uses the captured pre-add u/v/shade for the emitted pixel. Its unrolled sixteen-pixel loop has a remainder jump and destination adjustment. The validator checks all widths1 through640 and proves that it writes exactly consecutive offsets0 through count-1 and advances the packed state once per emitted pixel.

The negative-shade synthetic fixture is intentional: U starts at2.ffff, dU=0, and dS=-0.5. The literal packed carry advances integer u from2 to3 after the first sample. A clean-looking implementation with independent U/V/S updates would miss that behavior. The contract records what the instructions do without assigning intent to this quirk.

## Safe API and fixtures

A safe span API can return a start x and a vector of output indices after validating all accesses. Require exact FADE length16384 and shade byte below64; check every full-source address. Do not clamp texture coordinates to a32×32 tile or convert the source to a compact1024-byte array for this sampler: the actual address uses a256-byte pitch and byte coordinates. Invalid accesses must report an error, rather than extending the source bounds or changing the shade to manufacture output. Preparing output before applying destination writes avoids partial mutation on rejection; that is an explicit safety improvement over unchecked source accesses.

`mode5-span-evidence.json` contains eight complete synthetic fixtures using independently reproducible patterned source/FADE data. Four additional fixtures sample exact CD B32A tiles0,7,8,159 with exact FADE; their expected bytes remain under `private/ghidra/terrain-span`, with hashes and input parameters in the safe receipt. Reproduce with `python -B scripts/ghidra/validate_mode5_span_bridge.py`: 5,184 packed-carry cases,640 unroll widths, eight synthetic spans and four actual resource fixtures pass. Raw proprietary samples are not written to research outputs. No original executable is run.

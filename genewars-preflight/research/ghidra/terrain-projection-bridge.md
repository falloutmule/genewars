# Explicit terrain vertices, triangle binding, and shade inputs

This supplements `terrain-bridge.md` for the same exact CD candidate. It specifies the terrain strip path at `0x7B7F0`, with explicit camera and lighting inputs. It does not substitute the separate arbitrary-point routine `0x7AF00` or infer the original first camera position.

## Strip-derived vertex transform

Let integer `world_x/world_y` be unwrapped cell coordinates and `camera_x/camera_y` be the current signed32 globals `0x184FDC` and `0x184FE0`. Map record lookup masks each axis with 127; projection does not mask the deltas. A seam neighbor after world cell 127 is represented as world cell 128 while its height lookup wraps to record zero.

For `dx=world_x-camera_x`, `dy=world_y-camera_y`, signed16 height `h`, and explicit strip top input:

`buffer_x = (dx-dy)*32`

`buffer_y = (dx+dy)*16 - 2*(i32(h)>>6) - vertical_top`

The source performs 32-bit arithmetic. A safe checked port may reject overflowing intermediate coordinates; that restriction must be explicit. Additional screen anchoring is caller placement policy, not a newly discovered original camera setting. Source strip coordinates use anchor zero. The height shift occurs before multiplication and remains arithmetic for negative heights.

This transform follows from the alternating strip traversal, rather than assuming a conventional isometric projection. For column `c`, define `n=c>>1` and `p=c&1`. Initial map coordinates are `x0=camera_x+n`, `y0=camera_y-n-1-p`. The two primed cached vertices are A=(x0,y0) and B=(x0,y0+1). Their buffer x values are `32*(c+1)` and `32*c`; their height-free y values are `-16*(1+p)-top` and that value plus 16. The loop constructs C=(x0+1,y0+1) at the right x with y increased another 16, then D=(x0+1,y0+2) at the left x with y increased another 16. Each iteration advances x0 and y0 together and rotates the cached vertices. Substitution yields the two delta formulas for both column parities and every loop step.

The private listing is `terrain-map-refs/instructions-0007b7f0.txt`: horizontal setup at `0x7B810..0x7B82A`, camera and parity setup at `0x7B83B..0x7B8A9`, signed height displacement at `0x7B8F1..0x7B905`, and subsequent vertex construction in the same function. The receipt checks 307,200 vertices from explicit strip recurrence against the generalized formula and includes concrete even/odd examples.

## Actual triangle and UV order

Name one cell's four corners `[00,10,11,01]`. Call `0x7BB8E` passes cached A,C,B, which are that cell's `[00,11,01]`, and reads UV pairs at byte offsets 0,16,24. Call `0x7BD9D` passes B,C,D, which are the next attached cell's `[00,10,11]`, and reads UV pairs at offsets 0,8,16. Thus the two triangle index lists are `[[0,2,3],[0,1,2]]` in the stated corner order. Both share the 00-to-11 diagonal used by the height sampler. The result is established by actual strip cache traversal and argument order, not by visual convention.

Orientation index `(record_byte5 & 0x70)>>4` chooses one of eight four-pair rows at `0x1060EC`. The pairs are unsigned 16.16 coordinates with endpoints zero and `0x1FFFFF`; all eight values are pinned in `terrain-evidence.json` and expanded into triangle bindings in `terrain-projection-evidence.json`. The source leaves the final fractional bits set at the far edge; replacing that endpoint with `31<<16` changes interpolation. This contract supplies vertex attributes but does not yet establish which raster pixels own shared edges.

## Explicit lighting and FADE lookup

Vertex assembly at `0x7B910..0x7B937` sign-extends record+0, shifts it left eight, multiplies by unsigned byte record+12, divides by 64 with signed truncation, and adds `0x8000`. The product always has a factor 256, so the division is exact. The equivalent signed32 16.16 shade is

`shade_fixed = 4 * light_i16 * factor_u8 + 0x8000`.

The full signed16/unsigned8 product fits signed32. No meaning such as explored, visible, or brightness percentage is assigned to the factor without tracing its producer. Initial CONQ001 has factor zero; choosing a nonzero factor in an inspection is explicit caller policy, not an original startup-state claim.

Mode-five sampling reads a texture index, combines the interpolated shade row with that index, reads FADE, and always writes the output index. Exact CD `FADE.DAT` entry `root/2/disc/61`, compressed SHA-256 `ddef38a3e63fbb3680aac15569f12e02f141036ffa270ddd48cdc82bce361503`, decodes to 16,384 bytes with SHA-256 `b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392`. A safe lookup accepts explicit shade indices 0 through 63 and source indices 0 through 255, returning `fade[shade*256+index]`. Source index zero remains opaque. The original sampler can form a wider byte row; the allocation justifies the safe 64-row admission, not silent wrapping or clamping to manufacture valid output.

Camera setter `0x829A0` sign-extends two stored cell words into the camera globals. Target-anchoring helper `0x82E20` separately samples height and computes byte-based offsets before storing them; this does not identify an initial mission target by itself. The bounded API therefore accepts camera inputs directly. Visibility production, camera selection, clipping, gradient rounding, scanline coverage, overlays and timing remain separate work.

Reproduce with `python -B scripts/ghidra/validate_terrain_projection_bridge.py`. The safe receipt pins four private instruction contracts, strip recurrence comparisons, eight UV bindings, 262,144 shade arithmetic checks, all 64 FADE row hashes and small lookup vectors. No original executable is run.

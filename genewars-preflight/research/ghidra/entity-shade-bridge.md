# Entity shade from explicit map data

`entity-shade-evidence.json` pins `0x7B240` and its active family-1 consumer in `0x72960`. The validator checks 458,752 fraction/corner cases, 42 synthetic vectors and forty actual CONQ001 source points. The second actual fixture uses an explicitly modified diagnostic input with factor 63 everywhere, not a claimed original state.

Read four wrapping neighbors from the exact 128x128, 14-byte-cell map region. Each corner contributes `q=(signed16_light*unsigned8_factor)>>6`, with arithmetic floor before interpolation. Keep q as signed32. For low-byte position fractions fx/fy, interpolate on the same diagonal as the height helper: when fx<fy, `n=(q11-q01)*fx+(q01-q00)*fy`; otherwise `n=(q10-q00)*fx+(q11-q10)*fy`. Set `p=2*q00+(n>>7)`, then return `clamp(((p-7680)>>8)+16,0,64)` as a byte. All shifts floor toward negative infinity; all intermediates fit signed32 across the admitted light/factor domains.

The consumer passes active family-1 record position at native +7 and stores the result at native +0x17. This proves the field's producer, not its initial timing or visibility state. The helper differs from terrain vertex shading, and its result does not itself perform a FADE lookup. The separate foreground contract chooses direct indices for shades at least 30 or flags +0x15 mask0x02.

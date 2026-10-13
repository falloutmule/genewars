# Independent entity-shade review

The native helper passed 475,136 exact CONQ001 coordinates, including signed coordinate seams; 1,310,720 fraction pairs across 20 mixed corner fixtures; and every one of the 16,777,216 signed16 light/unsigned8 factor combinations at a vertex. Every result from 0 through 64 was exercised. Both short and long map extents reject. See [the receipt](entity-shade-review.json).

Direct `0x7B240` instructions corroborate signed light multiplied by the unsigned factor, arithmetic shift six at each corner, triangular interpolation, then bias and clamp. The independent oracle uses nonnegative corner weights summing to 256 and reduces the final arithmetic to `clamp(weighted_sum // 32768 - 14, 0, 64)`. Multiplication must be quantized before interpolation; interpolating raw light and factor separately would change results.

No defect was found. Endpoint 64 is intentional for this helper and must not be treated as an admitted FADE row (0 through 63). This review supplies explicit map snapshots and coordinates; it makes no entity mutation, callback, initial factor or visibility-state claim. No original executable or GUI ran.

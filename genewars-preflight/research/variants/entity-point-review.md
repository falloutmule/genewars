# Independent entity point projection

The native `entity_view_position` helper passed 4,276,264 cases: 475,136 coordinates on the exact CONQ001 region, all 40 active family-1 source points, every position/camera word value, all 65,536 fraction pairs on a signed-height checker fixture, both height extrema and unsigned margins 0/128/65535. Invalid map lengths reject. See [the receipt](entity-point-review.json).

The independent model uses centered ring distance modulo 32,768, nonnegative barycentric height weights and direct integer floor division. It agrees with the original word double/subtract/arithmetic-half sequence. Height interpolation precedes its shift six; substituting the separately rounded terrain-vertex displacement would change results.

No defect was found. The margin is zero-extended unsigned16, and outputs retain signed32 width; observed x values reached -69,630. The source preconditions remain height factor 2, horizontal scale 32 and vertical scale 16. Later caller word storage, clipping, sorting, timing, visibility and initial camera selection are outside this helper. No original executable or GUI ran.

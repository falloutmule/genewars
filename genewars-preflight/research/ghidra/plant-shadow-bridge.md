# Normal plant shadow

`plant-shadow-evidence.json` and `validate_plant_shadow_bridge.py` close the normal `0x96AC0 -> 0x92F80` pixel contract, separately from the color pass. Four private listing hashes, 757 cached-versus-stream comparisons and twelve actual resource/MAIN.GLA fixtures are recorded. No original game code is executed.

Admit component flags zero, per-call context word `0x19CFAC` zero and the extra argument zero. This word is distinct from source context byte `0x16E0C4`: caller `0x6C230` writes the zero per-call word when source mask0x02 is clear, ignoring the other source-byte bits. The composed normal path therefore admits any source context byte with mask0x02 clear. The caller supplies a signed16 frame origin. Each component's loader-halved signed x is added directly; its signed y is divided by two with truncation toward zero and added to the origin. These are full signed32 sums, unlike the foreground's final signed16 narrowing.

`0x92F80` halves the sprite descriptor height with an unsigned shift. Output row k uses the original sprite's opacity row `2*k+1`: even rows are consumed without drawing, and an odd final source row is discarded. Top clipping consumes twice the clipped shadow-row count and preserves this phase. Horizontal clipping intersects the opaque footprint normally. Source literal values are ignored, but their complete byte payload is consumed; zero-valued literals are still opaque.

For each opaque pixel, write `MAIN.GLA[(color<<8)+existing_destination]`. This is the opposite index order from terrain mode15. The exact CD MAIN.GLA load to `0x18CF48` is shared with the prior terrain modes receipt. Plant caller `0x6C230` supplies color zero; the underlying single-sample rule accepts any byte color.

The pure pixel helper takes validated cached opacity, explicit placement, clip and pitch. It assumes drawing is enabled at the `0x8F660` gate (`0x10AA15 != 0`) and excludes the clip helper's dirty-block bookkeeping. Empty safe clip dimensions may return no writes without asserting equivalence to original zero-dimension behavior. Flip/scale flags, alternate extra argument and entity visibility/order remain outside this contract.

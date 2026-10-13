# Independent terrain data and interpolation review

No defect found in the bounded assets/core APIs. A fresh private native harness compared all16384 cells, including every raw byte, signed height and texture selector, and all16032×32 atlas tiles against independent source offsets. Every one of163840 atlas bytes was used exactly once. Tiles160 and usizeMAX, coordinates128/usizeMAX, short/long atlas/map regions and short/long interpolation regions reject.

The instruction listing at7B140 establishes signed coordinate high-byte extraction masked by127, unsigned low-byte fractions, conditional triangle choice, signed height loads and arithmetic shift7 after two products. The independent oracle uses nonnegative barycentric weights summing256 and floor division128, rather than the implementation's difference formula. Actual native results match475136 real-coordinate samples and524288 synthetic fractions over eight height sets, including extrema and negative rounding. All cell vertices, every signed16 x at six y boundaries, and mixed-axis samples were covered.

Each height difference is bounded by65535 and fraction by255; even the conservative sum of two absolute products is33422850, safely inside i32. The weighted result is bounded[-65536,65534]. Unsigned extraction of the coordinate word is equivalent to the original signed shift followed by mask127, including negative coordinates and neighbor wrapping.

The atlas API returns raw indices with no transparency rule. Mode5's zero-write/shade lookup is separate static evidence, not implemented terrain rendering. Orientation, lighting and other cell bytes remain raw. Initial map bytes precede runtime normalizers; complete triangles, camera origin, visibility, overlays and frame parity are outside this review. Doubled point interpolation also must not be substituted for separately rounded vertex projection.

Reproduce with `python -B research/variants/terrain-review.py`. The safe JSON pins all12 instruction listings, exact source assets, crate sources, native harness and all160 tile hashes. No original or GUI was executed.

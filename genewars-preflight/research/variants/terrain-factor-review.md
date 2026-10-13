# Independent radius4096 factor-plan review

No defect found in the immutable arithmetic plan. Actual native output matched1356 plans over twelve real/synthetic maps and113 targets each:1476684 visited vertices and659023 ordered changes. Cases include signed/fractional/alias seams, flat previous factors0/1/31/62/63/64/255, extreme checkerboard heights and mixed signed heights/factors. All factors63 or higher skip, and constant minimum/maximum heights give the same planar result because the height difference cancels.

The independent model uses shortest displacement on the32768-period ring, barycentric height and integer square root. It preserves the33×33 byte-grid traversal, source-map modulo128 and callback-coordinate modulo256 separately. Complete ordered records, including old/new values and signed grid positions, are compared; native source-map extents reject short/long inputs.

The strict outer radius excludes distance4096 exactly. The inner boundary uses3840²; outside it but below4096² the integer falloff is floor((4095-isqrt(distance²))/4), admitted only when greater than the previous factor. The vertical difference arithmetic-shifts by3 before squaring. Three squared components are bounded by16384² each, so the total805306368 fits signed32 and signed/unsigned threshold comparisons agree in this domain.

This is a snapshot plan, not complete0x75B00 execution. It does not mutate map data, invoke0x84B40, evaluate virtual entity predicates or implement dirty-region/minimap/shade/visibility lifecycle effects. No initial mission target is inferred. Reproduce with `python -B research/variants/terrain-factor-review.py`; source, instruction and native harness hashes are pinned in the receipt. No original or GUI is executed.

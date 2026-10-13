# Independent terrain modes 0, 6 and 15

The native review passed 71,289 mode-6 spans with 688,378 compared samples, including every 16-bit shade fraction and every admitted span width. It compares full pixels and write masks for the three actual B32A overlay tiles 157–159 and all 65,536 destination/color lookup pairs in the actual MAIN.GLA table. The geometry review adds 1,229 cases, 8,616 rows and 1,552 complete mode 0/6/15 framebuffer comparisons. Exact counts, code/resource hashes and reproducing harnesses are in [the receipt](terrain-modes-review.json).

No defect was found. The independent sample expression retains all 24 bits of each U/V/S accumulator, independently of mode 5's coupled carry machine. Source zero skips FADE validation and painting but still requires a valid texture address. Nonzero source mapped to palette zero remains opaque. Negative clipping, source/address bounds, late shade failure and output sentinels were checked.

Geometry-only preparation admits the general rounded-width-zero fixture because it does not calculate attribute gradients. Mode 5/6 correctly reject that source case. Public prepared rows were checked with late invalid y, edge and inverted-span values; rejected draws preserve the entire original frame, including pitch padding and surrounding sentinels. Fully right-clipped rows remain empty safely. MAIN.GLA uses existing destination as the high byte and explicit color as the low byte, always writing covered pixels.

These checks supply explicit snapshots, colors, lighting operands and placements. They do not choose an initial camera/factor state or establish parity of a complete original scene. No original executable or GUI ran.

# BONES selector-4 render slice

The staged CD `BONES.ANB` is admitted by its exact hash and section layout in `BonesAnimationDatabase`. Its packed-four-bit RLE records decode to indexed pixels plus an explicit run-derived opacity mask; literal index zero remains opaque. The resource metadata pins five animations, ten frames/components, and eleven sprite records.

For the normalized selector-4 records in states 26 and 27, `0x59470` uses the type-1 wrapper with the normal owner-4 color branch at `0x5993B` (`0x94F30`), after the `0x95100` shadow call when global shadow mode is zero. The owner-4 wrapper inputs observed in the pinned trace are scale 150, alternating palette bases 192/208, threshold 135, fade row 0 for the fixture, and explicit wrapper flags. Owners 0–3 take a different `0x94D70` path and are outside this adapter.

`scripts/desktop/validate_bones_pixels.py` independently parses the ANB frame/component tables and packed4 RLE streams, then models the existing source-derived scale, flip, palette, fade, shadow-mask, and MAIN.GLA rules. It compares both complete pitched frame buffers against the Rust helper: normalized state26/frame0 and state27/frame1 records, 6,536 bytes total. The matching source fixture, wrapper trace, asset, and instruction listings are hash-pinned in `research/integration/bones-pixel-validation-current.json`.

This validates the bounded owner-4 color-and-shadow slice. It does not establish other-owner rendering, scene admission/order, or original-executable framebuffer parity. Wrapper projection, inherited global flags, clip seed, and global shadow mode remain caller inputs.

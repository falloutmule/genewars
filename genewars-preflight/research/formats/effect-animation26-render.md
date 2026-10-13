# EFF animation 26 draw slice

The hash-bound `EFF.ANB` resource is catalog entry `root/2/disc/59`, decoded SHA-256 `f45f1201e50ce67199f1e1e8448d250f221cad71f1328ee44b7bacf9ee62c253` (186,726 bytes). Effect code `0x1A` binds animation 26. Its descriptor spans byte offsets 2030 through 2086 in 14-byte steps: five frames, each with hold byte 1. Their component flags are `00, 00, 00, 84, 04`.

The bounded draw path uses the generic shadow pass and ordinary indexed8 color path. The frame with component flag `0x04` uses the proved MAIN.GLA blend mode; none of these frames enters the separate code-`0x31` special EFF66 layer. The helper requires the caller to supply the admitted effect record, slot binding, ground point, and global draw flags.

`scripts/desktop/validate_eff26_pixels.py` runs the focused Rust test and records four complete 512×480 indexed-framebuffer comparisons against an independently walked component/shadow/color model. Current source/resource pins and output hashes are atomically refreshed in `research/integration/eff26-pixel-validation-current.json`; earlier dated receipts remain historical snapshots. This is a bounded renderer validation, not an original-executable run or a claim about complete effect scheduling.

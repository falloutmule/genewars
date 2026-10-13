# Countdown family resource binding

## Finding

The family-8 resource base is bound to `data/ethereal.anb`. The static descriptor at mapped VA `0x103910` contains that NUL-terminated path; its pointer field at `+0x1C` is `0x0011681C`. The mapped input is `private/ghidra/actor-activation/memory-.object3.bin` (SHA-256 `aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87`, mapped from VA `0x100000`). This establishes the previously unresolved destination without relying on a filename guess.

The nearby descriptor at `0x1038E4` binds `data/eff.anb` to `0x116814`; its initializer writes the animation-table address to `0x116818`. The same 0x2C-byte descriptor stride is used by the generic loader at `0x154B0`/`0x15410`. The resulting family-8 resource hook at `0x10060` uses the initialized table at `0x116820`: selector byte `actor+0x55 == 1` selects that base, selectors 0 or 2 select base `+0x50`, and other values leave the existing resource pointer untouched.

The exact CD resource is manifest entry `root/2/disc/60` (`GAME/DATA/ETHEREAL.ANB;1`): packed SHA-256 `7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc`, 52,085 bytes; decoded SHA-256 `318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4`, 91,123 bytes. The bounded RNC corpus reports method 1, successful header/CRC validation, and decoded CRC16 5624. The same packed/decoded identity occurs in `root/1/261` and `root/4/82`; these are aliases of the same bytes, not separate variants.

## Draw-path boundary

The family-8 draw entry is vtable `0x11242C + 0x24 = 0x10EA0`. For a factory-initialized selector-0 record, the factory word at `+0x15` is `0x4020`, state byte `+0x56` is 4, and local animation byte `+0x19` starts at 0. The family resource hook maps that local animation to global ETHEREAL animation 8 (`base + 0x50`), which has 11 frames at byte offsets 112 through 252 in steps of 14; each frame has one untransformed component. Selector 1 binds the unadjusted base: its factory state-0 local animation/frame pair 0/0 resolves to global animation 0, a single 27×26 sprite. The frame display extents, component sprite extents, and loader-halved offsets are checked by `scripts/desktop/validate_family8_state4_pixels.py`.

The bounded renderer helper `family8_scene` admits selector-0 state 4 and state 5, plus selector-1 states 0 and 1 for direction animations 0 through 7 at frame 0. Selector-0 state 4 requires word `0x4020`; state 5 uses the arrival-cleared word `0x0020` and an animation-8 frame index 0 through 10. A literal 276-callback review pins the state-5 record as local animation 0/frame 6; its state transition clears the word's high-byte `0x40` bit. For selector 1, onward child records bind local direction animations directly to the matching global ETHEREAL descriptors. Selector-1 state 1 uses the transitioned word `0x0020`; the shared 0x10EA0 entry takes its ordinary color branch for this selector/state. Direction 4 has component horizontal-flip flag 1; the renderer mirrors indexed pixels and opacity for shadow and foreground while preserving the component position.

For the admitted normal path, the helper uses the 0x10EA0 order: shadow at the supplied ground origin, then indexed8 foreground at signed16 `(origin_y - ((physical_height - ground_height) >> 6))`. Raw shade zero selects fade row zero; otherwise the arithmetic-shifted height is added to raw shade. The caller's admitted record chooses the direct blitter when the effective shade reaches 30; otherwise the helper uses the checked FADE row. It rejects negative fade rows and unsupported record variants. It does not implement selector-specific decorations, visibility, ordering, or animation scheduling.

The repeatable validators atomically refresh `family8-initial-selectors-pixel-validation-current.json`, `family8-state5-pixel-validation-current.json`, `family8-selector1-direction4-pixel-validation-current.json`, and `family8-selector1-state1-pixel-validation-current.json`. They compare complete buffers against an independent Python parser/pixel model using exact decoded ETHEREAL, FADE, and MAIN.GLA hashes. Earlier dated receipts remain historical snapshots. No original executable was run for these pixel validations.

Other family-8 states, selector-2 decoration, and nonzero selector-1 frame indices remain outside this helper. In particular, this does not infer a complete countdown UI or animation policy from the resource binding alone; caller-owned draw ordering remains explicit.

## Pinned evidence

- `private/ghidra/mission-tail-init/instructions-00010060.txt`, SHA-256 `518f7e316012183b083623aca3e4b01b8feedee77b0476c56e24d6f76a7d441d` (resource-hook selector dispatch).
- `private/ghidra/countdown-expiry/instructions-00010ea0.txt`, SHA-256 `639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb` (family-8 renderer entry).
- `research/ghidra/mission-load-evidence.json` pins the `0x10060` loader instruction listing; `research/ghidra/landing-evidence.json` pins the adjacent EFF descriptor and animation table destination.

This note records addresses, source identities, and bounded behavior only; it contains no decoded asset bytes.

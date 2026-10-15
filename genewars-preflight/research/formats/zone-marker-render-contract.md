# Bounded code3C / EFF60 render contract

This packet admits rendering only. It does not add zone scheduling, credits, mission outcomes, original mixed-family picking or campaign acceptance.

## Original selector and shared renderer

The numeric type5 vtable at `112254` selects `4A060` through virtual offset `24h`. In the saved listing `private/ghidra/effect-draw/instructions-0004a060.txt`, code `3Ch` follows `4A07D` (greater than `3Bh`), `4A08B` (less than `42h`), then `4AEEC` (at most `3Dh`) into `4ABEC..4AC94`. Code `2Fh` selects exactly this same branch through `4AF19/4AF1B`.

This is the existing source-proved owner-color renderer, not the ordinary code6 shadow renderer. It selects `*(animation1F + localAnimation19*10) + frame1B*14`, then reads virtual4 as physical height27 minus ground11.

- Effective shade is raw byte17 plus arithmetic `(relative_height >> 7)`, including raw shade zero.
- Shade below30 selects `96990` (owner recolor then FADE); shade30 or above selects `96870` (owner recolor, no FADE).
- `4AC41/4AC44` uses byte arithmetic `owner*16 + C0h`. Neutral owner4 therefore yields bias0. Source color indices in C0h..CFh become bias plus their low nibble before fading. There is no original owner4 rejection on this branch; this port bounds owners0..4.
- Y is `i32(sign16 incomingY) - (relative_height >> 6)` with no intermediate signed-word wrap.
- No `96AC0` shadow, no special layer, and no flags15 bit2/bit10 direct/parity branch occur. Those ordinary-wrapper flags must not influence code3C rendering.

Saved listing SHA256: `c1562d884b99ef3145fb69593cd221aec76a0d711b8c780c110dcf911b58bfb9`. Existing shared-branch/primitive evidence is `research/ghidra/effect-draw-contract.json`, its code2F entry and pinned owner-color `96870/96990` / `95390/95E40` findings. No new original executable session was used.

## Actual resource admission

Private `effects/EFF.ANB` SHA256 is `f45f1201e50ce67199f1e1e8448d250f221cad71f1328ee44b7bacf9ee62c253`. Animation60 has six frames at offsets4088..4158 (stride14), flags0, zero frame holds. Each frame contains one component with flags0 and extra0. Component positions are (-8,-8), (-10,-10), (-12,-12), (-12,-12), (-10,-10), (-8,-8).

The host requires exact `EffAnimation60` binding, code3C, owner0..4, local animation byte0 and resource-bounded frame0..5. Initial inherited global flags remain restricted to0. Unknown resource layouts, owner values, bindings and frames reject before destination commit.

## Verification scope

Added actual-resource foreground pixel walks cover every frame for owners0,2,4, raw-shade0 with relative height512, the shade29/30 fade/direct boundary, ignored flags15 bits2/10, no shadow, and neutral palette remapping before FADE. A tall-height case checks the32-bit Y result; invalid owner/binding/frame cases check destination rollback. These checks reuse the established code2F pixel oracle style without calling the compositor to construct the expected image.

Root verification182 passed all12 effect-scene tests, full game80/runtime244 regressions (one opt-in ignored each), and the release build. Receipt: research/integration/zone-marker-render-182.json. This contract records static source/resource evidence and proposed bounded verification, not a new native visual acceptance claim. No proprietary sprite or executable bytes are published here.

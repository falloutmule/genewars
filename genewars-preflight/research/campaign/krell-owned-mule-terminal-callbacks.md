# Owned newborn: ejection, landing and command boundary

Observed literal source, QA BETA V5: source-compared terminal adapter fixture002 produces newbornslot1, owner0, quality50. On its original CONQ001 terrain the newborn completes17 callbacks and reaches ground. An ordinary opcode0x11 handler then installs a move; callback18 changes its position. Receipt status passed means this bounded source sequence completed; it is **not Rust equivalence, target arrival, Krell initialization or native gameplay acceptance**. Residual ejection motion has not been excluded as a contributor to the first post-command position change.

- Callback1:718 instructions, zero draws, position19839,12927 to19840,12928; height10368 to11135, ground6528.
- Callback2: owned idle AI after physics, flags60word0x0088, timer32766, six draws to seed48356/ring12/count13. The Mule remains airborne. This is command0 before any player movement request.
- Callback17: position19861,12953; height and ground both6528. The timer is32759.
- Opcode0x11 target20117,12953:189 instructions, zero draws. Exact5DF50 cleanup/install, no record injection.
- Callback18: position19863,12953; height6700, ground6528, flags0x8098, timer32766; two draws to seed6020/ring14/count15.

Full per-callback native memory/traces and command afterimage are preserved in `private/review/owned-mule-terminal-callbacks/d4ef0917a7494bde873f044a950fe6d3`. JSON contains exact fixture, listing and script hashes, key record deltas and terrain-change counts. Original missing59280 boundary remains at private/review/owned-mule-terminal-callbacks/edbdc16cb112481caccc8b55f88a3fda.

## Smallest engine requirements

1. Shared6DF30 physics must admit this owner0 factory/ejection record without changing its gravity, RNG or animation behavior. Existing neutral-only guards cannot simply be removed: owned physics calls virtual+2C, resolved to59280.
2. 59280 is visibility reveal. Species radius comes from1045FC+species*45. It scans radius+512; inside radius+256 it raises terrain factor to63, otherwise uses the original distance falloff. Each increase calls84B40, including dirty flags and linked-entity shade updates. Reuse existing reveal/terrain refresh mechanisms with the recovered radius/falloff; do not stub map effects. Newly exported exact listing is private/ghidra/owned-mule-collision-59280.txt (historical filename, actual behavior reveal).
3. Owner0 idle state8 has an AI branch at63ECC onward, using player+4A1. 6BFD0 tests only record60bit2. In this fixture it is false;6446E invokes43130 twice for selector masks4 and16 (selectors2 and4) withflags0x30. Preserve actual building-list queries; do not invent absent/present results. Its resulting wait action is demonstrated by callback2's complete afterimage, not implemented yet.
4. The same shared engine needs owned positive-wait continuation and cleanup/install for command7. The existing pure mule_owned_command module proves the guarded opcode handler separately. Compare its real landing record before broadening admission.

## Harness boundaries

The source input is a synthetic completed Genepod/work1 fixture on authentic CONQ001 terrain. Only newborn callbacks advance; other world callbacks are frozen. Frame numbering starts at1 for this isolated sequence. Genepod caller ejection is real and retained. Packed backing pool headers must be expanded and original vtables rebound for linked shade traversal: creatures, plants, actors, buildings and effects. Actor data is used only through its common header here. Audio and selected-panel refresh remain explicit inherited presentation ports. None of these boundaries authorizes claiming an ordinary campaign run.

The later full Rust world comparison passes all18 callbacks/5,237,190 bytes; see owned-mule-ai.md and owned-mule-world-review.json. The earlier gaps above describe the source-recovery checkpoint, not the final integration status.

# Original result presentation contract

Reference: verified CD `GENEWARS QA BETA V5`, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. This is a source-derived implementation contract, not native interactive acceptance. `result-presentation.json` pins the inspected source and private resource observations. The observed run-006 copies need normal manifest admission before production loading.

## Victory notification and continuation

72ED0 requests 2F810 on object185DFC. That function selects its widget at +595, copies seven dwords from font context1870FC to widget+36, clears widget+52, binds text pointer11673C to widget+10, positions it through28690 at (89,85), and sets object+582 to FF. Startup1DA48..1DA7D builds the TEXT pointer table at115E90; therefore11673C is entry555, the English level-complete/Return instruction. Existing Enter scan1C→packet21→82590 evidence supplies the actual continuation action. No invented objective or extra key is needed.

The successful continuation queues1863DC. Its entry7DBC0 loads `data/winscr.pal` and `data/winscr.dat`, plays `anims/biowin.smk`, closes the old widgets via2F0C0, and performs the numeric changes in `result_transition.rs`. In particular, progress increments on this entry, before its mission feedback is selected. Rendering7DE70 presents the framebuffer then calls7D800 with BL=0.

## Victory screen content

7D800 chooses the campaign feedback pointer at116604 + signed progress*4: TEXT index477 + progress. For Exceland, progress advances0→1 and selects entry478. Its meaning corroborates a successfully constructed base and continuation to subsequent planets. Preserve source text bytes through the existing TEXT parser instead of replacing this with invented mission prose. The feedback clip is (43,31), size560×114; source sets font context colors2/80 and flags4. Startup1DA13..1DA37 binds context1870FC (slot6) to descriptor118CBC; mapped descriptor10400C names FONT7.TAB, with matching FONT7.DAT. The renderer uses this exact font.

Six statistic labels use TEXT entries546..551. Their meanings, in order, are creatures born, enemy creatures killed, enemy buildings destroyed, new species discovered, specialists lost and points scored. Labels are at x218, y196/241/286/331/376/421. Their values are signed decimal dwords from selected player offsets6F9,6FD,701,705,709 and00D, drawn centered in the58-wide clip around x473 with the same y positions. Value clipping is (444,0), size58×480. These fields must come from the completed mission before72E50 clears the mission counters.

The victory view's advance condition is118779 bit1, produced by the existing input routine70E40. Its type2 branch consumes19CFB3. This report does not infer a second Return binding. The native input adapter must establish that route before claiming original continuation usability. Nonfinal campaign advancement queues186F19. Progress24 is a separate ending branch, not an established mission-total denominator.

## Loss scope

7DEA0 loads `data/losescr.pal`, `data/losescr.dat`, and `anims/biolose.smk`; its entry clears runtime loss bit80 after presentation. 7D800 with BL nonzero selects TEXT508 + signed progress for campaign loss feedback. The normal mission outcome proof establishes elimination/loss flags, but has not established the complete route into this loss view or its notification widget. Do not map an unsupported engine error to this view, and do not claim a proven loss continuation from these asset observations alone.

## Required715E0 palette service

715E0 writes eight bytes186F54..186F5B from A3F54 palette lookups, in this order: RGB6 black, white, red, green, blue, yellow, cyan, magenta. The palette pointer is object+2. This is rendering state, not campaign or player mutation.

A3F54 scans all256 RGB triples. An exact match returns its first index immediately. Otherwise it minimizes squared RGB distance; ties are narrowed by minimum Manhattan distance; remaining ties are narrowed by minimum `2*R*R + 2*G*G + B*B`, retaining the first index on equal final score. A simple first-minimum squared-distance helper is insufficient for nonexact ties. This algorithm is recovered from the full A3F54..A4378 listing; a production port still needs a focused source comparison, including adversarial ties. No palette-port execution is claimed here.

## Minimal integration sequence

1. Display the source notification text/widget when the shared evaluator requests it; dispatch ordinary Return through the existing command path.
2. Enter the bounded victory transition once; load the admitted original screen assets and preserve completed-mission statistics for rendering. Play the original animation using an admitted decoder or report that presentation service as unsupported.
3. Show the original feedback and six values. Bind the proven input edge to the campaign continuation, preserving the explicit ending boundary.
4. Apply the source-backed carry cleanup only when leaving the view, then initialize the next campaign view. Do not resume the cleared old mission world.

The reusable runtime should expose a presentation request with asset identifiers, TEXT indices and statistic field identities. Rendering and input dispatch belong to the desktop host. This report adds no host shortcut, no automatic mission victory, and no acceptance count.

## Native reusable helper

`rust/crates/genewars-desktop/src/bin/support/result_scene.rs` now provides `VictoryResources::load(root)` (compiled catalog admission) and `render(post_entry_player)`. The returned screen uses original decoded WINSCR, FONT7, English TEXT and the captured player statistics. Its receipt explicitly marks BIOWIN animation pending and native input acceptance false. The caller must supply normal campaign context; this bounded helper admits post-entry progress1..24 and rejects control/extended feedback bytes rather than inventing text-control behavior. ASCII word wrapping uses the source559-pixel available width and FONT7 space-glyph height. Full pixel equality against original1F750 remains untested.

`research/variants/result-scene-smoke.json` records a passing isolated compile/render and rejection of invalid progress0. The private frame was visually inspected: feedback and six labels/values are visible in the original screen regions. Its zero-value fixture is a renderer fixture, not a completed gameplay capture. This does not increase mission acceptance counts.

`python -B scripts/result_inputs.py` stages four exact manifest-bound CD members through the existing Rust RNC decoder, verifies packed and decoded hashes, and refuses changed copies. Their `results/` roles were appended to the runtime resource catalog. Root must call this staging helper before normal runtime bundle staging and register the new module in the native host. No shared host or runtime scheduler was edited.

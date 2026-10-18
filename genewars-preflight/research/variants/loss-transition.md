# Ordinary loss presentation and continuation

Reference: verified CD `GENEWARS QA BETA V5`, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Source evidence, not a completed native loss acceptance.

## Automatic entry

72ED0 sets runtime118770 bit80 for selected-player elimination or a different winner. In mode0 it does not call2F810 or queue a result view. The missing consumer is the main loop1BBC0: at1BC8E..1BCAF, bit80 set and mode118968 bit1 clear cause current view116974 to be copied to previous11697C and loss view186400 to be queued at116978. The following1BCBE..1BCD1 invokes the previous exit and new entry virtuals in that order. This happens on the next main-loop pass after the loss is produced. There is no intermediate victory-style loss widget or Return gate in this ordinary path.

Constructor8DD18..8DD3B binds object1863E8 with inner view186400 and vtable112810. Its entry8DF10 subtracts18 and jumps7DEA0; exit8DF00→7DFD0; update8DEF0→7E060; draw8DEE0→7E090.

## Loss view

7DEA0 selects the loss view, loads LOSESCR.PAL/DAT, closes old widgets with2F0C0, sets stage5 and modebit2, saves frame11877E into118786, and requests BIOLOSE.SMK plus the bounded video services. After those services, it clears loss bit80. It also clears runtimebit4. Player bytes and campaign progress remain unchanged; unlike victory, the current frame is not zeroed. No715E0 call appears in this entry.

7E090 calls7D800 with BL=1: normal campaign text uses TEXT508 + signed current progress, and the same six statistics described in `result-presentation.md`. The first mission's loss text therefore uses508. These values must be rendered before cleanup clears their mission counters. Unsupported engine behavior is not a loss condition and must never enter this screen.

## Continue and exit

7E060 tests runtime118779 bit1. Without it, nothing is queued. With it, the previous view is recorded and title view185008 is queued. This is a return to the title, not an automatic retry, campaign increment, or next mission. The existing70E40 input producer must provide the bit; a second Enter binding is not established by this branch alone.

Exit7DFD0 selects player0, runs72E50 on mission118D60, clears modebit1, performs palette/video cleanup and frees LOSESCR.PAL, then clears current116974 and the view byte+1. The source-backed `result_carry` projection supplies the bounded carried-state cleanup, with its existing link/mode restrictions. Entry into the title and subsequent player choices are separate host integration work.

## Implementation and verification

New `loss_transition.rs` exposes `pending_loss(flags,mode)`, `enter_loss(flags,mode,frame,video_lock_depth)` and `advance_loss(runtime_input_flags)`. It does not edit player state, execute presentation calls, or register itself in the host. Its ordered service addresses are explicit obligations. Entry rejects absent loss latch, alternate mode and unsupported video-lock context; these errors are separate from gameplay loss.

`python -B research/variants/loss-transition-review.py` freezes the production module and compares32 literal-source entries: all7220 player bytes unchanged, runtime flags/mode/stage/frame/savedframe and ordered presentation calls. It additionally source-executes the1BC8E prefix through the dispatch boundary for128 flag/mode cases and7E060 for128 input-edge cases. Presentation functions are explicit successful ports. The Ghidra absolute `MOV [118967],AL` is normalized to its encoded byte width in the interpreter; treating it as a dword produced a harness failure that was repaired before the passing run. This is not a source behavior change.

Latest passing receipt: `loss-transition-review.json`, private run `private/review/loss-transition/6f186214418c4236978fe8f5aa10f8e1`. Native loss rendering, ordinary input edge delivery, full exit integration and title interaction remain untested. No mission or campaign transition acceptance is added.

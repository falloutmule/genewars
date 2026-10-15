# Krell zone marker: selector2 states10/11

Reference QA BETA V5. Marker input is slot0 from `private/review/krell-zone-bootstrap-001/after/family8.bin`, created by the independently matched zone factory. The numerical comparison below now supplements the source contract.

## Shared callback order

Family8 vtable `11242C+64` resolves to `118E0`. It executes `6FB60` at `118FA`, then virtualAC at `11905`; virtualAC=`1EE40`, the exact predicate `record49 & 20 != 0`. If true, zero words `3F` and `41` (`11913..1191C`). Selector byte55 is then dispatched; selector2 takes `136DC` and indexes state56 minus10 into table118CC.

The five pinned table entries for states10..14 are `134D8,1355C,1357A,13593,136C6`. These are separate from selector0's state4 child/RNG behavior. Reuse only source-backed shared movement and tail helpers, not the complete selector0 state4 implementation.

## State10: descent and arrival

`134D8..134E5` calls virtualAC again. If false, continue to audio at134F5. If true, write state56=11 and word64=0 at `134EB..134EF`, then continue to the same audio/tail. There is no RNG draw, child allocation, zone-quota write or retirement in this branch.

The captured initial marker has owner4, selector2, state10, target-altitude dword+B=30464, physical-height dword+27=40064, current speed word+3D=0, requested/speed-limit words+3F/+41=1920. Its position is `[256,24320]`; it begins above the target and must descend through the genuine common-flight path. Selector0's owner<4 guard cannot be retained for this marker. Keep the source arrival threshold, width/wrap behavior, steering, physics, terrain interaction and map migration from the shared `6FB60` path. Do not force arrival or edit the marker altitude to skip it.

## State11: exact timer

`1355C..13564` increments word64 with 16-bit wrapping, sign-extends the new word, and compares it to20 (`13568`). Only equality changes state56 to12 and clears byte57 (`1356D..13571`). No RNG, quota, child or retirement. An incoming timer19 transitions; incoming20 becomes21 and does not transition. It is not a `>=20` predicate.

State12 at `1357A` waits while byte57 is zero. A nonzero byte57 changes state to13 and clears word64 (`13584..13588`). This establishes a wait boundary, not the source of the external event. No quota relation should be invented. State13 later waits20 then initiates departure; state14 retires through103F0 at altitude>=57600 (`136C6..136D5`). Those later behaviors are outside this requested movement contract.

## Audio, animation and display tail

At134F5 signed current speed3D>0 requests virtual44 sound38, mode2, priority255, volume127, initial parameter100; virtual80=`1EA90` returns1920 for selector2. `98210` receives adjusted signed parameter `100 + trunc(20*speed/1920)` (`13501..13552`). At speed<=0, `136F9..1371A` checks whether sound38 is playing through97830, and requests virtual50 mode4 only when playing. These are explicit audio-service obligations; do not fabricate playback state.

If record6 remains nonzero, `13735..13758` advances its animation with7AC00 using descriptor `animation_base + 10*record19` and state at1A, then `1375D..13765` submits the entity to83140. The callback returns AL=0. The marker needs its original selector2 animation binding and visible native render, not just successful headless descent.

## Minimum proof and unsupported boundaries

Run literal118E0 on the captured marker and current map through first descent, arrival, and timer19/20 boundaries; compare full nonpointer record, map links and unchanged RNG. Preserve explicit audio/query and renderer-cache ports. Do not route an unsupported collision, terrain query, animation state or external state12 event into success. Keep transactional rollback if any required service cannot be represented. A numerical state10/11 pass is not acceptance of later zone interaction or the Krell mission.

## Numerical review passed

`zone-marker-motion-review.py/.rs/.json` compared 110 callbacks and 12,320 bytes against 75,520 interpreted source instructions. Natural arrival occurred at frame46, state12 at frame66, followed by40 waiting callbacks. Additional explicit fixtures cover timer19,20,32767,-1 and state12's nonzero57 transition to13. Every 102-byte record afterimage, seven RNG bytes and moving-audio/stop-query/queue-request flag matched. Pointer bytes31..39 are normalized; map equality was asserted after each literal callback. The native adjusted sound parameter was checked arithmetically, not against a recorded audio device.

Private passing run: `private/review/zone-marker-motion/c59459093bcf4e978d5e0e5dc0b22706`. Production source pins were unchanged during compilation/comparison. An earlier comparison failed solely because the harness looked for inline83140 in the instruction trace instead of the call log; record, RNG and audio bytes already matched. That failed private run remains preserved.

The trajectory made no305F0 searches. Existing `family8-idle-review.md/.json` separately establishes the read-only blocked-query search closure with family8 A8=`1EA80` RET. Its owning-adapter query admission guard remains necessary; this numeric planner review does not prove arbitrary query bindings. Source audio playback query is ported false, queue requests are recorded, and no actual audio or native presentation acceptance is claimed here.

# Krell frame 276: neutral scan encounters the zone marker

The frozen actual `private/review/krell-mule-callbacks-006/frame-276-slot-1-before` source callback completes in 9,527 instructions with five RNG draws and one render queue request. Native comparison is pending. Literal afterimage and source/input hashes are preserved in `private/review/krell-mule-callback/7cd22f7a681640c7893dd234166191f7/result.json`.

Slot 1 is a neutral selector-4 Mule, state 8, skill 50, hunger 173, at [32373,24190]. Its wrapped neighborhood includes friendly Mule slot 2 and kind-8 zone marker slot 0 at [256,24320], cell 12161. The current native neighborhood guard and food-target scan reject kind 8; this is the identified unsupported boundary. Source does not call 5B260 in this callback.

Exact source contract in `private/ghidra/initial-creature/instructions-0006b700.txt`: 6BD3D loads entity kind at +6; kinds below 5 branch to 6B78B, kind 5 to 6B72A, and 6BD4F compares the remaining kind to 8. The JNZ at 6BD52 sends all other kinds to ordinary processing. Kind 8 falls directly through to 6BD58, advances to the next linked entity, and contributes no candidate, score, or RNG draw. This skip has no owner, state, or selector predicate.

The bounded repair is therefore to skip validated kind-8 entities in both native neutral-neighborhood admission and neutral-food scanning. It does not justify relaxing cell-entry, collision, or other family dispatch rules. Full callback comparison against the repaired native afterimage remains required.

Reproduce the source afterimage with:

`python research/variants/krell-mule-callback-review.py --before private/review/krell-mule-callbacks-006/frame-276-slot-1-before --frame 276 --slot 1`

Native repaired callback comparison: **passed**, all 427,369 bytes across eight buffers. Capture007 frame276 slot1, source 9,527 instructions, five RNG draws. Receipt: private/review/krell-mule-callback/ae61755610664179b310f46d6dde928f. Presentation ports remain explicit as in the receipt.

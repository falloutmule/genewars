# Krell frame498: food-approach cell entry

Actual fixture010/frame-498-slot-1-before: neutral selector4 Mule, state1, targetplant0, movementmask53=1, association4F=0. Literal complete5ED20 callback prepares3331 instructions, one RNG draw, one render queue request. Full native comparison pending. Expected eight buffers are private/review/krell-mule-callback/39c3434bd03f4dd8afaf30f67015c6a7.

The state1 cell-entry predicate differs from the state8 predicate only at5B100, before shared waypoint5B260. Source5B10E virtual10 is false (word49 mask0400 clear). State1 dispatch at5B234 reaches5B18C and resolves target55 through6D390. For this nonnull kind2 target,8B780 returns the last live kind1 in the new cell; it is not the entering Mule. Source5B213 reads target kind,5B21D compares2,5B220 branches5B16C to return true. Target health/reservation is not read in this kind2 branch. The existing owned approach admission can continue requiring a living plant separately. No effect11 fallback query8B7D0 occurs on this true branch.

If8B780 instead returns self,5B1AB makes the primary predicate false and the usual8B7D0 effect11 query remains relevant. Do not generalize null-target, kind1-target, or other target cases from this fixture.

The existing source5B260 body does not depend on state1 versus8; it generates the same ordered angular candidates and calls74D90. Its current Rust state8 domain guard can be extended to state1 only with the independently checked caller predicate. Actual afterimage: position[31353,22014], goal[31340,21884], waypoint[31155,21850], flags49 word0010 to0011, state1 retained. RNG63161/14/34897 becomes38021/15/34898. New cell matches the goal cell, so source also sets byte54=80 and clears flags4A bit10; it does not set that bit as in the different-cell branch.

Instruction/input pins are in krell-approach-cell-entry.json. Source listing: private/ghidra/initial-creature/instructions-0005b100.txt. Presentation ports are unchanged from the fullcallback interpreter and remain explicit.

## Post-physics waypoint timer branch

Capture011 still rejected after its corrected entry callback because native code rejected byte54>20. Source70631..7067A defines this branch: when waypointbits49&3 are nonzero and unsignedbyte54>20, require signed speed3D<=256, physicalheight27<=ground11, and distance736F0(current, saved workingtarget)<=1024 before ORing flags49 with3. If any condition fails, return from6FB60 without additional route mutation. This branch does not execute the byte54<=20 distance4096/OR8 behavior.

Actual498 speed607 fails the first condition; trace70647→7064C→70656→7065E→70676→7067A returns unchanged. Keeping byte54=80 and flags49=0011 is source-backed; there is no additional RNG draw.

The target is a packed coordinate value saved at70316..70332 before the6DF30 call70603. Initial lowwaypointbits are zero in this fixture, so it snapshots goal+B, not the subsequently generated waypoint+45. Do not refresh the saved workingtarget after cell-entry changes+45.

Capture012 frame498 slot1 complete native callback comparison PASSED:427369 bytes across all8buffers,3331 source instructions,one RNG draw. This includes cell unlink/prepend, state1 caller predicate,5B260 waypoint, sourcepostphysics timer>20 branch, lifecycle and animation. Receipt krell-approach-cell-entry-native.json; private6ce28369c423458d9be694e5e82c30f7. Presentation ports remain explicit. Earlier011 unsupported boundary is preserved and superseded only for this exact repaired callback.

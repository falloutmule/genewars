# Krell frame557: waypoint waiting branch

Fixture013/frame-557-slot-1-before is state1, flags49=2013, timerbyte54=80, goal[31340,21884], waypoint[31155,21850], current[31152,21872], targetplant0. Lowflags49 bits3 select a distinct6FB60 branch which the native route prefix currently rejects. Full literal callback prepares1508instructions,zeroRNGdraws,one queue request; native comparison pending. Expected eight buffers: private/review/krell-mule-callback/29908784036649a7b19f530b80c12549.

Source6FC5C..6FD80:

- If associated reference50 has nonzero kind52, resolve it and update goal+B from its position; absent in actual fixture.
- Compute goal cell. If wrapping(frame+signedID)&3==3, call virtual8C, which for this Mule resolves5B100, on that goal cell. A false result resets byte54=20, clears word49 withEFFC and flags16 bit80.
- Otherwise decrement byte54 with byte wrapping. If the decremented byte is zero, reset54=20, set49=(49&EEBC)|0140 and clear16 bit80.
- On odd(frame+ID), update heading4B using73970(current,goal+B).
- Save signed word3F, temporarily set3F=0, execute virtualB4=6F980 steering, run6DF30 physics, restore saved3F, then return from6FB60. The normal route, waypoint acquisition and post-physics arrival branches do not run.

Actual phase557+ID1=558 skips the every-four-frame query and oddheading refresh. Timer80 becomes79; speed3D192 becomes112 through braking; flags49=2013 remain. The callback then runs its ordinary5ED20 continuation/animation. Merely removing the lowbits3 rejection would invoke the wrong motion path.

Do not silently skip the upcoming goal-cell query or timer-expiry branch. An implementation may bound unsupported association/goal predicates explicitly while admitting the proved waiting callback. The source listing is private/ghidra/player-creation/instructions-0006fb60.txt; vtable111FEC+8C=5B100 and+B4=6F980. Full input/source hashes remain in krell-waypoint-wait.json.

## Bounded wait-prefix sweep

`krell-waypoint-wait-prefix-review.py/.json` passes24 synthetic cases: timers1/20/80, wrapping(frame+ID) phases0..3, and controlled predicate true/false. The5B100 result is explicitly stubbed, never described as a real goal-cell query. Literal execution stops at6FD58 before steering/physics. All122 nonheading record bytes agree with the independently expressed reset/expiry contract, totaling2928 bytes. Oddphase73970 is executed and its call cadence checked; its numeric heading is observed, not independently compared by this sweep. No RNG draws occur. Native comparison is not run.

The predicate is called only phase3. A false predicate takes precedence over timer decrement: timer becomes20 and flags49 are maskedEFFC. Otherwise timer decrements; reachingzero resets20, applies(49&EEBC)|0140, and clears16 bit80. Timers20 and80 remain19 and79 on nonreset cases.

### Minimum integration shape

Keep a distinct route continuation marker, for example `Normal` versus `Waiting { restore_target_speed }`, alongside the prefix record. The marker must reflect the branch selected from the input flags, not be recomputed from flags after reset. Wait-prefix computation needs source frame/ID, an optional associated-target position (or typed unsupported), and a lazy goal-cell predicate invoked only on phase3. It should return explicit unsupported when that required query is unavailable.

The owned callback then executes wait steering with temporary3F=0 and the existing full physics/cell-entry transaction, restores original3F, and bypasses the normal postphysics route block before resuming common5ED20 health/AI/animation. A reset may clear lowflags3, but it still follows this wait continuation for the current callback. This can be added without redesigning the established physics or shared scheduler. Real goal-query proof and full native557 comparison remain required.

Capture014 frame557 slot1 repaired native fullcallback comparison PASSED:427369 bytes,1508instructions,zeroRNGdraws. Receipt krell-waypoint-wait-native.json/private7df0e306503c48e694b922c948457127. This compares the real557 waiting path and entire remainingcallback; controlled-prefix sweep boundaries remain separately labeled.

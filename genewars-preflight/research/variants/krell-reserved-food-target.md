# Krell frame461: another Mule already reserved the food plant

Actual fixture008/frame-461-slot-1-before contains a neutral Mule in approach state1, targeting plant0 with flags1 and health1. Source5ED20 completes1871 instructions, zero RNG draws, leaves state1 and advances ordinary movement/animation. Native rejected its preliminary state1 test requiring plantflags&9==0. Literal expected world: private/review/krell-mule-callback/600179d512ee454498d6e419ecad7210. Native repaired comparison remains pending.

Source approach does not impose an exclusive reservation rule. At65969 the Mule's arrival virtualAC is checked, and flags60 bit80 may defer arrival. The plant arrival branch659A3 calls target virtual68. For plants vtable1120AC+68 resolves1EBD0, which tests signed native healthword2C<=0. If alive,659BF ORs plantflags2E with1 (idempotent), then sets Mule state7 and timer32767. Existing reservation bit1 is not read here. Dead target handling remains distinct and must not be admitted by simply dropping all guards.

For approach before arrival,65B4D..65B86 queries heights/distance and invokes5D360 (already bounded neutral skill<=90 no-op). The later plant special behavior is gated by Muleflags60 bit20; it is absent in this fixture. The frame461 callback has even(frame+ID) phase and skips the AI tail entirely, demonstrating why a premature reservation guard was particularly restrictive.

The minimal repair may admit alive reserved plants during approach and its normal arrival; retain unsupported dead/special target paths. The existing state1 arrival closure has a second flags&9==0 guard and must follow the same recovered rule to avoid moving the failure to arrival.

Reference source listings: private/ghidra/player-creation/instructions-0005ed20.txt (65957..65BC0), private/ghidra/plant-startup/instructions-0001ebd0.txt. Full source/input hashes are in krell-reserved-food-target.json.

Water-context regression observation: capture008 frames0/1, slots0..3, before/after, matches prior capture002(frame0)/003(frame1) across all8buffers:6,837,904 bytes unchanged. This preserves the earlier literal comparisons for those exact callbacks while recording that the native runtime now binds source11A9C7=2000.

Repaired frame461 slot1 comparison PASSED:427369 bytes,1871 literal instructions,0draws; receipt11771335663a4c308815468700175ae1. A separate synthetic source-only arrival slice65957..65B48 also passes with input plantflags0 and1,health1: both end flags1/state7/timer32767,96instructions each,346 total compared bytes. Its artifacts krell-reserved-arrival-review.py/.json explicitly record native comparison not_run; it verifies the literal contract without claiming a native synthetic mission or completearrivalcallback.

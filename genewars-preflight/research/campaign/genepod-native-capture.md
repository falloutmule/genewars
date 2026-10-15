# Genepod native capture contract

Reference: pinned CD QA BETA V5. `genepod-native-capture.json` pins the literal sources and records four passing manager sequences (ten input polls). Reproduce with `python -B scripts/ghidra/genepod_capture_contract.py`. The probe binds only control 106DBC and its original sprite; it does not establish whole-panel traversal or native acceptance.

## Ready state and selected species

21A10 connects the Genepod controls at 21ADC..21AE7, initializes panel offsets 5E=0, 60=1, 5F=1, clears the low two bits of 84, and copies selected building byte 3C to panel byte 7D (21B19..21B1C). Positive work sets panel byte8=0; idle initialization calls 22760. Do not substitute a constant species: player list 6AE indexed by panel7D selects it. Current Krell list index0 is species4.

Every 21E20 update first derives ready state. At 21F0C..21F4E, building flags must include 1 and exclude 40 and 100. If this fails, panel8 becomes0. If eligible and panel8<=1, building animation byte1B must be zero and work36<=0; then panel8 becomes3 at21F8A. Thus a newly opened idle panel becomes ready through its ordinary update; readiness is not a manually injected2. State3 is ready, state2 is captured/pressed. Action8 checks panel8>=2. Event8001 resets it to3 and returns; absence of event8 sets2 and returns; event8 proceeds to the producer gates and resets0 before dispatch.

22FD0 handles species-list animation separately: nonzero7E advances7F; at four renders (233A8 onward) it clears7F/7E, adds old7E to7D, and calls22760. This is not required for current list index0. Do not conflate render transition with production readiness.

## Actual mouse flow

Control106DBC has flags9128, action8, origin26,344, descriptor1 of GENEPANL.ANB (29x29 opaque hit mask). A87D0 resets manager origin to0 each poll; origin records can replace it when traversing the full list. A86D0 adds that origin before bounds and opacity testing. The isolated probe establishes origin0 behavior only.

* Opaque left-down (packet8 bit0): capture106DBC, capture mask8, draw1; returned low-word action8, event19FF70=0. Panel transitions ready3 to pressed2; no production packet.
* Held inside (packet8 bit2): actionFFFF, event2, capture remains. No new activation.
* Move outside while held: action8, event1, draw0, capture remains; panel resets3.
* Move back inside while held: action8, event2, draw1; panel becomes2.
* Release inside (packet8 bit1): action8, event8, clears capture and draw; producer gates may emit opcode12.
* Release outside: action8, event8008, clears capture and draw; canceled by producer8001 test. This differs from the custom-arrow control's capture semantics.
* Transparent corner press: actionFFFF, no capture.

Return values are signed/unsigned **16-bit action IDs**; upper EAX bits remain FFFF on several paths because source writes SI. Consumers must not compare full EAX to8.

Native implementation must feed ordinary down/hold/up packets through hit testing and persistent capture, then pass both action and event flags to the proven producer. No direct action8 injection. Remaining integration evidence: full list origin traversal/owned-panel enable and actual native dispatcher playback; existing world-click selection ordering is a separate packet.

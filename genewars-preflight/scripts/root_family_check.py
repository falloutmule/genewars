"""Root-side family checker for the GeneWars Mule family (F1).

Implements the strengthened comparison policy in
private/integration/development-policy-20261017.md:

* a positive gate must show EXECUTION SUCCESS and identity - Rust rc 0,
  result.json status == passed, the intended frame/slot, input roundtrip true and no
  callback error - BEFORE any byte comparison;
* all eight buffers must be compared at their EXACT pinned lengths
  (actors 4000, backing 343958, buildings 17152, creatures 37200, effects 16200,
  family8 1632, players 7220, rng 7; total 427369) before equality is credited;
* zero byte differences, the expected RNG draw count, and the rng.bin write
  (seed + ring = draw order) must match the interpreter output;
* negative gates are reported SEPARATELY and must prove the expected rejection/error
  AND complete transactional preservation (output buffers byte-identical to input);
  rollback-produced equality is never credited as a success.

Usage:
  C:/Python314/python.exe -B scripts/root_family_check.py --tag <unique> [--head <sha>]
        [--campaign <replay.txt>] [--no-focused] [--no-suite]
Each run writes private/integration/<tag>/summary.json and never reuses an output dir.
"""
import argparse
import json
import os
import subprocess
import time
from pathlib import Path

P = Path(__file__).resolve().parents[1]
PY = 'C:/Python314/python.exe'
BUFS = ['actors.bin', 'backing.bin', 'buildings.bin', 'creatures.bin', 'effects.bin',
        'family8.bin', 'players.bin', 'rng.bin']
SIZES = {'actors.bin': 4000, 'backing.bin': 343958, 'buildings.bin': 17152,
         'creatures.bin': 37200, 'effects.bin': 16200, 'family8.bin': 1632,
         'players.bin': 7220, 'rng.bin': 7}
TOTAL = 427369
FOCUSED = r'C:/Users/fallo/Documents/Codex/2026-10-15/9-router-is-installed-to-use/run195_focused_fresh.py'

# mode 'direct': byte-compare the port output against the interpreter's own output dir.
# mode 'oracle': run the owned/neutral oracle with --expected against the port output.
POSITIVE = [
    dict(id='R05-neutral-completed-attack', frame=5914, slot=5, draws=9, mode='direct',
         fixture='private/review/synthetic-source-5914-slot-5-before',
         expected='private/review/neutral-mule-captured-width/5683a83b87bb4ddcb207b4a26da2b84a'),
    dict(id='R06-owned-hit-hold', frame=5916, slot=1, draws=0, mode='direct',
         fixture='private/review/gw5914w9-campaign-20261017T064630/capture/frame-5916-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/567ca731e64c4ba092a31cc5e2a8ab87'),
    dict(id='R07-owned-hit-complete', frame=5922, slot=1, draws=0, mode='direct',
         fixture='private/review/gw5922-source-fixture/frame-5922-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/4036d5daf3834570827bfa7d1bd15c5b'),
    dict(id='R08-owned-command-ai', frame=5924, slot=1, draws=2, mode='direct',
         fixture='private/review/gw5922-attempt11-campaign/fixture/frame-5924-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/723f2066ee5748d4821f4d2de8c68c50'),
    dict(id='R09-neutral-zero-reserve-complete', frame=5938, slot=5, draws=8, mode='direct',
         fixture='private/review/gw5924-attempt12-campaign/fixture/frame-5938-slot-5-blocked-before',
         expected='private/review/neutral-mule-captured-width/54ce64d726194635840afa00f6f0e6bd'),
    # historical focused captures: compare through the oracle (--expected) because the
    # interpreter output dirs for these batches are not retained in the checkout.
    dict(id='R01-owned-light-4801', frame=4801, slot=1, draws=0, mode='oracle', oracle='owned',
         fixture='private/review/authentic-krell-capture4802/frame-4801-slot-1-owned-before',
         instructions=2224),
    dict(id='R02-owned-light-5893', frame=5893, slot=1, draws=0, mode='oracle', oracle='owned',
         fixture='private/review/authentic-krell-194/frame-5892-slot-1-blocked-before-owned',
         instructions=1714),
    dict(id='R03-owned-light-5939', frame=5939, slot=1, draws=0, mode='oracle', oracle='owned',
         fixture='private/review/authentic-krell-195/frame-5939-slot-1-blocked-before-owned',
         instructions=1503),
    # F1-R04 intentionally expanded (2026-10-17): the frame-5940 owned FULL-path
    # transition is now independently source-proved by the owned oracle run on
    # private/review/attempt13-fx-5940 (status source_prepared, 11594
    # instructions, 5 draws, output 606d12841ae14e6b87a2bd999f44e63a, slot1-only
    # deltas 0x19/0x1b..0x1e/0x4b/0x5e/0x66/0x68/0x6c); the former negative gate
    # becomes this direct positive comparison per the family policy's
    # "source-proved and intentionally expanded" condition.
    dict(id='R04-owned-full-path-5940', frame=5940, slot=1, draws=5, mode='direct',
         fixture='private/review/attempt13-fx-5940',
         expected='private/review/owned-mule-captured-width/606d12841ae14e6b87a2bd999f44e63a'),
    # F6 conversion of the attempt17 negative row to POSITIVE (2026-10-17), per the
    # family policy's "source-proved and intentionally expanded" condition. Source
    # basis: the literal executed path 0005ed37/0005ed3b -> 0005ee25-0005ee32 (parity
    # gate, FULL path) -> 0005eed5-0005ef1f -> 0005ef22 call 0x5e0b0 -> 0005ef32 call
    # 0x5eb30 -> 0005ef3a-0005ef40 (fresh timer, expired arm NOT taken) ->
    # 0005ef49-0005ef68 + 0006a20d-0006a21f -> tail 0006a290-0006a53e with 0006a4a0
    # DEC word(0x6c) -> epilogue 0005e640/0005e817 -> 0007ac00; zero RNG draws
    # (attempt18 note, research/variants/attempt18-owned-state20-fresh-timer-domain.md).
    # The owned oracle now admits exactly this measured shape and produced the
    # authoritative output below (source_prepared, 2307 instructions, 0 draws,
    # creatures.bin slot-1-only 20-byte delta); the port implements the same pinned
    # arm in plan_owned_after_prefix (fresh_hit). Same fixture as the former
    # negative gate; this is NOT a conversion of any other row.
    dict(id='R10-owned-state20-fresh-timer-5940', frame=5940, slot=1, draws=0, mode='direct',
         fixture='private/review/attempt17-live-5940-blocked-owned',
         expected='private/review/owned-mule-captured-width/2e412d033c0e43b9898e309d022b321e'),
    # F9 conversion of the F7 negative row to POSITIVE (2026-10-17), per the family
    # policy's "source-proved and intentionally expanded" condition. Source basis:
    # the literal executed path 0005ef3a/0005ef3e/0005ef40 (JLE 0x65384 TAKEN when
    # word(0x6c)==0) -> 0x65384-0x65397 state dispatch (jump table 0x5ecf0, entry
    # 0x14 -> 0x5f4c4) -> 0005f4c4-0005f4d3 (word(0x6a)>0) -> 0005f501-0005f521 ->
    # 0005f532-0005f569 (CALL 0x81c60) -> 0005f5bc JNZ 0x6029a -> 000602a2 state 8,
    # 00060291 word(0x6c)=0 -> tail 0006a290-0006a53e (0006a4a0 DEC SKIPPED;
    # 0006a4a7 CALL 0x5e640 animation 0x10->0x05); zero RNG draws. The first
    # triple 0x50..0x53 is admission-only, so its mismatch does NOT refuse this
    # shape. The reference model extended first (f8-interpreter-expired-timer-
    # extension-checkpoint-20261017.json: source_prepared, 21226 instructions,
    # 0 draws, deterministic across three runs, slot-1-only 21-byte delta), then
    # the port implemented the same pinned arm (expired_hit in
    # plan_owned_after_prefix + the 5F4C4/6029A completion in the choice callback).
    # Same fixture as the former negative gate; this is NOT a conversion of any
    # other row.
    dict(id='R20-owned-state20-expired-timer-triple-mismatch-5946', frame=5946, slot=1, draws=0, mode='direct',
         fixture='private/review/attempt20-live-5946-blocked-owned',
         expected='private/review/owned-mule-captured-width/2a54ba4cd4344ba98d35d210c2d3d9b1'),
    # F10: R22 was added FIRST as a measurement-first NEGATIVE gate pinning the
    # live 5962 refusal, then CONVERTED TO POSITIVE in the same change set
    # (2026-10-17) once the neutral oracle adjudicated the record, per the
    # family policy's "source-proved and intentionally expanded" condition.
    # Source basis: the literal executed path 0x5ed20 entry -> 0x5ef37/0x5ef3e/
    # 0x5ef40 (JLE 0x65384 taken when word(0x6c)==0) -> 0x65384-0x65397 state
    # dispatch -> 0x60332-0x6033b state-21 arm (target resolved from the
    # attacker's own 0x55 link) -> 0x6d390 distance squared -> 0x60340-0x60347
    # range compare (measured 85149 > 16384 FAILS) -> 0x60392-0x6039e
    # out-of-range leave-pursuit branch -> 0x1f270 draw -> 0x73770 -> 0x6152c
    # (animation 0x0b->0x05) -> 0x60291 -> 0x5ef46 -> tail 0x6a290-0x6a53e with
    # 0x6a4a7 CALL 0x5e640; exactly one draw (seed 0x0a25->0x0681, ring 17->18,
    # draw_count 50010->50011). Measured deltas are slot-5-only: state
    # 0x5e 0x15->8, animation 0x19 0x0b->0x05, frame counters 0x1c..0x1e ->0,
    # word(0x66) -2, word(0x6c) ->15; no position, enemy or other-buffer write.
    # The neutral oracle produced the authoritative expected output
    # 23a9b1f85a74452588e40de2712fe016 (source_prepared, 2376 instructions,
    # 1 draw); the port implements the same measured branch in
    # mule_neutral_completed_attack with its own field gate as the admission
    # shape. Same fixture as the former negative gate; this is NOT a conversion
    # of any other row.
    dict(id='R22-neutral-state21-pursuit-5962', frame=5962, slot=5, draws=1, mode='direct',
         fixture='private/review/attempt23-live-5962-blocked-neutral',
         expected='private/review/neutral-mule-captured-width/23a9b1f85a74452588e40de2712fe016'),
    # F11 conversion of the R23 negative row to POSITIVE (2026-10-17), per the
    # family policy's "source-proved and intentionally expanded" condition.
    # Source basis: the literal executed oracle path 0x5ed20 entry ->
    # 0x5ed37/0x5ed3b (word(0x6c)=15 != 0, JLE 0x65384 NOT taken) ->
    # 0x5ee1c/0x5ee1f in-pursuit fall-through -> shared body 0x6fb60..0x70790
    # with 0x6d390 distance queries and 0x5b400/0x8b520/0x8b340 cell walks ->
    # 0x5ee25 -> 0x5eb30 -> 0x5ec6b/0x5ec98 -> 0x5ee3f -> 0x5e640 ->
    # 0x5ee48 -> 0x7ac00 epilogue -> 0x5ee73/0x5eec6; NO 0x65384 state
    # dispatch, NO tail 0x6a290 DEC, and ZERO RNG draws. The neutral oracle
    # produced the authoritative output 4d36d9f1ee814938abd972b8808972b1
    # (source_prepared, 1517 instructions, 0 draws): all eight buffers
    # byte-identical to the input - an admitted in-pursuit hold, not a
    # transition. The port implements the same measured no-op hold in
    # step_mule_followup_inner ahead of mule_route_prefix, gated to owner 4
    # state 8 with the surviving pursuit fields (0x60=0x88, 0x61=0x40,
    # word(0x6c)>0, word(0x6a)>0, 0x52=1, 0x57=1, 0x5a=0, 0x78=0, 0x7a=4,
    # 0x4f=0, 0x5d=0, 0x68=0, word(0x58)=0, equal 0x50/0x55 triples and a
    # resolvable live target). Same fixture as the former negative gate; this
    # is NOT a conversion of any other row.
    dict(id='R23-neutral-state8-pursuit-88-40-5963', frame=5963, slot=5, draws=0, mode='direct',
         fixture='private/review/attempt24-live-5963-blocked-neutral',
         expected='private/review/neutral-mule-captured-width/4d36d9f1ee814938abd972b8808972b1'),
    # F14 CONVERSION of the R24 negative row to POSITIVE (2026-10-17), per the
    # family policy's "source-proved and intentionally expanded" condition.
    # Source basis (F13/F14): the corrected campaign blocked here ONLY because
    # the port had no hostile kind-1 candidate arm; the original EXECUTES a
    # specific branch. Scorer 0x6B700: alliance test 0x6B7DF..0x6B7FE computes
    # DL=[self_owner*0x70d+0x6e5], EAX=1<<cand_owner, TEST EDX,EAX with the
    # VALIDATED bytes 75 04 (JNZ 0x6B804; the Ghidra listing's 0x6B800 render is
    # wrong), so with player0's mask 0 the neutral Mule is HOSTILE; kind
    # dispatch 0x6B81E..0x6B830 -> jump table 0x6BD70[0]=0x6B9B0 (confirmed
    # executed); hostile arm 0x6B9B0..0x6B9C3 (state!=0xF), 0x6BA8A..0x6BAF3
    # (candidate state<0x1A, [ESI+0x60]&1==0). The owned idle AI then dispatches
    # the 5B580 case-1 acquisition (trace call order ... 0x62874 -> 0x5b580 ->
    # 0x5cc60 ... -> 0x62e24 -> tail 0x6a290..). The owned oracle executed the
    # whole callback on this exact fixture (trace
    # private/review/owned-mule-captured-width/9c48d067bdd04c5d807ca4c4ce3140d3:
    # 31638 instructions, 4 draws, slot1-only 32-byte creature delta + one
    # reveal backing byte) and adjudicates this row through --expected; the
    # port implements the same arm in owned_mule_single_hostile's hostile
    # kind-1 classification plus the widened
    # mule_owned_hostile_acquisition::plan_acquisition (enemy state8/0x88
    # pursuit shape; retained 0x5a=1 with 0x78=7). This is NOT a conversion of
    # any other row; the catch-all still refuses every unproved candidate.
    dict(id='R24-owned-arrival-idle-kind1-neighbor-5966', frame=5966, slot=1, draws=4, mode='oracle', oracle='owned',
         fixture='private/review/attempt25-live-5966-blocked-owned',
         instructions=31638),
    # F15 attempt28: the R28 measurement-first NEGATIVE gate (same fixture)
    # CONVERTED TO POSITIVE (2026-10-17) per the family policy's
    # "source-proved and intentionally expanded" condition. Source basis: the
    # literal executed path 0x5ed20 -> 0x5ed3b (JZ 0x5ee1c) -> 0x6fb60
    # virtual94 movement prefix -> 0x5ee25..0x5ee32 parity gate (JNZ taken:
    # byte[0x11877e]+[esi+0xf] odd) -> 0x5eed5..0x5ef1f -> 0x5ef22 CALL
    # 0x5e0b0 -> 0x5ef32 CALL 0x5eb30 -> 0x5ef37..0x5ef40 (word(0x6c)=0x7ffe,
    # fresh-timer side) -> 0x5ef49 -> 0x6a22b..0x6a2dd -> 0x65743..0x65de6
    # (state-1 movement continuation) -> tail 0x6a290..0x6a53e (0x6a4a0 DEC
    # word(0x6c) executed) -> 0x7ac00 epilogue; 2542 instructions, 0 draws,
    # and ZERO accesses to the enemy record region (probe
    # private/review/attempt28-5968-probe-2a44a5da): the degraded target
    # (enemy state 8, 0x55-ref back at the owned mule) is admission-only
    # data on this path. The reference model extended first
    # (owned-mule-captured-width-review.py degraded-enemy admission; output
    # source_prepared 22da7a68d74f407b80611ca51ad602d0, 2542 instructions,
    # 0 draws, slot-1-only 12-byte delta), then the port implemented the
    # same pinned arm (degraded_pursuit admission at owned_mule_updates.rs
    # + the 5EB30 no-AI continuation in the choice callback). Same fixture
    # as the former negative gate; this is NOT a conversion of any other
    # row. R21/R25 remain as fail-closed negatives.
    dict(id='R28-owned-state1-degraded-target-5968', frame=5968, slot=1, draws=0, mode='direct',
         fixture='private/review/attempt28-5968-measure/capture/frame-5968-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/22da7a68d74f407b80611ca51ad602d0'),
    dict(id='R29-owned-state1-pursuit-target-cell-5992', frame=5992, slot=1, draws=0, mode='direct',
         fixture='private/review/attempt29-5992-measure/campaign/capture/frame-5992-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/d076d8fe60f547eb8c391064778a55aa'),
    # F17: R30 was added FIRST as a measurement-first NEGATIVE gate pinning the
    # measured 6008 refusal (Creatures(UnsupportedNeighborhood) at the owned
    # callback, exact rejection + all eight buffers byte-identical, 0 draws,
    # private/review/attempt30-6008-portrun/out2), then CONVERTED POSITIVE in
    # the same change set. Source basis: the measured record IS the B9
    # degraded-pursuit shape (0x5e=1, 0x60=0xc8, 0x61=0, 0x78=7, 0x52/0x57/0x5a=1,
    # word(0x6c)=0x7fea, 0x76=37, word(0x49)&0x100==0) reaching ARRIVAL: the
    # source5EB30 body ran its zeroing block (0005eb9b AND byte[0x60],0x7f ->
    # 0x48; 0005ebd9 MOV word[0x6c],0), then 0005ef3a/0005ef40 reads the FRESH
    # word(0x6c)==0 and JLEs to 00065384 whose jump table 0x5eca0 entry 1 ->
    # 0005ef46 - the same no-AI/no-6B700 continuation as the fresh side - and
    # the continuation's near-target block 00065d80..00065dce (73770 distance
    # <= 0x4000; 6EDA0/7A6C0 shortest delta 551 > 0x100) writes word(0x4b)=0
    # through 73970/AE028's zero delta. The former word(record,0x6c)>0
    # freshness test on the POST-prefix record was port-authored (map rule 1):
    # the prefix itself produces the zeroed timer. Oracle: source_prepared,
    # 2818 instructions, 0 draws, output
    # owned-mule-captured-width/b3ac36ab78474896ab7e200a37250f8b; port
    # byte-identical over all eight buffers (427369). A PRE-prefix expired
    # timer (word(0x6c)==0 before the callback), the aligned virtual38 side
    # and other degraded-target kinds keep refusing (fail-closed).
    dict(id='R30-owned-degraded-pursuit-arrival-6008', frame=6008, slot=1, draws=0, mode='direct',
         fixture='private/review/attempt30-6008-measure/campaign/capture/frame-6008-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/b3ac36ab78474896ab7e200a37250f8b'),
    # F18 fail-closed negative gate (added 2026-10-18, attempt31): pins the
    # MEASURED 6010 refusal - the pre-prefix EXPIRED-TIMER side of the B9
    # degraded-pursuit continuation, the boundary F17 explicitly left
    # unsupported. Owned Mule slot 1 in the B9 arrival shape (0x5e=1,
    # 0x60=0x48, 0x61=0, 0x78=7, 0x52/0x57/0x5a=1, word(0x49)=0x2000) but with
    # word(0x6c)=0 ALREADY BEFORE the callback (F17's arrival side carried
    # word(0x6c)=0x7fea pre-prefix), so both the F15 fresh arm (word(0x6c)>0)
    # and the F17 arrival arm (pre-prefix word(0x6c)>0) skip and the record
    # falls into owned_mule_neighborhood_no_action, whose food guard
    # (word(0x68)=5 <= word(0x62)>>1=65) refuses at owned_mule_updates.rs:401.
    # The source oracle ADMITS this record: source_prepared, 2431 instructions,
    # 4 draws, output
    # private/review/owned-mule-captured-width/59dc2afb22b64ab6bf89e1ccb85df79b
    # (slot-1-only deltas: 0x5e 1->8, 0x60 0x48->0x88, word(0x6c) -> 0x7ffe,
    # 0x52/0x57 cleared, byte 0x54=0x14, word(0x49)|0x10, waypoint dword 0xb
    # re-planned, word(0x3f)=79, bytes 0x2d..0x31 zeroed). The trace settles the
    # DECISION STRUCTURE but not yet every mechanism: 65c7d JNZ
    # ((word(0xf)+rng_ring)&3)==3 -> 65c9a; 65cb2 (0x7fff-word(0x6c))>0x100;
    # 65cfe draw; 65d42 CALL 5cc60 -> flag -> 65d58 JNZ 65deb; 65deb draw;
    # 65e68 byte[0x5e]=8; virtual+0x8c; 73c20 waypoint; 65ea1 0x49|=0x10;
    # 65e97 byte[0x54]=0x14; 65ed6 word(0x6c)=0x7fff; 65eeb CALL 6bdd0; tail
    # 6a4a0 DEC word(0x6c) -> 0x7ffe. NOT yet settled from listing text: 5c920
    # (clears 0x52/0x57/0x2d..0x31), the [EBX+0x8c]/[EDX+0x80] virtuals (8b780),
    # 6bdd0/6c190, the remaining 2 of 4 draw sites, and the creature-side
    # activation-RNG refs (0x1185ac/0x11862c/0x1186ac). Deliberately NOT
    # hand-fabricated; the gate asserts the exact expected rejection AND
    # complete transactional world/RNG preservation. Never silently expanded.
    # F21 CONVERSION of the R31 negative row to POSITIVE (2026-10-18, attempt34),
    # per the family policy's "source-proved and intentionally expanded"
    # condition. Source basis: the MEASURED 6010 record is the PRE-prefix
    # EXPIRED-timer side of the B9 degraded-pursuit continuation (0x5e=1,
    # 0x60=0x48, 0x61=0, 0x78=7, 0x52/0x57/0x5a=1, word(0x49)=0x2000, word(0x6c)=0
    # ALREADY before the callback), reaching the 65c60 ring gate on the state-1
    # continuation 65743..65de6: LONG side ((word(0xf)+[0x11877e])&3)==3 =
    # (6010+1)&3==3 ([0x11877e] = the mission tick, modeled as counter=args.frame
    # per the F20 receipt, producer INC dword ptr [0x11877e] at 0x1bd4f); 65cb2
    # timer gate (0x7fff-word(0x6c))>0x100; 65cbe activation write 0x9aa/0x10e4d8
    # (cursor 26->27); 65cfe draw 1 (LCG -0x19d3, counter 50243->50244);
    # 65d42 CALL 5cc60 -> 5c920 (0x60&2=0) -> jump-table entry 0 -> the 5cca0
    # zeroing block (word[0x35]=0, byte[0x15]&=0xdf, word[0x33]=0,
    # word[0x60]&=0xd71a 0x48->0x08, byte[0x5d]=0, byte[0x57]=0, word[0x6c]=0)
    # with the flag (BH=1) preserved -> 65d58 JNZ 65deb taken; 65deb activation
    # write 0xa03/0x10e4e5 (cursor 27->28) + draw 2 (LCG -0x44bf19d3, counter
    # ->50245); 65e68 byte[0x5e]=8; 65e7a CALL [vtable+0x8c]=8b780 (measured
    # AL!=0; the ECX arg 10..29 is never read by 8b780); 65e88 CALL 73c20:
    # activation writes 0x59c/0x10e6b7 (cursor 28->29, counter +=2) and
    # 0x59d/0x10e6c0 (cursor 29->30), draws 3-4 (counter ->50247), and the
    # waypoint dword 0xb replanned from the 0x107ee8/0x1086e8 slope tables with
    # index norm(rem3*8): measured 0x??48/0x??73 -> 0x23a/0x5d6c; 65e97
    # byte[0x54]=0x14; 65ea1 word(0x49)|0x10; 65eaf CALL 13e50 (OR 0x4); 65eb4
    # CALL 13e70 (AND word[0x49],0xeffc; AND byte[0x16],0x7f); 65ebb/65ec3 CALL
    # 13eb0 (+0x4d, +0x50: byte[0x4f]=0, byte[0x52]=0); 65ed0 CALL
    # [vtable+0x80]=5a990 -> word(0x3f)=((75+min(0x76,100)*25/100)*
    # dword[0x1045fc+45*0x7a+6]/100*(0x7b+1))>>8 = 591; 65ed6 word(0x6c)=0x7fff;
    # 65ee3 byte[0x60]|=0x80 -> 0x88; 65eeb CALL 6bdd0: AL = byte[0x104a64 +
    # 2*byte[0x7a]]==3, species 4 reads 0x04 -> 65ef2 JZ 6a290 taken; shared
    # tail 6a4a0 DEC word(0x6c) -> 0x7ffe. Seed 37665 -> 3889, ring 26 -> 30,
    # counter 50243 -> 50247 corroborated by rng.bin. Oracle: source_prepared,
    # 2431 instructions, 4 draws, output
    # private/review/owned-mule-captured-width/59dc2afb22b64ab6bf89e1ccb85df79b;
    # the port implements the same pinned arm in
    # mule_owned_expired_replan::plan_expired_reacquisition (wired at
    # owned_mule_updates.rs, pre word(0x6c)==0 gate). Table data extracted to
    # private/integration/game-runtime/creature/b11-tables.bin from
    # GW-CD-candidate.exe (receipt private/integration/b11-tables-extraction.json);
    # missing/unpinned data refuses fail-closed. Every unmeasured side of the
    # block stays fail-closed: ring-gate short side 65d4d/65d5e, timer-gate
    # short side 65cb8 (and the unmodeled 65c84 kind==3 ECX doubling), draw-1
    # flag JG side 65d3c, 8b780 AL==0 side 65c57, 6bdd0 true side 65ef8 with
    # 6c190 (never executed on the measured path). R21/R25 remain fail-closed.
    dict(id='R31-owned-state1-pursuit-expired-preprefix-6010', frame=6010, slot=1, draws=4, mode='direct',
         fixture='private/review/attempt31-6010-measure/campaign/capture/frame-6010-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/59dc2afb22b64ab6bf89e1ccb85df79b'),
    # F35 measurement-first gate (added 2026-10-18, attempt35; CONVERTED from
    # negative to POSITIVE in the same change set per the family policy's
    # "source-proved and intentionally expanded" condition): pins the measured
    # 6060 record - the B11 ring-gate SHORT side (65c7f JMP 65d4d), the F21
    # fail-closed boundary. Owned Mule slot 1 in the same B9/B11 shape
    # (0x5e=1, 0x60=0x48, 0x61=0, 0x78=7, 0x52/0x57/0x5a=1, 0x76=0x25,
    # word(0x49)=0x220, word(0x6c)=0 pre AND post) at frame 6060, where the ring
    # gate ((word(0xf)+[0x11877e])&3)==3 = (6060+1)&3 = 1 FAILS. Source basis
    # (addresses; listing private/ghidra/player-creation/instructions-0005ed20.txt
    # and trace private/review/owned-mule-captured-width/d386493180e14ca7bc14f28d32e22e5f,
    # 1736 instructions, 0 draws): the ONLY entry to 65c60 is 65951 JZ (taken
    # when CALL [ctx+0x68] returns AL==0), so byte[EBP+0x186] = entry AL = 0
    # deterministically; 65c7f JMP 65d4d -> 65d56 TEST CL,CL NOT taken ->
    # 65d5e virtual pair (1f270 twice) -> the 65d80..65dce near-target block
    # (73770 <=0x4000, 6eda0 FALSE side 65da5 JZ 65dbe, 73970 -> 65dce
    # word(0x4b)=0) -> JMP 6a290. The timer gate/draws/5cc60/65deb block are
    # NOT executed on this side. The near-target block is the SAME block the
    # F17 arrival arm already proved (R30); F35 moved it into the shared helper
    # mule_owned_expired_replan::near_target so both sides run one
    # implementation. The port arm: ring-gate failure inside
    # plan_expired_reacquisition returns near_target(record). Oracle:
    # source_prepared, 1736 instructions, 0 draws, output
    # private/review/owned-mule-captured-width/d386493180e14ca7bc14f28d32e22e5f
    # (slot-1-only deltas: 0x19 4->2, 0x27 0x1e->0x1d, 6df30 vector rebuild
    # 0x2d..0x31 / 0x39..0x3d, word(0x49) 0x220->0, word(0x4b) 0x31a->0,
    # 0x66 0x59->0x57; rng.bin byte-identical). Port vs oracle: zero diffs over
    # all eight buffers (427,369 compared). 65da7..65db5 (aligned virtual38)
    # and 65dd7.. (>0x4000 distance) stay fail-closed; R21/R25 remain
    # fail-closed negatives. Never silently expanded.
    dict(id='R32-owned-state1-pursuit-ring-short-6060', frame=6060, slot=1, draws=0, mode='direct',
         fixture='private/review/attempt35-6060-measure/capture/frame-6060-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/d386493180e14ca7bc14f28d32e22e5f'),
    # F23/attempt36: R33 was added FIRST as a measurement-first NEGATIVE gate
    # pinning the measured 6100 refusal, then CONVERTED TO POSITIVE in the same
    # change set (2026-10-18, attempt37) per the family policy's
    # "source-proved and intentionally expanded" condition. Source basis: the
    # 6100 refusal was a TWO-STAGE PORT OVER-ADMISSION, not a source behaviour.
    # Stage 1: the port's mule_route_prefix consumes the timer (prefix record
    # 0x60=8, word(0x6c)=0, 0x52=0/0x57=0; probe private/review/f23-instrument5.py).
    # Stage 2: owned_mule_single_hostile's walk matched branchB on TWO
    # neutral-Mule candidates (slot 2 then slot 5, both 0x5e=8/0x60=0x88/owner4/
    # MuleSelector4) and refused on the SECOND (the line-395-region DUP refusal).
    # The SOURCE does not refuse: scorer 6b700's hostile kind-1 tail
    # 6b9cf..6b9e9 ACCUMULATES the best candidate per category (CMP EDI,
    # [EBX+0x1dc]; JLE 6b9ea keeps the existing one; the strictly closer
    # candidate replaces it at 6b9de/6b9e4 storing the candidate pointer at
    # [EBX+0x1cc]) - there is NO second-candidate refusal (listing private/
    # ghidra/initial-creature/instructions-0006b700.txt). The measured winner is
    # slot 5 (closer: walk-subject distance weight 768 vs slot 2's 0; slot 2's
    # 0x50/0x55 refs are null so the 5b580 Enemy admission cannot take it), and
    # the acquisition path 5b580/5cc60/5a990 then runs through the 65743
    # continuation whose tail DEC writes word(0x6c)=0x7ffe. The source path runs
    # through the ORDINARY AI (0 draws prefix/continuation + the 4-draw 5b580
    # acquisition = the 4-draw oracle total). Port: the state-8 acquisition
    # admission is bounded to the timer-expired domain (word(0x6c)==0
    # post-prefix) and the walk accumulates the best same-arm branchB candidate
    # instead of refusing on the second. Oracle: source_prepared, 12109
    # instructions, 4 draws, output
    # private/review/owned-mule-captured-width/e4f43b9e6d9a4f8aa13054f5d4760c06;
    # port zero diffs over all eight buffers (427,369), byte-identical rng.bin.
    # Mixed first-arm/second-arm candidates stay refused (unmeasured).
    dict(id='R33-owned-state8-timer-consumed-two-pair-candidates-6100', frame=6100, slot=1, draws=4, mode='direct',
         fixture='private/review/attempt36-6100-measure/frame-6100-slot-1-blocked-before-owned',
         expected='private/review/owned-mule-captured-width/e4f43b9e6d9a4f8aa13054f5d4760c06'),
]
NEGATIVE = [
    # F16 measurement-first NEGATIVE gate (added 2026-10-17, attempt29): pins the
    # measured 5992 refusal. Owned Mule slot 1 in the B9 degraded-pursuit shape
    # (0x5e=1, 0x60=0xc8, 0x61=0, 0x78=7, 0x52/0x57/0x5a=1, word(0x6c)=0x7ff2)
    # whose continuation now commits into the DESTINATION cell (plan.new_cell =
    # 11906) that already contains creature slot 5 - the record's own ref55
    # pursuit target (degraded to state 8, its own cell == the queried cell).
    # The port's mule_cell_entry_predicate first branch runs the blanket
    # check_mule_cell_entry head-check (any kind1 occupant refuses), while the
    # source predicate 5B100 has a dedicated STATE-1 ARM 0005b18c..0005b204 that
    # special-cases the ref55 target (5b1af..5b204: occupant==self -> admit;
    # occupant kind1 at its own cell == queried cell -> admit unless the target's
    # virtual+0x68 refuses; the measured oracle run admitted: slot1 committed,
    # link word -> 0x2e82). Oracle: source_prepared, 2856 instructions, 0 draws
    # (private/review/owned-mule-captured-width/d076d8fe60f547eb8c391064778a55aa).
    # The gate asserts the exact expected rejection AND complete transactional
    # world/RNG preservation. Converted to POSITIVE in this same change set; R21
    # and R25 remain fail-closed. Never silently expanded to other occupants.
    # F9 fail-closed negative gate (added 2026-10-17 in the same change set as the
    # R20 conversion so the checker keeps at least one negative row). Pins the
    # domain the F8 evidence EXPLICITLY leaves unexecuted: the identical expired
    # state-20 shape (fixture attempt20-live-5946-blocked-owned, word(0x6c)=0,
    # 0x60=0x18, 0x61=0x80, 0x52=0, 0x57=1, 0x5a=1) with word(0x6a) forced to 0.
    # Source basis: the 0005f4d5-0005f4fc completion side of 5F4C4 requires
    # word(0x6a)>0 (0005f4c4-0005f4d3 JG) and was never executed on the measured
    # path, so word(0x6a)<=0 stays unsupported; the port's generic admission gate
    # (w(0x6a)<=0) must refuse it. The fixture is DERIVED, not measured: slot 1
    # word(0x6a) 58 -> 0, all other bytes of all eight buffers byte-identical to
    # the live capture. The gate asserts the exact expected rejection AND complete
    # transactional world/RNG preservation (verified: blocked
    # Creatures(UnsupportedMotionState), all eight buffers byte-identical to the
    # input, 0 draws). Never silently expanded.
    dict(id='R21-owned-state20-expired-timer-zero-6a-5946', frame=5946, slot=1,
         fixture='private/review/attempt22-owned-5946-expired-timer-zero-6a',
         expect_error='Creatures(UnsupportedMotionState)'),
    # F14 fail-closed negative gate (added 2026-10-17 in the same change set as
    # the R24 conversion so the checker keeps at least two honest negatives).
    # Pins the domain the F14 evidence EXPLICITLY leaves uncounted: the same
    # 5966 owned idle walk with the kind-1 candidate's state forced to 0x1A,
    # where the scorer's hostile arm 0x6BA8A..0x6BAF3 requires candidate
    # state<0x1A, so the Mule is not counted and the downstream idle
    # continuation is UNEXECUTED by the oracle (31638-instruction trace
    # 9c48d067bdd04c5d807ca4c4ce3140d3 ran with state=8). The fixture is
    # DERIVED, not measured: creatures.bin slot 5 byte 0x5e 8 -> 0x1A, all
    # other bytes of all eight buffers byte-identical to the live capture.
    # The port's catch-all must refuse it. The gate asserts the exact expected
    # rejection AND complete transactional world/RNG preservation. Never
    # silently expanded.
    dict(id='R25-owned-idle-kind1-state-over-19-5966', frame=5966, slot=1,
         fixture='private/review/attempt27-owned-5966-kind1-state-1a',
         expect_error='Creatures(UnsupportedNeighborhood)'),
    # F23/attempt37: R33 moved to POSITIVE above (converted in the same change
    # set, 2026-10-18, after the accumulation + timer-expired-domain bound made
    # the port byte-identical to the oracle). This row replaces it as the new
    # fail-closed negative, keeping at least two honest negatives alongside
    # R21/R25. DERIVED, not measured: the 6100 measured fixture
    # (attempt36-6100-measure/frame-6100-slot-1-blocked-before-owned) with the
    # scan winner's (slot 5) state byte 0x5e 8 -> 0x1A, all other bytes of all
    # eight buffers byte-identical. Source basis: the scorer's hostile kind-1
    # arm 0x6BA8A..0x6BAF3 requires candidate state<0x1A, so the winner is not
    # counted and the walk yields no hostile; the record then falls through the
    # timer-expired acquisition gate to the neighborhood catch-all, whose
    # kind-1-under-food_guard arm is UNPROVED source (B0) and must refuse. The
    # gate asserts the exact expected rejection AND complete transactional
    # world/RNG preservation (verified: blocked Creatures(UnsupportedNeighborhood),
    # 0 draws, all eight buffers byte-identical to the input). Never silently
    # expanded.
    dict(id='R34-owned-state8-timer-consumed-scan-winner-state-1a-6100', frame=6100, slot=1,
         fixture='private/review/attempt37-owned-6100-scan-winner-state-1a/frame-6100-slot-1-blocked-before-owned',
         expect_error='Creatures(UnsupportedNeighborhood)'),
]

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--tag', required=True)
    ap.add_argument('--head', default=None, help='expected HEAD sha (skips the assertion when omitted)')
    ap.add_argument('--campaign', default=None, help='replay.txt to run headless')
    ap.add_argument('--no-focused', action='store_true')
    ap.add_argument('--no-suite', action='store_true')
    a = ap.parse_args()

    run = P / f'private/integration/{a.tag}'
    run.mkdir(parents=True, exist_ok=False)
    env = {k: v for k, v in os.environ.items() if not k.startswith('GENEWARS_CAPTURED_MULE_')}
    env.update(CARGO_BUILD_JOBS='2', CARGO_PROFILE_TEST_OPT_LEVEL='1')
    results = []

    def sh(args, logname, extra=None):
        e = dict(env)
        if extra:
            e.update(extra)
        log = run / logname
        with log.open('wb') as f:
            rc = subprocess.run(args, cwd=P, env=e, stdout=f, stderr=subprocess.STDOUT).returncode
        return rc, log.read_text(errors='replace')

    def rec(step, ok, detail, kind='check'):
        results.append({'step': step, 'ok': bool(ok), 'kind': kind, 'detail': detail})
        print(('PASS ' if ok else 'FAIL ') + step + ' :: ' + json.dumps(detail)[:420], flush=True)

    out_root = run / 'out'
    out_root.mkdir()

    def captured(fixture, frame, slot, outdir, expect=None):
        e = {'GENEWARS_CAPTURED_MULE_INPUT': str(fixture), 'GENEWARS_CAPTURED_MULE_OUTPUT': str(outdir),
             'GENEWARS_CAPTURED_MULE_FRAME': str(frame), 'GENEWARS_CAPTURED_MULE_SLOT': str(slot)}
        if expect:
            e['GENEWARS_CAPTURED_MULE_EXPECT'] = expect
        return sh([PY, '-B', 'scripts/cargo.py', 'test', '-p', 'genewars-runtime', 'captured_mule_callback',
                   '--offline', '--locked', '--', '--ignored', '--nocapture'],
                  f'{outdir.name}.log', e)

    def geometry(port_dir, expected_dir=None):
        """Exact-length gate over the port output (and, when given, the interpreter output)."""
        got, bad = {}, []
        for b in BUFS:
            p = port_dir / b
            if not p.exists():
                bad.append(f'{b}:missing'); continue
            n = p.stat().st_size
            got[b] = n
            if n != SIZES[b]:
                bad.append(f'{b}:{n}!={SIZES[b]}')
            if expected_dir is not None:
                q = expected_dir / b
                if not q.exists() or q.stat().st_size != n:
                    bad.append(f'{b}:expected-mismatch')
        tot = sum(got.values())
        if tot != TOTAL:
            bad.append(f'total:{tot}!={TOTAL}')
        return got, bad

    def diffs(port_dir, expected_dir):
        out = {}
        for b in BUFS:
            p, q = port_dir / b, expected_dir / b
            if not (p.exists() and q.exists()):
                out[b] = 'missing'; continue
            x, y = p.read_bytes(), q.read_bytes()
            if len(x) != len(y):
                out[b] = f'length {len(x)}!={len(y)}'; continue
            n = sum(1 for i in range(len(x)) if x[i] != y[i])
            if n:
                first = [i for i in range(len(x)) if x[i] != y[i]][:4]
                out[b] = f'{n} diffs first@{first}'
        return out

    rc, _ = sh([PY, '-B', 'scripts/cargo.py', 'test', '--workspace', '--offline', '--locked', '--no-run'], 'compile.log')
    rec('compile-workspace', rc == 0, {'rc': rc})

    for c in POSITIVE:
        fixture = P / c['fixture']
        expected = P / c['expected'] if c.get('expected') else None
        outdir = out_root / c['id']
        rc, _ = captured(fixture, c['frame'], c['slot'], outdir)
        res = json.loads((outdir / 'result.json').read_text()) if (outdir / 'result.json').exists() else None
        # 1) execution/identity gate
        exec_ok = (rc == 0 and res is not None and res.get('status') == 'passed'
                   and res.get('frame') == c['frame'] and res.get('slot') == c['slot']
                   and res.get('input_roundtrip') is True and not res.get('error'))
        rec(f"{c['id']}-execution", exec_ok, {'rc': rc, 'rust': res}, kind='positive')
        got, bad = geometry(outdir, expected if c['mode'] == 'direct' else None)
        rec(f"{c['id']}-exact-lengths", not bad, {'lengths': got, 'problems': bad}, kind='positive')
        pd = diffs(outdir, expected) if expected is not None else {}  # oracle mode: the oracle compares
        rng_in = (fixture / 'rng.bin').read_bytes().hex() if (fixture / 'rng.bin').exists() else None
        rng_out = (outdir / 'rng.bin').read_bytes().hex() if (outdir / 'rng.bin').exists() else None
        rng_exp = (expected / 'rng.bin').read_bytes().hex() if expected is not None and (expected / 'rng.bin').exists() else None
        draws_ok = res is not None and res.get('rng_draws') == c['draws']
        if c['mode'] == 'direct':
            rec(f"{c['id']}-rng+bytes", (not pd) and draws_ok and rng_out == rng_exp,
                {'draws': None if res is None else res.get('rng_draws'), 'expected_draws': c['draws'],
                 'rng_in': rng_in, 'rng_out': rng_out, 'rng_expected': rng_exp, 'buffer_diffs': pd}, kind='positive')
        else:
            rc2, txt2 = sh([PY, '-B', f"research/variants/{c['oracle']}-mule-captured-width-review.py",
                            '--before', str(fixture), '--expected', str(outdir),
                            '--frame', str(c['frame']), '--slot', str(c['slot'])], f"{c['id']}-oracle.log")
            try:
                d = json.loads(txt2)
            except Exception as ex:
                d = {'parse_error': repr(ex), 'tail': txt2[-200:]}
            ok = (rc2 == 0 and d.get('status') == 'passed' and d.get('compared_bytes') == TOTAL
                  and not d.get('differences') and d.get('draws') == c['draws'] and draws_ok)
            rec(f"{c['id']}-oracle+bytes", ok,
                {k: d.get(k) for k in ('status', 'draws', 'instructions', 'compared_bytes', 'differences', 'parse_error')}
                | {'expected_instructions': c.get('instructions'), 'rust_draws': None if res is None else res.get('rng_draws')},
                kind='positive')

    for c in NEGATIVE:
        fixture = P / c['fixture']
        outdir = out_root / c['id']
        rc, _ = captured(fixture, c['frame'], c['slot'], outdir, expect='blocked')
        res = json.loads((outdir / 'result.json').read_text()) if (outdir / 'result.json').exists() else None
        rec(f"{c['id']}-rejection", rc == 0 and res is not None and res.get('status') == 'blocked'
            and c['expect_error'] in (res.get('error') or ''), {'rc': rc, 'rust': res}, kind='negative')
        got, bad = geometry(outdir)
        pd = diffs(outdir, fixture)
        rec(f"{c['id']}-transactional-preservation", (not bad) and (not pd),
            {'lengths': got, 'problems': bad, 'buffer_diffs_vs_input': pd}, kind='negative')

    if not a.no_focused:
        rc, txt = sh([PY, '-B', FOCUSED], 'focused195.log')
        rec('focused-batch-9of9', rc == 0 and '"passed": 9' in txt,
            {'rc': rc, 'tail': txt.strip().splitlines()[-4:]})
    if not a.no_suite:
        rc, txt = sh([PY, '-B', 'scripts/cargo.py', 'test', '-p', 'genewars-runtime', '--offline', '--locked'], 'suite.log')
        lines = [l for l in txt.splitlines() if l.startswith('test result:')]
        rec('runtime-suite', rc == 0 and any('260 passed' in l and '0 failed' in l for l in lines), {'results': lines})

    if a.campaign:
        exe = P / 'private/build/rust-target/release/genewars-game.exe'
        rc, _ = sh([PY, '-B', 'scripts/cargo.py', 'build', '--release', '-p', 'genewars-desktop',
                    '--features', 'genewars-runtime/callback-captures', '--offline', '--locked'], 'release.log')
        rec('release-build', rc == 0, {'rc': rc, 'exe': str(exe)})
        camp = run / 'campaign'
        camp.mkdir()
        e = {'GENEWARS_MULE_CALLBACK_CAMPAIGN_PROGRESS': '1', 'GENEWARS_MULE_CALLBACK_FIXTURE': str(camp / 'capture')}
        (camp / 'capture').mkdir()
        rc, _ = sh([str(exe), str(P / 'private/integration/game-runtime'), '--reference-startup', 'fresh',
                    '--headless', str(P / a.campaign), '--report', str(camp / 'report.json'),
                    '--bmp', str(camp / 'frontier.bmp')], 'campaign.log', e)
        rep = json.loads((camp / 'report.json').read_text()) if (camp / 'report.json').exists() else {}
        m = rep.get('mission', {})
        rec('campaign-run', rc == 0 or bool(m),
            {'rc': rc, 'steps': m.get('steps'), 'blocked': m.get('blocked'),
             'world_sha256': m.get('world_sha256'), 'rgb_sha256': m.get('rgb_sha256'),
             'campaign_progress': m.get('campaign_progress')})

    if a.head:
        head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=P.parent, capture_output=True, text=True).stdout.strip()
        rec('head-unchanged', head.startswith(a.head), {'head': head, 'expected': a.head})
    staged = subprocess.run(['git', 'diff', '--cached', '--name-only'], cwd=P.parent,
                            capture_output=True, text=True).stdout.strip()
    rec('staged-empty', staged == '', {'staged': staged.splitlines()})

    pos = [r for r in results if r['kind'] == 'positive']
    neg = [r for r in results if r['kind'] == 'negative']
    summary = {
        'tag': a.tag, 'run': str(run.relative_to(P)), 'head': subprocess.run(
            ['git', 'rev-parse', 'HEAD'], cwd=P.parent, capture_output=True, text=True).stdout.strip(),
        'positive': {'passed': sum(1 for r in pos if r['ok']), 'total': len(pos)},
        'negative': {'passed': sum(1 for r in neg if r['ok']), 'total': len(neg)},
        'checks': {'passed': sum(1 for r in results if r['ok']), 'total': len(results)},
        'all_ok': all(r['ok'] for r in results),
        'steps': results,
    }
    (run / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n', encoding='utf-8')
    print('===== FAMILY SUMMARY =====')
    print(json.dumps({k: summary[k] for k in ('positive', 'negative', 'checks', 'all_ok')}))


if __name__ == '__main__':
    main()

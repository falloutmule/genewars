# Attempt21 - reference-model domain change: owned state-20 EXPIRED-timer arm in `owned-mule-captured-width-review.py`

Untracked working note (2026-10-17, worker attempt21, model GLM-5.3 Flash). The interpreter
diff is unstaged; see `C:/Users/fallo/Documents/ChatGPT/genewars-routing-probe/attempt21-interpreter-expired.patch`.
The port (`rust/`) was NOT modified and still refuses this record at 5946.

## What was admitted

A third, OR'd arm now admits state 20 exactly when

    0x5e==20 && 0x60==0x18 && 0x61==0x80 && word(0x6c)==0 (EXPIRED) && word(0x6a)>0
    && 0x78==7 && 0x7a==4 && 0x4f==0 && 0x5d==0 && 0x52==0 && 0x57==1 && 0x5a==1
    && [0x58:0x5b]==[0x55:0x58]

on top of the unchanged pre-existing gates (target identity at lines 41-43, live neutral
enemy checks at lines 46-47). The two prior arms (state-1/21 hostile pursuit; state-20
fresh timer with word(0x6c)>0) are byte-identical to before. This is the measured shape of
the live step-5946 refused record (`private/review/attempt20-live-5946-blocked-owned`,
slot 1, enemy slot 5): triples 0x50..0x52 = 05 00 00 do NOT match 0x55..0x57 = 05 00 01,
and this arm, unlike the hostile arm, does not require them.

## Evidence (all addresses in the literal listings pinned by `vm.files`)

1. `0005ef3a` MOVSX EAX,word[0x6c]; `0005ef3e` TEST EAX,EAX; `0005ef40` JLE 0x00065384
   (`player-creation/instructions-0005ed20.txt`). word(0x6c)=0 => signed <=0 => TAKEN.
   (The F5/F6 fresh arm only documented the not-taken side.)
2. `00065384`-`00065397`: AL=byte[0x5e]; CMP 0x18; JA 0x5ef46 NOT taken (state 0x14<=0x18);
   `00065392` AND EAX,0xff; `00065397` JMP dword ptr CS:[EAX*4+0x5eca0]. Jump table read
   from pinned `private/ghidra/validated/memory-.object1.bin` (base 0x10000): entry 0x14
   (state 20) at file offset 0x5ecf0-0x10000 = 0x4dcf0 contains 0x0005f4c4.
3. `0005f4c4`-`0005f4d3`: word[0x6a]=58 > 0 => `0005f4d3` JG 0x5f501 TAKEN (the
   word(0x6a)<=0 completion side at 5f4d5..5f4fc is NOT this record's path and stays outside
   the proven domain).
4. `0005f501`-`0005f521`: byte[0x78]=7 nonzero (JZ 5f5a9 not taken); word[0x49]=0x10,
   AND AL,0x3 = 0 != 3 (JNZ 5f5a9 not taken).
5. `0005f532`-`0005f569`: byte[0x78]==7 => NOT JNZ 5f56b; word[0x72], word[0x74] staged,
   byte[0x18] (owner 0) IMUL 0x70d + 0x118d74; `0005f564` CALL 0x81c60 (pinned
   `mission-array-refs/instructions-00081c60.txt`); JMP 5f5a9.
6. `0005f5a9`-`0005f5bc`: BH=byte[0x78]=7 nonzero => `0005f5bc` JNZ 0x6029a TAKEN.
7. `0006029a`-`000602b0`: `000602a2` MOV byte[0x5e],0x8 (state 20 -> 8);
   `000602a8` CALL dword ptr [EBX+0x8c] -> 0x5b100 (pinned
   `initial-creature/instructions-0005b100.txt`, pure predicate, no record write);
   `000602ae` TEST AL,AL -> AL=0 => `000602b0` JZ 0x60291 TAKEN.
8. `00060291` MOV word[0x6c],CX (CX=0 from `000602a6` XOR ECX,ECX; 0x6c stays 0);
   `00060295` JMP 0x5ef46.
9. `0005ef46`-`0005ef54`: state now 8: CMP AL,0x8 (JC 6a22b not taken), `0005ef54`
   JBE 0x6539f TAKEN. `0006539f`-`000653c3`: byte[0x18]=0 != 4 (JNZ 6a290 not taken);
   byte[0x5f]=0 (JNZ 65735 not taken); `000653c3` JMP 0x6a290 -> the SAME generic
   continuation tail 6a290..6a53e as the fresh arm.
10. Tail: `0006a495`-`0006a49b` TEST word[0x6c] -> 0 => JLE taken, so `0006a4a0`
    DEC word[0x6c] is SKIPPED; `0006a4a7` CALL 0x5e640 (animation selection via 5cc2a/5e817,
    animation byte 0x19 0x10 -> 0x05); `0006a4d5` CALL 0x7ac00 render request;
    `0006a4e2` CALL 0x83140 (queued, returns 0); `0006a53e` RET.

## Draw proof

Zero RNG draws. The run's `draws` (counter 0x118782 delta) is 0 and `rng.bin` is
byte-identical to the input. The 11872c/11878a ring manipulations on the executed path
(81c60 body) are journal-ring writes, not 0x118782 draw increments; no draw site executes.

## Word(0x66) delta of -2, both sites cited

- `0005e0e1`-`0005e0e9` (body of the `0005ef22` CALL 0x5e0b0, executed before the timer
  branch): MOV DX,word[0x66]; DEC EDX; MOV word[0x66],DX (8651 -> 8650).
- `0006a2c4`/`0006a2c9` in the tail: DEC ECX; MOV word[0x66],CX (8650 -> 8649).
- `0006a4a0` skipped (0x6c=0). Measured output word(0x66)=8649. Both sites are in the trace.

## Oracle result (live 5946 record, three runs, all byte-identical)

status `source_prepared`, 21226 instructions, 0 draws; output dirs
`private/review/owned-mule-captured-width/2a54ba4cd4344ba98d35d210c2d3d9b1`,
`.../7598c93a21e34436bdf80f77bc56ba93`, plus the post-restore rerun. Only `creatures.bin`
changes (21 bytes, all inside slot 1's 124-byte record): 0x07(a6->a5), 0x09(e3->e9),
0x11(3f->5f), 0x17(39->3a), 0x19(10->05), 0x1b(0->1), 0x1d(0->2), 0x1e(0->2), 0x27(3f->5f),
0x2d(d3->d4), 0x2f(ac->97), 0x31(f9->19), 0x32(0->1), 0x39(22->23), 0x3b(55->3c),
0x43(40->90), 0x44(70->cc), 0x4b(23->24), 0x57(1->0), 0x5e(0x14->0x08), 0x66(-2).
State 0x5e becomes 8; word(0x6c) stays 0. Enemy slot 5 and all seven other buffers
byte-identical.

## Regressions (all unchanged)

- authentic-krell-capture4802 frame 4801 slot 1: source_prepared, 2224 instr, 0 draws
- authentic-krell-194 frame-5892 fixture (run at frame 5893, matching the attempt18
  baseline parity): source_prepared, 1714 instr, 0 draws
- authentic-krell-195 frame 5939 slot 1: source_prepared, 1503 instr, 0 draws
- attempt13-fx-5940 frame 5940 slot 1: source_prepared, 11594 instr, 5 draws
- attempt17-live-5940-blocked-owned frame 5940 slot 1: source_prepared, 2307 instr, 0 draws

## What remains rejected

- State 20 with word(0x6c) < 0 (signed negative), or word(0x6c)==0 with word(0x6a)<=0
  (the 5f4d5/5f4fc completion side is unexecuted and unproven here).
- State 20 with any other 0x60/0x61/0x78/0x7a/0x4f/0x5d/0x52/0x57/0x5a combination or
  unmatched [0x58:0x5b] triple.
- Any state 1/21 not matching the unchanged hostile predicate; enemy shapes failing
  lines 46-47.
- The 6029a-block continuation when the virtual call returns AL!=0 (602b2..6032d, which
  writes 0x6c=0x7fff and could call 6c190) is NOT admitted by this shape change alone; for
  this record AL=0 (transactionally visible in the trace). Admission of that side would
  need its own measured record.

## Status

Observed for the executed literal path (trace + listing addresses + jump-table bytes above).
Not provisional: every executed address is pinned original listing or pinned memory-object
data. Remaining uncertainty: 0x1d/0x1e/0x32/0x39/0x43/0x44/0x4b deltas come from inside the
pinned helper bodies (81c60/5b100/5e640 chain) whose full internal writes were not
line-cited; they are measured outputs of pinned original code, not invented values. The
port-side implementation of this arm is a separate bounded assignment.

# Attempt18 - reference-model domain change: owned state-20 fresh-timer arm in `owned-mule-captured-width-review.py`

Untracked working note (2026-10-17, worker attempt18). The interpreter diff is unstaged;
see `C:/Users/fallo/Documents/ChatGPT/genewars-routing-probe/attempt18-interpreter-domain-scoped.patch`.

## What was admitted

`research/variants/owned-mule-captured-width-review.py` line-44 prefilter previously admitted
states 1/20/21 only in the hostile attack-entry shape (`0x60` in {8,0x88}, `0x61==0`,
`0x52==1`, both pointer triples equal). A second, OR'd arm now admits state 20 exactly when

    0x5e==20 && 0x60==0x18 && 0x61==0x80 && word(0x6c)>0 && 0x78==7 && 0x7a==4
    && 0x4f==0 && 0x5d==0 && 0x52==0 && 0x57==1 && 0x5a==1 && [0x58:0x5b]==[0x55:0x58]

(target-identity validation at lines 41-43 and the live neutral state-21 enemy checks are kept
unchanged and still gate this arm). This is the measured shape of the live step-5940 refused
record (`private/review/attempt17-live-5940-blocked-owned`, slot 1, enemy slot 5).

## Evidence (all addresses in the literal listings already pinned by `vm.files`)

The prefilter is a shape whitelist, not source semantics; the executed path is literal
original code and every executed address is inside `player-creation/instructions-0005ed20.txt`
(5ED20..6A53E), `activation-rng-refs/instructions-0005e0b0.txt`,
`player-creation/instructions-0005e640.txt`, `mission-array-refs/instructions-0005cc60.txt`,
`landing-contact/instructions-0007ac00.txt`, `initial-creature/instructions-0006b150.txt`
+ `23800/23820`. Executed order for the 5940 record (trace of output dir
`private/review/owned-mule-captured-width/2e412d033c0e43b9898e309d022b321e`):

1. `0005ed37` TEST byte[0x61],1; `0005ed3b` JZ 0x5ee1c - 0x61=0x80 has bit0 clear, light/head
   branch taken.
2. `0005ee25`-`0005ee32` parity gate: (byte[0x11877e] + record[0xf]) & 1, JNZ 0x5eed5.
   Frame 5940 low byte even + slot 1 => odd => FULL path (the 5ee38 light tail is NOT taken).
3. `0005eed5`-`0005ef1f` terrain cell pointers from word(7)/word(9) (SAR 8, AND 0x7f,
   base 0x11c2be); `0005ef22` CALL 0x5e0b0 (motion body, activation-rng-refs listing).
4. `0005ef27` TEST AL / `0005ef29` JZ 0x6a297 - 5e0b0 returned AL != 0, not taken
   (the bare-tail entry at 6a297 is not this record's entry).
5. `0005ef32` CALL 0x5eb30; `0005ef3a`-`0005ef40` MOVSX word[0x6c], TEST, JLE 0x65384 -
   word(0x6c)=32767>0, so the expired-timer completion path (the interpreter's other
   state-20 arm, 5E0B0/5CC60 -> state 8) is NOT taken.
6. `0005ef49`-`0005ef68` state dispatch: AL=0x14 (20); CMP 8 (JC not taken), CMP 0x15
   (JC 0x6a20d taken); `0006a20d` CMP 0xe (JBE no), `0006a217` CMP 0x11 (JZ no),
   `0006a21f` JMP 0x6a290. State 20 enters the SAME generic continuation tail at 0x6a290
   that the previously admitted state-20 hostile shape reaches through this same dispatch
   (state 20 can only enter the tail via 6a21f; no source branch on 0x60/0x52 selects a
   different entry for state 20 in 5ef46..6a21f).
7. Tail `0006a290`..`0006a53e`, all executed: `0006a293` AND byte[0x4a],0xfc (0x4a=0,
   no change); `0006a2a0` CALL [0x23+0x68] (virtual 0x1ec50); `0006a2ae` CMP 0x5e,0xf (no);
   `0006a2c4` DEC word[0x66] (8657->8655 across the two DEC sites); 6b150/23800/23820
   animation-progress chain x4 (`0006a2ed`, `0006a30e`, `0006a319`, ...); 0x77/0x62/0x64
   writes; `0006a4a0` DEC word[0x6c] (32767->32766, matching the sibling neutral slot-5
   timer value 32766 in the same fixture); `0006a4a7`..`0006a53e` epilogue via
   `0005e640` (which reaches `0005e817` animation selection through `0005cc2a`) and
   `0007ac00` render request.
   The tail reads none of 0x52/0x57/0x5a/0x78/0x50-triple/0x4f/0x5d in the executed range;
   those prefilter fields are kept at the measured values to stay fail-closed, not because
   the tail branches on them.

No RNG draw is taken anywhere on this path (0 draws; draw sites such as 5ef8f/5f008 belong
to other state blocks).

## Oracle result (live 5940 record)

status `source_prepared`, 2307 instructions, 0 draws; only `creatures.bin` changes (20 bytes,
all inside slot 1's 124-byte record): 0x07,0x09,0x11,0x17,0x1b(1->2),0x27,0x2d,0x2f,0x31,
0x3b,0x3d,0x3e,0x41,0x42,0x43,0x44,0x66(-2),0x68(11->8),0x6a,0x6c(32767->32766).
State 0x5e stays 20, animation 0x19 stays 16, 0x57 stays 1. Enemy slot 5 and all seven
other buffers byte-identical. Output dir
`private/review/owned-mule-captured-width/2e412d033c0e43b9898e309d022b321e`.

## Regressions (unchanged counts)

- authentic-krell-capture4802 frame 4801 slot 1: source_prepared, 2224 instr, 0 draws
- authentic-krell-194 frame 5893 slot 1: source_prepared, 1714 instr, 0 draws
- authentic-krell-195 frame 5939 slot 1: source_prepared, 1503 instr, 0 draws
- attempt13-fx-5940 frame 5940 slot 1: source_prepared, 11594 instr, 5 draws
- second deterministic rerun of the 5940 record produced byte-identical buffers.

## What remains rejected

Everything outside the two arms: state 20 with word(0x6c)<=0 but not the proven
expired-timer completion shape; any state 1/21 not matching the hostile predicate;
any 0x60/0x61/0x52/0x7a value outside the measured shape (0x7a must be 4 because the tail
indexes the 0x1045fc+0x4a table with it); enemy shapes failing lines 46-47. The port
(`rust/`) was NOT modified and still refuses this record at 5940
(`R10-owned-state20-fresh-timer-5940-rejection` stays a negative PASS with full
transactional preservation).

## Status

Observed for the executed literal path (trace + listing addresses above). The extension is
NOT provisional on reachability: the dispatch and tail are literal original code. Remaining
uncertainty: the two DEC-word-0x66 sites (`0006a2c4` DEC ECX then `0006a4a0` DEC word[0x6c])
account for the measured 0x66 delta of -2; each executed address is cited, but the port-side
implementation of this arm is a separate bounded assignment.

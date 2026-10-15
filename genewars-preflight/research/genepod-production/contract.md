# Completed Genepod: next-action contract

Reference: verified CD **GENEWARS QA BETA V5**. This is source investigation, not mission acceptance or a Rust implementation claim.

## Proven idle callback

`genepod_idle_source.py` interprets full `451F0` with exact BGENE animations and source-created completed building fixtures. Eight whole-world runs (four owners; stack fills0/255), each409,793 bytes, agree across stack fills. The first callback changes only building byte`19` from0 to1; RNG, map, player records and all other entity records remain unchanged. The render queue at`83140` is an explicit presentation port. No compiled original code executes.

Idle is **not a general no-op**. Common power accounting subtracts mapped`104D10[0]` (zero for this build) plus extra demand byte`3F`; the shared tail caps timer`3D` at200 and reduces positive extra demand. Power exhaustion can clear flag4 and set the player's warning timer. Selector0 with signed work dword`36<=0` sets animationcounter`1C` according to frame`1B!=0`; the shared tail selects grade+1 animation and runs literal`7AC00`. Disabled states reset work. The initial fixture happens to need only the animation-selector write.

## Source-backed request, not yet implemented

The source command dispatcher reads byte`payload+A`; mapped jump table`1C094[12]` is`1C5C2`, calling`814A0`. Payload words are species at0, building slot at2, start/cancel at4. An independent AI caller`36920` builds opcode12 and calls the same handler. The human UI producer/hit target remains untraced; do not invent an original shortcut.

Starting reads cost`10461C + species*45`, sets work dword`36=cost>>grade`, stores its low word at`34`, and species at`33`. It pays the remainder`cost & ((1<<grade)-1)` from caller primary then secondary funds. For a human caller it writes caller player ID into building byte`3A`. Mule species4 has mapped base cost75: grade0 starts75 work ticks with zero remainder debit. These are literal listing/table observations, not yet an independent request test.

For positive work, `451F0` sets demand byte`3F=3`; powered flag4 or frame divisible by6 permits spending`1<<grade` and decrementing work. Thus grade0 Mule nominal cost is75 total, paid incrementally, not an up-front75. Effective payer depends on original player type and byte`3A`. Resource shortage stalls; it is not a gameplay loss.

On the final tick, the source reads species research from player`61A+species*4`, shifts11, caps100 for grade<2, and performs a tagged RNG quality/failure decision. Success calls creature factory`5A210`, then initializes launch motion with additional RNG and raises physical height by`F00`; capacity failure and mutation/failure effects are separate branches. Audio, research changes, factory allocation, launch physics, and RNG ordering all need literal fixtures before admission. Do not turn this into a guaranteed Mule spawn.

## Minimal next work

Implement and compare bounded completed-idle selector0 first. Keep positive work explicitly unsupported until opcode12, eligible productive tick, final success/failure and factory-capacity behavior are proved. Capture the actual completed Krell Genepod when ordinary gameplay reaches it; current proof inputs are construction fixtures. Then recover the player UI producer and expose the same source command through native dispatch. No broad genetics research is required yet.

Reproduce idle evidence: `C:/Python314/python.exe scripts/ghidra/genepod_idle_source.py`. Pins and private output paths are in `idle-source.json` and `contract.json`.

## Bounded idle implementation follow-up

`genepod_idle.rs` now matches96 varied full building/player source cases (179,712 bytes). Eight construction-fixture callbacks additionally match complete409,793-byte world afterimages with unchanged RNG. These compose the pure callback result into the preserved world; a separate focused owned-world test verifies actual ordered scheduler dispatch and whole-world rollback when a later Genepod has positive work. No positive production, disabled/destruction branch, or authentic mission acceptance is claimed. Numeric idle exposes its render-queue request; it makes no audio request. Source/build hashes are in the comparison and rollback receipts.

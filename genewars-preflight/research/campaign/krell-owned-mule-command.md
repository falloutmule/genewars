# Krell owned Mule: minimum command boundary

Pinned QA BETA V5 source inspection establishes an ordinary **single-creature opcode11 hex** movement path. It does not yet establish the owned live-callback/arrival implementation. The Genepod creates an owned Mule in state8; do not change owner to4 to reuse the current neutral-only gates.

## Selection and input

MAINPANL kind1 world selection calls278D0 (root panel offset1C4), after the same hover visibility/event checks described in the building-selection contract. 278D0 installs its module through24770, records the creature pointer at panel+4, configures creature controls, and refreshes selection presentation. General world hit priority remains83140/83930; do not select by nearest center.

The selected pointer118740 must be nonnull and owner18 equal the selected player's owner byte1. With no active creature-selection group, MAINPANL2CD44..2CEAC dispatches kind1 to2CE26. It writes packet opcode11 at2CE4E, then words **[creature slot, projected world X, projected world Y,0]**. Projection is current16E0B4/16E0B6, not invented zone coordinates. Input5's parent branch excludes19FF70 high0A and118968 bit2; event10 (right release) takes the alternate panel/deselect route. Therefore the earlier proposed right-click label is **not established as original input**. Bind the exact action5 event/118770 flow before claiming authentic controls. The current development right-click convention must not silently define the original Mule controls.

1C120 routes opcode11 to81C60. This rejects state5E=12hex or0Fhex, dead virtual68, or flags61 bit1. For eligible creatures it requests feedback and calls5DF50 with target. The handler does not itself repeat an owner check; the UI producer checks ownership. A safe shared command API should retain explicit caller ownership validation.

## Movement installation

5DF50 retains state2 only when flags60 bit2 is set, adding bit1. Otherwise it calls5CC60 cleanup and writes state5E=0. It copies target to72 and6E, sets60 bit8 and61 bit80, and clears5F. For owner!=4 it additionally sets78=7,60 bit10, and clears61 bit40. These owned fields are meaningful and must not be dropped.

On retained state2 it obtains virtual80 requested speed, writes3F and6C=7FFF, sets60 bit80, installs destinationB/D, movement flags49|14 then&EFFC, clears4F/52, and resets54=20. The species-table type3 branch also alters vertical/presentation state; Mule's table value must be pinned before admitting that branch.

5CC60 has real task cleanup:5C920 precedes releasing plant or partner references for the applicable old state. It clears33/35,15 bit20, masks60word withD71A, conditionally clears5D when oldbit2 absent, clears57 and6C. Newly ejected state8 avoids the listed partner/plant branches, but5C920 must still be composed; do not replace cleanup with zeros based only on this summary.

## Actual implementation gates and smallest next work

Current `creature_world.rs` live Mule planners explicitly require owner18=4 in the motion/idle/AI paths (including lines around384,623,695,711). Owned command78=7 and flags60 bit10 lead to additional behavior not covered by the neutral proof. Merely loosening owner validation is insufficient.

Next bounded implementation: literal81C60→5DF50→5CC60/5C920 on the actual owned Genepod newborn after ejection; preserve complete record/RNG/world deltas. Then compare one full5ED20 callback on that result, finding the first owned-only branch and porting it. Follow ordinary motion until destination/zone accounting triggers. This is the minimum needed to convert a production success into a player-directed zone delivery; broader group commands and genetics need not block it.

No arrival, ejection-complete, zone-credit, native-selection, or mission acceptance is claimed here. Existing source contracts for factory/ejection and zone evaluation remain separate evidence.

## First literal composition

`scripts/ghidra/krell_owned_mule_command_probe.py` now reproduces the existing null-parent owner0/quality35 factory fixture with seed5428 and zero native prior, then executes81C60→5DF50→5CC60→5C920 for target[1536,1536]. Audio41910 and selected-panel refresh are explicitly bounded presentation ports. The command consumes zero additional RNG draws: [52932,7,22] remains unchanged. It changes state8→0, flags60word to8018, command78 to7, and both target pairs to the requested point.

One complete literal5ED20 callback then executes9857 instructions and three RNG draws, ending [13716,10,25], state8, timer6C=19, energy reduced2, command78=7 and flags8018 retained. Position remains[1024,1024]; this is not arrival. Complete records and callback memory/trace are saved privately and hashed in `krell-owned-mule-command-probe.json`. These are source-prepared afterimages, **not a Rust comparison**. The fixture still excludes Genepod caller ejection overwrites and uses synthetic flat terrain; use the real terminal-adapter newborn next before asserting the campaign path.

## Pure command implementation

`mule_owned_command::plan_move` implements the state0/8, no-attachment/no-peer cleanup branch. Caller ownership is checked; actual virtual68 death and selected-panel predicate are explicit context inputs. Dead/state0F/state12/flag61bit1 commands return an unchanged typed ignored outcome; attachment, peer cleanup, state2 and other unproved tasks reject. The caller remains responsible for actual pool identity/resource validation and transactional commit. No neutral callback guard was loosened.

`krell_owned_mule_command_review.py` passes full124-byte comparisons for two exact source-derived inputs: original factory record and the first complete callback result, with two destination commands. Eight unsupported/ownership guards and six unchanged ineligible cases also pass. Receipt `krell-owned-mule-command-review.json` pins the module and source-probe receipt. Real Genepod ejection and the live owned callback are outside this pure command proof.

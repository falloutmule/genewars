# Original world control5 and Mule selection

Read-only static analysis of the pinned CD QA BETA V5 source. This records the original mouse path; it does not claim a new native acceptance run or complete mixed-family picking integration.

Startup8DCB0 passes control list106894 to constructor2B7E0.2B7E1 preserves that pointer inECX;2BAA0 writes root+4. MAINPANL entry2BD70..2BD7E installs that list in19FF64 and binds its resources throughA8320.

The mapped40-byte list has container106894 flags2, then world rectangle1068BC: flags00041A08, geometry[0,0,640,480], eventID5, callback pointer0. The following record1068E4 is the sentinel. This is a world input rectangle, not a sprite-center or entity hit test.

## Hardware edge to action5

The separately recovered mode2 mouse packet has byte8 bit0 left press, bit1 left release, bit2 left held; bit6 right press, bit7 right release; byte9 bit0 right held.70E40 produces these bytes at118771 with pointer118970. The original callback registration/physical edge proof is recorded in research/ghidra/input-evidence.json.

A87D0 reads packet byte8 bit0 atA881A..A8825 and bit6 atA8829..A8833. An uncaptured world press hits the rectangle throughA86D0 and ANDs press masks8/10 against control flags atA8D04..A8D28. World control second flags byte1A takes capture pathA8D96..A8E20: capture pointer19FF88, countdown1, mask+26, and event high08. A left press therefore creates event0808; MAINPANL2C965 suppresses this through19FF71&0A. The press itself does not issue the movement packet.

While captured, A8871..A8894 maps packet left release bit1 to event08 and right release bit7 to event10, then ANDs the captured control mask+26. A88EA tests world flags10; A88FA checks the release hit, A8904..A890E selects controlID5 on success. A8A20..A8A54 clears capture/mask. Thus a captured **left release inside the world rectangle** returns action5 with event08.2D0D5..2D0D9 routes action5 to2C965. A captured right release maps event10;2C97F takes the alternate panel/deselect route. This establishes left-release selection/movement under these explicit ordinary capture gates, not a right-click movement convention.

A87D0 returns the control ID atA8CD7. MAINPANL2C38A..2C38F stores it in169A2; pause/disabled entry can replace it withFFFF at2C395..2C3A2. Child handlers2C501..2C533 may consume it before2D085 dispatch.

## Selection before movement

2C965 excludes event high0A and pause118968 bit2. On the no-event10 branch,2C9D0..2CA16 derives CL: with no group, an already-selected hovered pointer makes CL1 and suppresses further action. A live group is a separate opcode12 domain; it is not admitted by the single-creature producer.

For a different nonnull published hovered pointer16E0C0, visibility byte17 nonzero, simulation118770 bit4 set, and pause clear,2CA1C..2CA65 dispatches entity kind. The mapped kind1 jump target at2C2D4 is2CBE8. The saved decompile shows kind1 resetting16E0C7 and manager31, then calling278D0 on root+1C4 with CL1;278D0 preservesECX, so this selection consumes CL. Its24770 call stores the selected creature pointer at118740 (247B1), installs the panel and updates presentation. Selection itself has no owner check in this kind1 case; command issuance later requires selected owner18 equal selected player's owner byte1 (2CD44..2CD7B and2CE26..2CE30).

On a subsequent ordinary left release over ground, no hover selection consumes CL. With group526/52F zero, producer2CE4E..2CE7E writes opcode11, words[creatureSlot,projectedX,projectedY,0], preserving word+8, and manager58C=0,58D=1,58E=1,599=projectedPoint. Projection must be actual16E0B4/16E0B6. The producer does not run terrain projection or move the world itself.

The actual picked entity is produced by83140 and published by83930, not by nearest center. Existing research/campaign/krell-building-selection-contract.md records shared visibility/viewport gates, ground-Y ordering (later entry wins equalY), virtual20 opaque component picking, and modifier-key exclusions. The equivalent Mule resource/frame hit and complete mixed-family order still require native integration evidence.

## Limits

- Static source/data composition; no new fullA87D0 literal test or hardware/native replay in this analysis.
- The instruction export30100 omits the kind1 switch island2CBE8; that target is established by mapped jump-table bytes and the saved decompile plus independently saved278D0/24770 callees. Do not claim that omitted island has a new literal proof.
- Rectangle capture, release hit, enabled flags, current module/list, and child-handler consumption must remain explicit. Keyboard action synthesis and unusual multi-button/capture replacement cases are outside this bounded route.
- Group selection/movement, modifier actions, resource loading, projection and world picking are separate boundaries. No mission completion is implied.

## Source pins

- `private/ghidra/terrain-constructor/instructions-0008db90.txt` SHA256 `b68c2cedc7bf7904a58293c448423f5408f65061548769d78245f5961f0a7c8c`
- `private/ghidra/entity-shadow-panel/instructions-0002b7e0.txt` SHA256 `aa968be1655cb0a96d153c2d8fca757cbef5848dafb53968acb3b0c7b57b37bd`
- `private/ghidra/panel-activation-refs/range-2bac0.txt` SHA256 `2d1d7028f91ef808ade385c7b89684925582d5c577dcbc077a2501480d7508e9`
- `private/ghidra/ui-input/instructions-000a87d0.txt` SHA256 `1fe8f247f251c7b1a5a333b71d830d93ddca2feb1f6d557448894131510afd58`
- `private/ghidra/packet-fields/instructions-00070e40.txt` SHA256 `d9e28b4cd2565e951d112522fde407f6186667f3e7f3840837f0619304ddc83d`
- `private/ghidra/entity-order/instructions-00030100.txt` SHA256 `174fa08085b159b3277ccbc2108913459f5d5f59d4b222d81ac3a9aa067f6c15`
- `private/ghidra/entity-order/decompile-00030100.c` SHA256 `377c1ca5665fe278aa14be2495568ebeb0e98bae56426759d0c3c9ae74edee97`
- `private/ghidra/display-geometry/instructions-000278d0.txt` SHA256 `a9fa5f62eedc266105284536037cb64a831a234e000ab617840f4bb8839fb29a`
- `private/ghidra/display-geometry/instructions-00024770.txt` SHA256 `dd59f4f3b8fc7a3cffbd69473a05c96feb80954bec0752792d5c1c673ae9bbcb`
- `private/ghidra/validated/memory-.object1.bin` SHA256 `92cb549a81ab565d62bbd233cf69b2470fad8186a52df31de6a5cd0faab280fe`
- `private/ghidra/validated/memory-.object3.bin` SHA256 `aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87`

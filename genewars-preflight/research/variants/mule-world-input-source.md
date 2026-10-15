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


## Bounded owned-Mule picking integration findings

Mapped Mule vtable111FEC+20 is5A140, available in private/ghidra/initial-creature/instructions-0005a110.txt. Frame virtual1C is1E840 (entity-plant listing). Unlike building1F3C0/98C40,5A140 calls **scaled rectangle** hit95300. It calculates clamp((unsignedbyte76*unsignedbyte77)/100,150,256), always subtracts relative physical height (virtual4=1F260)>>6 from the supplied groundY, sign-extends input coordinates, obtains the source-clamped animation19/frame1B through1E840, and accepts95300's component result other than-1. Rendering59470's shadow-mode and scale fast paths are not interchangeable with hit5A140:76=100/77=255 produces hitfactor255, while the current render fast path chooses256. Do not infer hit coordinates from a rendered bounding rectangle or substitute building_pick::hit_animation.

Existing host surfaces:

- runtime_resources::CreatureAnimationDatabase::decode_frame_packed4 provides selected frame metadata and components with local positions, flags, sprite pixels and explicit opacity. It currently rejects an out-of-range frame; picking must first apply1E840's clamp to the actual animation extent.
- creature_scene::draw_supported_mule_record admits the bounded owned state0/8 visual domain. Its successful render proves resource availability, not pick admission. actor_scene already uses entity_view_position(camera,128) into a512-wide slice beginning at framebuffer offset128; picking must consistently use absolute screen coordinates including that margin.
- runtime_scene::Game::packet routes mission events only to genepod_packet. Game has GenepodSelection, but no selected-creature/world-control5 capture state. SharedMission.selected is an actor slot; it cannot be reused as a Mule index.
- ActorWorld::dispatch_owned_mule_action5 already composes the proved producer and owned command transaction. It accepts resolved selected slot, typed packet, manager[59D], ProducerInput and explicit selected-panel-refresh predicate. A host adapter should stage world, selection, manager/packet and rendered frame together, and commit only after success.

The faithful bounded route is source-control5 left capture/release, exact current Mule scaled-component rectangle hit, selection release consumed asCL1 through278D0/24770, then a later ground release reaching action5 with actual projected16E0B4/B6. Picking must respect visibility17, viewport family mask/flag1C, original83140 eligibility and modifier exclusions, ground projectedY priority, and83930 publication. A no-overlap route must account for every competing family, not merely other Mules/buildings. Current actor_scene explicitly uses caller-supplied family order; it does not establish original mixed queue order. Existing Genepod selection opens on press and excludes only overlapping buildings, so it is not proof of this original creature selection path. Selected creature panel presentation/refresh and original control-state priors remain separate obligations.

### Missing scaled hit and local export command

Bounded inventory found no standalone95300 saved listing. actor-draw/instructions-00095390.txt begins at95390 and is a different function. The existing local exporter scripts are scripts/ghidra/ExportExistingRange.java (read-only listing; no disassembly) and ExportRange.java (disassembles a known start only if needed). Existing project and program are private/ghidra/projects/GW_CD_Candidate and /GW-CD-candidate.exe, as confirmed by mule-render-export.log. The pinned Java21/Ghidra12.0.1 and existing MemoryPreferencesFactory classes are already installed locally.

The following bounded command is prepared for later use, **not executed in this analysis**. It uses the Java entry/classpath established by support/launch.bat, explicit process-local memory/preferences isolation, project read-only/no-analysis mode, and a new output path. It does not download, import, analyze the whole game or save the project. Run from repository root when no other heavyweight job is active:

```powershell
$taskRepo = (Get-Location).Path
$taskJava = Join-Path $taskRepo 'private/tools/jdk/jdk-21.0.12.1+1/bin/java.exe'
$taskGhidra = Join-Path $taskRepo 'private/tools/ghidra/ghidra_12.0.1_PUBLIC'
$taskClassPath = (Join-Path $taskRepo 'private/ghidra/launcher-classes') + ';' + (Join-Path $taskGhidra 'Ghidra/Framework/Utility/lib/Utility.jar')
& $taskJava '-Xmx768M' '-XX:ParallelGCThreads=2' '-XX:CICompilerCount=2' `
  "-Duser.home=$taskRepo/private/ghidra/user-home" `
  "-Dapplication.settingsdir=$taskRepo/private/ghidra/settings" `
  '-Djava.util.prefs.PreferencesFactory=MemoryPreferencesFactory' `
  '-Djava.system.class.loader=ghidra.GhidraClassLoader' '-cp' $taskClassPath `
  ghidra.Ghidra ghidra.app.util.headless.AnalyzeHeadless `
  "$taskRepo/private/ghidra/projects" 'GW_CD_Candidate' `
  '-process' 'GW-CD-candidate.exe' '-readOnly' '-noanalysis' `
  '-scriptPath' "$taskRepo/scripts/ghidra" `
  '-postScript' 'ExportExistingRange.java' `
  "$taskRepo/private/ghidra/mule-picking/instructions-00095300.txt" '00095300' '0009538f'
```

If the read-only existing listing is empty, that is not a successful export.95300 is an exact call target established by5A140, so the same `-readOnly -noanalysis` invocation may use ExportRange.java at that known start to disassemble transiently without saving project changes. Inspect the result/callees before extending the helper graph; neither a successful command nor a guessed generic scaler proves the hit routine. This command has not been launch-tested here, and actual Ghidra startup/project locking remains an execution-time check.

Additional bounded source/export pins:
- `private/ghidra/initial-creature/instructions-0005a110.txt` SHA256 `5121591cbf165c63dc16fb3de9fb95a8b7772b96162f72a44191d8a8b97d8910`
- `private/ghidra/entity-plant/instructions-0001e840.txt` SHA256 `339ecbf75d0b4fccaed84e606ad9be40a7fe63c271b33f9a6b1b47493ceff54f`
- `private/ghidra/actor-activation/instructions-0001f260.txt` SHA256 `0f0c7ee7a26b87418a794e92c763f2dffb749d896335288aa052cfa40aba9ff2`
- `private/ghidra/actor-draw/instructions-00095390.txt` SHA256 `8ae130718f8bb1c699ccfc7e44a6a83fc9f41c10770fb5b9b4050efc96bfcfa5`
- `scripts/ghidra/ExportExistingRange.java` SHA256 `0d61409c46955ed2df76ad96cf189893a1895c58a1afe1737561bc393fb15659`
- `scripts/ghidra/ExportRange.java` SHA256 `bffea11b1c8354ca61c65dc21f76cb5cc82fbdac4ccc3f7b89e57d7df33cce3d`
- `scripts/ghidra/MemoryPreferencesFactory.java` SHA256 `5b1b81db827f84d6b55037763fe4f79ed00b816f827eb701ab1e4f9ef9e0f84b`
- `private/tools/ghidra/ghidra_12.0.1_PUBLIC/support/launch.bat` SHA256 `4ce74f930f8ef9e37e3f05375f2cda45831139bb5819cae4e734ea676506af60`


### Correction after read-only export148

Root successfully exported95300 and its callee952A0 to private/ghidra/mule-picking-148. The earlier opaque-hit description was incorrect:952A0 reads the component sprite pointer, sprite byte4 width/byte5 height and component flags8 bits1/2, but never reads pixels or an opacity mask. A transparent pixel inside the scaled rectangle is a hit.95300 visits components in reverse order and returns the last matching component index, or-1. Signed component offsets multiply by factor and truncate toward zero on division by256; unsigned dimensions multiply and shift right8. Horizontal/vertical flips transform the local coordinate using extent-coordinate-1 before the same half-open bounds check. Null sprite pointers reject that component. Empty frames are not safe source inputs (the source decrements byte component count before the first read).

The new pure creature_pick helper proves these numeric boundaries and5A140 with explicitly supplied virtual-relative-height and already-resolved frame components. It does not establish resource1E840 resolution, world visibility/mixed queue ordering, mouse selection or native acceptance. Sprite decoding remains useful to obtain dimensions, but opacity must not participate in selection. The earlier command remains a historical prepared command; actual bounded exports now exist at the following pinned paths:
- `private/ghidra/mule-picking-148/instructions-00095300.txt` SHA256 `734798af301743f2e484cfd2ed60873c899917047702de7f30c54d480be9ccc3`
- `private/ghidra/mule-picking-148/instructions-000952a0.txt` SHA256 `96d01e06dc49edfd09ca4d3828afc89dbe31260688f3734eb837c8465c4930de`

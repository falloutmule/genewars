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


## Viewport and input priors from saved source (read-only follow-up)

These are static source findings, not native Mule selection acceptance. Viewport base16E0A8 gives flags+1C=16E0C4, sidebar margin+1D=16E0C5, family mask+1F=16E0C7, published hover+18=16E0C0. The mapped executable's initial viewport bytes are zero; mapped zero storage is not proof of live mission defaults.

### Mission module and view geometry

- `private/ghidra/entity-shadow-panel/instructions-0002b7e0.txt`:2B7E8 installs MAINPANL root vtable11259C on the object initialized at185DFC by8DC9D..8DCBD in `terrain-constructor/instructions-0008db90.txt`. Pinned object3 dword11259C+4=5A6, giving185DFC+5A6=1863A2. `entity-order/instructions-0002bbe0.txt`2BBE8..2BC0B computes that subobject and writes current module116974, establishing the real mission active-module identity required by83140.
- `mission-bootstrap-mode/instructions-0007d090.txt`7D7C7..7D7D5 schedules next module116978=1863A2 while preserving previous module11697C; it does not immediately write current module116974.
- `display-geometry/instructions-00082bd0.txt`82BDA writes margin+1D=128.82BE3..82BF0 writes origin+8=[128,0];82BF5..82C14 writes dimensions+33=[screenWidth-128,screenHeight]. Source82AC9 in `display-geometry/instructions-00082ac0.txt` is the separate full-width margin0 route, with corresponding origin/dimensions.
- MAINPANL entry `entity-order/instructions-0002bbe0.txt`2BE54..2BE74 chooses82AC0 versus82BD0 from the existing margin. Sidebar state is an explicit original input, not a universal fresh-zero assumption. `display-geometry/instructions-00084310.txt`8432C..84363 refreshes origin/dimensions from the existing margin.
- `display-geometry/instructions-0001caa0.txt`1CACE..1CAE0 explicitly installs screen dimensions640x480 on its software-mode branch. The other branch requests mode0D at640x480 through979A0. Preserve the existing admitted640x480 host extent rather than generalizing to arbitrary display sizes.
- Absolute entity hit coordinates use `entity_view_position(...,margin0)`. Rendering into the512-wide viewport slice uses margin128 and destination offset128; adding another128 to margin0 projection is wrong.83140 adds viewport+1D to its local projectedX before virtual20.

### Family mask, publication and unresolved flag

- `actor-resource-refs/instructions-00020880.txt`20953..2095D installs maskFF5B; Genepod panel `actor-panel-font-color/instructions-00021a10.txt`21A6E..21A79 also installsFF5B.
- `display-geometry/instructions-00025150.txt`2520E..2521A first installsFF5B and25249..2525D later installsFF5F. Mule selection `plant-command/instructions-0002cbe8.txt`2CBE8..2CBF0 installsFF5B then calls278D0; `display-geometry/instructions-000278d0.txt`27A78..27A81 installsFF5F. Both masks allow kind1;FF5F additionally allows plant kind2. Do not use one mask for every panel state.
- `display-geometry/instructions-00083140.txt`83688..836A4 checks the kind bit in viewport word1F.834AA..834B3 requires ground projectedY >= current bestY; equalY may replace the winner.834BD..834C9 requires viewport byte1C bit0. Shade/visibility byte17 must be nonzero before this path. Preserve both groundY priority and source hit admission.
- `display-geometry/instructions-00083930.txt`83ABB resets bestY+11D3 to-1;83ACA resets queue count+11BB;83ADC clears queuedirty+11D8;83AD9..83AEA publishes candidate+37 into hover+18 and clears candidate+37. Completed publication establishes these reset values, not the first-frame state before publication.
- `display-geometry/instructions-00086c20.txt`86C2A..86C4C sets suppress-picking counter+11D6=3 when viewport flag1C bit1 is set; otherwise it decrements a positive counter. This affects plants/buildings, not the ordinary kind1 branch.
- Remaining gap: saved bounded listings do not establish the initial writer/value of viewport byte1C bit0. Known direct writers modify drag/toggle bits (MAINPANL2C453 bit10,2D31D bit2; menu26890 higher bits). Mapped initial byte0 is not a live default. Do not manufacture flags1C=1 from successful rendering; bind this prerequisite with initializer/caller proof or a genuine live viewport capture.

### Modifiers and original world control

`keyboard-event-refs/instructions-000760b0.txt`760D8..760E1 extracts input packet byte4 bit2 into19D01A;760E6..760EB copies it into19D00E. An admitted unmodified input packet with bit2 clear proves both modifier bytes0 without guessed global defaults. Hardware mapping of that bit to a named keyboard key is not inferred.83140's83504..8354E applies friendly/allied exclusion when either byte is nonzero.

The previously recorded source world control5 capture/release, parent suppression19FF71&0A, pause118968 bit2, simulation118770 bit4, group526/52F and child-handler-consumption gates remain necessary. Visibility, ownership and correct sprite hit alone do not establish this control context.

### Competing-family gap before native selection

Current Game `runtime_scene.rs::open_clicked_genepod` scans visible Source-resource buildings and rejects multiple building hits. It does not collect all original83140 candidates or model viewport mask/flags/publication. `actor_scene.rs` explicitly uses caller family order; a successful rendered frame does not prove original hover eligibility or queue capacity/order.

A bounded no-overlap Mule route must account for every potential eligible hit: other creatures, plants, actors, buildings, source-selectable kind5 effects (code3B or code31 with extra predicates), source-selectable kind8 (55=2/56=12), and remaining active families admitted by the mask. Kind7 is explicitly unpickable. Unknown resource/hit geometry must return unsupported unless a source-proved enclosing bound excludes the mouse. A unique Mule hit plus zero building hits is insufficient. Shared queue capacity256 and source admission must also be established before treating a Mule as queued. No native selection, command, mission completion or full mixed-family ordering is claimed.

### Viewport follow-up159/160: cleanup versus enable candidates

New read-only export `private/ghidra/mule-viewport-enable-159/instructions-0002c0c0.txt` proves2C200..2C257 is selected-panel reference cleanup, not redraw. It conditionally clears MAINPANL selected references+19F/+1CC after comparing active panel+C8 and supplied entity pointer. Earlier redraw-candidate wording must not be carried forward; the callback interpreter already treats it as deferred UI cleanup.

Preceding2C0C0 cleanup clears current module116974 at2C111, calls viewport resource teardown82A80 at2C117 with16E0A8, then calls86DF0 at2C121 with the same viewport. Mapped86DF0..86DF4 bytesC6401C00C3 are `MOV byte ptr[EAX+1C],0; RET`: explicit viewport flags reset on cleanup, not mission-entry defaults.

Fresh reference graph159 exposes25AF6/25B06 without a containing function. Mapped25B06 bytes8025C4E01600BF clear bit40; follow-up25B12 calls86170. This is not the bit0 enable writer; do not export solely to reconfirm.

A root-requested read-only raw-byte scan found exact OR-immediate1 candidate30400 bytes80481C01C3 (`OR byte ptr[EAX+1C],1; RET`), adjacent303F0 bytes80601CFEC3 (`AND byte ptr[EAX+1C],FE; RET`). These are raw mapped-code candidates, not a proved viewport call path. No direct E8/E9 branch or absolute DWORD30400/303F0 pointer was found in mappedobject1, nor an absolute30400 pointer in mappedobject3. Do not infer live mission input invokes either helper.

Next exact bounded evidence request: `ExportExistingRange.java <new-output> 000303f0 0003040f`, plus `QueryReferences.java <new-output-directory> 00030400 000303f0` for caller discovery. Root coordinates future read-only Ghidra runs; none was launched by this analysis. Viewport bit0 remains unresolved until caller/data-flow is established.

- New read-only MAINPANL range SHA256 `9dcc28bc36c3e7f608f91094f66b1732982e6ae0f1cd85974d3769db763f23a8`.


## Authentic3573 corpse acquisition and bounded pursuit

Capture: `private/review/authentic-krell-159/frame-3573-slot-8-blocked-before`. Original afterimage/trace: `private/review/krell-mule-callback/e7dca4ea6c7b4281b67b983e6aaa8732`. Root reports160 integrated acquisition passed all427369 world bytes with2 RNG draws. This clears an authentic callback, not Krell completion.161 isolated pursuit comparisons were still running at this append; no result is presumed.

### Exact neutral food score and winner

`player-creation/instructions-0005ed20.txt`624C6..624DD and62527..62552 bind Mule descriptor1045FC+45*4=1046B0; its food coefficients+1D/+1E/+1F are100/100/100. Query baseEBP-7C2 gives bonus bytes+1BD/+1BE/+1BF at stackf9FB/f9FC/f9FD. Literal625C3..626AC initialization seeded those bytesA5 and returned0/0/0 (143 instructions,0 RNG).62697 clearsBH;62699 storesBH to each bonus byte. Do not invent a preference bonus from the sorting prelude.

`initial-creature/instructions-0006b700.txt`6BA8A..6BB57 treats friendly Mule24/25 as edible: virtual68=1EC50 testsstate>=24. Corpse60bit1clear doubles food-direction contribution and best-food score at6BB4B..6BB53:2*(sampleWeight+bonus0). With bit1set, score remains sampleWeight+0; reservation is not unconditional exclusion.6BC64..6BC72 replaces best-food only on strict signedgreater, preserving first source sample/link winner on ties. Plant6BCFC..6BD23 score=sampleWeight+bonus0. Preserve8D930's two RNG traversal draws, radius1024 sampled-cell circle and linked order.

Bones>=26 jumps6BAF3 directly6BD58, publishing neither food nor mate while retaining directional contributions. Live friendly same-Mule food is suppressed by6BBAE (sentinel score/zero direction); caller77!=FF skips mate publication6BBCB..6BBDD. Combat/foreign/other-species domains remain unsupported.

Actual3573 caller8 is immature77=75, reserve6. Its circle has exactly one edible corpse: Mule1state25 owner4/selector4 flags60=08, celloffset(+1,+1), weight662 score1324; no active plants. Nearby2/4/5/7 are live friends;6 is Bones27. This also fits narrower unique-corpse/no-plant admission.

###5B580 kind1 acquisition differs from plants

`terrain-map-refs/instructions-0005b580.txt` calls5CC60 local task cleanup first. Targetvirtual10 must pass5B59C..5B5B6; actualcorpse49=0 makes1EE20's0400 testfalse.5B5BC..5B5C9 writes target55=[corpseID,kind1]. Caller78=0 leaves optional58 untouched.

Mapped5B56C[kind1-1]=5B6C3, unlike plantkind2's5B6F9.5B6C3..5B6F1 clears caller16upperbits, sets49=(49|14)&EFFC and54=20, clears reference4Dkind, and sets live association50=[corpseID,kind1]. It deliberately does NOT copy corpseXY into goal0B.5B773..5B7D1 sets requestedspeedvirtual80, timer32767, state1 andlow60|C0; hunger61|40 was set before acquisition. Commoncallbacktail decrements timer32766. Actualafterstate1 flagsC8/40 target55/association50kind1 retainsoldgoal; corpseunchanged.

Newstate1 continuation executes in SAME callback:5B7E1 ->62EA2 ->63049 ->5EF46 ->65743. Initializer-only output is insufficient unless later branch has been proved no-action.

### Associated pursuit and explicit arrival boundary

`player-creation/instructions-0006fb60.txt` association50 refresh is state-agnostic.49bit4/8 triggers immediately; otherwise6FDE4..6FE23 needs nonnullassociation, phase(frame+signedID)&3=3 and49&3=0.6FE8C..6FEA2 resolves associated currentXY into goal0B. Validatedcorpse1 can share pairedapproach refresh gates without sharing readiness behavior.

6FEAB..6FEB7 calls74D90 forkind1 with nominatedbuildingfrom4D (null here), checking buildingobstacle and terrain; blockedgoal invokes74A50 at6FECE. Use `common_move_point_accepted` plus existingresolver/ray predicates. The pairedadapter's initial terrain-only `mule_accepts` shortcut is not additionalsourceproof and must not be copied into corpseadmission.

VirtualB8=5AD70 measures association physicalXYZ distance. Neutral76<=90 skipsflyspecial5ADBC..5ADFE and5AE57..5AE7B calls bothvirtual0 getters then73770. Existing `mule_pair_distance` implements this for twoMules. Repairedgoal terrainheight is wrong if corpsephysicalheightdiffers or74A50 repairsgoal.

State1 in `player-creation/instructions-0005ed20.txt`65861..65957 accepts deadvirtual68 target. If arrivalvirtualAC (49bit200) isfalse OR60bit80 remainsset,65973/6597D goes65B4D: speedrefresh/distance/5D360. `initial-creature/instructions-0005d360.txt`5D399..5D3E2 returns withoutmutations for neutral76<=90; phase!=3 returns earlier. This nonarrivaldomain does not startcombat/feeding.

Arrival remains explicitly unsupported:49bit200true AND60bit80clear dispatcheskind1via5ED04 to65AEF.65AEF..65B10 sets callerstate5 for corpse24/25 (state4 forBones>=26), ORs corpse60bit1 and sets callertimer32767.65B35..65B44 stores neutralcallerorigin72, sets60bit8 and clears60bit40. This changes BOTHrecords and enters a newfeeding lifecycle, not plantstate7. Preserve atomicrollback until independently proved.

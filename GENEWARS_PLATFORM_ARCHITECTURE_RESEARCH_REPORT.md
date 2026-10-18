# GeneWars platform architecture research report

Research completed 18 October 2026. Decision report; no implementation authorized or performed. Public revisions and local evidence are pinned below. This report is outside the live game checkout.

## 1. Executive verdict

**Build a reference-led, portable, mixed-language platform around the valuable Rust implementation already present. Change the reconstruction workflow before changing engines.** Use the original DOS game as an executable behavioral reference and a way to reach later mechanics. Keep Rust as the initial simulation implementation, asset layer and host, but remove the policy that every future routine must be Rust. Admit a C/C++ implementation when a bounded, tested subsystem demonstrably costs less to integrate and maintain. Build one authoritative simulation, with a stable command/state boundary, explicit edition adapters, complete saves, data-defined scenarios and eventually sandboxed behavior mods.

This combines option D, an original-game reference lane, with option F, a modular mixed-language product. It does **not** mean maintaining two new engines or calling original assembly opportunistically whenever a Rust callback fails. The current callback snapshots do not contain enough state for that substitution. The original executable should initially remain an external reference process, with an explicit exchange format and recorded observations.

**Second-best: a compatibility-first native x86 assembly port, followed by gradual routine replacement.** This could produce a substantially complete Windows game sooner because it preserves existing gameplay. There is a credible GeneWars-specific report of a Watcom/assembly bridge, and SyndWarsFX demonstrates the general technique. However, the GeneWars bridge and its full harness are private; the public attachment is a symbol/mapping package, not a playable engine. No inspected public artifact proves an ordinary person can complete a GeneWars mission through that native bridge. It is a candidate for a short feasibility gate, not grounds to abandon the Rust work. [GeneWars discussion][gw-issue], [public package][gw-pack], [SyndWarsFX build][sw-build]

**Premature decisions:** replacing the entire runtime with that private port; choosing a Magic Carpet engine as the GeneWars engine; committing to whole-program decompilation; declaring QA BETA V5 equivalent to retail; or promising complete creature/AI modding through the existing fixed-layout structures.

The evidence that would change this recommendation is specific:

- An inspectable, reproducible GeneWars native-assembly build that supports normal input, autonomous gameplay, mission completion, save/reload and progression, with a credible distribution/provenance strategy. That would justify making assembly compatibility the first product lane while portable replacements continue behind it.
- A verified original-save route that exposes advanced gameplay immediately. That would strengthen the recommended reference-led strategy and remove the campaign frontier as the discovery bottleneck.
- A portable lift of one complete gameplay family that beats hand reconstruction on **total** implementation, verification and integration effort, not lines generated. That would justify broader targeted lifting.
- Demonstrated byte/function equivalence of a specific Bullfrog service in the exact GeneWars build. That would justify importing that service; a shared engine credit would not.

No inspected alternative supplies a complete, openly reusable GeneWars gameplay engine today. Conversely, much of the supporting work has already been solved: asset extraction, numerous data contracts, symbols and class organization, assembly bridging techniques, original-input recording, portable host patterns, modern save containers, and renderer/simulation separation. Repeating those discoveries is unnecessary.

The proposed change preserves the final goal: all 24 missions, authentic rules where established, normal interaction, essential presentation/audio, save/load, portability and meaningful modding. It relaxes **universal implementation and presentation identity**, not game-rule correctness. A Windows/x86-only assembly product would postpone portability and source independence; any decision to make that the final endpoint would explicitly reduce the goal and needs separate approval.

## 2. Evidence standard and current local baseline

### Trust classes used in this report

| Class | Meaning | What it does not establish |
|---|---|---|
| **I — inspected implementation** | Relevant source, declarations, build configuration or static arithmetic examined during this assignment. | Successful execution or complete game behavior. |
| **R — recorded result** | Existing receipt, CI status or reference report inspected, with its revision and scope. | A fresh test by this researcher, or coverage beyond the receipt. |
| **C — contributor claim** | Maintainer documentation, issue report or private-repository account. | Independently reproducible functionality. |
| **X — inference/recommendation** | Conclusion drawn from the evidence, including proposed design and estimates. | An implemented feature. |
| **U — unresolved** | Needed evidence was absent, inaccessible or insufficient. | Permission to assume the favorable answer. |

No game, port, external executable, build or test suite was run for this assignment. Repository trees, source, receipts, CI metadata, documentation and one public ZIP's text entries were inspected. Static layout calculations are original analysis, not runtime observations. “Original-observed,” “instruction-interpreted,” “Rust-versus-Rust,” and “synthetic” remain different evidence origins even when all produce equal bytes.

### Live state, not the dated summary alone

The live checkout is `C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight`, inside the parent Git repository `C:/Users/fallo/Documents/ChatGPT/Freecad`. Read-only Git inspection found branch `codex/genewars-reconstruction`, HEAD **`3bcf6ff9b2d40212c2f501a9bff489fbf83e8f71`**, dated 18 October 2026, with nothing staged. Numerous historical tracked modifications and untracked files remain, including gameplay files, documentation and unfinished loss/escape work. They were preserved. Therefore HEAD alone does not describe the entire working state.

The most recent integration checkpoint, rather than the older top section of `CURRENT_STATE.md`, records:

| Item | Evidence and qualification |
|---|---|
| Campaign | Exceland accepted, **1/24**; Krell unfinished. This is a reconstruction acceptance count, not proof of broad human playability. |
| Latest frontier | **6119**, with `Creatures(UnsupportedNeighborhood)` still unmeasured. |
| Focused checks | **60 positive, 6 negative, 73 checks**, all passing in the retained F24 receipt. Not rerun. |
| Native agreement | **6100** accepted in F22. **6119 native parity remains pending.** |
| F24 world hash | `1184f851372039bd982200b2274c613273cfe922744e616849589790d8804f37`. |
| Reference identity | QA BETA V5 candidate executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; equivalence to retail remains unresolved. |
| Receipt identity limitation | The retained F24 root-check summary identifies `e282fb226315158651b8169d16fb33b49acc710e`, before the checkpoint commit. It is working-tree evidence, not a fresh clean-HEAD run of `3bcf6ff`. |

Sources: [current state][local-state], [F24 checkpoint][local-f24], [F24 root summary][local-f24-private], [F22 native checkpoint][local-f22]. The F24 checkpoint also records a correction from the prior timer-word attribution to `0x6c`, and source evidence that the neighborhood walk accumulates candidates rather than rejecting a second candidate. These are examples of genuine semantic progress, but also of the cost of relying on a narrow admitted state at each frontier.

The private tree retains original captures, Ghidra exports, harness inputs/outputs, native acceptance material and built binaries. `.gitignore` excludes `/private/`. Public source and receipts alone cannot reproduce tests requiring those inputs or original game assets. Clean-checkout reproducibility must distinguish ordinary source tests from an explicitly provisioned private conformance suite. The prior scenario-engine report named in the handoff was not located in the accessible search; its conclusions are not treated as independently reviewed evidence here.

A process inspection saw Hermes desktop executable processes. That does not establish that its reconstruction director or workers were running, and their inactivity was not independently proven. This assignment did not start, stop or alter those services. There was one procedural error: three research-only delegates were started before the handoff's explicit no-agent instruction was noticed; they were immediately interrupted and supplied no findings used here. No implementation was assigned. All subsequent research was performed in this task.

### What survives, component by component

The six workspace crates already provide useful boundaries: `genewars-assets`, `genewars-core`, `genewars-render`, `genewars-tools`, `genewars-desktop`, and `genewars-runtime`. Release builds explicitly enable overflow checks; original wrapping operations therefore need deliberate semantics rather than accidental release behavior. [Workspace][local-cargo]

| Existing asset | Decision | Reason and required adaptation |
|---|---|---|
| Resource discovery, RNC, palettes, animation/sprite readers | **Keep** | Existing recorded coverage is substantial: the dated feature matrix reports 331 RNC records and extensive image/animation extraction. Preserve exact edition/hash identities; do not replace working codecs merely to match another engine's language. |
| Mission/source parser and serialized layouts | **Keep, wrap in edition adapters** | Exact range/tag checks and unknown bytes are valuable. Its 343,958-byte contract is deliberately narrow, not a generic save or all-edition level parser. |
| Fixed-point math, RNG, source-backed numeric helpers | **Keep** | High-value fidelity foundations; compare third-party candidates against these contracts before substitution. |
| Implemented actor, creature, plant, building and campaign behavior | **Keep, generalize by behavior family** | Preserve valid source semantics and regressions. Replace frontier-shaped admission logic with complete supported families and explicit remaining unsupported behavior. |
| Original-instruction interpreter and negative tests | **Keep as focused research tools** | Excellent for exact arithmetic, mutation, ordering and disputed branches. They are bounded interpreters, not substitutes for an executing whole original game. |
| Native host, input and terrain/rendering | **Keep initially** | Already integrated. Extract command/view interfaces; do not make a renderer rewrite a prerequisite for gameplay progress. |
| Native/headless captures | **Keep, label correctly** | Useful host-path regressions. Agreement of two Rust routes is not a DOS reference comparison. |
| Campaign carry and deterministic replay | **Keep, broaden** | Reuse for regression and mission transitions. Add verified direct-entry scenarios, human interaction and alternative play strategies. |
| Eight-buffer callback captures | **Keep as callback fixtures** | They lack a demonstrated whole-runtime restoration contract. Do not rebrand them as saves. |
| Hardcoded mission/resource identities | **Adapt at boundaries** | Keep immutable original fixture identities, but remove fixed `CONQ001`/`CONQ002` assumptions from public scenario APIs. |
| Current development gate and worker scheduling policy | **Replace after approval** | Optimize for complete mechanics and product milestones, while retaining strict evidence requirements where they matter. |

The runtime is not merely a byte array. `ActorWorld` owns backing bytes plus players, RNG, optional plants, effects, creatures, family pools, buildings and collision information; binding and terrain-job context also exists outside the serialized source image. `creature_world.rs` distinguishes **116-byte packed** records from **124-byte native** records and has narrow animation/support states. Adding a row to a 25-species table cannot create arbitrary new behaviors. The desktop `save_new` helper writes reports, BMPs and source-world bytes; it is not whole-game save/load. [Actor world][local-actor], [creature world][local-creature], [desktop entry][local-desktop], [feature matrix][local-features]

## 3. The direct GeneWars ecosystem

### Public C++ reconstruction: valuable map, not a replacement engine

`mefistotelis/genewars`, default `main` at `72ec4c4…` (26 April 2022), is a meaningful source of recovered names, class organization, field offsets, tables and some implementations. Its README's warning about unimplemented functions is borne out by code. A lightweight scan of all 77 `bio/src/*.cpp` files found approximately 1,592 function-shaped bodies, about 1,501 empty after comments. This is a heuristic census, not an AST count or a completion percentage; constructors and initializer syntax affect the count.

Specific inspection is more decisive: `Creature` gameplay bodies, `Computer::UpdateCreatures`, map operations, save/load and editor methods are largely stubs. `xy.cpp` contains real coordinate operations; `pcmain.cpp` contains logging, argument handling and program setup; several constructors initialize meaningful defaults. `IFCEditor` has constructor/resource evidence but empty load/save/update/draw methods. `IFCCustom` has session-related names and setup, not a completed custom-game implementation. [Source tree][gw-code], [computer][gw-computer], [editor][gw-editor], [custom interface][gw-custom]

The offsets are informative without proving ABI equivalence. The public `Player` declaration is **1,769 bytes**, with research at decimal 1,562 and technology at 1,767. The historical save editor and local serialized player are **1,805 bytes**. Public `GridTile` is **16 bytes**, whereas serialized terrain cells are **14 bytes**. These may reflect runtime/serialized differences and build differences; they are not evidence that one structure can be copied over another. Use the recovered classes to name concepts and investigate call graphs, not as authoritative layout replacements. [Player declaration][gw-player], [grid declaration][gw-grid]

### Symbols and original builds

`genewars-re-helpers`, `a85d5c5…` (21 September 2026), contains a small collection of original executable variants, MAP/XML/byte-oriented symbol material, generated declarations and modified Watcom tooling. Its README explains that some binaries carry debug information and that the symbol formats support Ghidra/IDA interchange. The MAP files were inspected; the large XML content was not fully inspectable through the connector, and original binaries/tool executables were not run. [Helpers][gw-helpers]

Keep these identities separate: Beta2 July 1996; September retail lineage; November patched 1.1; demos; and the local QA BETA V5 candidate. For example, Beta2 `IFCEditor::PlugIn` appears at MAP `0001:00015C0C`, while a C++ comment uses `0001:00005c0c`. That supports a particular address-model difference, not universal `+0x10000` rebasing. The 1.1 MAP includes `LbSqrL` at `0x96260` and `UnpackM1` at `0xB9CFC`. [Beta2 map][gw-map-beta], [1.1 map][gw-map-retail]

The reusable output is a **versioned symbol crosswalk**: executable hash, LE object, address model, function range, candidate name, ABI, table references, evidence and confidence. Never import an entire foreign-build symbol map into the active project as though names proved matching boundaries.

### What the native-assembly collaboration actually establishes

All five issue comments and the issue's corrections were inspected. A contributor reports a Windows-native ASM/Watcom bridge in both directions, explicit DGROUP/DSEG/BSS and startup handling, and migration of a 634-byte production function into C. The claimed test covers 29 seeded classes, a full `0xA8C50` DGROUP comparison, eight poisoned variants and nine audit attacks, including a discovered false positive subsequently repaired. This is materially more specific than “it reaches a menu.” It remains **C**, because the full bridge, gameplay implementation and native harness are not public in the inspected material. The public `computer.cpp` does not contain that claimed migrated function. [Bridge comment][gw-bridge]

The accessible upstream ZIP was downloaded into isolated temporary storage and read as text, without executing its scripts. SHA-256: `b78ccd3c448a44d087626ff37b6833e606777fdbc1e3788d991ac6ef1f4d55f2`. It supplies methodology, address modeling, symbol-binding reports and limited reproducibility examples; it explicitly excludes the game binary, full assembly/native harness and several private corpora. No license file was present in its 36 entries. [Attachment][gw-pack]

Its corrected S3a report classifies 811 priority code symbols as 22 PROVEN, 146 STRONG, 425 PLAUSIBLE and 218 UNKNOWN. These are the package author's classifications of mapping evidence, **not this report certifying 811 equivalent implementations**. It reports 94/97 wrapper candidates passing, but only 39 strong bindings with ABI precedent. A previously reported 139 missing symbols becomes 68 renamed, 34 merged, 10 unknown and 27 actually missing. It also warns that a naive `+0x10000` transformation matches only 51/2,710 candidates across two related layouts. Those corrections make the package more useful and rule out simplistic rebasing.

The package describes a patched 1.1 LE model with code at `0x10000`, a small 16-bit callback object at `0xF0000`, DGROUP at `0x100000` and DSEG at `0x1B0000`; these are not proven QA BETA V5 locations. Its limited `LbSqrL`/RNC examples align with the public 1.1 MAP. Mapping agreement still does not prove complete gameplay or service emulation.

If later authorized, the smallest respectful maintainer questions are: can the exact bridge revision and build instructions be shared; which executable hash is supported; has an ordinary mission and save/reload been demonstrated; which original instructions are required at distribution; and what rights apply to the shared mapping scripts? No outreach was performed.

## 4. Save editor, format reconciliation and later-game research

### What the historical editor really does

At `3b754f7…` (19 October 2021), `src/gwsaved.c` is substantial working editor logic, not just declarations. It addresses eight slots, `save/save0.sav` through `save/save7.sav`, and exposes four players, 25 creature types, 32 specialist records per player, 12 plant entries, 300 packed creature records and a 128×128 terrain array. Its readme records earlier feature additions, including live-creature and underground-resource editing. No edition matrix or automated test suite was found in its small tree. These are **implemented operations**, not a new execution test of compatibility. [Editor source][save-c], [editor readme][save-readme]

| Surface | Actual implementation and limitation |
|---|---|
| Genetics/research | Reads/writes four-byte little-endian research lanes; enables/disables species, edits ordering/discovery and blessing data. Its display calculation divides by 2,048. A comment describing big-endian/2,068 is wrong relative to the code. |
| Available plants | Edits 12 four-byte entries, including availability; preserves other bytes where not explicitly changed. |
| Specialists | Edits names, race/character, occupation, ability, health and orbital/mission flags. Thirty-two stored records do not imply 32 simultaneous selectable specialists. |
| Living creatures | Edits owned creatures and native organisms (owner value 4), species, energy, health, maxima and lifespan. Serialized pool slots are not all necessarily active. |
| Terrain | Exposes height, texture, rotation, reveal and underground GOOP fields. Several bytes remain unknown; it does not establish a complete plant-growth/ecology model. |
| Player GOOP | The editor warns this is starting GOOP rather than a reliable edit of all accumulated/current economy state. |
| Campaign level | Changes a player level/technology byte; its warning says this can unlock buildings and accidentally skip missions unless restored. This is not proof of loading a different planet. |
| Partial export/import | Supports selected data blocks. The research/plants/order/blessing block spans 198 bytes and excludes the following discovered-species count, so import alone need not produce a consistent roster. |
| Campaign/custom map editing | Menu commands `M`/`U` print unfinished functionality and exit. The path-selection code also has a fallthrough and space-padded `%3d` naming. These paths are not a usable level editor. |

The source does not validate an edition-specific magic, length, checksum or active-record invariants before interpreting the file. It reads one `sizeof(SAVEDATA)` item without establishing a complete read, then writes with `fopen(...,"wb")` and `fwrite(sizeof(SAVEDATA))`. This truncates any longer file. It is useful reverse-engineering documentation, but should **not** be run on the user's original saves or assumed safe on copies from an unidentified edition. [File I/O and structures][save-c]

### 343,935 versus 343,958: the discrepancy is real

The editor's serialized structures consist of bytes and nested byte-array structures. Their member alignment is one; ordinary C padding cannot explain the 23-byte difference. Summing the declarations gives **343,935 bytes**. This was calculated statically, without compiling the historical editor.

The Rust mission reader instead requires **343,958 bytes**, a leading `0x55336` tag for `cd-conq-fast-0x55336-v1`, explicit group lengths and an extension tag. It explicitly rejects a known 375,830-byte alternate layout rather than pretending a filename establishes identity. [Mission parser][local-mission]

| Region | Historical editor declaration | Local 343,958-byte parser | Finding |
|---|---|---|---|
| First player | File offset 20; stride 1,805; four records | Matching player stride and observed fields | Strong common serialized contract, not whole-file identity. |
| Terrain | Offset 13,662 (`0x355E`), 16,384 × 14 = 229,376 bytes | Same start/extent in actor-world backing | Agrees. |
| Creatures | Offset 243,038 (`0x3B55E`), 300 × 116 = 34,800 | Group 0, same start/extent | Agrees through offset 277,838. |
| Following object pools | Opaque chunks 20,996 + 45 + 38,097 = **59,138** bytes | Groups 1–7 sum **59,160** | Editor is **22 bytes short** by this boundary. |
| Next globals | 26 + 25 = **51** bytes | Tail globals **52** bytes | One additional byte discrepancy. |
| Viewport | x/y at 337,027 / 337,029 | Four-byte viewport starts **337,050** | Editor is now 23 bytes early. |
| Final block | 6,904 bytes at **337,031** | Four-byte extension tag at **337,054**, followed by 6,900-byte payload | Equal extent, different start and unproved interpretation. |
| Total | **343,935** | **343,958** | Not interchangeable. |

For audit, local groups 1–7 are: 512×41 at 277,838; 20×140 at 298,830; 256×59 at 301,630; 200×73 at 316,734; 32×41 at 331,334; 32×89 at 332,646; and 16×94 at 335,494, ending at 336,998. The historical editor's opaque grouping does not identify where the missing 22 bytes truly belong. Appending 23 zeros cannot repair displaced fields.

The likely explanations are an older edition, an incomplete historical declaration, or both. The available code cannot adjudicate which. Equal prefixes are enough to motivate read-only field research, not enough to certify a writer. The editor treats `magic_id` as opaque; the local parser recognizes a specific mission format. Neither proves that every DOS save uses the same header, pointer fixups or extension. The editor has no decompression/checksum path, which establishes only what **that implementation** assumes, not that every original save is uncompressed and checksum-free.

Useful player-relative offsets calculated from the editor are:

| Field | Offset | Contract/qualification |
|---|---|---|
| Specialist count / records | `0x15` / `0x16` | 32 records × 36 bytes. |
| GOOP | `0x4A3` | Two-byte value; warning about starting balance applies. |
| Research lanes | `0x61A` | 25 × four-byte values, unaligned. |
| Plants | `0x67E` | 12 × four bytes. |
| Species order / blessing | `0x6AE` / `0x6C7` | 25 bytes each. |
| Discovered count | `0x6E0` | Must agree with order/availability edits. |
| Level/technology | `0x6E7` | Do not equate changing this with valid mission progression. |
| Player name | `0x6E9` | 16 bytes, explaining part of the extension beyond the public 1,769-byte player class. |

Packed creature fields include owner `+0x18`, energy/health maxima `+0x5A/+0x5C`, lifespan `+0x5E`, energy/health `+0x60/+0x62`, species `+0x72`. They are **not** the offsets of the local 124-byte native creature, whose species is at `+0x7A`. A tool must distinguish inactive slots, live records and serialized references before changing ownership or injecting organisms. [Editor declarations][save-c], [local creature representation][local-creature]

### Independent research corroboration and visible retractions

The contributor reports a byte-exact **343,958-byte** save parser with 50/50 checks and DOSBox-X save/capture work. The full parser and corpus are private; the report is **C**, not reproduced here. Crucially, the issue corrects its earlier research interpretation. The current account uses an unaligned 32-bit value at player `+0x61A + 4×species`, displayed by arithmetic shift 11. The earlier high-word/divide-by-512 interpretation and a minimum-22% production explanation are superseded. [Corrected issue body][gw-issue]

Two independent inspectable artifacts support the corrected lane layout: the old editor calculates `+0x61A` and divides a little-endian four-byte value by 2,048; the local `player_state.rs` writes **`0x11800` at `+0x62A`**, yielding 35% for that species lane. This corroborates layout and initialization, not every retail UI or AI rule. A raw value `0x17592` is about 46.70 in fixed-point units but displays **46** under integer `>>11`; fractional analysis and integer UI percentages must not be conflated. [Player initialization][local-player]

The contributor's corrected cap formula is `(tier×25 + 50)×0x800`, corresponding to 50/75/100%. Its claimed rendering, research and production function addresses refer to its retail lineage, not the local QA build. The original manual independently supports the 50% initial research ceiling and a 75% upgraded stage, but does not by itself prove the exact storage formula or all three code branches. Therefore: lane representation **I**, exact local initializer **I**, retail cap implementation and live observations **C**, general tier behavior partly supported by the manual. [Manual][manual]

### Can an edited original save bypass early research?

**It is a strong, testable route; it is not yet demonstrated reliable for the user's exact edition.** An original game that accepts a minimally edited copy can execute advanced mechanics even when Rust rejects them. That is a far better research source than inventing an advanced Rust fixture and calling it authentic. However, technology, discovered-species order, research amounts, plant availability, specialist state, live pools and mission objectives are coupled. Arbitrary high bytes can produce impossible states, out-of-range table access or incorrect campaign transitions.

Prefer naturally reached later-game saves or genuine original mission entry when available. Use cheat/debug entry only as labeled research provenance. The author's site documents cheats, but a cheat-created condition remains a modified original run rather than proof of normal progression. [Author's editor/cheat page][editor-site]

**Smallest safe experiment — specified, not executed:**

1. Pin the user's executable hash, assets, emulator version/configuration and save-producing edition. Use a separate writable research directory with copied game data, disconnected from real saves. Record hashes of originals and copies.
2. Start an ordinary original mission through normal controls, save it, quit and reload the **unchanged** copy. Record the file length, header, a screenshot and whether normal commands still work. If this baseline fails, do not edit anything.
3. Parse the copy read-only. Compare the 1,805-byte player stride, research lane and map/creature boundaries against the identified edition. Reject unidentified lengths/tags and overlapping ranges; preserve every unknown byte. Do not use the old editor's 343,935-byte writer.
4. Make exactly one diff: raise an already-known species' research within a plausible current tier, keeping the lane's low fractional bits deliberately specified. Write a new file of identical size in an unused slot, keeping an immutable backup and a machine-readable byte-diff record. Make no simultaneous creature or campaign-level edits.
5. Load through DOS GeneWars. Check the displayed research, actual production/research behavior, normal commands and a subsequent save/reload. Compare unaffected player fields and event behavior; merely reaching a screen is insufficient.
6. In separate copies, test species discovery/order/count consistency, then one technology unlock, then a specialist change. Stop on mismatch. Only after those pass attempt a later mission using a genuine level-start/save transition; never infer that changing `+0x6E7` loads the corresponding map.

**Pass:** unchanged and singly edited saves are accepted, the intended visible and gameplay effect occurs, subsequent save/reload works, unrelated state remains accounted for, and the resulting observation has an edition/hash/diff provenance record. **Fail:** rejection, truncation, unrelated mutations, inconsistent roster, missed progression or inability to explain the format. Budget **2–5 engineer-days** including original input setup; this is a planning estimate, not a promise. A pass immediately enables later research without waiting for Krell. A failure redirects effort toward original load/save tracing or genuine later saves, not increasingly broad mutations.

### A modern inspector is useful before a full editor

Use separate documents and APIs for four jobs:

1. **Original-file inspector/patcher:** immutable source plus edition adapter, interpreted fields and unknown-byte overlay; staged diff, validation, undo and export to a new path. Exact preservation is mandatory for untouched bytes.
2. **Research scenario:** original observation, input recording or controlled synthetic condition, with explicit provenance and assumptions. May be sufficient for a function test without being a valid whole game.
3. **Mission authoring document:** portable terrain, entities, player starts, rules, objectives and dependencies. A compiler validates and constructs runtime state. Original-format export is allowed only for a verified subset.
4. **Modern running-game save:** complete simulation state at a supported tick boundary, including scheduler state, RNG, jobs, references, campaign carry and mod state. UI preferences may be separate, but resumption-critical context cannot be omitted.

The **427,369-byte eight-buffer callback capture** is category 2. It is neither a certified original `.sav` nor category 4. Local fields outside those buffers include runtime bindings, subsystem setup and terrain jobs; mid-callback restoration also needs scheduler/call context. A future save must prove `save → new process → load → continue` equals uninterrupted execution. Equal captured arrays after a rejected callback prove neither execution nor restoration. [Current-state capture description][local-state], [ActorWorld][local-actor]

### Original editor, custom scenarios and multiplayer

There is real evidence of internal editor affordances: `IFCEditor`, `IFCCompEditor`, `LevelHeader` and `IFCCustom`, including an editor animation-bank resource. There is **no verified released, functional official GeneWars level editor** among the inspected materials. The public C++ methods are stubs; the save editor's map menu is unfinished. An internal class name is not a creator tool.

The original manual documents IPX multiplayer, eight selectable network levels, host selection, chat, creature advantages and zone-scoring or elimination outcomes. That supports a historical custom/network-game interface, not a completed single-player skirmish mode or arbitrary user-authored maps. Four serialized player records agree with a four-side model, but record count alone is not a network protocol specification. A patch-page mention of multiplayer changes must not erase the manual's earlier network support. New skirmish/sandbox belongs in the explicitly modern rule space. [Manual][manual], [patch information][patch-site], [IFCCustom source][gw-custom]

## 5. Magic Carpet genealogy and concrete reuse

### Is official, licensed Bullfrog engine source available?

**No official, authentic Bullfrog Magic Carpet engine source release under a legitimate reuse license was verified.** The inspected projects describe binary recoding, decompilation, disassembly or reimplementation. None provides a verified EA/Bullfrog grant releasing the original engine. A source repository can be useful and openly developed without turning the original proprietary instructions and assets into open-source material. In particular, remc2's acknowledgement of Bullfrog copyright is not such a grant. [remc2 README][remc-readme], [mgcarpet README][mgc-readme], [C++ port README][mcport-readme]

The historical engine-family connection is credible at a broad level: the original GeneWars manual credits Glenn Corpes for engine design, and the reverse-engineering author's material describes common resource technology across Bullfrog games. That does not establish a stable public engine SDK, matching terrain records or shared creature AI. The best demonstrated reuse is **common services and porting techniques**, not GeneWars gameplay hidden inside Magic Carpet. [Manual credits][manual], [author's format notes](https://genewars.lubiki.pl/html/gwrework_main.php)

### Side-by-side evidence

| Area | GeneWars evidence | Magic Carpet/Bullfrog evidence | Adjudication |
|---|---|---|---|
| Compiler/service lineage | Watcom symbol recovery; `LbSqrL`, `UnpackM1`, library groups in the mapping package | Real `Lb*` math, RNC and SDL implementations in Bullfrog library; Watcom wrappers in SyndWarsFX/SyndicatFX | **Demonstrated names and service families.** Exact cross-build bodies/ABIs require comparison. |
| RNC compression | Local decoder and recorded 331-record validation; 1.1 `UnpackM1` symbol | `rnc_1fm.c`; mgcarpet's importer explicitly derives its RNC code from remc2/RNC tooling | **Actual reusable codec implementations.** Local implementation already has high salvage value. |
| Terrain/world | 128×128 serialized 14-byte cells; runtime/public C++ tile differs; authored mission backing | remc2 `Terrain.cpp` has separate 65,536-entry arrays for 256×256 height, shade, type, angle and entity indexing; seeded terrain generation | **Not the same serialization or world API.** Shared mathematical ancestry is plausible, but a drop-in map engine is refuted by these contracts. |
| Creatures/entities | 300 packed 116-byte creatures, native 124-byte records, specialized player/building/ecology pools | C++ MC1 `Thing` is 164 bytes, retail-sized pool 1,000, separate initialization records | **Different entity systems.** Generic allocation ideas transfer; gameplay code needs reconstruction. |
| RNG | Local `blueprint.rs` uses a wrapping `0xE62D` multiplier with additional ring/draw accounting | MC1 `mc_types.h` documents per-entity recurrence using `0x24A1` and `0x24DF` | **Specific mechanisms differ.** This does not assert every RNG in either game differs; it rules out blanket substitution. |
| Renderer | Local source-backed terrain/projection/shading and sprite work; public `MapDisplay`, `TopoGrid`, `Nibble` names | remc2 exposes `DrawWorld_411A0(x,y,yaw,z,pitch,roll,fov)` with Original/HD/NG implementations | **Architectural analogy, not proved equivalent renderer.** Camera, globals, world layout, clipping and palette dependencies must be mapped. |
| Resources | RNC, palette and indexed-art conventions | Related DAT/TAB, palettes, texture/sprite import pipelines | **Shared conventions**, not universal file compatibility. Individual format versions and flags matter. |
| Sound/network/input | Original service names and import groups, unfinished public host methods | Mature SDL/audio hosts and replacement networking in ports | **Reusable host patterns and perhaps isolated services.** Neither IPX packets nor GeneWars sound events are supplied by MC gameplay. |
| AI, genetics, plants, economy, objectives | GeneWars-specific semantics and incomplete local families | Magic Carpet spells, wizards, mana, castles and flight | **No demonstrated implementation reuse.** These are the largest remaining GeneWars requirements. |

Sources: [local mission parser][local-mission], [local creatures][local-creature], [local RNG][local-rng], [MC1 types][mcport-types], [MC2 terrain][remc-terrain], [renderer interface][remc-interface], [Bullfrog library][bflib], [mgcarpet RNC][mgc-rnc]. No binary-fingerprint corpus or paired call-graph proof was found establishing GeneWars↔Magic Carpet terrain-renderer identity. The mapping attachment supplies narrower service/name evidence, not that missing proof.

### remc2 and magic-carpet-2-hd: practical ports, not independent confirmations

`turican0/remc2` and `thobbsinteractive/magic-carpet-2-hd` have separate active default-branch heads, but the latter is a fork of the former and they share substantial history. Treating them as two independent reverse-engineering validations would overstate the evidence. Their modern C++/SDL code, real terrain implementation and multiple renderers are inspectable. Project documentation reports high-resolution output, controller support and LAN/IPv4 work; the MC1 roadmap remains incomplete. [remc2][remc-readme], [HD fork][hd-readme]

Current-head CI metadata showed successful Windows, Linux and ARM64 jobs for remc2, and successful corresponding jobs for the HD fork. That is useful portability/build evidence, not a fresh human campaign test. The regression code is more informative than an old README statement: `ResetRegressionRunState` resets globals between runs, and `run_regtest` can fall back to packed `data.binz` when a CD tree is absent. Thus an assertion that regression testing simply cannot run in CI is stale. Availability and provenance of packed reference/game material still need review before reuse. [Regression implementation][remc-regression], [remc2 ARM64 run][remc-ci], [HD Linux run][hd-ci]

`GameRenderInterface` shows a good seam, but the HD renderer is a large module coupled to engine globals and MC terrain/entities. Copying it is not equivalent to installing a renderer library. A GeneWars adapter could cost more than improving the existing renderer. Audio/input libraries are less game-specific, but local winit/softbuffer already provides a functioning host path.

Licensing is materially ambiguous: README text distinguishes original Bullfrog copyright and MIT-licensed additional code, while `docs/LICENSE` contains GPLv3 and bundled dependencies have their own terms. Some inspected engine files lack a clarifying per-file grant. Do not label the whole repository “MIT” or assume the GPL file grants rights to original executable-derived material. [remc2 license][remc-license], [HD license][hd-license]

### mgcarpet: strongest architectural model, not compatible GeneWars simulation

`raincz/mgcarpet` is an actively developed Rust implementation using wgpu/winit/cpal. Its README reports MC1, Hidden Worlds and MC2 campaign play and explicitly distinguishes behavioral fidelity from a newly written renderer. It also describes deliberate improvements; a default entity pool larger than the retail limit means default “faithfulness” must be qualified by configuration. These are useful design choices to inspect, not proof of every reported campaign result. [README][mgc-readme]

The important mechanisms are implemented across separate import, format, simulation, rendering, audio and conformance crates:

- Import/baking converts user game data into versioned local resources and levels; direct game/level selection avoids replaying an entire campaign to investigate one situation.
- Simulation runs independently of render frame rate. Rendering interpolates presentation rather than making GPU frames advance game logic.
- `snapshot.rs` has explicit snapshot/restore contracts and validation; `mgcs.rs` defines a versioned save container. A save format, simulation snapshot version and level schema are separate concerns.
- Fidelity documentation distinguishes traced code, oracle comparison, player validation and stronger retail-recorded coverage. Intentional patches have classifications and can be forced to retail behavior for applicable records.
- `mgc-conform/tests/suite.rs` has an explicit **`MGC_REQUIRE_GOLDENS`** path: missing baked/LFS evidence becomes a failure, including a run with no usable fixtures. Without that strict mode, missing evidence can be skipped.

These are inspected mechanisms, not merely README aspirations. [Fidelity policy][mgc-fidelity], [snapshot][mgc-snapshot], [save container][mgc-save], [conformance runner][mgc-suite]

The conformance report is dated **1 October 2026**, tied to an earlier revision than the inspected HEAD. It reports 633 fixtures across 62 levels and 34 roster entries, 31 classified as retail-side divergences and three as recording-capture limitations. Whole-campaign recordings are not all public; some certified material covers prefixes. Current-head CI succeeded, but that does not turn the older report or absent private recordings into fresh all-campaign evidence. [Conformance report][mgc-conformance], [current CI][mgc-ci]

Its implemented mod system is notably narrower than a comprehensive mod platform. `docs/MODDING.md` specifies one overlay root containing replacement existing `LEVnnnnn.DAT` records. It rejects duplicate overrides, does not yet support general insertion into empty slots, and requires rebaking when an overlay changes. Hash metadata and a MODDED designation help prevent altered content from silently entering faithful fixtures. This is an excellent first content-mod milestone, but not arbitrary genes, new species logic, package dependency resolution or script mods. [Modding contract][mgc-modding]

Reuse the **boundaries, evidence labeling, direct scenario entry, versioned saves and mod provenance**. Potential code reuse is limited to independent utilities or proven format/math contracts under compatible licensing. Its terrain, entity logic, physics, save schema and MC-specific import pipeline are not GeneWars modules.

### Magic-Carpet-port: concrete replay, normalization and save-anywhere techniques

`jessperler88/Magic-Carpet-port` targets native Windows x64 with C++17/SDL2, with an SDL-free `mcengine`, a C asset layer and a host. Documentation claims the base/Hidden Worlds campaigns, exact reference comparisons, debug scenarios, networking and save-anywhere. No current CI workflow was found in the inspected tree; those campaign claims were not executed here. [README][mcport-readme]

The actual reference runner compares selected original fields after normalizing pointers into indices, including entities, player state, RNG and sampled terrain. It handles edition-specific record-size differences explicitly. Its movie-0 path compares every tick for an early interval and later ticks at a lower cadence, with still sparser terrain checks. That is valuable scope-controlled validation, not proof that every byte, frame and campaign is checked. Both `reference_test.cpp` and `render_reference_test.cpp` return success when their required dumps are absent. A green exit without a coverage/skip report is therefore insufficient. [State reference test][mcport-ref], [render reference test][mcport-render-ref]

`savegame.cpp` separates the original save representation from extended `state%02d.mcs` state, including versioned GLOB and RULE chunks. This directly addresses the fact that original-looking main state may omit simulation globals and modern rule settings. `scenario.cpp` contains command parsing, controlled input, stepping and assertions such as health/mana, rather than only a debug console screenshot. The same command vocabulary can serve researchers, tests and an eventual editor. [Save implementation][mcport-save], [scenario implementation][mcport-scenario]

These mechanisms support the recommendation to decouple headless simulation, replay, save state and presentation. They do not establish compatible GeneWars structs, or make a C++ rewrite intrinsically faster. The project asks users to provide game data and extracts original tables from an executable; its code license does not license those assets or settle every derived-code question.

### Bullfrog library, assembly ports and other relevant work

**`jopadan/bullfrog` is not merely a header collection.** `src/general/gtrig.c` and `rnc_1fm.c` implement math/decompression; `src/x86-win-sdl/sscreen.c` contains substantial screen code; `tests/bflib_test_math.c` exercises square-root behavior. Root GPLv3 and GPLv2-or-later headers require compatible reuse. It is a historical library snapshot, last default-head update in 2023, not an official Bullfrog source release. Consider maintained descendants and exact ABI/behavior before importing it. [Library][bflib], [math test][bflib-test]

**SyndWarsFX demonstrates gradual replacement of original assembly.** Its build includes `src/*.sx` alongside C and generates wrappers. `configure.ac` explicitly selects i386 and 32-bit/no-PIE-style assumptions; modern host APIs do not make the original assembly ARM-native. The related SyndicatFX wrapper configuration demonstrates Watcom register arguments and special handling for C library calls/varargs. These are credible bridge precedents; they do not implement GeneWars initialization, globals, callbacks or gameplay. Current SyndWarsFX HEAD's build jobs were still in progress when inspected, so no current-head success is claimed. [Build][sw-build], [configuration][sw-config], [SyndicatFX wrappers][syndicat-wrappers]

Its licensing warning is decisive: `COPYING` grants GPL terms for the port's work but explicitly excludes original Bullfrog disassembly in `src/*.sx`. A good-faith takedown statement is not a reuse license. This distinction applies directly to an assembly-first GeneWars distribution. [SyndWarsFX copying][sw-license]

**remc1/remc1hw** are older binary-recoding references with very large generated-style `sub_main.cpp` files, 2023 heads, Visual Studio-era setup and hardcoded regression paths. Their comparison machinery can help identify behavior, but they are weaker modern product foundations than the newer ports. The old regression helper's hardcoded path and deliberately crashing mismatch mechanism should not be copied as a workflow. [remc1][remc1], [remc1hw][remc1hw]

**MagicBalls** is the clearest additional host/simulation experiment: a Godot front end connected to remc2 through GDExtension. Actual `MBEXmain.cpp` bindings expose `RunGameStep(Dictionary)`, entity and player queries, and terrain height access while including remc2 modules. This proves a practical integration seam exists in that project. Its April 2026 release documentation describes a playable but incomplete/poorly optimized implementation; Android is experimental. It is not GeneWars gameplay, and choosing Godot would still require a state/view adapter, new UI and editor integration. Use it to understand the seam, not to justify a fashionable host rewrite. [MagicBalls][magicballs], [binding implementation][magicballs-bind]

**Magic Carpet File Format** is a small research repository containing a historical draft specification, including procedural terrain parameters and entity records. It is useful corroborating format research, not an engine or a verified license to copy source. No license grant was established. [Format research][mc-format]

### Reuse decision map

| Candidate | Dependency/version issues | License/provenance | Decision now |
|---|---|---|---|
| Existing Rust asset/math modules | QA/asset identities already recorded | Local licensing needs explicit project audit | **Keep.** Highest integration advantage. |
| Bullfrog RNC/math helpers | Exact arithmetic/tables/ABI; compare supported inputs | GPL-family headers; check chosen file and dependencies | **Optional targeted reuse**, only if missing locally or demonstrably better. |
| Watcom wrapper generation | Register/stack conventions, varargs, x86, callbacks, globals | Port code and original assembly have different rights | **Study/use compatible wrapper code conditionally** for experiment B. |
| GeneWars names/maps/class layouts | Build-specific addresses and runtime versus packed layouts | Metadata/tool code/original binaries differ | **Use as versioned research evidence**, no blind symbol transplant. |
| remc2 renderer or terrain core | MC-specific globals/world/camera; substantial adapter | Mixed/ambiguous grants plus original provenance | **Do not transplant now.** No proved cost advantage. |
| mgcarpet save/replay architecture | Different simulation and container schemas | GPLv3 project; third-party asset notices | **Adopt design**, not MC state schemas. |
| C++ scenario/assertion approach | MC command semantics must become GW commands | GPLv3-family code; original tables external | **Adopt approach**; copy only if useful and license-compatible. |
| SDL/modern I/O | Replace service layer, not simulation | SDL2 uses zlib terms; other libraries differ | **Available option**, no reason to discard existing winit host immediately. |
| MagicBalls/Godot adapter | Godot/remc2 dependencies, dictionary boundary cost | Root GPLv3 plus third-party licenses | **Editor/host precedent**, defer wholesale integration. |
| Original lifted gameplay/ASM | Exact executable, state closure, instruction semantics | Original rights unresolved by port license | **Research/reference lane first**; distribution is a separate decision. |

## 6. Architecture comparison

### Consistent definition of a working vertical slice

A working GeneWars slice lets a person start an identified original mission, select and command specialists/creatures through native controls, build and interact with the ecology, receive essential visual/audio feedback, encounter autonomous simulation, reach a legitimate win/loss, and resume from a real save. It must tolerate an alternative sensible sequence of actions, not only one replay. Where the mission has no opposing AI, a separate adversarial scenario is needed before claiming AI gameplay.

Menus, compiled binaries, a rendered terrain, scripted visible replays, exact isolated callbacks and emulator execution of the unchanged DOS game each prove useful but different things. None alone meets that product definition.

The table rates **prospective routes**, not measured completion dates. Confidence concerns the evidence supporting the judgment. “Fast” always means relative to other reconstruction routes under the stated prerequisites.

| Option | Route to a human-playable original campaign | Modding and fidelity | Portability / observability / maintenance | Effort, evidence and failure signal |
|---|---|---|---|---|
| **A. Rust only, improved workflow** | Extend current runtime by whole mechanics; use original saves/direct scenarios instead of serial frontier discovery | Strong eventual data/script surface; original semantics must still be recovered. Existing source-exact assets survive | Best fit to current portable code. Excellent local testability; original observation still external. One-language maintenance | **High remaining gameplay effort; medium confidence.** Best continuity, but no proof of fast full-campaign convergence. Fail if repeated guard repairs dominate while no new human mission becomes playable. |
| **B. Native x86 ASM + modern I/O** | Preserve broad original gameplay, replace platform services, then make normal input/save/progression work | Strong inherited behavior if initialization/services/timing match; arbitrary mechanics mods initially difficult. Assembly and fixed pools constrain extension | Windows/x86 feasible; 64-bit hosts require 32-bit execution or a much larger translation effort. ARM needs emulation or replacement. Excellent internal instrumentation once bridged; complex ABI maintenance | **Potentially shortest compatibility route; low-to-medium confidence for GW.** GeneWars bridge is private claim, sibling precedent real. Fail if no whole mission/state roundtrip or if most dependencies require wholesale lifting before use. |
| **C. Portable C/C++ or IR lift** | Translate coherent original subsystems, replace services, normalize pointers/state, integrate a host | Can retain exact low-level rules and gradually expose data. Decompiled structure alone is poor mod API | Potential x64/ARM portability, but compiler UB, aliasing, flags, x87 and runtime/library dependencies need control. Debuggable after cleanup | **High upfront and verification effort; low confidence for whole program.** A bounded family may be efficient. Fail if generated code needs pervasive manual semantic repair or keeps hidden global dependencies. |
| **D. Original reference + Rust product** | Original runs unlock observation; portable implementation advances by mechanics and scenario evidence | Strong provenance and a modern mod surface in the portable product; original lane remains baseline | Emulator can run on modern hosts; product portability remains independent. Maintain a thin capture protocol, not a second rewritten game | **Moderate reference investment plus high remaining gameplay work; medium confidence.** Most justified accelerator. Fail if original input/capture cannot provide repeatable states or if dual tooling consumes more effort than it removes. |
| **E. Magic Carpet components** | Reuse specific codecs/math/host pieces; GeneWars gameplay still needed | Helps peripheral systems, not genetics/ecology/mission rules. Fidelity depends on demonstrated matching contract | C++/Rust/SDL implementations offer practical platform examples; integration and licensing can dominate | **Low cost for proven small services, high risk for whole-engine transplant; high confidence in that distinction.** Fail if adapter exceeds replacement value or unproved behavior is imported. |
| **F. Mixed-language modular product + data/scripts** | Keep current platform; use the cheapest verified implementation per coherent subsystem behind stable interfaces, with D for reference | Best long-term content/behavior extension; faithful rules remain a pinned package. More boundary design than A, no automatic fidelity loss | Portable core and stable C ABI; sandbox isolates mod faults. Requires ownership discipline and versioned schemas | **Medium architectural investment, high gameplay work; medium confidence. Recommended with D.** Fail if split ownership duplicates mutable state or FFI dominates fine-grained simulation. |

Every option requires user-supplied original content unless replacement assets are independently created/licensed. B and C add particularly direct original-instruction/code provenance questions. A/F are not automatically legally “clean” because they use Rust; copied implementations, tables and assets retain their own provenance. D's private reference execution avoids publishing a new original-instruction engine, but does not grant redistribution rights. E requires per-module license review. No option has a demonstrated license shortcut.

### The hardest blocker in each route

**A: breadth, not syntax.** Current parsing, rendering and verified families would survive almost intact. The obstacle is completing behavior under varied legitimate player actions: AI, creatures beyond admitted paths, ecology, technology, outcomes, audio and persistent state. A language restriction has no demonstrated fidelity benefit. A may still win if whole-family implementation throughput improves; measure that with product behavior and maintenance time, not frame count.

**B: a complete executable environment.** Original code expects register conventions, fixed data objects, callback transitions, runtime initialization, memory ownership, timers, interrupts/platform services and sometimes segmented/16-bit paths. A working call bridge is necessary but insufficient. The first meaningful result is a complete mission and reload through ordinary controls, not a bridged function. Rust parsers, fixtures and renderer knowledge survive as tools even if the first playable product uses assembly. Converting 32-bit x86 assembly to a native x64 build is not changing one compiler flag.

**C: preserving semantics while escaping the original machine model.** Ghidra output is evidence and scaffolding, not a verified portable implementation. Correctness can depend on signed widths, unaligned access, wraparound, flags, pointer arithmetic, aliasing, indirect calls, calling conventions and table addresses. IR lifting can retain those details but also retain a giant emulated memory model that is difficult to mod. Prefer a complete function family with explicit dependencies, then remove accidental machine representation behind tests. No broad lift was run here and no inspected source proves that automated whole-GeneWars conversion is a short task.

**D: original access and trustworthy capture.** The local feature matrix records only original menu/options entry, not a proven original mission-control pipeline. Fixing that research-access problem is high value and may be a small fraction of another engine's cost. Capture normalized state at tick/mission checkpoints plus commands and visible outcomes. Retain the executable/configuration so evidence can be regenerated, but archive routine reference tooling once stable contracts exist; only reopen it for disputes, new mechanics or edition differences.

**E: coupling.** The useful overlap is concentrated in already-understood services. Rendering and terrain are alluring because they are visible, but they are not the current campaign blocker. Import a module only after demonstrating compatible inputs/outputs and a net reduction in work. Do not turn investigation of a renderer into a new prerequisite for AI or saves.

**F: choosing honest boundaries.** One subsystem owns each state field. A C ABI must use explicit widths, opaque handles and bounded buffers; no cross-boundary C++ object layouts or Rust references, and no unwinding through FFI. Batch operations per system/tick, not per pixel or per creature property. One authoritative scheduler and RNG policy remain. Mixed language is an available optimization, not a mandate to rewrite existing Rust in C++.

### What costs can responsibly be estimated

The first decision experiments below are bounded in engineer-days. A complete 24-mission platform is still a **multi-month engineering effort** absent a verified near-complete native port. A provisional planning envelope of **6–18+ engineer-months** for full campaign breadth, save reliability, product finishing and an initial mod/editor surface is deliberately broad and low confidence; it is not extrapolated from frame 6119 or a sibling author's one-month anecdote. Comprehensive new-species behavior tooling and polished multiplayer can extend that substantially.

For B, a verified near-complete bridge could compress compatibility delivery considerably, while ARM portability and rich mods remain later work. Without that bridge, B may cost more than continuing A/F. For C, whole-program automation does not remove integration and semantic auditing. For all routes, account for original-game capture, licensing/provenance work, test-corpus maintenance, debugging, human playtesting and rework—not just code-generation time or model cost.

## 7. Proposed target architecture

Everything marked **proposed** below is a design recommendation, not a claim of implemented functionality. Existing components remain in place until a bounded change earns its cost.

```text
 User-owned original assets / original saves        Creator packages
                  |                                      |
 [Existing asset readers + proposed edition adapters / package validator]
                  |                                      |
       Immutable resource store                Portable mission definitions
       + source/hash/license manifest          + versioned rules and objectives
                  \                                      /
             [One authoritative simulation boundary — proposed]
             load_scenario / enqueue_command / step_tick
             snapshot / restore / inspect / emit_events
                              |
             Existing Rust runtime, expanded by whole mechanics
             + optional verified C/C++ subsystem implementations
             + explicit faithful-edition rules OR custom rules
             + bounded sandbox hooks, after data mods work
                              |
                  Read-only view + event stream
                    /                       \
       Existing renderer/native host      Headless runner / debugger
       modern audio/input/accessibility   replay / scenario assertions
                    \                       /
               Proposed editor and save/scenario inspector
               same commands, schemas and validation as the game

 Separate reference lane, never an implicit failed-callback fallback:
 [Original DOS EXE in pinned emulator, or validated native ASM host]
            -> recorded commands, state projections, observations
            -> provenance-bearing conformance corpus
            -> comparison with the portable simulation
```

### Simulation, commands and ownership

Retain the current runtime as the implementation starting point. Introduce an interface at a complete tick boundary with explicit scenario identity, commands, deterministic state and events. Inputs become commands stamped for simulation ticks; window events and frame rate do not directly mutate gameplay. Pause and single-step belong to the host/debug layer. Rendering receives a view and can interpolate without consuming simulation RNG or changing collision state.

Faithful mode preserves original tick scheduling, entity iteration order, integer behavior, state transitions and random draw order wherever evidence makes them observable. Do not silently “clean up” wraparound, pool exhaustion or target tie-breaking. Modern render frame rate, output resolution, window management, key rebinding and readable UI may improve independently. Presentation changes that influence information, targeting or timing need explicit gameplay classification.

Keep original packed layouts in import/export and evidence adapters. Runtime ownership may become typed, but legacy slot order and handles must survive when faithful behavior depends on them. Use stable IDs with generation checks for modern references; preserve a legacy-slot mapping for original comparisons. Do not force new content into fixed 25/300 limits merely because the original serializer has those limits. Faithful packages can enforce original capacities, while custom packages declare supported limits.

A mixed-language subsystem is justified only when it has a closed state contract: explicit input/output, known mutation ownership, RNG consumption and scheduling. Cross-language objects are opaque; ownership and allocator boundaries are documented. Do not split a creature's live state between Rust and C++ mirrors updated opportunistically.

### Scenarios, saves and replays

Use a versioned scenario document with a format ID, edition/rule profile, resource manifest, map, player starts, initial entities, campaign carry, objectives, AI settings and optional controlled seed. Original mission import is one producer, not a special path embedded throughout the core. Direct mission entry must distinguish original fresh entry, verified carried entry and synthetic research setup.

A save contains the authoritative simulation snapshot, current tick/scheduler phase, all RNG state, pending commands/jobs, pool/handle state, campaign progress, resolved content hashes and script state. Save only at supported boundaries until mid-tick serialization is genuinely implemented. Load validates schema and dependencies before changing the running game, then reconstructs derived caches and checks invariants. A checksum detects corruption; it is not a security or authenticity claim. Use atomic new-file writes and retained previous saves. Importing an original `.sav` requires an edition-specific conversion and a visible statement of any unsupported fields.

A replay contains the initial scenario/save identity, rule/mod manifest and ordered commands, with periodic semantic/state hashes. It should reproduce the same simulation under different rendering rates. Pointer values and renderer caches are excluded from portable state hashes, but relevant entity ordering is not normalized away. Version migrations are explicit and tested; old recordings must either retain their compatible runtime/profile or fail with a precise incompatibility message.

### Concrete mod technology

**Start with data, then one sandbox technology.** Use a ZIP/directory package containing a TOML manifest and JSON definition documents validated against versioned schemas. These are proposed choices selected for simple inspection and tooling, not claims that the local engine already supports them.

The manifest should include a unique namespaced ID, semantic version, engine API range, schema version, required packages, content hashes, license/attribution and capabilities. Resolve dependencies deterministically and store a lock manifest in saves/replays. Reject cycles, duplicate IDs and conflicting scalar replacements unless the user explicitly selects a resolution. Avoid accidental filesystem-order “last file wins.” Additive definitions use namespaced IDs; original IDs remain stable in the faithful base package. Hash the **resolved content**, not just the package version string.

| Surface | Minimum useful version | Eventual comprehensive version |
|---|---|---|
| Planets/maps | Edit/import a terrain map, starts and existing entity placements | Biomes, terrain materials, procedural generators and larger custom maps within declared limits |
| Missions/objectives | Declarative conditions/actions using supported mechanics | Branching campaign graphs, custom events, victory/loss logic and scripted objectives |
| Creatures/genes | Parameter variants of existing implemented archetypes, explicitly labeled | New traits, hybrid recipes, animation/art mappings, senses, abilities and behavior hooks |
| Plants/ecology | Existing plant definitions and validated parameters | New growth models, soil/biome interactions and ecosystem rules |
| Buildings/research/economy | Costs, prerequisites, production and research tables within supported operations | New technology graphs and bounded custom production/effect logic |
| AI | Configure existing policies and priorities | Scripted decision hooks or behavior modules issuing the same validated commands |
| Presentation/UI | Assets, localization, color/readability settings and permitted layout metadata | Sandboxed UI panels with separate presentation capabilities; no direct simulation memory access |

Data edits alone cannot promise every row's eventual behavior. A new creature type requires real simulation, animation, targeting, save and UI support. The first successful mod should add a useful custom mission or alter a supported technology definition; it should not require rebuilding the engine or editing proprietary binaries.

For behavior mods, **WebAssembly through Wasmtime is the preferred eventual boundary**: portable modules, bounded memory and explicit imports. Expose a narrow game API—query a supplied state view, emit validated commands/events, use host-managed deterministic RNG, and serialize declared mod state. Provide no ambient WASI filesystem, networking, clocks, process access or shared memory. Bound memory, calls, output queues and per-tick execution. For deterministic quotas, Wasmtime documents fuel-based interruption; epoch deadlines are useful host protection but not equivalent deterministic accounting. Pin runtime/configuration and define whether an exhausted budget disables the mod, rejects the scenario or aborts the tick transaction. Do not let a timeout leave partially applied commands. [Wasmtime interruption](https://docs.wasmtime.dev/examples-interrupting-wasm.html), [security](https://docs.wasmtime.dev/security.html)

WebAssembly alone does not guarantee reproducibility. Imports, iteration order, floating-point NaNs, relaxed SIMD and memory-growth behavior require a defined policy. Prefer integer/fixed-point simulation APIs, canonicalize or restrict relevant floating behavior, and keep scripts from choosing nondeterministic host services. Save script-visible persistent state through an explicit schema rather than assuming opaque VM memory is a stable cross-version save format. [Deterministic execution guidance](https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html)

**Lua 5.4 is the lower-authoring-cost alternative** if actual creator demand prioritizes handwritten scripts and the team accepts additional sandbox/determinism work. It has a permissive license and supports environment control and hooks, but simply removing `os` is not a complete resource/determinism policy. Do not ship both Lua and Wasm initially: two APIs, save models and security surfaces would multiply maintenance. A small scripting spike should decide against a concrete authoring task. [Lua manual](https://www.lua.org/manual/5.4/manual.html), [Lua license](https://www.lua.org/license.html)

Native plugins should be reserved for trusted developer integrations. They can corrupt process memory, defeat deterministic guarantees and bind to platform/ABI details. They are inappropriate as the default community mod format. If supported later, load them only through an explicit trusted-plugin mode and mark resulting sessions accordingly. A native plugin boundary is not a way to evade applicable code-license obligations.

For multiplayer, all peers need the same resolved content/rule/API manifest and deterministic simulation configuration. Replay hashes can detect divergence, not prove that an opponent is honest. Server-authoritative validation or another explicit trust model is a later product decision. Do not represent rewritten networking as original IPX protocol compatibility without separate evidence.

### Editor and user experience

Build the first standalone inspector/editor on the same schemas and validators used by the runtime. It should show terrain/entities, field explanations, unknown-byte regions for original imports, warnings, staged diffs and undo. “Test scenario” launches a disposable session through the same commands as ordinary play, then returns diagnostics linked to the responsible definition. Follow with in-game pause/edit/preview when simulation snapshots are reliable.

A creature designer needs to show gene/trait dependencies, resulting stats, supported behaviors, art/animation bindings and a contained test encounter. A mission editor needs objectives, prerequisites, player starts and failure conditions, not just a terrain paintbrush. Economy/research editing should visualize dependencies and flag unreachable technologies or cycles. Save import/export remains a distinct action from authoring a reusable mission.

Retain the existing desktop/render path initially. Godot may later lower editor/UI authoring costs, as MagicBalls demonstrates a possible native seam, but it must earn the integration cost through a small editor experiment. It is not necessary to settle the renderer or editor framework before completing a playable GeneWars mechanic.

### Two rule spaces with visible identities

**Faithful:** a pinned executable/edition profile, original content hashes, documented implemented scope and only changes classified as presentation-only. It must be called “QA BETA V5 faithful” until retail equivalence or a separate retail profile is established. Unreconstructed behavior remains unsupported; it is not silently approximated.

**Modern/custom:** explicit patches, new rules, capacities, missions, scripts or convenience features. The launcher, save and replay display that identity and its package hash. A mod cannot claim faithful status by keeping the same filename. Original quirks can be retained in faithful mode and corrected in modern mode, with a documented reason and regression example.

This separation does not require two engines. It requires one engine with explicit rule/content profiles and audited extension points. Avoid a forest of undocumented per-function booleans; group changes into named, versioned patches with clear effects.

## 8. A more economical acceptance and development model

The current strictness solved real errors. The mistake would be either abandoning it wholesale or applying its most expensive form to every visible frame. Use layered acceptance, each with a declared claim:

| Layer | Required evidence | Appropriate scope |
|---|---|---|
| **L0: identity and execution** | Exact inputs, executable/source hashes, fixture origin, successful execution, nonzero tested coverage; skipped cases separately counted | Every test and receipt. Error/rollback equality can never count as success. |
| **L1: exact numeric/state contracts** | Original-instruction or original-run comparisons, boundary cases, RNG draws, mutation footprint and relevant ordering | Arithmetic, serialization, research/economy, targeting, collision and deterministic simulation mechanics. |
| **L2: behavior/scenario equivalence** | Original commands/observations, semantic checkpoints, win/loss, entity/resource outcomes, varied inputs | Complete gameplay families and missions; normalize pointers, not meaningful behavior. |
| **L3: playable product** | Human input, alternative strategies, autonomous AI/ecology, essential audio/feedback, save/reload, stability and progression | Real user-facing vertical slices. A scripted replay is supporting evidence only. |
| **L4: presentation equivalence** | Golden views where useful, visibility/picking alignment, readable UI, animation/audio events and timing tolerances | Renderers, scaling and audio. Pixel exactness is a diagnostic option, not a universal release requirement. |
| **L5: platform/mod contract** | Package validation, reproducibility, dependency resolution, deterministic replay, schema migration, invalid-mod handling | Custom scenarios, content and scripts; separate from faithful campaign certification. |

Accept renderer modernization when objects are correctly visible/selectable and cues remain faithful, without requiring identical rasterization or audio samples everywhere. Preserve exact simulation collision/projection boundaries when they determine a click target or outcome. Presentation tolerances must be set before reviewing a failure, not widened until a test passes.

Specific changes to workflow:

1. **Organize work by closed mechanics and their call dependencies.** For example, complete target selection across legitimate candidate counts, tie cases and ownership combinations, rather than add one accepted captured state. Preserve all corrected F24 evidence.
2. **Use a scenario coverage matrix across the campaign.** Sample representative genetics, ecology, construction, combat, AI, advanced technology and mission objectives through original-game access. Campaign progress remains an end-to-end gate, but no longer the sole source of requirements.
3. **Separate observed and synthetic fixtures.** Synthetic boundary tests are valuable for integer overflow and rare paths, but they cannot certify that a whole scenario is reachable in the original. Store provenance in machine-readable manifests.
4. **Make missing evidence visible.** Follow mgcarpet's strict-golden concept; do not copy the C++ port's success-on-missing-dumps convention into a release gate. Print executed/skipped counts and require the intended suite.
5. **Keep oracle independence explicit.** A bounded interpreter can mis-model an instruction or environment. Triangulate disputed families with original DOS observation when possible. Rust/headless agreement belongs to the host layer, not the original-equivalence column.
6. **Measure total throughput.** Track new supported mechanics, human-playable missions, scenarios covered, integration defects, capture cost and time spent maintaining guards. A larger frame number or more generated functions is not a product metric.
7. **Separate cheap tests from expensive conformance.** Run relevant family checks on edits; run provisioned original/native suites at meaningful integration points. Avoid repeatedly rebuilding everything when no affected behavior changed, while making release-level skipped evidence impossible to hide.

The prior 5913 divergence, rollback false positives and two-Rust-path comparisons are reasons to improve the evidence model, not reasons to forbid all abstraction. Exactness should protect outcomes and stateful rules; it should not require reproducing every internal renderer implementation before a player can complete a mission.

## 9. Smallest decisive experiments — not executed

Each experiment has a decision consequence. These are the remaining empirical gates of a completed architectural analysis, not substitutes for the analysis. They require later authorization because this assignment is read-only. Time estimates assume one experienced engineer, existing lawful game access and the current workstation; they include inspection and recording, not just coding.

| Option / question | Prerequisites and exact bounded procedure | Objective pass/fail and decision | Time/resources |
|---|---|---|---|
| **A — can whole-family Rust work escape frontier churn?** | Freeze the current working tree/evidence separately. Select one already-evidenced target-selection family encompassing the F24 accumulation behavior. Enumerate source branches and dependencies, implement the complete declared family in isolation, then run existing cases plus independently derived nearby and adversarial cases. Exercise it under two real input sequences in a mission. | **Pass:** all declared reachable branches execute correctly, RNG/state contracts agree, no new shape guard simply excludes valid neighbors, and the product gains a measurable interaction. **Fail:** mostly new narrow guards or unverifiable dependencies. A pass supports A/F; failure strengthens B/C investigation, not permissive acceptance. | **4–7 engineer-days**, existing Rust/oracle tools; no new engine. |
| **B — does the reported native bridge yield actual GeneWars play?** | Obtain authorized inspectable bridge source, exact binary identity and build/provenance instructions. Reproduce its build in a disposable checkout; trace service coverage. Use ordinary controls to complete Exceland and enter Krell, save/reload, and demonstrate an autonomous creature/AI interaction without invoking Rust gameplay. Record unsupported services and every original-code dependency. | **Pass:** reproducible gameplay and state continuation with no unexplained no-op services, plus a credible local/distribution boundary. **Fail:** unavailable source, only menu/callback success, corrupt state, or large unbounded service gaps. A pass can promote B to first product lane; unavailable source rules it out as a present dependency. | **5–10 engineer-days after source access**; x86 toolchain/emulator/reference assets. Source access waiting time excluded. |
| **C — is a portable lift cheaper for a complete family?** | Choose one coherent not-yet-complete gameplay family with tractable dependencies and an original oracle. Pin instructions/ABI; generate or manually derive a C/IR implementation, make dependencies explicit, compile for x64 and a second architecture in an isolated experiment, and compare ordinary/boundary cases and mutation/RNG footprints. Log translation, debugging and integration time. | **Pass:** exact declared behavior on both architectures, no unresolved calls/host pointers, reviewable state contract, and lower total cost than the comparable Rust family. **Fail:** UB/ABI emulation spreads across the engine or audit cost dominates. A pass warrants targeted lifting only; it does not certify whole-program conversion. | **5–10 engineer-days**, C/IR toolchain and cross-architecture runner when available. |
| **D — can original saves unlock advanced research?** | Execute the copy-only procedure in section 4: unchanged reload first, identify format, one research-lane mutation, observe effect and resave/reload. Then obtain one genuine later-game scenario or independently validated entry/carry. Capture commands and semantic state with edition identity. | **Pass:** repeatable original-run advanced mechanic independent of the Rust frontier, with no unexplained state corruption. **Fail:** original input/save access or format identity remains unresolved. A pass makes this the primary discovery lane; failure focuses the next effort on original access, not engine replacement by default. | **2–5 engineer-days**, pinned DOS environment, disposable saves and capture tools. |
| **E — is there one genuinely reusable foreign module?** | Select a bounded missing service or exact math helper with candidate correspondence, such as `LbSqrL`; do not start with the whole renderer. Map exact-edition instructions/tables and calling conventions, inspect the candidate implementation/license, compare a representative domain and boundaries, and estimate adapter/dependency cost against the current module. | **Pass:** equivalent defined behavior plus a net reduction in implementation/maintenance cost and usable license provenance. **Fail:** only matching names, incompatible tables/semantics, or no advantage over current Rust. A failure closes that reuse proposal; no need for a graphics prototype. | **1–3 engineer-days per service**, static evidence and existing numeric harness. |
| **F — can the current runtime support a useful package and complete state boundary?** | Define a minimal manifest and one external scenario using existing mechanics. Load it through the normal host without recompilation. Save at a supported boundary, restart the process, reload and compare continuation to uninterrupted play. Change one supported objective/technology parameter through a separately identified package and verify the faithful baseline is unaffected. | **Pass:** custom mission is playable, package identity persists in save/replay, dependency errors are explicit, state continuation matches, and no duplicate simulator was introduced. **Fail:** hidden host globals/hardcoded identities prevent resumption or mods bypass validated commands. A pass establishes the platform seam; failure identifies concrete state ownership work. | **5–8 engineer-days**, existing host/runtime, one original-data package and one small custom package. |

After F's data-mod pass, an optional **2–3 day script spike** can implement the same simple objective with one Wasm module and one Lua prototype, measuring authoring effort, deterministic replay, budget exhaustion and save-state handling. Select one or defer scripting. This spike must not block the first data-defined scenario.

Experiments may overlap in research preparation after approval, but integration should remain controlled. D and B have the greatest power to change near-term strategy; A/F measure whether the current investment can converge; C/E are bounded alternative suppliers. Stop an experiment at its limit and record the negative result rather than converting it into an unapproved engine project.

## 10. Shortest honest path to playable and moddable

The sequence below is a product path, not a promise to implement it now. Timeboxes for discovery do not replace acceptance. A minimum save is part of a real vertical slice; the later save milestone broadens durability and compatibility rather than postponing all saving until the campaign is finished.

| Milestone | Deliverable and acceptance | What it resolves |
|---|---|---|
| **0. Reliable research access and identity** | A pinned original mission can be controlled, saved, reloaded and observed; an edition/format manifest distinguishes QA and retail. One advanced mechanic is reachable independently of the current frontier. | Removes the serial discovery bottleneck and prevents cross-edition claims. |
| **1. One original mission a person can play** | Exceland from start through legitimate outcome with ordinary controls, readable UI, essential feedback/audio and a real supported-boundary save/reload. Two different sensible human strategies work. Add a contained Krell interaction exercising autonomous creature/hostile behavior before claiming combat/AI coverage. No pre-scripted victory. | Converts the existing accepted replay into an actual product slice. If B wins its gate, the same standard applies to that lane. |
| **2. Faithful multi-mission progression** | Exceland→Krell→the next mission with validated carry, research, roster, outcomes and loss/retry. Direct entry with a verified carry produces the same defined start state as progression. Human variation is included. | Establishes campaign flow as reusable logic rather than mission-specific patches. |
| **3. Durable saves and resume** | New-process reload matches uninterrupted continuation across construction, creatures, RNG, terrain jobs and campaign transitions; corrupt/missing-content saves fail safely. Save schema/version and mod identity are documented. Original-save import is separately gated by edition. | Makes longer play and reproducible research practical. Raw callback dumps never satisfy this milestone. |
| **4. One authored/imported custom mission** | A creator edits terrain/placements/objectives using documented tools, validates it, launches it in the normal game and reaches its intended outcome. No engine recompile or fixture editing. Unsupported original import fields are disclosed. | Proves the scenario platform, not just a map viewer. |
| **5. One independently installable mod** | A small package changes a supported creature/technology definition or adds a mission; dependency/hash identity, uninstall, save/replay behavior and error handling work. A second person can follow the documentation. | Proves a usable creator contract. Data-only is sufficient initially. |
| **6. Full original campaign breadth** | All 24 missions have mechanic coverage, faithful-profile results, ordinary human completion or documented playtest coverage, save/reload and progression. Audio/UI/movies needed for comprehension and flow work. Missing AI/ecology families cannot be hidden by scripted paths. | Meets the core GeneWars product goal. Content counts and one deterministic route are insufficient. |
| **7. Extended creatures, skirmish and sandbox** | Custom AI and objectives, configurable factions/economy/terrain, new trait/behavior support and a usable creator preview. Skirmish opponents act autonomously; sandbox is explicitly custom. Multiplayer, if pursued, has a defined integrity/compatibility model. | Expands beyond the historical campaign without contaminating its verification. |

Milestones 4–5 can proceed alongside campaign breadth once the relevant mechanics are stable. They should not require implementing arbitrary new creature behaviors first. Editor work earns priority when it improves researcher scenario production and creator usability together. Full public distribution additionally needs reproducible packaging, dependency/license notices and user-owned asset import.

## 11. Source, revision and legal provenance appendix

### Scope of the source audit

The repository index below records current default-branch heads observed during the assignment. File links elsewhere in this report pin those revisions. Public README feature claims are **C** unless supported by an inspected mechanism (**I**) or a separately scoped recorded result (**R**). A recent commit date is activity evidence, not maturity. No repository was built locally. CI runs were read from GitHub, not rerun.

Licensing findings describe the inspected grants and ambiguities; they do not conclude that every reverse-engineered artifact is legally distributable in every jurisdiction. Before publishing lifted original instructions or a combined distribution, obtain appropriate rights/provenance review. This is a release gate, not a reason to stop read-only research or preserve unhelpful architecture.

### Repository revision index

| Repository / default branch | Observed HEAD and commit date (UTC) | Key inspected evidence / trust / license |
|---|---|---|
| [mefistotelis/genewars](https://github.com/mefistotelis/genewars/tree/72ec4c4908d26304dd373132cb3187c855d4bf58) / `main` | `72ec4c4908d26304dd373132cb3187c855d4bf58` · 2022-04-26 | I: class/field reconstruction, mostly stubs; GPLv2-or-later source headers. |
| [mefistotelis/genewars-saveedit](https://github.com/mefistotelis/genewars-saveedit/tree/3b754f7d576c3b54d3fc34052487cc4fd7d7440d) / `main` | `3b754f7d576c3b54d3fc34052487cc4fd7d7440d` · 2021-10-19 | I: structures, menus and unsafe fixed-size I/O; GPLv2-or-later program header. |
| [mefistotelis/genewars-re-helpers](https://github.com/mefistotelis/genewars-re-helpers/tree/a85d5c517b524aa4906a720330ed1435bf8b65cf) / `main` | `a85d5c517b524aa4906a720330ed1435bf8b65cf` · 2026-09-21 | I: MAPs/README/tree; XML only partly accessible; original binaries not executed; rights vary/unresolved. |
| [turican0/remc2](https://github.com/turican0/remc2/tree/a0b9feb7808b3df2b9445949fbe706d8b2d43a19) / `development` | `a0b9feb7808b3df2b9445949fbe706d8b2d43a19` · 2026-10-16 | I: terrain/render/regression code; R: current-head CI; C: gameplay claims; mixed license findings. |
| [thobbsinteractive/magic-carpet-2-hd](https://github.com/thobbsinteractive/magic-carpet-2-hd/tree/ebd02b32039958a0a3702a6ef3f1fb353eb184ac) / `development` | `ebd02b32039958a0a3702a6ef3f1fb353eb184ac` · 2026-10-15 | I: fork/configuration/render sources; R: current-head CI; mixed license findings. |
| [raincz/mgcarpet](https://github.com/raincz/mgcarpet/tree/9a82c640cc4fa5921f67e328588ff4ecc02e2052) / `master` | `9a82c640cc4fa5921f67e328588ff4ecc02e2052` · 2026-10-18 | I: formats/snapshot/mod/conformance code; R: dated report and CI; C: campaign claims; GPLv3-family. |
| [jessperler88/Magic-Carpet-port](https://github.com/jessperler88/Magic-Carpet-port/tree/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036) / `main` | `18cbfd5da0b3d7fc5481595b0b6c9b32abd58036` · 2026-10-08 | I: reference/save/scenario/types; C: campaign claims; GPLv3-or-later stated. |
| [jopadan/bullfrog](https://github.com/jopadan/bullfrog/tree/be5ccc6e32f25a9121a453eb06f73b767db4bd93) / `main` | `be5ccc6e32f25a9121a453eb06f73b767db4bd93` · 2023-11-23 | I: math/RNC/SDL/test code; GPLv3 root and GPLv2-or-later headers. |
| [swfans/syndwarsfx](https://github.com/swfans/syndwarsfx/tree/49502ca5bd66de9b3bebcfd74a1bb8905bb1d807) / `master` | `49502ca5bd66de9b3bebcfd74a1bb8905bb1d807` · 2026-10-18 | I: mixed assembly/C build and exclusions; current-head CI was in progress; GPL plus unlicensed-original exceptions. |
| [swfans/syndicatfx](https://github.com/swfans/syndicatfx/tree/36320614c4c76363a9d225ab5022123ca70cfd83) / `master` | `36320614c4c76363a9d225ab5022123ca70cfd83` · 2025-12-17 | I: Watcom wrappers and COPYING; GPL plus original-code exceptions. |
| [turican0/remc1](https://github.com/turican0/remc1/tree/65b9d026bf85cc2d7277e50e2d6c15538e68545e) / `main` | `65b9d026bf85cc2d7277e50e2d6c15538e68545e` · 2023-03-21 | I: tree/large recoding and regression helper; historical reference; GPLv3-family. |
| [turican0/remc1hw](https://github.com/turican0/remc1hw/tree/c23fdee46135ac5e8182b458e64596fbeccc0fda) / `main` | `c23fdee46135ac5e8182b458e64596fbeccc0fda` · 2023-03-12 | I: tree/recoding structure; historical Hidden Worlds reference; GPLv3-family. |
| [turican0/MagicBalls](https://github.com/turican0/MagicBalls/tree/a017321853d3c169c41c0f473390a0e01b418f53) / `main` | `a017321853d3c169c41c0f473390a0e01b418f53` · 2026-10-04 | I: README, GDExtension binding and project configuration; C: playability; root GPLv3, dependencies separate. |
| [michaelhoward/MagicCarpetFileFormat](https://github.com/michaelhoward/MagicCarpetFileFormat/tree/41981e796cf96655533bf584510a76d2442c4fea) / `master` | `41981e796cf96655533bf584510a76d2442c4fea` · 2026-04-04 | I: small format-research tree/document; no reuse license established. |

### Concrete license implications

- **GeneWars C++ reconstruction:** inspected source headers use GPLv2-or-later. Its third-party/original material remains separately attributable. Public availability does not make it official source.
- **Save editor:** `src/globals.h` states GPLv2-or-later for the program. Retain attribution and audit individual files when reusing code; this grant does not license game saves/assets. Prefer a new safe writer over preserving its dangerous I/O design. [Header][save-license]
- **Helpers and public attachment:** no comprehensive reuse grant was established for the original binaries and all accompanying tools/metadata. The ZIP has no license entry. Read as research evidence; clarify rights before copying tools or redistributing contents.
- **Bullfrog library:** root GPLv3 text and GPLv2-or-later source headers; choose a compatible licensing path and retain file-level notices. It is a reconstruction library, not evidence of an official engine release.
- **SyndWarsFX/SyndicatFX:** GPL port code with explicit exceptions for original disassembled instructions. Those exceptions are unresolved rights, not permissive licenses. [COPYING][sw-license]
- **remc2/HD:** README MIT wording for additions conflicts with an oversimplified “whole repo MIT” interpretation because GPLv3 text, original-code notices and independently licensed dependencies are present. Determine the intended grant for each imported file and its dependency closure before copying. [License][remc-license]
- **mgcarpet / Magic-Carpet-port / remc1 / remc1hw / MagicBalls:** GPLv3-family project licenses; inspect “or later” grants and third-party notices at the relevant files rather than assuming every asset shares the root license. mgcarpet's RNC source explicitly identifies its upstream derivation. Do not remove that chain. [mgcarpet license][mgc-license], [C++ license][mcport-license]
- **SDL2:** zlib license; SDL 1.2 has different terms. A host-library license says nothing about game code or assets. [SDL license](https://www.libsdl.org/license.php)
- **Local Rust project:** the inspected workspace manifest has no explicit workspace license declaration. Do not advertise a settled permissive/open license until ownership, contributor provenance, copied code and tables have been inventoried.

GPL-covered code incorporation can bring corresponding-source, notice and distribution obligations to a combined work; assess the actual integration. A process boundary or plugin label is not an automatic licensing exemption. Rust translation does not erase a copied implementation's provenance. Conversely, studying interfaces and writing independently reviewed behavior specifications is different from copying an entire derived source file.

Ship engine/tools and schemas separately from user-owned game assets. The importer can identify known editions, read local data and produce a private cache; it should not embed the original executable, music, art, maps or extracted proprietary tables in a public source release. Public test fixtures should be synthetic or independently distributable, with private original conformance inputs provisioned separately. User-generated mods need their own licenses and must not silently bundle the original game's content.

## 12. Handoff decision brief for Hermes/Codex

**Remain frozen now:** the live checkout and all historical dirty/untracked work; HEAD and branch; fixtures and expected outputs; source candidate and executable hashes; Ghidra project; private captures and integration receipts; current 6119 frontier; unfinished loss/escape work; Hermes director/worker services. This report does not authorize any restart, build, mutation, branch, prototype or external contact.

**One future approval should cover a bounded direction change:** adopt D+F as the working architecture and replace frontier-only acceptance with the layered model; authorize experiments D and F, a bounded A family completion, and read-only acquisition/assessment of the reported B bridge if made available. Specify separate disposable work areas and preserve the frozen baseline. B/C/E implementation experiments can be included in that approval with their stated time limits, or selected after the first results. No further permission is needed merely to read this report or review its conclusions.

After that approval, useful parallel responsibilities are original-game input/save access; format/inspector/scenario design; one complete gameplay family; and native-bridge/provenance assessment. Assign state ownership and immutable evidence identities before integration. Parallel work must not edit the same fixtures or silently alter expected outputs. This is a proposed future allocation, not a worker launch.

**Do not do yet:** abandon Rust; import a Magic Carpet renderer wholesale; treat public symbols as QA addresses; run the old editor on real saves; publish original assembly or assets under a port's license; call a private bridge a finished game; restore every callback capture as a full save; or relax guards solely to advance a frame counter.

The next decision is therefore concrete: **retain the existing work, change the discovery and verification model, and test the few alternatives that can actually shorten delivery.** Promote native ASM to the first playable lane only if its whole-mission/save gate succeeds. Prefer selective portable implementations over a wholesale lift. Build the mod/scenario/save boundary around one authoritative simulation, and measure progress by human-playable mechanics and missions.

[gw-code]: <https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/README.md>
[gw-computer]: <https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/src/computer.cpp>
[gw-editor]: <https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/src/ifceditor.cpp>
[gw-custom]: <https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/src/ifccustom.cpp>
[gw-player]: <https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/include/player.hpp>
[gw-grid]: <https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/include/gridtile.hpp>
[gw-helpers]: <https://github.com/mefistotelis/genewars-re-helpers/blob/a85d5c517b524aa4906a720330ed1435bf8b65cf/README.md>
[gw-map-beta]: <https://github.com/mefistotelis/genewars-re-helpers/blob/a85d5c517b524aa4906a720330ed1435bf8b65cf/symbols/gw-genewars-beta2-jul_1996.map>
[gw-map-retail]: <https://github.com/mefistotelis/genewars-re-helpers/blob/a85d5c517b524aa4906a720330ed1435bf8b65cf/symbols/gw-genewars-1_1-nov_1996.map>
[save-c]: <https://github.com/mefistotelis/genewars-saveedit/blob/3b754f7d576c3b54d3fc34052487cc4fd7d7440d/src/gwsaved.c>
[save-readme]: <https://github.com/mefistotelis/genewars-saveedit/blob/3b754f7d576c3b54d3fc34052487cc4fd7d7440d/readme_gwsaved.txt>
[save-license]: <https://github.com/mefistotelis/genewars-saveedit/blob/3b754f7d576c3b54d3fc34052487cc4fd7d7440d/src/globals.h>
[remc-readme]: <https://github.com/turican0/remc2/blob/a0b9feb7808b3df2b9445949fbe706d8b2d43a19/README.md>
[remc-terrain]: <https://github.com/turican0/remc2/blob/a0b9feb7808b3df2b9445949fbe706d8b2d43a19/remc2/engine/Terrain.cpp>
[remc-interface]: <https://github.com/turican0/remc2/blob/a0b9feb7808b3df2b9445949fbe706d8b2d43a19/remc2/engine/GameRenderInterface.h>
[remc-regression]: <https://github.com/turican0/remc2/blob/a0b9feb7808b3df2b9445949fbe706d8b2d43a19/remc2-regression-test/regression-tests.cpp>
[remc-license]: <https://github.com/turican0/remc2/blob/a0b9feb7808b3df2b9445949fbe706d8b2d43a19/docs/LICENSE>
[hd-readme]: <https://github.com/thobbsinteractive/magic-carpet-2-hd/blob/ebd02b32039958a0a3702a6ef3f1fb353eb184ac/README.md>
[hd-license]: <https://github.com/thobbsinteractive/magic-carpet-2-hd/blob/ebd02b32039958a0a3702a6ef3f1fb353eb184ac/docs/LICENSE>
[mgc-readme]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/README.md>
[mgc-fidelity]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/docs/FIDELITY.md>
[mgc-snapshot]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/crates/mgc-sim/src/snapshot.rs>
[mgc-save]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/crates/mgc-formats/src/mgcs.rs>
[mgc-suite]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/crates/mgc-conform/tests/suite.rs>
[mgc-conformance]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/docs/CONFORMANCE-REPORT.md>
[mgc-modding]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/docs/MODDING.md>
[mgc-rnc]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/crates/mgc-import/src/rnc.rs>
[mgc-license]: <https://github.com/raincz/mgcarpet/blob/9a82c640cc4fa5921f67e328588ff4ecc02e2052/LICENSE>
[mcport-readme]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/README.md>
[mcport-types]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/src/mcengine/mc_types.h>
[mcport-ref]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/src/tests/reference_test.cpp>
[mcport-render-ref]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/src/tests/render_reference_test.cpp>
[mcport-save]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/src/mcengine/savegame.cpp>
[mcport-scenario]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/src/mcengine/scenario.cpp>
[mcport-license]: <https://github.com/jessperler88/Magic-Carpet-port/blob/18cbfd5da0b3d7fc5481595b0b6c9b32abd58036/LICENSE>
[bflib]: <https://github.com/jopadan/bullfrog/blob/be5ccc6e32f25a9121a453eb06f73b767db4bd93/README.md>
[bflib-test]: <https://github.com/jopadan/bullfrog/blob/be5ccc6e32f25a9121a453eb06f73b767db4bd93/tests/bflib_test_math.c>
[sw-build]: <https://github.com/swfans/syndwarsfx/blob/49502ca5bd66de9b3bebcfd74a1bb8905bb1d807/src/Makefile.am>
[sw-config]: <https://github.com/swfans/syndwarsfx/blob/49502ca5bd66de9b3bebcfd74a1bb8905bb1d807/configure.ac>
[sw-license]: <https://github.com/swfans/syndwarsfx/blob/49502ca5bd66de9b3bebcfd74a1bb8905bb1d807/COPYING>
[syndicat-wrappers]: <https://github.com/swfans/syndicatfx/blob/36320614c4c76363a9d225ab5022123ca70cfd83/conf/wrappers_libc.conf>
[remc1]: <https://github.com/turican0/remc1/tree/65b9d026bf85cc2d7277e50e2d6c15538e68545e>
[remc1hw]: <https://github.com/turican0/remc1hw/tree/c23fdee46135ac5e8182b458e64596fbeccc0fda>
[magicballs]: <https://github.com/turican0/MagicBalls/blob/a017321853d3c169c41c0f473390a0e01b418f53/README.md>
[magicballs-bind]: <https://github.com/turican0/MagicBalls/blob/a017321853d3c169c41c0f473390a0e01b418f53/gitextension/godot-cpp-template/src/MBEXmain.cpp>
[mc-format]: <https://github.com/michaelhoward/MagicCarpetFileFormat/blob/41981e796cf96655533bf584510a76d2442c4fea/readme.txt>
[gw-issue]: <https://github.com/mefistotelis/genewars/issues/1>
[gw-bridge]: <https://github.com/mefistotelis/genewars/issues/1#issuecomment-5468089134>
[gw-pack]: <https://github.com/user-attachments/files/31619787/gwfx_upstream_pack_share.zip>
[remc-ci]: <https://github.com/turican0/remc2/actions/runs/38858530114>
[hd-ci]: <https://github.com/thobbsinteractive/magic-carpet-2-hd/actions/runs/38809801130>
[mgc-ci]: <https://github.com/raincz/mgcarpet/actions/runs/39086001000>
[manual]: <https://genewars.lubiki.pl/docs/genewars_manual.htm>
[editor-site]: <https://genewars.lubiki.pl/html/gwedit_main.php>
[patch-site]: <https://genewars.lubiki.pl/html/gwedit_patches.php>
[local-state]: <genewars-preflight/docs/CURRENT_STATE.md>
[local-f24]: <genewars-preflight/research/integration/f24-b12-b0-closed-checkpoint-20261018.json>
[local-f24-private]: <archive/private-research/content/f0b25c9ea1cc643452f0537302fcf6e9d84bc67871bc9f8ffc08bffdf48a59d4.json>
[local-f22]: <genewars-preflight/research/integration/f22-b11-ringgate-short-side-checkpoint-20261018.json>
[local-cargo]: <genewars-preflight/rust/Cargo.toml>
[local-actor]: <genewars-preflight/rust/crates/genewars-runtime/src/actor_world.rs>
[local-creature]: <genewars-preflight/rust/crates/genewars-runtime/src/creature_world.rs>
[local-desktop]: <genewars-preflight/rust/crates/genewars-desktop/src/bin/genewars-game.rs>
[local-features]: <genewars-preflight/docs/GAME_FEATURE_MATRIX.md>
[local-mission]: <genewars-preflight/rust/crates/genewars-assets/src/mission_source.rs>
[local-player]: <genewars-preflight/rust/crates/genewars-runtime/src/player_state.rs>
[local-rng]: <genewars-preflight/rust/crates/genewars-runtime/src/blueprint.rs>




# CODEX DEEP RESEARCH HANDOFF — GeneWars as a modern, open, moddable platform

**Research-only architectural decision assignment — 18 October 2026**

## Mission

Reconsider the GeneWars reconstruction **from first principles**, using the existing Rust work, independent GeneWars research projects, original-game save/level editing, reverse-engineering artifacts, and the possible Magic Carpet engine lineage. The target is **a finished, playable, modern, openly developed and genuinely moddable GeneWars platform**, not an endless supply of exact callback tests.

**Do not assume Rust must be retained as the sole runtime. Do not assume it should be discarded. Do not assume an x86 assembly route is automatically faster. Do not assume “built on Magic Carpet” means compatible source or drop-in engine code.** Investigate and then recommend an evidence-based strategy that salvages everything useful and avoids needless duplication.

Deliver a *completed research report and a concrete architecture decision*, not another plan to do research later. This task is **read-only with respect to the GeneWars project and its services**. Do not restart Hermes, launch its workers, modify Rust, edit fixtures, build binaries, change the active Ghidra project, create new Git branches or commits, overwrite saves, or silently substitute a different implementation. You may inspect public repositories, read local source and receipts, perform lightweight static analysis in isolated temporary space, and create your final report **outside the working game checkout**. Do not run untrusted external executables or redistribute proprietary game material. No contact with external maintainers or public PRs without a further instruction.

This is a deliberate pause for architectural reevaluation. **No implementation is authorized yet.**

## Desired final product — criteria, not a prescribed engine

We want:

1. **Actually playable GeneWars:** the full original 24-mission experience, normal human controls, AI, ecology, genetics, construction, presentation, outcomes, save/load, audio and other essential systems; faithfully reproduced original rules as a selectable baseline where evidence supports them.
2. **A modern application:** maintainable, reasonably portable, supported Windows first if priorities must be chosen, with sensible resolution scaling, controls, stability, save support, diagnostics, accessibility and optional quality-of-life improvements that do not silently rewrite the authentic campaign.
3. **A modding *platform*, not just a port:** creators should eventually be able to add and edit planets, missions, maps/terrain, creatures/hybrids, traits, plants, buildings, research/technology, AI behaviors, events and objectives; support user-made scenarios, skirmish, and sandbox as explicitly labeled extensions.
4. **An open, sustainable codebase:** documented, reproducible builds, transparent formats, publicly usable mod tooling and clear separation of engine code from third-party copyrighted assets. Identify and respect actual licenses and binary-distribution constraints.
5. **Research acceleration:** the ability to study later-game mechanics using original save files, direct mission entry, original-game observation, source symbols, or independent scenario fixtures instead of advancing one callback at a time through Krell.
6. **Reliable provenance:** source behavior, emulator-observed behavior, reconstructed-instruction oracles, artificial/derived scenarios, and new design choices must remain distinguishable. Correctness must not be simulated by permissive guards, changed expectations, copied fixtures or matching post-error rollback bytes.

Consider additional goals or architecture patterns the project has overlooked. Identify genuine trade-offs rather than claiming all goals can be met equally quickly.

## Verified background and current stopping point — recheck, do not assume current

Our Hermes/DeepSeek director with a pinned GLM worker recently reconstructed GeneWars gameplay in native Rust using Ghidra, a bounded Python interpreter of original x86, exact game-state comparisons, deterministic scripted campaign replays, and native/headless framebuffer agreement. The last **reported** state at deliberate stop was:

- Repository: `C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight`; Git parent `C:/Users/fallo/Documents/ChatGPT/Freecad`; branch `codex/genewars-reconstruction`.
- Reported HEAD `3bcf6ff`, 18 October 2026; Hermes processes stopped; nothing staged. Historical dirty/untracked files and private captures must be preserved.
- Exceland accepted (1/24), Krell unfinished. Source-corrected campaign reaches 6119 with focused checks; native acceptance confirmed at 6100, **6119 native parity pending** at stop.
- Family checker last reported 60 positive, 6 negative, 73 green checks; `Creatures(UnsupportedNeighborhood)` at 6119 remains unmeasured.
- Source candidate currently identified as QA BETA V5, SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; equivalence to retail releases unresolved.
- The eight-buffer 427,369-byte captured callback format is useful, **but not a complete saved running game**; runtime context, bindings, scheduler cursor, host/UI state and terrain jobs may be absent.
- Existing Codex report `GENEWARS_SCENARIO_ENGINE_ARCHITECTURE_REPORT.md` identified a viable limited callback-scenario laboratory and a narrower possibility of direct Krell entry with a verified Exceland carry. This is a starting hypothesis, not your mandated answer.

Read **the current checkout and actual `docs/CURRENT_STATE.md`, receipts and code** first. Identify new work or a different HEAD and distinguish it from this dated handoff. Do not turn a reported success into a fresh test claim. Check which evidence is retained under `private/`, what is reproducible, and what is unavailable in a clean checkout.

### Prior false-positive and productivity lessons

Some exact-world comparisons only proved a source interpreter agreed with Rust *after the Rust callback rejected and rolled back*; the gate was hardened to require successful execution before byte equality. Some renderer/headless native checks compare two Rust routes, not directly to DOS GeneWars. An original interpreted instruction store exposed a defect in an earlier accepted Rust timeline. A narrowly admitted guard could also reject a legitimate nearby branch; several failures were port-authored rather than original behavior. Preserve these lessons, and ask whether there is a more economical fidelity/acceptance architecture than requiring every intermediate frame to be fully source-exact **without hiding user-visible wrong behavior**.

## Mandatory external project inventory

Inspect repositories, their **actual relevant files and current default-branch HEADs**, activity, documentation, licenses and build/test evidence. Do not rely on README claims or stars alone.

### Direct GeneWars ecosystem

- https://github.com/mefistotelis/genewars — reverse-engineered C/C++ class, data, method, renderer/engine and UI organization; README says most functions are unimplemented and it does not display a game. Check which source files contain meaningful implementations versus stubs, and any class/offset/table similarities to our exact Rust reference build.
- https://github.com/mefistotelis/genewars-saveedit — historical GeneWars save editor. **Inspect `src/gwsaved.c`, structures, fields, file I/O, import/export, supported game editions and existing tests.** It recognizes four players, 25 creature types, specialists, plants, underground GOOP, creature stats and terrain. Its campaign/custom level edit menu commands appear unfinished; verify rather than assuming it is a working mission editor.
- https://github.com/mefistotelis/genewars-re-helpers — publicly shared original-build symbols, MAP/XML/bytes, Beta2 and retail v1.1 builds, Watcom tooling, and ongoing symbol recovery. Trace provenance and versions. Do **not** transplant an address from Beta2 or v1.1 into QA BETA V5 without verified rebasing and ABI matching.
- https://github.com/mefistotelis/genewars/issues/1 — substantial 2026 community discussion, including the original-assembly-plus-modern-I/O strategy, a second contributor's reported Windows-native ASM/Watcom bridge, DOSBox-X instrumentation, binary-first RE work and save-parser evidence. Inspect every relevant comment and public attachment **if accessible through authorized means**; separate firsthand reproducible material from private-repo or self-reported progress. Do not assume the private collaboration source is available or working. Identify minimal respectful questions for its maintainers only as a recommendation, not outreach.

### Magic Carpet and Bullfrog common technology — major research thread

GeneWars is often credited as using the **Magic Carpet engine**. Investigate what this actually means for **code genealogy**, not just game credits or visual similarity. A shared terrain/renderer, shared math and compression libraries, or common development tools are not equivalent to a shared gameplay engine.

Research at least:

- https://github.com/turican0/remc2 and https://github.com/thobbsinteractive/magic-carpet-2-hd — C/C++ reverse-engineering of Magic Carpet 2; check working features, reused SDL/modern I/O, graphics, terrain, input, and provenance. Distinguish forks.
- https://github.com/raincz/mgcarpet — modern **Rust** implementation of Magic Carpet 1, Hidden Worlds and MC2 (wgpu/winit); inspect how it separates simulation, rendering, game assets, scenario entry, configuration, saved games and fidelity tests. Are there reusable techniques or *actual modules* compatible with GeneWars?
- https://github.com/jessperler88/Magic-Carpet-port — C++17/SDL2 decompilation/port claiming tick/pixel-exact original reference comparisons, debug inspector, scenario scripts and save-anywhere; assess verifiable mechanisms, portability, and license.
- https://github.com/jopadan/bullfrog — historical Bullfrog emulation library; determine whether it has substantial reusable implementations or mostly interfaces.
- https://github.com/swfans/syndwarsfx — original-assembly disassembly port with SDL; architecture precedent for preserving original mechanics and gradually replacing routines. Distinguish practical porting technique from GeneWars-compatible game code.
- Related `remc1`, `remc1hw`, `MagicBalls`, `Magic Carpet File Format` research and other relevant Bullfrog projects discovered during the investigation. Avoid a long irrelevant directory of links; explain their specific use.

**Answer plainly:** Is any *official, authentic Bullfrog Magic Carpet engine source code* publicly available under a legitimate reuse license? If not verified, say so; identify available decompilations/reimplementations and how they differ. What code or data overlap between Magic Carpet and GeneWars is **demonstrated**, probable, merely reported, or refuted? Look for symbol names, compiler/build structure, binary fingerprints, terrain interpolation, world data structures, palette/draw routines, RNG, RNC, shared `bflibrary`, networking/sound input layers, relevant functions, call graphs and resource naming. Where possible give **specific side-by-side source/offset evidence** and compatible edition identity. Attribute licenses per module: do not conflate a repo's code license with permission to distribute original game assets or lifted executable instructions.

## The saved-game editor and original-running-game research route

Give this its own thorough chapter. Do not relegate it to an appendix.

1. Reconstruct what `genewars-saveedit` **really** supports: original `.sav` locations; `SAVEDATA` packing/size; four player records, creature records, 128×128 terrain, growth/reveal/underground resources, genetics/research levels, available species/plants, specialists, native organisms, campaign-level byte(s), partial import/export, menu paths that are unimplemented, missing validation.
2. Explicitly adjudicate the **343,935-byte size calculated from the editor's C declarations** versus the **343,958-byte mission/capture backing** in our Rust materials; their matching 1,805-byte player, 116-byte packed creature, and 14-byte terrain-cell contracts matter. Compare by format and edition, include struct packing/alignment, magic/header, active-vs-inactive records, offsets, serialization extent and checksum/compression as applicable. Do not declare the layouts interchangeable until proved. Never write to an original save as an experiment.
3. Compare against the GeneWars contributor's claimed byte-exact **343,958-byte save parser** and live DOSBox-X save/capture work, including the reported unaligned genetics-research lanes (`+0x61A`, 32-bit `>>11`) and 50/75/100% research caps. Independently corroborate with inspectable code or source/binary evidence if accessible; distinguish contributor reports and their superseded claims from current verified facts.
4. Determine whether editing an **original-game save on a copy** can reliably bypass early campaign unlocks to observe advanced specialists, hybrids, AI, construction and tech *in the actual DOS game*, even when current Rust cannot run the corresponding mission. Separate a genuine original save accepted by DOS from a synthetically mutated snapshot and from a Rust callback fixture. Identify the smallest safe, falsifiable validation experiment; **do not execute it in this read-only assignment**.
5. Explore whether an extensible **modern save inspector/level editor** could serve both researchers and mod creators, with exact format identities, unknown-byte preservation, staged diffs, rollback and safe versioning. Distinguish research tooling, game save/import, mission generation and full portable game-state serialization.
6. Identify legitimate original level editor and `IFCEditor`/`IFCCustom` affordances. Investigate original custom maps/scenarios and multiplayer options; do not assume either a full single-player skirmish mode or that the C++ class stubs are functional.

## Technical architectures to compare without favoritism

Go beyond these if better alternatives arise. For each option state *which existing work survives*, first verified playable outcome, likely hardest blocker, modding story, maintenance/portability, source authenticity, legal concerns, and failure signals.

**A. Retain Rust as the sole native engine**, but stop using only campaign frontier for discovery: original save-game observation, scenario manifests, direct verified mission entry, reusable behavior-family mapping, source/data tables, and a modern modding shell. Evaluate whether this will actually converge in reasonable time.

**B. Native x86 assembly-based GeneWars execution** with DOS/platform services replaced using SDL and suitable Bullfrog libraries, gradually rewriting game routines into C/C++ or Rust. Evaluate the actual GeneWars proof of concept (not only SyndWarsFX), ABI/calling conventions, x86 limitations, licensing and options for portable modern builds.

**C. Decompile/lift original gameplay into portable C/C++ or intermediate representation**, retain testable original behavior, then extract modular interfaces. Evaluate automated decompilation reliability, Ghidra/p-code/etc. evidence, original data layout, wrong semantics and cost.

**D. Hybrid reference path:** use native original ASM/emulator or original DOS executable as an **executable oracle/prototyping lane**, and use Rust as the long-term open game. Assess whether the reference helps enough to justify dual maintenance, or whether it can be archived after extracting evidence.

**E. Leverage compatible components of Magic Carpet ports** (terrain projection, renderer, shared libraries, asset readers, input/audio), only where a concrete common interface is proved; don't make a graphics-layer analogy into gameplay compatibility.

**F. An alternative that is actually better**, including a mixed-language native engine, game-data-driven simulation, host-plus-sandboxed mods, phased compatibility mode, or other viable solution. Do not recommend new middleware because it is fashionable.

**Do not score an option higher merely because it can reach a menu, compile a binary, run a synthetic callback or draw attractive graphics.** Distinguish original-game execution, visible but scripted Rust replay, autonomous complete mission play, and real human-playable/interactive gameplay. Establish a consistent definition of a working vertical slice.

## Moddability must drive engine design

Research concrete technology choices, not vague “plugin support.” Specify a minimum feasible and eventual comprehensive mod surface:

- Content package/discovery/versioning/dependency and conflict rules; modular definitions for creatures, traits/genes, building technology, plants/biomes, missions/planet maps, objectives, AI/effects and UI.
- Data-driven editing versus sandboxed scripts versus native plugins: safety, determinism/replays, performance, stability, multiplayer integrity, licensing and realistic authoring skills.
- In-game and standalone editors: map/planet editor, scenario editor, gene/creature designer, research/economy configuration, test-and-preview, validation, diff/undo, save import/export.
- **Two clearly separated rule spaces:** faithful original campaign behavior and opt-in modern/custom modes. Reproduce original quirks in compatibility mode, while allowing mods/QoL without contaminating campaign verification.
- Modern runtime architecture: simulation time vs frame rate, renderer swap, native input, sound, save/load, debugging and replay; avoid hardcoding campaign and original resource identities into core APIs.
- Distribution: engine/tool source versus user-supplied original EA/Bullfrog data, allowed licenses, original-binary-derived code questions. Identify **specific** repo license obligations, source-file header exceptions and uncertainty requiring counsel or maintainer clarification.

Research how the Magic Carpet projects actually expose equivalent capabilities, especially debug scenarios, save-anywhere, data baking, fidelity toggles and rendering independence. Reuse *ideas* where code copying is legally or technically incompatible.

## Research method and standards

- Start with live local repository status, read-only. Map real crates/modules/call sites and existing evidence. Preserve work and services untouched. If local paths are inaccessible, say so; do not invent measurements.
- Conduct meaningful **external** research: code files, revision/tag pinning, relevant issues/PRs, licenses, technical docs and historical engine provenance. Follow cross-project references and discover important alternatives the above list misses.
- Prefer primary sources, but challenge contributor assertions with reproducible artifacts. For every significant claim give URL/path, symbol/function and SHA/commit/date where available. Split confirmed implementation, recorded-but-unrerun test, maintainer claim, plausible inference and unknown.
- Keep edition/version compatibility explicit: QA BETA V5, Beta2 July 1996, retail 1.0 September 1996, patched 1.1 November 1996, Magic Carpet 1/2, etc. Addresses/struct layouts may not carry across builds.
- Treat **the current Rust work as an asset with salvage value**, including parsers, models, byte-comparison harness, input host, terrain/rendering, source-verified behavior, campaign carry and fixtures. Give component-level keep/adapt/replace judgments rather than “rewrite everything.”
- Estimate costs and timelines as ranges or relative effort based on evidence, with uncertainties. Measure total verification/integration/rework cost, not just cheap AI tokens. A buildable repo or 100% source-file count is not proof of feature completion.
- **No hidden substitution** of a faster but non-equivalent goal. If you recommend a different final architecture or changing the authenticity target, state exactly which requirements would be lost and require a separate decision before any implementation.
- Don't demand maximal byte-perfect behavior of presentation or audio when an appropriately justified equivalence goal serves playability better, but never loosen numeric source requirements silently. Propose a layered acceptance model with clear labels for precise source comparison, behavior, playable product and mod functionality.
- A negative result is valuable if it rules out an attractive but unsupported approach. Report contradictions and retractions visibly.
- Do not spend this run doing a large build, spawning agents, modifying the game, or trying to make a prototype. This is comparative research and a platform decision.

## Required final deliverable

Write one complete Markdown report **outside the live project checkout** called `GENEWARS_PLATFORM_ARCHITECTURE_RESEARCH_REPORT.md` and present it to the user. The report must contain:

1. **Executive verdict** in plain language: recommended architecture now, second-best alternative, what decision is premature, and what critical evidence could change the recommendation.
2. **What we can actually prove today**: live Rust baseline, other GeneWars project maturity, save editor abilities/limitations, publicly available original RE symbols, reports from the native-ASM collaboration, Magic Carpet implementations and original-engine lineage. Link all decisive evidence.
3. **Genealogy and reuse map** from Magic Carpet/Bullfrog projects into GeneWars: actual overlap vs superficial similarity; tables of reuse targets, dependencies, known version issues and licenses.
4. **Save/editor and later-game access study**: format reconciliation; a safe original-save experiment design; a route to editable scenarios; limitations of 427,369-byte callback fixtures as whole saves.
5. **Deep comparison table** (Rust, ASM, lifted C, hybrids and any novel candidate): route to *human-playable* original campaign, high-quality modding, source fidelity, portability, original-assets and code licensing, original-reference observability, total maintenance and relative development effort. Explicit evidence/confidence for each rating.
6. **Proposed target architecture** (clear text diagram) with source-faithful simulation, asset/resource readers, rendering/host, scenarios/saves, mod packages, script/API boundaries, original-compatibility path and user-facing editor. Distinguish existing vs proposed components and no hand-wavy second engine.
7. **One smallest decisive experiment** for each leading architectural option, with prerequisites, exact steps, objective pass/fail criteria, expected time/resources and what result would change the recommendation. Do not execute them. Favor high-information experiments over premature long builds or code translation.
8. **Practical path to playable and moddable**: the shortest honest milestone sequence to (a) one original mission a human can play, (b) faithful multi-mission progression, (c) actual save/reload, (d) importing or authoring a custom mission, (e) loading a small community mod, (f) eventual skirmish/sandbox. No arbitrary MVP theater; each step must be a real vertical slice with acceptance criteria.
9. **Source and legal provenance appendix**: repository links, commit SHAs, key files/functions, trust class, license findings/ambiguities. Do not reproduce proprietary binaries or publish copied game data.
10. **A handoff decision brief for Hermes/Codex**: exactly which current work should remain frozen; what could be investigated next in parallel; which findings merit one user approval to change direction; and what not to do until those findings are validated.

**Final instruction:** Seek the shortest technically credible path to a complete, maintainable, open and moddable GeneWars platform. Being faithful matters; being usable and finishing the game matters too. Don't let past investment in one method dictate the conclusion, and don't discard verified work without evidence that an alternative earns the cost.

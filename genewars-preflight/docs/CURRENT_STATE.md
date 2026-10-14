# GeneWars current checkpoint — 2026-10-14

**Original campaign missions completable: 0 / unknown recovered total.** Automated mission acceptance: none. Native interactive acceptance: none. Integration-tested campaign transitions: 0. Reference remains verified CD **GENEWARS QA BETA V5**, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Retail equivalence is unestablished.

## Current gameplay position

Verified dependency batches are committed through `a7a3caf` (ethereal visits, atomic outcomes, victory resources, fresh host/native fail-fast and Krell contract); later host, input and rendering integration remains uncommitted. The full original campaign is the goal, followed by release acceptance and SFHS feasibility.

- Fresh source startup is integrated behind `--reference-startup fresh`: participating player count1, flags3E, frame0, countdown1000, serialized RNG followed by64 blueprint draws, actual mask5 specialist preparation and bootstrap visibility. Explicit zero opaque blueprint scratch priors and diagnostic RNG ring remain fidelity qualifications. No full original startup acceptance claim.
- Original opcode16 building placement now follows selected footprint snapping, movement, assignment and same-callback construction arrival. Moving architects are admitted by source comparison. Powered completion checks the actual completion bit.
- Development-path native probes reached completed Extractor and Distributor but blocked on effect33 drawing. EFF33 binding, component offset and flipped-shadow rendering have now been repaired with focused source comparisons; integrated fresh replay has not yet reached that point.
- **Fresh building-chain001 completes Extractor, pays588 from2000 and stops at1137 with `Family8Motion(UnsupportedRecord)`.** This is an actual mission-path blocker. See `research/integration/fresh-building-chain-001.json`. The old replay loop also dispatched the later Distributor placement after blocking; preserve this limitation and capture the pre-boundary world independently. Fail-fast repair is assigned.
- Mission1 Exceland requires Extractor11, Distributor3 and three Photon Processors6. Shared source-backed outcome, result transition and campaign-carry plans exist; host outcome/result integration is incomplete. Victory artwork renderer is available, animation/audio and usable result flow remain pending.
- Mission2 Krell minimum contract is recovered in `research/campaign/mission-002-contract.md`: Genepod, Mules and zone credit, with score threshold1000. Root/1 variant matches the CD resources; root/4 differs. No variant behavior mixed.

## Active work and exact resumption

- Fresh ordinary-command chain003 passes2500 callbacks: Extractor280 and Distributor240 complete/powered, funds1074, no blocker. `research/integration/fresh-building-chain-003.json`. Native chain passes twice with identical intermediate images and world hashes. Receipt `research/integration/native-building-chain-fresh-building-chain-003-passed.json`.
- Ethereal source114B0 completed-building selection and positive-timer hover/expiry repaired:64 initial-target cases,134 exact fresh1226 hover callbacks throughstate3. Historical1137/1226 failures/captures retained.
- Next normal Photon command `build-place 6 430 350` stops2523 with `Setup(UnsupportedSite(Blocked))`; report `research/integration/fresh-photon-chain-001.json`. Source507D0 ordinary rejection is recovered. Luna owns construction cancellation and shared post-motion continuation;48 cleanup cases and8 regressions pass, whole57A10 comparison pending. Sol2 owns independent whole-callback oracle. Root owns host feedback integration. Do not treat this as loss or suppress the error.
- Headless and native replay now fail fast before later commands; focused regressions pass. Native validator consumes explicit blocked artifacts and expected-boundary tests require exact reason/frame.
- Outcome adapter committed with3 focused atomicity/latching tests. Root has wired it into SharedMission's source72ED0 frame tail, fresh ten-frame host smoke passes Running/countdown990. Later host JSON/HUD changes compile; full terminal path remains pending. Victory/result view and continuation remain unfinished.
- Luna owns native chain verification; no other broad research running. Root next: finish source-supported invalid-site response, verify ordinary Photon placement/construction, add all3 Photons and visible outcome/result flow. Counts remain0/unknown accepted missions and0transitions.

Build with `CARGO_BUILD_JOBS=2 C:/Python314/python.exe -B scripts/cargo.py build --release -p genewars-desktop --bin genewars-game --offline --locked` from this project. Reuse frozen chain003 resources and replay; each new integration run gets a fresh private directory. No full current-suite claim. Crash archive check remains unchanged; proprietary/private files are untracked.

## Historical recovery evidence

# GeneWars recovery checkpoint â€” 2026-10-14

Recovered from the filesystem and saved command results. This file is the current entry point; older status sections are history. The native Rust reconstruction is a working development prototype with several verified gameplay components. It is not a complete playable GeneWars campaign or a demonstrated behavioral match to the original.

## Verified position

- Branch: `codex/genewars-reconstruction`; latest commit: `e2493cc9f8cf82981e12a5f9bb29ae61996737b1`. Substantial later work is present but uncommitted. Recovery initially found 164 changed tracked paths and 114 untracked paths; most are code, validators and generated evidence.
- The broad run finished at 2026-10-14 00:04:51 UTC with **270/271 checks passing**, including **416 Rust tests**. All 132 pinned production paths remained stable during that run and still match at recovery.
- Its sole failure was a validator dependency: the BTURR pixel oracle pinned another test's regenerated JSON receipt. The failure occurred before pixel comparison. That unnecessary dependency is removed, retaining actual source-listing, resource and implementation pins. The focused rerun passes **896 complete framebuffer cases and six atomic rejection cases**. The historical aggregate remains 270/271; this is a separate repair receipt, not a rerun of the aggregate.
- Three checks were registered after that aggregate started: `mission_bootstrap_context`, `saved_architect_return`, and `turret_target_tracking_pass`. Their separate evidence exists; they were not included in the 271-check run.
- The original archive was rehashed during recovery and remains SHA-256 `8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906`. The dated manifest and comparison hashes are also unchanged. No private files are tracked. The completed aggregate checked 181,375 ignored private files.
- No subagents, original game, emulator, or project game process remained active when recovery checked them. The earlier review agents reported usage-limit errors. The old aggregate source-edit hold has ended because that run is complete.

Evidence: [recovery JSON](../research/integration/recovery-20261014.json), [aggregate](../research/integration/verification.json), [focused repair](../research/integration/verification-repair-20261014.json).

## What works

| Area | Completed scope |
|---|---|
| Inventory and analysis | Archive provenance, separate executable variants, LE mapping/fixups, isolated Ghidra analysis and exact-build evidence. |
| Assets and native presentation | RNC decompression, terrain, fonts, menu, panels, plants, admitted actor/creature/effect artwork, bounded native windows and resource hash admission. One GPF animation resource remains unresolved. |
| Shared simulation | Player/blueprint preparation, landing and actor activation, selected-actor display, terrain move orders, arrival/idle, plant reproduction, Mule feeding/mortality/BONES retirement, water effects and the admitted ethereal lifecycle. |
| Construction | A real picked move and build order completes the admitted selector-1 building, pays 688 from starting funds 2000, reaches health 120, returns the architect to idle and leaves zero terrain jobs. Two headless runs and a window agree over 689 ticks. |
| Turret tracking | Target search, retained/replaced targets, rotation, cooldown and pending audio requests; complete mixed building passes have independent comparisons. Firing still rejects transactionally. |
| Long replay | A pinned native build completes 32,768 steps twice headless and once with matching window presentation. It predates later construction/tracking changes; its success must not be relabelled as a current-build 32,768-step result. |

## Unfinished work and exact resumption points

1. **Close actor idle relocation.** The private alternate-context experiment uses one active player and countdown 1000. Seeds 0 and 5428 pass 32,768 numeric steps; seed 65535 stops at step 14,111 in actor 1. The captured world is under `private/review/bootstrap-context/fbc3cdc5f63a4ebfa01d0e78c2cf5354/seed65535-players1-timer1000/frame14111/boundary`. Static tracing identifies periodic `6FB60` current-position rejection, nearby search, then actor callback `52280`. The new source harness stopped on a missing `1EBB0` listing; no repair has been proved or integrated. Start with `private/integration/actor-relocation-source.py` and its `actor-relocation-source-058.log`.
2. **Make startup context explicit before mission outcomes.** The host still uses development conventions for active-player loops, countdown, callback frame numbering, seed and native priors. The static bootstrap contract proves specific count/timer/frame differences, but does not establish complete original startup. The private mission-tail draft has 1,024 prefix fixtures and 477 admitted complete nonterminal cases; its owned-world adapter is a draft. Terminal outcomes and deferred landing remain unsupported. Files are in `private/ghidra/construction-command/mission_tail*.rs` and `validate_mission_tail*_draft.py`.
3. **Finish projectile behavior before enabling firing.** Geometry and a private code-42 factory/lifetime-prefix draft have independent checks (64 factories and 240 prefixes). The live callback also performs a randomized nearby query and can create child effect 20, whose damage/reaction behavior is not yet integrated. Continue from `research/variants/turret-projectile-source.py` and the private receipt `private/review/turret-projectile/7715adbb9c09463ab685b149cd88c7e7/result.json`. A factory-only proof does not establish a working weapon.
4. **Integrate the turret status overlay.** Ten complete padded framebuffer cases match independently written Python and Rust models for PLGBL overlays. Production drawing is unchanged; see [overlay contract](../research/formats/bturr-status-overlay.md). The reviewed building draw path has no separate shadow call; do not invent one based on old status wording.
5. **Expand to a complete mission loop.** Other building types, resource production/power, harvesting, wider specialist work, creatures and AI, genetics/breeding, mission win/loss and campaign progression, save/reload, audio and full UI remain incomplete. Multiplayer and optional phone packaging are later work. No confirmed original-game mission has been observed; DOSBox remains an optional narrow reference investigation.

## Workflow changes

Implemented during recovery:

- `python -B scripts/recover_checkpoint.py --check-archive` reads current evidence, compares production hashes, identifies aggregate failures and later-added checks, reports Git state, and optionally rehashes the archive. It does not compile, execute the game or rewrite historical verification.
- Removed the generated-report hash dependency that caused the sole aggregate failure, then reran the affected oracle successfully.
- Saved a checked local ZIP of **1,158 non-ignored project code/metadata files**, plus a tracked-change patch and hash manifest, under `private/recovery/20261014T090523Z`. This snapshot precedes this recovery document. Existing private assets, tools, captures and experiments remain in place; the ZIP is not a full backup.

Recommended next:

1. Use this current-state file as the single short checkpoint. Keep chronological research in WORK_STATUS, and link to it instead of repeating changing counts in four documents.
2. Review and commit completed components in smaller batches. Do not leave a whole session's verified code mixed with unfinished proof scripts in hundreds of changed paths.
3. Run focused checks for the changed subsystem, then an integration smoke test. Run the long full suite at integration checkpoints. Preserve the original failure and attach focused repair evidence rather than rerunning everything to fix metadata alone.
4. Add a resumable check registry with dependency hashes and per-check durations. Snapshot validators as well as production sources; the current aggregate pins production but does not freeze every script it executes. Long runs should use isolated source snapshots/build directories so other work need not stop.
5. Give probes a structured `passed`, `blocked`, or `error` result. The exploratory trajectory probe currently prints `STOP` but exits 0; process exit alone is insufficient evidence of success. Expected boundary fixtures can explicitly assert a blocked result.
6. Reduce duplicated fixture/interpreter setup, centralize the local toolchain environment, and retain minimal reproducible failing worlds. Avoid repeatedly reconstructing the same paths or rereading large JSON reports.
7. Near usage exhaustion, finish the current bounded check and save the exact next command, source hashes and experiment state before assigning more work. Agent identity is not durable state; filesystem evidence is.

The next product milestone should be a repeatable first-mission loop with correct startup context, usable construction/economy, a complete combat interaction and explicit mission outcome. Raw test-case counts measure covered components, not percentage completion of the game.

Latest startup correction: mission-view exit7D020 resets the frame at7D038 before MAINPANL entry; earlier conditional frame1 inference is superseded by frame0 evidence under review. Fresh constructor numeric priors are now source-proved zero; intervening panel state and opaque scratch provenance remain explicit.

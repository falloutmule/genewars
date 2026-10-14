# GeneWars current checkpoint — 2026-10-14

## Campaign delivery status

**Original campaign missions completable: 0 / unknown recovered total.** No mission has passed both ordinary-command automated gameplay and native interactive mission acceptance. Integration-tested campaign transitions: **0**. The reference remains the verified CD **GENEWARS QA BETA V5**, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; retail equivalence is unestablished.

The governing task is the complete Rust campaign, delivered one authentic mission at a time. Milestones are first mission, first original combat mission, ordered campaign progression, complete single-player systems/presentation/save continuity, native release acceptance/full traversal, then SFHS feasibility. A mission counts only after both gameplay paths pass against the same implementation; unsupported behavior is never a loss.

## Current work and next player-visible capability

- Preserved verified runtime baseline in `ab61abb` and desktop/resource integration in `92b3fff`. Native input replay is committed in `8de6fbc`. Earlier uncommitted validators and evidence still need dependency-ordered preservation; private drafts remain distinguishable.
- Native replay now feeds pointer/button/key events through the same window dispatch as live input, with presentations between events and five intermediate captures. Two native runs agree through menu entry, landing, selection, movement and arrival after339 ordinary steps; premature replay termination rejects. All61 game tests pass. This establishes a **development control sequence**, not authentic mission completion. Receipt: `research/integration/native-input-development.json`.
- First mission contract identifies **Exceland**, campaign progress0, `CONQ001/HDRCO1`. Required construction chain: **Extractor(selector11) → First Stage Distributor(3) → three Photon Processors(6)**. The unsigned completed-selector6 counter>=3 is the source victory predicate within the enclosing eligibility rules. Turrets require later campaign progress. See `research/campaign/mission-001-contract.json` and its source-pinned generator.
- Root is extending shared construction for these source-backed selectors; selector6 generic completion already matches64 literal-source fixtures and selector1 regression64 fixtures. This isolated proof does not enable the entire building lifecycle yet.
- Luna owns native input/resources; Sol1 owns campaign/availability/result evidence; Sol2 owns projectile impact evidence and secondary relocation investigation. Root owns runtime integration and checkpoint/commits. Keep ownership non-overlapping.
- Next visible capability: select the originally available Extractor, place it through ordinary interface commands, and complete construction through the shared engine. Then unlock Distributor and Photon Processors using recovered menu rules.

## Active blockers and exact resumption

1. Extend the admitted construction/resource/callback paths for selectors11,3,6, starting with Extractor. Existing selector1-only assumptions must not be relabelled as general support. `building_progress.rs` and `actor_world/building_storage.rs` contain the initial selector6 extension under focused validation.
2. Integrate explicit source-backed startup context (active players, flags/countdown, frame ordering); RNG provenance/native priors remain unresolved. Existing landing-seed route stays labelled development.
3. Recover and implement complete eligibility/failure/result/continue predicates and campaign carry. `2F810` result-widget preparation alone does not establish progression.
4. Continue native action replay for construction and visible outcomes; current replay already supplies the needed input-dispatch mechanism. No user test is required.
5. Seed65535 step14111 relocation remains a captured secondary defect, a mission blocker only if authentic initialization or required play reaches it. Projectile work remains independent of this noncombat mission.

Resume by reading this checkpoint and the small mission contract, inspecting Git changes, and checking active agents before touching their files. Run focused tests for changes; do not repeat the historical aggregate solely to raise its count. Save actual `passed`, `blocked` or `error` outcomes and preserve earlier failures.

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

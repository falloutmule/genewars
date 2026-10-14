# GeneWars current checkpoint — 2026-10-14

## Campaign delivery status

**Original campaign missions completable: 0 / unknown recovered total.** Automated mission acceptance: none. Native interactive mission acceptance: none. Integration-tested campaign transitions: **0**. Reference: verified CD **GENEWARS QA BETA V5**, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; retail equivalence is unestablished.

The complete Rust campaign remains the goal. First mission, first combat mission, campaign progression, full single-player systems, release traversal, then SFHS feasibility are sequential milestones. Unsupported behavior is never a gameplay loss.

## Current implementation and acceptance

- Verified dependency batches are committed locally through `73ddf79`: shared construction, captured relocation repair, terrain height/jobs and linked-entity updates, bounded Extractor production, Distributor/Photon production and effect33 delivery, and generated terrain-border initialization. Later registered startup/outcome modules and native receipts are pending review/commit.
- **Extractor construction completes using ordinary commands and native input replay**, with the corrected generated terrain table. Probe006 runs1289 ticks, pays588 (2000→1412), leaves completed selector11 health280, idle architect and no terrain jobs. Native replay passes twice with11 intermediate observations through the real dispatcher/hit testing. Receipts: `research/integration/extractor-command-006.json` and `native-extractor-generated-passed.json`. This uses development startup and is not an accepted mission.
- Startup884D0 generates an8192-byte terrain table; the old mapped table was allzero. The corrected table matches literal source and is now generated from hash-bound640-byte pattern input. Historical probes remain preserved with their original input policy.
- **Distributor command probe001 stalls** after assignment: architect state9/stage0, target speed0, no movement to the cell center, no new building after1000 ticks. Full captures1479–1510 and before/after assignment preserve source/world/RNG. Exit0 and no engine error are not success. See `research/integration/distributor-command-001.json` and `distributor-stall-capture.json`.
- Grade0 Distributor/Photon production, power-transfer factory/flight/delivery/retirement are source-proved and integrated. The current stall precedes those systems. Full-pool, higher-grade and unsupported contact branches remain explicit boundaries.
- Exceland progress0 requires Extractor11 → Distributor3 → three Photon Processors6. Availability is source-backed. The new shared mission-outcome evaluator passes640 literal cases and returns typed Running/Won/Lost plus result requests; it is registered, but not yet integrated into mission gameplay. Ordinary mode0 victory requests185DFC; continuation/campaign carry remains separate.
- New explicit startup context and player preparation compose source-backed RNG/blueprints/activation preparation. CONQseed5428,count16855 becomes10548,count16919 after64 draws. Mask5 prepares architect/geneticist. Blueprint scratch/native priors and final startup sequencing remain explicit. Bootstrap reveal/world adapter is registered and under source comparison; host still labels development startup.
- Native construction chooser and movement bindings remain development controls. Original input mapping, usable result flow, campaign continuity and full saves remain required. No current full-suite claim: old270/271 aggregate and32768-step runs are historical.

## Active ownership, blockers and exact resumption

1. **Luna/native_input:** compare captured Distributor stall with actual57A10, arrival and52340/526C0/507D0 speed semantics. Own comparison harness/report; no production changes until source establishes the fix. Do not evade the defect by choosing a favorable site.
2. **Sol1/campaign_contract:** source-proof72AD0 bootstrap world adapter and finish minimum explicit startup inputs. Own mission_bootstrap.rs and actor_world/bootstrap_updates.rs. Root registered both and made terrain entity replacement pub(super); runtime cargo check passes.
3. **Sol2/combat_review:** recover result/continue transition and carried state after proven mission_outcome.rs. Own new transition module/evidence, no host edits.
4. **Root:** integrate reviewed modules and real mission context, fix the proven construction defect, then replay the same ordinary Distributor sequence followed by native input acceptance and three Photon Processors. Complete outcome/result flow only under source-backed initialization. Continue to the next mission after both acceptance paths pass.

Resume with this checkpoint, agent status and scoped Git changes. Read the captured stall receipt before rerunning anything. Use release builds for integrated gameplay (1289 ticks≈27 seconds; debug exceeded240 seconds). Focused source comparisons remain appropriate; avoid broad aggregate reruns until an integration milestone. Preserve frozen binaries/resources/inputs and report passed/blocked/error explicitly. Secondary relocation seed65535 fixture repair passed, but no new full32768-step run is claimed.

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

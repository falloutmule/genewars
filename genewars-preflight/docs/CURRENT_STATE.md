# GeneWars authoritative continuation - 2026-10-16

Authentic campaign missions accepted: **1 / 24**. Exceland and its ordinary retained-carry transition to Krell are verified. Krell is not complete. User authorization is continuous implementation without interactive testing/login; old stop instructions are superseded. Complete faithful native Rust GeneWars remains the goal.

## Current repository and live frontier

**RESUMED 2026-10-16** under the Hermes `genewars` profile (director DeepSeek V4.1 Flash, worker GLM-5.3 Flash; routing proven by one real delegated patch, child invocation recorded in the profile agent log). Prior stop handoff: `C:/Users/fallo/Documents/Codex/2026-10-15/9-router-is-installed-to-use/GENEWARS_PROGRESS_AND_WORKFLOW.md`.

Latest verified integration/native acceptance: **60ee31d** (batch195 attack-miss + retained-target wait), following **e5ee81c**, **ff1df5d** (neutral hit completion194), **f3c897f** (waypoint primitive), **a9285cf** (owned attack/neutral hit holds193) and f6bc152. Parent Git Freecad; branch codex/genewars-reconstruction. Preserve all unrelated historical modifications/untracked files and private captures; stage explicit files only.

Latest ordinary AND native accepted frontier: **195/frame5939**, outcome Running, **zone credit84/1000**, resources311, campaign **1/24**. World SHA `a4da3dd344fe195648208c603f341e47a189862e905d4d0e662e808a5a39ffd4`. Headless private/integration/continued-krell-195; native private/integration/native-carried-krell-195. Native steps/blocker/world/actor audit/buildings/outcome/creatures/commands all agree on one frozen executable, SHA `335d85ad947c480a1f0c174a0c7e4854ddb82d78bda236029f4844dfd85c6107`. Native framebuffer byte-identical to headless, SHA `11b943cbbc211580cd453bf0eec988f1c04336b142c71eb5062e826865b41c0a`; native frame inspected (coherent Krell scene, "KRELL - RECONSTRUCTION - STEP 5939", "ZONE CREDIT: 84/1000 - RESOURCES: 311"). Native exit1 is the explicit saved gameplay blocker, not a crash. Receipt research/integration/native-carried-krell-parity-195.json.

**195 resolves the former blocker.** The owned Mule1 attack at5892 is a MISS: original source range16481 exceeds16384, so the miss consumes oneBBD RNG draw, keeps target references/reserve/speeds and enters the exact state8/task7 retained-target wait instead of the hit path. Final guard placement puts the hit-only operator/reserve checks AFTER the range test (owned_mule_completed_attack). Focused re-verification with FRESH paths after that guard change is 9/9: the literal packet case, actual5892 plus the retained holds5893/5894/5895, synthetic range exact16384(8draws)/outside16385(1draw), and the prepared zero-reserve/operator miss fixture (1draw) - each comparing427369 world bytes with zero differences. Original pre-guard-change focused receipt preserved at research/integration/owned-attack-miss-195-focused.json; the fresh rerun is research/integration/owned-attack-miss-195-focused-rerun.json. Regression195: runtime260passed/0failed/8ignored; desktop11+10+80(+1 ignored)+6 passed; release195 built.

Current campaign blocker: step5939 `Creatures(UnsupportedMotionState)`, not yet localized. Owned Mule1 sits at[693,23904] heading[696,23863], state8/animation7/frame0, reserve14, target_kind1; mule trace PortMuleFocus:1 and PortMuleOrder:1:[658,24211]. Per-frame captures are at private/review/authentic-krell-195. Exact next step: localize the5939 unsupported callback from that capture, obtain independent original-source evidence, then implement - do not guess semantics.

Authentic zone credits progressed 0->24@5760, 24->54@5824, now 84@5939. Require ordinary credit1000, original mission outcome, retained carry and native acceptance before2/24. No resources/quality/seeds/flags/outcomes injected into the campaign. Blood/lethal/neutral attack completion/operator/alternate query branches stay guarded until source-proved; actual5892 was a miss.

Separate unfinished experiment: unregistered mule_neutral_hit_escape.rs and its private proof. Do not stage as release-ready. Registered mule_flee_waypoint.rs is independently literal-tested8cases but no flee caller is campaign integrated. Actual5882 chose four-draw reacquisition rather than the frozen seven-draw flee. Preserve unrelated loss_transition.rs and historical files.

One heavyweight job at a time; Rust2jobs. No Ghidra/Java work was needed for195. Original Ghidra project and private evidence remain intact. Candidate-vs-retail CD equivalence remains unresolved: GW.EXE SHA256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; proofs are scoped to that pinned candidate and the stated environment.

## Focused verification and supporting primitives

Runtime152: **232passed,1opt-in ignored**. Source-equal authentic callback corrections include1279 feeding-to-mating (2draws),1927 retained-pair retry (1),2051 circle scan (5),2601 idle mating (2),2602 satiated pair approach (0),2907 starvation (0), each comparing427,369worldbytes. Receipts are scoped under `research/integration/authentic-krell-*`.

Source8D930 scans a circle of radius1024; cells withdxÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â²+dyÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã¢â‚¬Â ÃƒÂ¢Ã¢â€šÂ¬Ã¢â€žÂ¢ÃƒÆ’Ã†â€™Ãƒâ€šÃ‚Â¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡Ãƒâ€šÃ‚Â¬ÃƒÆ’Ã¢â‚¬Â¦Ãƒâ€šÃ‚Â¡ÃƒÆ’Ã†â€™Ãƒâ€ Ã¢â‚¬â„¢ÃƒÆ’Ã‚Â¢ÃƒÂ¢Ã¢â‚¬Å¡Ã‚Â¬Ãƒâ€¦Ã‚Â¡ÃƒÆ’Ã†â€™ÃƒÂ¢Ã¢â€šÂ¬Ã…Â¡ÃƒÆ’Ã¢â‚¬Å¡Ãƒâ€šÃ‚Â²>16 are skipped and boundary16 retained. Do not ignore real mate candidates: idle state8 CAN initiate mating. Both pair speeds use the initiating Mule. Neutral5D700 returns true without a specialist.

Isolated pair rendezvous147 and three explicitly synthetic hunger-flag variants prove additional20/60 state10/postbirth8/retry domains. Three variants compare1,282,107worldbytes, zero mismatches, draws0/0/1. `research/variants/mule-pair-hunger-flags-review.json`. Other callbacks frozen/synthetic fixtures are NOT campaign acceptance. This avoids repeated full replays for already source-proved numeric variants.

Earlier synthetic-carry diagnostic: Genepod2275, ordinary production2276, owned Mule2724, rendered through2853. Its actual produced Mule lands2759; ordinary owned move, splash2800, arrival2830 and2831 each source-compare fullworld. Supporting receipts `research/integration/krell-owned-production-105.json`, `krell-owned-command-water-124.json`; do not substitute these for current authentic carry progress.

Original human action5 producer and atomic command adapter pass28literalcases/40,600bytes. Hardware world action5 is captured LEFT RELEASE; selection consumes that release, a later ground release issues opcode11. Actor selection slots must not be reused as creature slots. Source95300/952A0 picking uses SCALED RECTANGLES, not transparency; pure `creature_pick` passes320source/Rustcases/640results,0mismatches. `research/variants/creature-pick-review.json`. Real-resource height/frame adapter now passes288 literal source cases, including local frame clamping without record mutation; receipt `research/variants/mule-pick-resource-review.json`. Mixed-family hit ordering, visibility, selection manager and native Game wiring remain separate. Source/control evidence `research/variants/mule-world-input-source.md`.

Zone arithmetic46cases/86,914bytes, interleaved marker factory24cases/389,448bytes and marker motion593cases/48,626bytes are verified primitives. Live zone scoring/exhaustion and EFF60/code3C lifecycle/render integration remain pending. Marker iteration requires THREE F6 draws immediately followed by its real factory229 draw before the next marker; do not batch draws. Code3C needs its own typed factory/retirement binding, owner4 admission specifically for source-proved neutral markers, source priority11, skipped obstacle branch and signed counter handling. Full-pool replacement remains unsupported.

## Headless and native evidence

Native137 ordinary port construction controls reach exactly the same authentic2051 boundary as headless: step/reason/worldSHA `7ef1574a92c8874ab8ca6a79161048bdbf8d9eff34d4505544c5f8631239c643`, actor dispatch audit, buildings and outcome agree. Receipt `research/integration/native-carried-krell-parity-137.json`. This covers prior1279/1927 corrections through ordinary native gameplay. Later circle/idle/satiated/starvation corrections need latest native verification at the next integration checkpoint.

Frozen125 Exceland victory at4630 matches two native runs exactly in world/actor audit/scene dispatch, worldSHA `ebdc252d94215d920a43bed88c9715472aabfdb6866a84433edd7ef35421b805`; native retained Krell entry atstep0 passes. `research/integration/native-headless-parity-125.json`. Legacy native harness receipts retain historical zero-acceptance/presentation caveats; authoritative gameplay count here remains1/24. Result movie/audio and full presentation remain incomplete. Never claim Krell complete from startup, primitives or a blocked replay.

## GLM through existing 9router

Isolated `C:/Users/fallo/.codex/genewars-glm53.config.toml` uses Responses atlocalhost20128, exactmodel `openrouter/z-ai/glm-5.3`, lowreasoning, retries0, profile-only MCPdisabled. Auth helper supplies saved gateway key internally; never print/run its credential stdout. Existing9router0.5.69 runs hidden. Gateway source strips `openrouter/` then calls OpenRouter. Desktop defaults and Luna/Sol routes unchanged. No Flash/model substitution.

Full streaming read-only tool call/resume cycle passed. One bounded Extractor metadata coding task was root-reviewed/integrated. Four CLI requests:65,133input/663outputtokens; gateway estimated$0.045453. OpenRouter key-wide billing delta afterfour$0.01991058; after tiny upstream-IDprobe$0.019988565. CLI counters incorrectlyzero. Exact per-change cost and root review-token savings are unavailable. Model reply/source route support intended upstream; generation-ID lookup404 means independent generation-record lookup unverified. Key has no cap; funds/billing limits unchanged. Account privacy controls unavailable through key API and unchanged. No proprietary game bytes sent toGLM. `docs/GLM_ROUTE.md`, `research/integration/glm53-gateway-evaluation-001.json`.

## Resources, original evidence and ownership

One heavyweight job at a time; Rust2jobs. Windows Event41/6008 unexpected shutdown evidence Oct2/3,Sep21/22,Sep15/16; latestBugcheckCode0, no requestedEvent1001/minidump found. Cause unresolved. Workspace `windows-crash-evidence.json` preserves inspection; load limits do not prove a crash cause.

Existing Ghidra project preserved. Bounded read-only/no-analysis exports95300/952A0 succeeded, private directory `private/ghidra/mule-picking-148`, logs148/149; JVM768MiB/2compiler andGCthreads. No import, whole-program analysis or project save. Saved QA BETA V5 CD evidence remains private; retail equivalence unresolved. GW.EXE SHA256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.

Root owns integration, gameplay/native checks, Git/checkpoint. One supplementary agent `/root/owned_scheduler_review` contributed bounded source review and independent primitives; existing Luna/Sol configuration remains available, no inferred OpenRouter subagent routing. Supplementary review confirmed Bones exclusion and same-callback pair pursuit; no worker compile or source edits. One heavyweight root job at a time remains required.

Exact fast2907 reproduction (fresh output path required; other callbacks frozen):

```powershell
$env:CARGO_BUILD_JOBS='2'
$env:CARGO_PROFILE_TEST_OPT_LEVEL='1'
$env:GENEWARS_CAPTURED_MULE_INPUT=(Resolve-Path private/review/authentic-krell-150/frame-2907-slot-6-blocked-before).Path
$env:GENEWARS_CAPTURED_MULE_OUTPUT=Join-Path (Get-Location) private/review/authentic-mule-replay-NEXT
$env:GENEWARS_CAPTURED_MULE_FRAME='2907'
$env:GENEWARS_CAPTURED_MULE_SLOT='6'
$env:GENEWARS_CAPTURED_MULE_EXPECT='passed'
$env:GENEWARS_CAPTURED_MULE_EXPECT_AUDIO='31'
C:/Python314/python.exe -B scripts/cargo.py test -p genewars-runtime captured_mule_callback --offline --locked -- --ignored --nocapture
```

For ordinary continuation, reuse the frozen153 executable/replay with NEW report/BMP/capture paths. Large JSON reports/world comparisons stay local; inspect mismatch counts, relevant fields and paths only.

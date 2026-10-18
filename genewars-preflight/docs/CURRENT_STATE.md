# GeneWars authoritative continuation - 2026-10-16

## Family F9 checkpoint - 2026-10-17 (campaign advances 5946 -> 5962, native-accepted)

Same evidence chain as F4->F6, applied to the expired-timer boundary: F7 measured the refusal, F8 documented the TAKEN side of the
timer test from the listing (0005ef40 JLE 0x65384 -> jump table entry 0x14 -> 0x5f4c4 -> 000602a2 state 20->8, 00060291 word(0x6c)=0
-> rejoin 0005ef54/000653c3 -> the shared tail with 0006a4a0 DEC skipped and animation 0x10->0x05; the executed tail never reads the
pointer triples, so the old 'mismatch' pin was an admission artifact), and F9 implemented the port arm against that output.

- Authoritative output: source_prepared, 21226 instructions, 0 draws, 21 changed bytes in slot 1.
- Port: `expired_hit` arm in owned_mule_callback.rs (+21/-3) + the choice arm in owned_mule_updates.rs (+16), placed FIRST so the
  broader 5922 arm cannot pre-empt it; the first pointer triple is deliberately not pinned (admission-only, per the evidence).
- Campaign: **5946 -> 5962** (world `f6e22870...`, rgb `417550fa...`). Native acceptance on one frozen executable (`0ad57ae2...`):
  5962 headless and native, all eight keys equal, framebuffers byte-identical.
- Negative coverage: R20 converted to positive with the source basis in-row; **R21 added** as a fail-closed negative (the same shape
  with word(0x6a)=0, the side the evidence leaves unexecuted - derived fixture, labelled as such). Checker 33/2/42.
- Next: the 5962 refusal is unmeasured. Family F10 measures it before anything is implemented.
- Cumulative since the policy went operational: **5940 -> 5946 -> 5962**, with no gate relaxation.

## Family F6 checkpoint - 2026-10-17 (campaign advances 5940 -> 5946, native-accepted)

The 5940 boundary was resolved by evidence in three ordered steps, not by relaxing a gate: F4 measured the live
refused record (owned state 20, animation 16, fresh timer 0x6c=32767, 0x52=0, 0x60=0x18) and showed the bounded
interpreter could not adjudicate it; F5 extended the reference model with a separate OR arm justified by the
literal original listing (0005ed37/0005ed3b -> 0005ee25-0005ee32 -> 0005eed5-0005ef1f -> 0005ef22 -> 0005ef32 ->
0005ef3a-0005ef40 -> 0005ef49-0005ef68 + 0006a20d-0006a21f -> 0006a290-0006a53e with 0006a4a0 DEC word(0x6c)
-> 0005e640/0005e817 -> 0007ac00, zero RNG draws); F6 implemented the port arm against that authoritative output.

- Authoritative output: `source_prepared`, 2307 instructions, 0 draws, 20 changed bytes all inside slot 1.
- Port arm `fresh_hit` in `owned_mule_callback.rs` pins exactly the measured shape; focused fixture byte-exact at
  exact lengths with 0 draws and an identical rng.bin.
- Campaign: **5940 -> 5946** (world `59c95764...`, rgb `1a065942...`). First genuine frontier movement since F1,
  and it came from source evidence.
- Native acceptance for the F6 build on one frozen executable (`95433f47...`): 5946 headless and native, all eight
  compared keys equal, frontier framebuffers byte-identical. Director checker: 30/30 positive, 37/37 checks.
- New blocker at 5946 (owned state-20 EXPIRED timer with a non-matching 0x50 triple) is reported, not measured;
  the checker again has zero negative rows, so family F7 pins that domain as a negative row *before* implementing.

## Family F2 checkpoint - 2026-10-17 (campaign reaches step 5940, native-accepted)

F2 (the step-5940 advance boundary) resolved by measurement first: an env-gated dump captured the exact refused
record (private/review/f2-5940-dump/refused-record.json; instrumentation removed before the gate run) - an owned
Mule in state 20, animation 16, frame 1, 0x60=0x18, 0x61=0x80, 0x6c=32767, 0x52=0, 0x57=1 - and the call graph
showed the refusing guard is the owned-ordinary conjunction in draw_supported_mule_record (not the startup
wrapper, and not the F1-R04 simulation gate).

- Bounded admission `owned_hit_hold = state 20 && animation 16 && 0x60==0x18 && 0x61==0x80`, recorded as a
  presentation SCOPE decision (no interpreter oracle covers rendering). No test, expected value, oracle or other
  guard changed: the existing negative pin for (20,16) with 0x60=0x08 stays green because the arm pins the carried
  command bytes exactly.
- Verified: family checker positive 24/24, negative 2/2, 33/33. Campaign now commits **step 5940**
  (world `53a4d4ea...`, rgb `08e84edc...`) and stops in the SIMULATION at the declared unsupported owned
  full-path transition (F1-R04), which stays explicitly blocked with transactional preservation.
- Native parity for the 5940 frontier on one frozen executable (sha256 `af9ca177...`): all eight compared keys
  equal and the frontier framebuffers byte-identical.
- Next family **F3**: the owned full-path motion transition at this boundary - it requires independent source proof
  (trace + interpreter) before any admission; F1-R04 stays the explicit negative gate until then.

## Family F1 closure checkpoint - 2026-10-17 (campaign step 5939 + native acceptance)

F1 (Mule light/full callback sequencing, shared completed-animation and hit-reaction
continuation, affected owned/neutral callers) is closed for its **declared domains**. Row F1-R14
(the 5939 presentation refusal) moved `in-flight` -> `integrated/proved`.

- **Corrected campaign now passes step 5939**: world `eb2fabf8...`, rgb `b67579e5...`; the next
  refusal is presenting step 5940 (`creature slot 1: UnsupportedRecordState`), which is F1-R04's
  declared unsupported boundary - kept as the explicit negative gate, not admitted.
- **The 5939 fix is a bounded presentation-scope decision, not an oracle-proved rendering**: the
  draw layer has no interpreter oracle, and the campaign reaches `draw_supported_mule_record`
  (the startup wrapper `draw_initial_mule_record` has no production caller - an earlier packet
  mis-attributed the guard from a line number). The owned-ordinary gate now admits the measured
  carried hit-reaction shape (owned, state 20, animation <=7) while the completed-animation arm
  keeps requiring the attack-entry shape; every other guard is untouched.
- **Native acceptance on one frozen executable (sha256 18442a3d...)**:
  private/integration/f1-native-20261017T1220 - headless and native both stop at step 5939 with
  the identical block string; `steps`, `blocked`, `world_sha256`, `actor_dispatch_audit`,
  `buildings`, `outcome`, `creature_states`, `issued_move_commands` all equal, and the frontier
  framebuffers are byte-identical (BMP sha256 `775a718d...`). Visual check: coherent scene, HUD
  `KRELL - RECONSTRUCTION - STEP 5939`, `ZONE CREDIT: 84/1000`, `RESOURCES: 311`.
- **Director gates** (private/integration/F1-R14-root-check-20261017T1115, via
  `scripts/root_family_check.py`): positive 24/24, negative 2/2, 33/33; focused batch 9/9;
  runtime suite 260 passed / 0 failed / 8 ignored.
- **Anomalies resolved and documented** (private/integration/verification-anomalies-20261017.md):
  the 4801 instruction count is a stable 2224 - the 2271 reading came from an intermediate
  line-ending-churn build whose leftover output dir also disagrees by 14 bytes; the empty
  background-task notifications were redirect artifacts, and every genuine tooling failure was
  recovered without affecting a verification result.
- **Open**: F1-R13 (the 6BA47 food-guard path is skipped for the 0x61=0x80 domain - state-invisible
  but a path-fidelity gap) and the next out-of-family boundary (step 5940's owned full-path
  transition). Effort today (state.db estimates, not provider-reconciled): workers 6 assignments
  ~$1.77, director session ~$2.66.


## Development policy adopted - 2026-10-17 (bounded behaviour families)

The user authorized a workflow change on 2026-10-17 and it is now installed in the places that
actually drive future work (not in a plan file):

- **Active skill `genewars-reconstruction`** - `SKILL.md` reframed from frame-per-assignment to
  family assignments, plus two new references the next packets must follow:
  `references/family-assignment-policy.md` (standing policy) and
  `references/worker-task-packet-template.md` (required packet shape). The short-lived duplicate
  skill `genewars-callback-reconstruction` was absorbed and deleted.
- **Repo copies** - `private/integration/development-policy-20261017.md` (same policy, citable
  from inside the checkout) and `private/integration/mule-family-caller-domain-matrix.json`
  (F1 caller/domain matrix with `integrated/proved`, `focused-only`, `unsupported`, `unknown`,
  `provisional`, `in-flight` rows).
- **Strengthened root result consumer** - `scripts/root_family_check.py`. It now gates a
  positive case on execution first (cargo rc, `result.json.status == passed`, intended
  frame/slot, `input_roundtrip`, no error), then **exact lengths of all eight buffers before any
  equality check** (actors 4000, backing 343958, buildings 17152, creatures 37200, effects 16200,
  family8 1632, players 7220, rng 7; total 427369), then zero byte differences plus the expected
  RNG draw count and the `rng.bin` seed/ring (draw order). Negative gates are reported and
  scored separately and must prove the expected rejection AND full transactional preservation;
  rollback-produced equality is never a pass.

**Initial family F1** - Mule light/full callback sequencing, shared completed-animation and
hit-reaction continuation, affected owned/neutral callers; movement/cell-commit and linked-AI
dependencies only where source evidence connects them. Rows R01-R13 are in the matrix JSON:
4801/5893/5939 light fixtures, 5914 neutral attack, 5916/5922 owned hit hold+completion, 5924
owned command -> AI dispatch (with a Provisional 6BA47 guard path), 5938 neutral zero-reserve
completion (in flight), 5940 as the explicit negative gate, plus the still-unknown domains.
Frame numbers are diagnostics: closure needs evidence-backed caller coverage plus ordinary
campaign and same-executable native acceptance, not a larger step count.

**Current acceptance status:** HEAD `b85098e` is a checkpoint (native parity for the corrected
trajectory has NOT been run). The corrected ordinary campaign reached step 5938; the old batch195
"5939 source-authentic frontier" stays retracted. Krell remains incomplete (1/24 missions).


## Source-corrected Krell trajectory checkpoint - 2026-10-17 (supersedes the 5939 frontier claim)

The batch195 "5939 frontier" rested on a campaign state that is now **provably wrong from frame 5913 onward**. The bounded original-instruction interpreter (research/variants/neutral-mule-captured-width-review.py, fed the accepted baseline's own captured before-state at 5913 slot 5) shows the source writes slot5 word(0x6c)=0 there; the accepted baseline left 32756, and the candidate now reproduces the interpreter byte-for-byte. So world `a4da3dd3…` at 5939 is a port-internal state, not a source-authentic one: 5939 remains the last port-internal accepted point, but it is retracted as a source-authentic frontier. Everything after 5913 is being rebuilt one interpreter-adjudicated callback at a time.

Committed checkpoint (this commit) — four source-corrected change sets, all root-verified on fresh paths:
- **GW-5939F** owned Mule light callback (source 5EE38..5EEC6): `mule_route_prefix(..., light)` applies the association admission only to the full path; the 5EB30 5ebdf..5ec69 completed-animation dispatch is added to the shared body's even branch; the light branch drives the shared admitted body through the same staged cell-entry closure as the full path. Files: actor_world/creature_updates.rs, creature_world.rs, actor_world/owned_mule_updates.rs.
- **Neutral Mule completed attack, frame 5914 slot 5**: hits its owned target (state 8->20, health -2, 0x6c 32767) consuming the source's 9 RNG draws. File: actor_world/creature_updates.rs.
- **Owned hit-reaction draw admission**: state 20 with animation 16, or the carried-over directional animation <=7, in the owned hostile shape. File: genewars-desktop/src/bin/support/creature_scene.rs.
- **Owned hit-reaction motion, frame 5916 slot 1**: state 20 admitted for the measured record domain; the 5E0B0/5EB30/5E640/5E817/7AC00 path performs the source's animation frame advance, energy -2 and countdown decrement with 0 draws. File: actor_world/owned_mule_callback.rs.

Gates on this tree (root-run, fresh exclusive output dirs): captured 4801 (2224 instr), 5893 (0x6c 32759->0), 5939 (no-op, 1503 instr) and 5940 (explicitly blocked, world unchanged) each byte-exact over 427,369 bytes; frame-5914/slot-5 neutral fixture byte-exact vs the interpreter with 9 draws; frame-5916/slot-1 owned fixture byte-exact with 0 draws; batch195 focused 9/9; runtime suite 260 passed / 0 failed / 8 ignored; runtime and desktop both compile.

Campaign: the ordinary carried Krell replay (`private/integration/continued-krell-5939F/replay.txt`, an extension of the accepted 195 replay) commits steps 5914-5921 and stops at **5922** on the next source-authentic behavior — owned state-20 hit-reaction completion (source deltas 0x19 16->7, 0x1c 1->0, 0x57 1->0, 0x5e 20->8, 0x60 0x08->0x18, 0x61 0->0x80, 0x66 227->225, 0 draws, 2345 instructions; input preserved at private/review/gw5922-source-fixture/frame-5922-slot-1-blocked-before-owned). **Native parity for this corrected trajectory has NOT yet been run**, so this is a checkpoint, not a new acceptance.

Method (per new callback): capture the failing before-state with GENEWARS_MULE_CALLBACK_CAMPAIGN_PROGRESS=1, run the matching interpreter oracle (`research/variants/{owned,neutral}-mule-captured-width-review.py --before <capture> --frame N --slot S`) to obtain the authoritative output and instruction/draw counts, implement to byte-equality over the eight buffers, then re-run gates A-D + focused 9/9 + suite + the campaign. Adjudicate divergences against the interpreter before changing any port behavior; never adjust tests, oracles or expectations.

Authentic campaign missions accepted: **1 / 24**. Exceland and its ordinary retained-carry transition to Krell are verified. Krell is not complete. User authorization is continuous implementation without interactive testing/login; old stop instructions are superseded. Complete faithful native Rust GeneWars remains the goal.

## Current repository and live frontier

**RESUMED 2026-10-16** under the Hermes `genewars` profile (director DeepSeek V4.1 Flash, worker GLM-5.3 Flash; routing proven by one real delegated patch, child invocation recorded in the profile agent log). Prior stop handoff: `C:/Users/fallo/Documents/Codex/2026-10-15/9-router-is-installed-to-use/GENEWARS_PROGRESS_AND_WORKFLOW.md`.

Latest verified integration/native acceptance: **60ee31d** (batch195 attack-miss + retained-target wait), following **e5ee81c**, **ff1df5d** (neutral hit completion194), **f3c897f** (waypoint primitive), **a9285cf** (owned attack/neutral hit holds193) and f6bc152. Parent Git Freecad; branch codex/genewars-reconstruction. Preserve all unrelated historical modifications/untracked files and private captures; stage explicit files only.

Latest ordinary AND native accepted frontier: **195/frame5939**, outcome Running, **zone credit84/1000**, resources311, campaign **1/24**. World SHA `a4da3dd344fe195648208c603f341e47a189862e905d4d0e662e808a5a39ffd4`. Headless private/integration/continued-krell-195; native private/integration/native-carried-krell-195. Native steps/blocker/world/actor audit/buildings/outcome/creatures/commands all agree on one frozen executable, SHA `335d85ad947c480a1f0c174a0c7e4854ddb82d78bda236029f4844dfd85c6107`. Native framebuffer byte-identical to headless, SHA `11b943cbbc211580cd453bf0eec988f1c04336b142c71eb5062e826865b41c0a`; native frame inspected (coherent Krell scene, "KRELL - RECONSTRUCTION - STEP 5939", "ZONE CREDIT: 84/1000 - RESOURCES: 311"). Native exit1 is the explicit saved gameplay blocker, not a crash. Receipt research/integration/native-carried-krell-parity-195.json.

**195 resolves the former blocker.** The owned Mule1 attack at5892 is a MISS: original source range16481 exceeds16384, so the miss consumes oneBBD RNG draw, keeps target references/reserve/speeds and enters the exact state8/task7 retained-target wait instead of the hit path. Final guard placement puts the hit-only operator/reserve checks AFTER the range test (owned_mule_completed_attack). Focused re-verification with FRESH paths after that guard change is 9/9: the literal packet case, actual5892 plus the retained holds5893/5894/5895, synthetic range exact16384(8draws)/outside16385(1draw), and the prepared zero-reserve/operator miss fixture (1draw) - each comparing427369 world bytes with zero differences. Original pre-guard-change focused receipt preserved at research/integration/owned-attack-miss-195-focused.json; the fresh rerun is research/integration/owned-attack-miss-195-focused-rerun.json. Regression195: runtime260passed/0failed/8ignored; desktop11+10+80(+1 ignored)+6 passed; release195 built.

Current campaign blocker: step5939 `Creatures(UnsupportedMotionState)`, localized to the owned Mule1 callback. Capture private/review/authentic-krell-195/frame-5939-slot-1-blocked-before-owned; per-frame captures are in private/review/authentic-krell-195. Owned Mule1 sits at[693,23904] heading[696,23863], state8/animation7/frame0, reserve14, target_kind1; mule trace PortMuleFocus:1 and PortMuleOrder:1:[658,24211]. Independent original-source run at5939 completes normally with 1503 instructions, 0 RNG draws, and ZERO change to all eight world buffers (427369B byte-identical in and out): the source callback is a genuine NO-OP at this frame, while Rust rejects it. Trace span 5ed20..5ee1c..6fb60..1ec50..7acxx..5eec6..5eed4. Exact next step: determine the source branch that yields the no-op by bounded source-authorised investigation, define the contract, then patch and verify - do not guess semantics. Evidence: private/integration/next-blocker-5939-source-195.json.

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

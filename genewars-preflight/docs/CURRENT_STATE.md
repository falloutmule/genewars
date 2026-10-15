# GeneWars authoritative checkpoint — 2026-10-15 UTC

**Original campaign missions completable: 0 / 24 corroborated standard campaign missions.** Exceland automated and native sequences reach victory and static results, but full mission acceptance remains pending. Integration-tested mission-to-mission transitions: 0. Reference: verified **GENEWARS QA BETA V5 CD**, executable SHA256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`; retail equivalence unestablished.

## Current position

Local branch `codex/genewars-reconstruction`, dependency commits through `b32a791` before checkpoint batches. Significant historical research and unfinished changes remain; do not bulk-stage or discard. Full current-tree aggregate has not run. Original archive unchanged (`8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906`); bounded crash integrity receipt `private/integration/crash-recovery-20261014.json`.

- **Exceland normal commands:** Extractor11 → Distributor3 → rejected Photon site with source-normal cancellation/feedback → three legal Photon6 sites. 4630 steps, five completed powered buildings, funds510. Victory source frame4262, committed step4263. Enter opens original static victory screen, stage5/progress1/TEXT478. No resources or outcome injected.
- Headless frozen run `private/integration/fresh-photon-chain-003`; metadata `research/integration/fresh-photon-chain-003.json`. Native dispatch/hit-testing twice with equal intermediate BMP/world hashes: `research/integration/native-building-chain-fresh-photon-chain-003-passed.json`, private native run `dac1570c322b4fcfa3edeca1f2ebfbb3`.
- Repaired actual-path blockers1137/1226 (ethereal building targets/hover),2523 (normal blocked-site cancellation),2636 (source invalid actor destination resolver). Historical fixtures preserved. Their failure classifications are unsupported valid source behavior, now independently repaired; never gameplay losses.
- **Current blocker:** second Enter from Victory reaches campaign carry projection but rejects `ActorLink` on real nonisolated map chains. Frozen `private/integration/fresh-campaign-flow-001`, report remains Victory/error `campaign carry: ActorLink`. Luna owns source-backed shared-registry carry wrapper; root owns host integration. Do not suppress or replace this failure.
- Campaign24 sequence corroborated by all24 manifest-pinned CD CONQ/HDR pairs, sourceprogress and ending boundary: `research/campaign/campaign-sequence.json`. Exceland first, Biosphere last. Extra maps unassigned; not retail equivalence.
- Krell mission2 contract committed: Genepod, owned Mules and zone credit>=1000. `research/campaign/mission-002-contract.md`. Next player-visible capability: Continue → campaign view → load Krell while retaining exact original campaign ranges/blueprints and RNG ring.

## Fidelity and acceptance limits

Fresh runtime uses source player count1/flags3E/frame0/countdown1000, serialized RNG plus64 blueprint draws, mask5 actor preparation. Scratch blueprint18 original prior is unknown; activation copies18→1B. Construction path has no identified dependency, but read-use invariance proof pending. Explicit zero prior remains a qualification, not historical fact. Diagnostic RNG ring provenance remains explicit.

Victory media decoder/mixer unavailable. Source ignores BIOWIN media failure and continues; this supports static result progression, not a claim of original video/audio playback. Campaign planet controls/system presentation remain pending. Native acceptance requires actual intermediate input observations, not `--window-replay` final-state display.

Krell initialization still not implemented. Source restore selected player ranges[0,4A1)/[61A,6F7), preserve oldworld150bytes[34BA,3550), inherit RNG ring/mode0, load serializedseed47117/count33620/frame0, countdown1000, skip blueprint regeneration, use mask7 and retained actor native priors. Shared view dispatch1BCE0 clears mode2/restores savedframe when entering Campaign flags6=0; do not clear mode arbitrarily in mission factory.

## Exact resumption

1. Integrate Luna `ActorWorld::plan_campaign_carry` shared-registry wrapper, source-compare nonisolated links and check original isolated regressions.
2. Build release offline with jobs2: `C:/Python314/python.exe -B scripts/cargo.py build -p genewars-desktop --bin genewars-game --release --offline --locked`.
3. Freeze a NEW `fresh-campaign-flow-002` directory; copy latest release, replay from001. Run headless with `--reference-startup fresh --headless replay.txt --report report.json --bmp frame.bmp` against `private/integration/game-runtime`. Preserve001 failure.
4. If Campaign passes, run `C:/Python314/python.exe -B scripts/desktop/validate_native_building_chain.py --probe private/integration/fresh-campaign-flow-002 --expect completed --reference-startup fresh --photons --result-view --campaign-view` (two actual native runs).
5. Commit verified dependency packets; implement reference-bound carried Krell startup and continue ordinary gameplay. No user test/permission gate. One heavy replay/compiler job at a time.

Secondary seed65535/step14111 relocation, combat projectiles, saves/audio/full presentation and later campaign remain. Complete native campaign traversal/release/SFHS not achieved. No original game, installer, patch, global MCP change or upload this continuation.

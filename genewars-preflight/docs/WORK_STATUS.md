# Working state — 2026-10-12 / 2026-10-13 UTC

Active reconstruction; no playable game yet. The CD executable is a provisional analysis candidate, not established canonical retail.

## Implemented and verified

- The project-local Rust 1.99.0 GNU toolchain builds the resource CLI, native asset inspector and event-driven control inspector. The full integration gate passes 41 checks and 84 Rust tests: assets 29, core 23, controls 11, desktop 10 and renderer 11.
- Ghidra 12.0.1, the matched LX/LE loader and JDK 21 loaded the actual LE code. Four objects, 244 pages, 26,883 fixup instances and entry 0xA6E40 reconcile with independent reconstruction. Raw exports remain private.
- RNC1 and prefix validation pass all 331 source records (179 unique inputs), including the two compound INSTALL resources. Strict ANB bounds account for all 263 records: 262 pass, one GPF wrapper/CRC case remains blocked. The 11,001 sprite records include 25 independently verified packed4 creature inputs and a separately traced BONES input. The superseded unsafe full-file result remains preserved.
- The original GENEPANL/MAIN.PAL binding renders correctly, with all 61,440 pixels independently compared. All 56 tiles from the four PL texture resources also pass independent pixel and opacity comparisons (622,592 samples). PLANETS.DAT remains uninterpreted; its filename is not a format specification.
- Portable core helpers cover wrapped distance/sqrt, opaque record queries, grid addressing, custom-screen selection, arrow capture, mode-two button packets, bounded list names and construction-stage resource debit. The construction helper passed 3,870,016 independent transaction comparisons; surrounding world/callback behavior is still absent.
- The winit/softbuffer host preserves ordered input transitions. A supervised Windows test observed arrow events 27 and 28, a nonactivating margin click and Escape exit. Its nominal 60 Hz diagnostic update is not original-game timing. Physically held release-outside behavior was not established by that UI test; synthetic tests cover the state contract.
- The 18-page manual is indexed. Manual claims remain distinct from executable and runtime evidence.

## Original reference observations

Run 001 started DOS/4GW 1.97 and stopped at missing config.ini. Static analysis then proved the configuration writer/parser contract without running the installer. Run 002 used a generated 126-byte configuration and reached animated introduction, main-menu artwork and Options artwork. Dynamic labels were absent and the guest cursor remained near the center despite host clicks/capture attempts, so no mission, save/reload or gameplay comparison was established.

Run 002 ended after 782 seconds by verified-PID operator termination when UI close did not work. Exit 15 is not a game failure code. Private screenshots are window captures, not raw framebuffer dumps. Only disposable GW.ERR changed (45 bytes to empty); generated config and both executables stayed unchanged. ADDRESS.TXT and the host-created STARTGW.BAT were added. Archive, source disc, copied media and tool integrity checks passed. See research/variants/reference-run-002-result.json and the accompanying outcome note.

The latest direct user instruction enables full access and authorizes necessary reference reruns without another permission prompt. Use fresh private staging and bounded supervision. No installer or patcher has run, and no public upload, push, MCP/global configuration change, symbol import or SFHS claim is part of this work.

## Ownership and next work

Root owns integration, core modules, Cargo workspace/lock and aggregate docs. SOL-01 owns Ghidra/static evidence and is tracing menu text/font initialization and DOS mouse movement. LUNA-01 owns asset/render/desktop implementation and is checking the exact staged font/text resources. SOL-02 independently reviews evidence, ports and reference-run accounting. The manual lane is complete; its slot has been reused. Model routing is recorded in research/integration/agent-routing.json.

Run `python -B scripts/cargo.py test --workspace --offline --locked` for headless Rust checks and `python -B scripts/verify.py` for the complete current gate. The verifier preserves the dated 12-check preflight and checks every private file is ignored (26,112 at this run). It does not rerun broad extraction or execute the original game.

Branch codex/genewars-reconstruction contains checkpoint 750b9cd plus the current verified tranche awaiting its next scoped commit. Never stage private or unrelated parent-workspace files. Local .gitattributes preserves byte-hashed receipt files without global Git changes. Continue from NEXT_ACTIONS.md; do not restart preflight.

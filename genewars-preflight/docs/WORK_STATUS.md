# Working state - 2026-10-12

Active reconstruction; no playable game yet. CD executable is a provisional analysis candidate, not canonical retail.

## What works

- Local Rust1.99.0 GNU builds the native resource CLI. RNC1/prefix validation passes331records (179unique inputs), with both INSTALL compounds accounted for.
- Ghidra12.0.1 + matched LX/LE loader + JDK21 loaded LE code. Four objects,244pages,26883fixup instances and entry0xA6E40 reconcile; decompilation exports stay private.
- Rust core implements wrapped distance/sqrt, opaque record queries, grid addressing, custom-screen offset/selection, delay-one arrow capture, mode-two button packets and bounded list-name reads. Eighteen tests pass; Sol independently reviewed the instructions and ports.
- Manual18PDFpages indexed with cited systems/controls. Documentation is not assumed behavior.
- The exact 128x480 GENEPANL and MAIN.PAL binding renders correctly: all 61,440 pixels independently checked. ANB header/sprite parsers and masked compositing are implemented. The native panel and sprite inspectors launch successfully.
- Strict ANB bounds accept 260 of 263 source records (10,990 sprite records), using indexed8 for the original structural probes and a distinct packed4 mode for 25 hash-bound creature inputs. Independent review compares all 2,673 active creature sprites' pixels/masks/consumption with zero metadata crossings or width overshoots. BONES (two copies) and GPF remain blocked. This replaces the unsafe prior full-file result, preserved in research/variants/full-file-sprite-corpus-superseded.json; the matching totals do not mean the old implementation was correct.

## Ownership

SOL-01 (gpt-6.1-sol): research/ghidra, scripts/ghidra, private tools/projects; continuing sprite/UI traces.
LUNA-01 (gpt-6-luna): assets/render/desktop crates, research/formats, scripts/desktop, private samples; latest focused tests are 26 assets, 9 renderer and 10 desktop. ANB metadata, actual normal/selected arrow rendering and explicit sprite4 inspector are implemented. Assessing an event-driven host because minifb loses quick mouse clicks.
SOL-02 (gpt-6.1-sol): research/variants, scripts/reference and private review/runtime tools; reference staging/empty-emulator restrictions verified; sprite-boundary review active.
LUNA-02 (gpt-6-luna): manual lane completed; slot reused by SOL-02.
Root: workspace/lock, core/tools crates, integration scripts and aggregate docs.

Accepted routes/results: research/integration/agent-routing.json. No global Codex/MCP config change. Three concurrent child slots; explicit file ownership.

## Commands

From project root:

```text
python -B scripts/cargo.py test --workspace --offline
python -B scripts/cargo.py build -p genewars-tools --offline
python -B scripts/verify.py
```

The repaired full integration run passed all 29 checks: 63 Rust tests, seven inventory boundary tests, all 331 RNC records, strict ANB exception accounting, all 2,673 active creature pixel/mask comparisons, real panel pixels, native window presentation, LE/UI/input/list contracts, unchanged original/baseline hashes and 18,981 ignored private files. See research/integration/verification.json. The previous 24-check receipt is preserved as verification-before-sprite-bound-audit.json and is superseded.

Verifier preserves research/verification.json (12dated preflight checks) and avoids broad re-extraction. Tool/build homes stay private. Existing Python/Cargo warnings are retained; exit statuses determine success. PowerShell policy unchanged.

## Boundaries/continuity

The user-approved original session completed: DOS/4GW 1.97 started, then GW rejected missing config.ini before a menu. Emulator exit was 2026-10-13T00:58:41.747427Z; its zero status does not mean game startup succeeded. No mission/input/save/reload or framebuffer capture occurred. Two earlier unreleased setup attempts were preserved. The original archive/disc/tool/config hashes remain unchanged; only disposable GAME/GW.ERR changed, and C:/ADDRESS.TXT was created. See research/variants/reference-run-result.json. Static configuration analysis continues. No installer/launcher fallback, public upload/push, MCP change or SFHS test occurred.

Resume NEXT_ACTIONS.md and evidence/receipts; do not restart preflight. Branch codex/genewars-reconstruction is created; the first scoped Git checkpoint awaits the repaired integration run. Some earlier source/metadata is staged; no commit exists yet. Never stage private content or unrelated workspace files. Local .gitattributes preserves byte-hashed JSON/text receipts without global Git changes.

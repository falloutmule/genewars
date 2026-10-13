# GeneWars reconstruction workspace

This is an active, evidence-backed Rust reconstruction using the user's local GeneWars archive. It provides native resource tools and an asset window, RNC1 and sprite decoding, a CPU renderer, and statically verified arithmetic, record/grid and UI transitions. It does not yet run a playable game.

The original [preflight](research/preflight.md) remains a dated baseline. Current work is tracked in [WORK_STATUS.md](docs/WORK_STATUS.md), [GAME_FEATURE_MATRIX.md](docs/GAME_FEATURE_MATRIX.md), and the [evidence ledger](research/EVIDENCE_LEDGER.md).

## Build and verify

Run from this directory using the existing project-local Rust toolchain:

```text
python -B scripts/cargo.py test --workspace --offline --locked
python -B scripts/cargo.py build -p genewars-tools -p genewars-desktop --offline --locked
python -B scripts/verify.py
```

`scripts/cargo.py` confines Cargo/Rustup homes and build output to `private/`. Core/assets/render use only the standard library; the desktop host pins minifb 0.29.0, with dependency checksums in Cargo.lock and a local license receipt under `research/formats/`. Tool installation receipts are in `research/integration/rust-toolchain.json`; the wrapper does not install tools while building. GNU Rust is used on this Windows host. Existing Python location and Cargo filesystem warnings do not imply a failed exit status.

`scripts/verify.py` checks Rust, all RNC records, the ANB accounting, real panel pixels, a bounded native-window launch, static arithmetic/LE mapping, private-file Git exclusion, archive SHA-256 and dated preflight preservation. Three ANB records (two distinct inputs) retain explicit unresolved failures. The verifier does not redo broad extraction or overwrite the preflight receipt. Results go to `research/integration/verification.json`.

## Native research utility

The built executable is `private/build/rust-target/debug/genewars-tools.exe`:

```text
genewars-tools.exe rnc-info INPUT
genewars-tools.exe rnc-verify INPUT [MAX_BYTES]
genewars-tools.exe rnc-decode INPUT OUTPUT [MAX_BYTES]
genewars-tools.exe rnc-prefix INPUT OUTPUT [MAX_BYTES]
genewars-tools.exe panel-render PANEL_RNC MAIN_PAL OUTPUT_BMP
genewars-tools.exe sprite-verify DECODED_ANB DECODED_TAB
genewars-tools.exe sprite-decode DECODED_ANB DECODED_TAB RECORD INDEX_OUTPUT MASK_OUTPUT
genewars-tools.exe sprite4-verify SPRITE_DATABASE_PREFIX DECODED_TAB BASE
genewars-tools.exe sprite4-decode SPRITE_DATABASE_PREFIX DECODED_TAB RECORD BASE INDEX_OUTPUT MASK_OUTPUT
genewars-tools.exe distance 0 0 3 4
```

`INPUT` may be `-` for binary stdin. Inputs are bounded to 64 MiB. The output limit defaults to 64 MiB and can be tightened. `rnc-decode` requires a complete single RNC stream. `rnc-prefix` explicitly reports consumed and remaining bytes; callers must account for the tail. Existing differing output files are never overwritten. The distance command returns squared distance using the CD candidate's wrapped arithmetic, with no assumed gameplay units.

`scripts/source_member.py ENTRY_ID --output private/PATH` copies one manifest-identified member, validates source and output hashes, and rejects ambiguous filename selection. Nested ZIP and existing raw-CD reads are targeted; no bundled programs are executed. Full-corpus validation is `python -B scripts/validate_rnc_corpus.py`.

`python -B scripts/render_panel.py` reconstructs the exact CD GENEPANL/MAIN.PAL pair and writes a private BMP/PNG. RGB6 display conversion is explicit floor(component*255/63), not a measured VGA monitor transfer. `python -B scripts/validate_sprites.py` accounts for every ANB; it intentionally exits nonzero while the documented BONES/GPF cases remain unresolved.

Sprite CLI commands use an explicitly supplied database extent and table. For an embedded ANB, pass the original prefix ending at the first metadata offset, not the entire file; the corpus script does this automatically. Indexed8 and packed4 are distinct caller-selected codecs. The latter is admitted for 25 exact creature hashes by executable caller evidence. BASE is an explicit 0-255 diagnostic palette offset; runtime creature palette selection remains separate. Desktop ANB modes enforce the metadata bound internally.

The native asset window can be opened with:

```text
python -B scripts/cargo.py run -p genewars-desktop --offline --locked -- panel ../private/integration/panel/GENEPANL.RNC ../private/integration/panel/MAIN.PAL
```

Paths passed through Cargo resolve from `rust/`. Escape or window close exits; `--frames 3` limits an automated presentation check. Actual three-frame launches passed. Physical keyboard/close input has not yet been independently exercised. The window displays original decoded assets; it is an inspector, not a game menu or simulation.

## Evidence and distribution boundaries

The CD candidate is SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Its static analysis does not establish canonical retail behavior. Standalone Ghidra 12.0.1 uses the matched LX/LE extension; REA has not analyzed this executable. No incompatible symbol maps have been applied.

One separately approved DOSBox-X reference attempt reached DOS/4GW 1.97 and stopped at a missing `config.ini` check. No menu or mission was reached. The original archive and disc inputs were unchanged; logs and disposable-copy changes are recorded in [reference-run-result.json](research/variants/reference-run-result.json). No installer or launcher fallback ran.

All original resources, downloaded tools, raw decompiler exports and build caches stay under ignored `private/`. No commercial assets are embedded in Rust or synthetic tests. Third-party research license findings are in [independent-review.md](research/variants/independent-review.md). No product distribution license has been selected, and no remote publication is part of this work.

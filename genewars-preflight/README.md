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

`scripts/cargo.py` confines Cargo/Rustup homes and build output to `private/`. Core/assets/render use only the standard library. The desktop binaries pin minifb0.29.0 and winit0.30.13/softbuffer0.4.8, with checksums in Cargo.lock and a Windows dependency audit in `research/variants/windows-dependency-review.json`. GNU Rust1.99.0 is used on this host. Its bundled GNU import-library tool lacks an assembler; `scripts/prepare_llvm_dlltool.py` prepares a checksum-pinned official LLVM equivalent privately. See `research/integration/llvm-dlltool.json`. The build wrapper only sets child-process environment variables and never installs tools or changes global PATH.

`scripts/verify.py` checks Rust, all RNC records, ANB accounting, real panel pixels, bounded native-window launches, static arithmetic/LE mapping, private-file Git exclusion, archive SHA-256 and dated preflight preservation. One GPF ANB record retains an explicit unresolved failure; BONES now has a separately traced codec admission. The verifier does not redo broad extraction or overwrite the preflight receipt. Results go to `research/integration/verification.json`.

## Native research utility

The built executable is `private/build/rust-target/debug/genewars-tools.exe`:

```text
genewars-tools.exe rnc-info INPUT
genewars-tools.exe rnc-verify INPUT [MAX_BYTES]
genewars-tools.exe rnc-decode INPUT OUTPUT [MAX_BYTES]
genewars-tools.exe rnc-prefix INPUT OUTPUT [MAX_BYTES]
genewars-tools.exe panel-render PANEL_RNC MAIN_PAL OUTPUT_BMP
genewars-tools.exe sprite-verify SPRITE_DATABASE_PREFIX DECODED_TAB
genewars-tools.exe sprite-decode SPRITE_DATABASE_PREFIX DECODED_TAB RECORD INDEX_OUTPUT MASK_OUTPUT
genewars-tools.exe sprite4-verify SPRITE_DATABASE_PREFIX DECODED_TAB BASE
genewars-tools.exe sprite4-decode SPRITE_DATABASE_PREFIX DECODED_TAB RECORD BASE INDEX_OUTPUT MASK_OUTPUT
genewars-tools.exe distance 0 0 3 4
```

`INPUT` may be `-` for binary stdin. Inputs are bounded to 64 MiB. The output limit defaults to 64 MiB and can be tightened. `rnc-decode` requires a complete single RNC stream. `rnc-prefix` explicitly reports consumed and remaining bytes; callers must account for the tail. Existing differing output files are never overwritten. The distance command returns squared distance using the CD candidate's wrapped arithmetic, with no assumed gameplay units.

`scripts/source_member.py ENTRY_ID --output private/PATH` copies one manifest-identified member, validates source and output hashes, and rejects ambiguous filename selection. Nested ZIP and existing raw-CD reads are targeted; no bundled programs are executed. Full-corpus validation is `python -B scripts/validate_rnc_corpus.py`.

`python -B scripts/render_panel.py` reconstructs the exact CD GENEPANL/MAIN.PAL pair and writes a private BMP/PNG. RGB6 display conversion is explicit floor(component*255/63), not a measured VGA monitor transfer. `python -B scripts/validate_sprites.py` accounts for every ANB; it intentionally exits nonzero while the documented GPF case remains unresolved.

Sprite CLI commands use an explicitly supplied database extent and table. For an embedded ANB, pass the original prefix ending at the first metadata offset, not the entire file; the corpus script does this automatically. Indexed8 and packed4 are distinct caller-selected codecs. The latter is admitted for 25 exact creature hashes and one separately traced BONES hash by executable caller evidence. BASE is an explicit 0-255 diagnostic palette offset; runtime palette and lighting selection remain separate. Desktop ANB modes enforce the metadata bound internally.

The native asset window can be opened with:

```text
python -B scripts/cargo.py run -p genewars-desktop --offline --locked -- panel ../private/integration/panel/GENEPANL.RNC ../private/integration/panel/MAIN.PAL
```

Paths passed through Cargo resolve from `rust/`. Escape or window close exits; `--frames 3` limits an automated presentation check. Actual three-frame launches passed. Physical keyboard/close input has not yet been independently exercised. The window displays original decoded assets; it is an inspector, not a game menu or simulation.

The separate event-driven control inspector uses the verified CD IFCUSTOM arrows:

```text
python -B scripts/cargo.py run -p genewars-desktop --bin genewars-controls --offline --locked -- ../private/integration/panel/ifcustom-controls-smoke-20261012.anb ../private/integration/panel/MAIN.PAL
```

A supervised Windows test observed events27/28, a nonactivating margin click and Escape exit. See `research/integration/native-controls-observation.json`. It preserves ordered button transitions and uses current cursor coordinates at each diagnostic60Hz packet update. This is not original game cadence or a full interface. The portable core also exposes a bounded construction-stage resource debit; callback, cancellation and later world mutations remain outside that helper.

## Evidence and distribution boundaries

The CD candidate is SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Its static analysis does not establish canonical retail behavior. Standalone Ghidra 12.0.1 uses the matched LX/LE extension; REA has not analyzed this executable. No incompatible symbol maps have been applied.

The first DOSBox-X reference attempt stopped at missing `config.ini`. A second session, using a statically derived configuration, displayed the introduction and menu/options artwork. Missing dynamic labels and unresolved guest mouse motion prevented a mission test. It ended by operator termination after UI close failed; its exit status is not a game result. Window screenshots and logs remain private. The original archive, media and executable hashes are unchanged; see [run 002 results](research/variants/reference-run-002-result.json). The latest user instruction authorizes necessary reference reruns. No installer or patcher ran.

All original resources, downloaded tools, raw decompiler exports and build caches stay under ignored `private/`. No commercial assets are embedded in Rust or synthetic tests. Third-party research license findings are in [independent-review.md](research/variants/independent-review.md). No product distribution license has been selected, and no remote publication is part of this work.

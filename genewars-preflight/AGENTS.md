# GeneWars reconstruction

The user requested execution of GENEWARS_CONTINUOUS_MULTIAGENT_CODEX_HANDOFF.md on 2026-10-12. Scope is faithful evidence-backed Rust reconstruction with isolated standalone Ghidra/LX-loader tooling. Original DOS programs/installers must not be executed without separately scoped authorization. No public uploads, global tool/MCP changes, or proprietary bytes in tracked files.

Preserve existing preflight report, manifest, diagnostics and verification as the dated baseline. Do not rerun broad extraction. Use targeted, hashed reads of the supplied archive or existing private disc. The CD is a provisional analysis candidate, not established canonical retail behavior.

All source binaries, manual text/images, raw decompilation, downloaded tools, caches and build outputs stay under ignored `private/`. Public-facing research is metadata, concise summaries and independently written code/tests. Check third-party licenses before copying code or symbol maps.

## Ownership and routing

- Root owns Cargo workspace/lock, integration scripts, aggregate evidence/status/feature docs, and cross-lane integration.
- SOL-01 (`gpt-6.1-sol`) owns `research/ghidra/`, `scripts/ghidra/`, and private Ghidra/JDK installations/projects.
- LUNA-01 (`gpt-6-luna`) owns the assets/render/desktop crates, `scripts/desktop/`, `research/formats/`, and private asset samples.
- LUNA-02 (`gpt-6-luna`) owns `research/manual/`, `docs/game-system-matrix.md`, `scripts/manual/`, and private manual files.
- SOL-02 (`gpt-6.1-sol`) owns `research/variants/`, `scripts/reference/`, private review files and the prepared DOSBox/reference-run directories. One separately approved original session reached DOS/4GW and stopped at missing config.ini; see reference-run-result.json. Do not treat that completed one-session approval as blanket original/installer authorization.
- Three concurrent children are the available maximum alongside root. Schedule independent Sol review after a lane frees. Do not change model family without user authorization.

Do not edit another lane's files or revert others' changes. Share exact artifact paths, hashes, actual command/test results and unresolved evidence gaps. Mark claims Observed, Corroborated, Provisional, Unknown or Contradicted, with manual/binary/execution source status. A function name alone is not behavioral proof.

## Commands

- `python -B scripts/cargo.py test --workspace --offline --locked` uses project-local Rust homes and private build output.
- `python -B scripts/verify.py` runs current integration checks and preserves the dated preflight. Documented ANB failures are accounted for, not silently marked decoded.
- `python -B scripts/verify_preflight.py` is the existing preflight verifier; preserve its baseline results and use the integration verifier for further work.
- Original archive: `C:\Users\fallo\Downloads\genewars.zip`; expected SHA256 `8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906`.

No playable game or behavioral parity is established yet. Do not invent simulation defaults to make an incomplete milestone appear finished.

# GeneWars reconstruction and platform research

Private preservation repository created on 18 October 2026. It contains the existing reconstruction, unfinished work, research scripts and evidence summaries, plus the completed platform architecture report. **This is not a finished playable GeneWars release.**

## Start here

- [Complete platform architecture research report](GENEWARS_PLATFORM_ARCHITECTURE_RESEARCH_REPORT.md)
- [Research assignment and constraints](GENEWARS_CODEX_FRESH_EYES_PLATFORM_RESEARCH_HANDOFF.md)
- [Decision and timeline clarification](RESEARCH_DECISION_SUMMARY.md)
- [Existing reconstruction workspace](genewars-preflight/README.md)
- [Current-state history](genewars-preflight/docs/CURRENT_STATE.md)
- [Latest F24 checkpoint](genewars-preflight/research/integration/f24-b12-b0-closed-checkpoint-20261018.json)
- [Archive scope, provenance and exclusions](ARCHIVE_SCOPE.md)

## Preserved state

The source checkout was on `codex/genewars-reconstruction` at `3bcf6ff9b2d40212c2f501a9bff489fbf83e8f71`. The archive preserves 214 GeneWars commits on `archive/committed-source`; `main` adds the actual working-tree state, including 136 previously untracked files, and the research materials. Existing source files are retained under `genewars-preflight/` so their relative paths remain intact.

Exceland is accepted in the reconstruction evidence (1/24); Krell is unfinished. The latest recorded source-corrected frontier is 6119, with native agreement accepted at 6100 and 6119 native parity still pending. These are retained results, not fresh tests performed for this upload. Source identity is QA BETA V5; retail equivalence is unresolved.

The recommendation is to preserve verified Rust work, use the original game as a broader behavioral reference, and build one portable simulation that can admit proven C/C++ components. The reported native-assembly route remains conditional on a reproducible whole-mission/save demonstration. No implementation of that recommendation has been authorized by this archival upload.

## What is included

All 1,970 current non-ignored source/research files from the live GeneWars project are preserved, including tracked modifications and unfinished files. [The manifest](archive/current-files.json) records their SHA-256 hashes and original tracked/untracked status. Historical authored sources and selected summaries from ignored private research directories are preserved separately in a [deduplicated research archive](archive/private-research/README.md).

The original game files, raw binary captures, screenshots rendered from copyrighted game assets, decompilation exports, Ghidra databases, downloaded tools and build caches remain local. Their omission is intentional; this repository is a source/research archive, not a complete backup of all private workstation data. See [the scope document](ARCHIVE_SCOPE.md).

## Reproduction and rights

Build/research commands are documented in the existing workspace. Many conformance tests need user-owned game data and excluded private captures. No build or gameplay test was run during publication. Historical instructions may describe superseded goals; read the new report and current user instructions first.

No new blanket license is granted by this upload. Original EA/Bullfrog content, third-party tools and source-file notices retain their respective rights. The repository remains private while provenance and any future public licensing are reviewed. No credentials were intentionally included; common token/private-key patterns were checked in the imported history and selected source files.


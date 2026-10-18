# Archive scope and provenance

This repository was created at the user's request to preserve the GeneWars work and completed architectural research on GitHub. The original checkout, its index, branches, files, Ghidra project and Hermes services were not altered by the archival process.

## Git and working-tree preservation

- Source parent repository: `C:/Users/fallo/Documents/ChatGPT/Freecad`.
- Selected project: `genewars-preflight/`; unrelated FreeCAD files are excluded.
- Source commit: `3bcf6ff9b2d40212c2f501a9bff489fbf83e8f71`.
- 214 relevant commits were exported/imported into a separate repository. In this export the final imported commit ID matches the original ID.
- `archive/committed-source` points to that original committed state. `main` additionally records the current modified and previously untracked files.
- [Source status](archive/source-working-tree-status.txt), [file hash manifest](archive/current-files.json), and [import metadata](archive/import-provenance.json) distinguish committed and unfinished work.

Files were copied without rewriting production source, receipt expectations or line endings. No integration claim is added for unfinished work. Creating an archival commit does not imply those files passed tests or were approved for integration.

## Research report

The root report is the GitHub-readable edition: its local-file citation targets were changed to repository-relative links, with the F24 private summary linked to its archived content. The exact original report is preserved under `archive/reports/`. The later discussion clarified that 6–18+ engineer-months is a speculative planning allowance, not a measured completion forecast; see the decision summary.

## Private research and omissions

The private source archive preserves authored `.rs`, `.py`, `.java`, `.toml`, `.lock`, `.md`, `.patch`, and `.ps1` files, plus selected `summary.json` and attempt-log JSON files, from four research directories. Files are deduplicated by SHA-256, with original paths in its index. No archived script was run. Source-file notices and provenance still apply; private visibility is not a substitute for licensing review.

The following remain on the original workstation:

- User-owned game archives, disc contents, executables, maps, saved games, audio/art resources and raw captured world buffers.
- Original disassembly/decompilation exports and active Ghidra project files.
- Rendered screenshots and other binary reference captures.
- Downloaded toolchains, third-party installations, caches, generated binaries and dependency build trees.
- Large/generated or nonselected private metadata and logs. The four inventoried research directories have a path/size inventory of their omitted files; other private directories are described but not recursively archived.

The archive is therefore not a disaster-recovery backup of the full private evidence corpus. It preserves the source/history/report and additional authored research while leaving original copyrighted material and large generated data local. Do not report private-input-dependent tests as reproducible from GitHub alone.

## Privacy and verification

Common GitHub/OpenAI-style keys, AWS access-key IDs, private-key headers and credentials embedded in URLs were scanned without printing credential values. No game builds/tests, worker restarts, executable downloads or external maintainer messages were performed. Repository visibility is private. A future public release needs an explicit visibility decision and a complete code/content rights review.


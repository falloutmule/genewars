# Private research source preservation

This directory preserves authored research sources and selected small summaries from the original ignored `private/integration`, `private/review`, `private/recovery`, and `private/associated-activation-validation` directories. These are historical/unfinished research artifacts, not certified production code.

`index.json` maps each original project-relative path to its exact SHA-256 content under `content/`. Identical files are stored once. Keep the listed original extension and bytes when recovering a file into a separate research copy. No archived scripts were executed during publication.

`local-only-index.json` inventories omitted files from those four directories by path and byte length. Original game images/assets/saves, raw instruction exports, binary fixtures, rendered captures, generated executables, caches and nonselected metadata remain on the source workstation. The other private directories (`assets`, `build`, `disc`, `ghidra`, `manual`, `reference-run`, `tests`, `tools`) are not copied; they contain user game inputs, tool installations, projects, captures and generated outputs. This GitHub repository is not a complete backup of the tens of gigabytes of private data.

Source and selected summary files were scanned for common credential patterns. This is not a license audit or a proof that all test dependencies are public. Original rights and file-specific notices remain applicable. See the root archive README and architecture report.

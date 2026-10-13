# GeneWars preflight assessment

Assessment date: **2026-10-12 (America/Denver)**. Scope: static inventory and toolchain assessment of the user-supplied archive. The attached handoff was reference material; the approved preflight plan governed the work.

## Result

**The static inventory is complete for the discovered archive and filesystem layers.** It accounts for **1,681 file records and 84 directories**, including containers and their members, with **457 groups of identical file contents**. No archive/filesystem traversal is blocked. These are logical inventory records across several packages, not a count of unique game assets or files installed on a machine.

There are **four distinct `GW.EXE` binaries, all LE**. REA's documented target admission rejects LE; no local REA diagnostic was run because REA was not found. A metadata-matched standalone Ghidra 12.0.1 / LX-loader v12.0.1 pair exists, but import and runtime compatibility remain **untested**. The discovered Java 8 installation is below its JDK 21 requirement.

**No authoritative retail/reference build is established.** The disc volume labels itself `GENEWARS QA BETA V5`; one RIP contains demonstrably modified bytes, and the other carries an unverified bad-dump label. Preserve all four executables as separate evidence.

## Inputs, integrity, and outputs

Original: `C:\Users\fallo\Downloads\genewars.zip`, **508,689,422 bytes**. SHA-256 before and after inspection:

```text
8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906
```

The original archive was unchanged. Scratch copies occupy `genewars-preflight/private/`; all nine retained files there were verified Git-ignored, with none tracked. Only controlled archive members were copied to generated scratch paths. No disc was mounted, bundled program executed, patch applied, software installed, MCP setting changed, or content uploaded.

Primary deliverables:

- `game-files-manifest.json`: nested source paths, stable ancestry IDs, sizes, SHA-256, signatures, classifications, executable evidence, statuses, duplicate references, and checks. Directory hashes are null by design.
- `toolchain-diagnostics.txt`: exact discovery commands, outputs, exit codes, bounded search limits, and explicit **not run** entries for `rea doctor --json` and `rea providers --json`.
- This report.

Supporting evidence: `disc-inventory.json`, `variant-comparison.json`, `tooling-sources.json`, and `verification.json`. Reproduction scripts are in `../scripts/`. The JSON schema is version 1; no application API was introduced.

## Archive and disc evidence — observed

| Outer member | Bytes | Inventory result |
|---|---:|---|
| `Genewars_Manual_DOS_EN.pdf` | 3,825,521 | Hashed; PDF signature identified; pages not interpreted |
| `Genewars_DOS_EN_RIP-Version (1).zip` | 23,174,103 | 341 files, including patcher source/build material |
| `Genewars_DOS_EN_ISO-Version.7z` | 431,901,858 | 7 files + 1 directory; raw disc filesystem inspected |
| `Genewars_Patch_DOS_EN_Patch.zip` | 1,850,177 | 8 files; `Gwp.exe` additionally exposes a ZIP of 25 files |
| `Genewars_DOS_EN_RIP-Version.zip` | 48,158,199 | 419 files |

`Gwp.exe` is a DOS MZ self-extracting ZIP with a PKLITE marker. Its archive members were read directly using Python's ZIP reader, including `patch/GW.EXE`; the executable was never run. Every read ZIP member passed the reader's CRC check.

The disc package's `.img` is **630,947,520 bytes = 268,260 × 2,352-byte raw sectors**. Its `.sub` is **25,752,960 bytes = 268,260 × 96 bytes**. CUE and CCD metadata agree on one MODE1/2352 data track and four audio tracks. Audio starts are LBAs 101726, 142728, 187202, and 246792; the first audio pregap begins at 101576. Leadout agrees with image length.

The ISO9660 primary volume descriptor declares 101,424 logical 2,048-byte blocks and volume name `GENEWARS QA BETA V5`. Its creation field claims 1996-09-02. Those are observed metadata values, **not independently authenticated release dates or edition identity**.

The filesystem contains **876 files and 54 directories**. All 1,037 directory records reconcile to 876 files, 53 non-root directories, and 108 dot/dotdot records. Both endian path tables agree with the directory tree; all extents are in bounds. **101,408 MODE1 sector headers** were checked with no invalid sync/mode headers. All four CRC32 values supplied in the SFV match the corresponding bytes; that establishes internal consistency, not external authenticity.

Track ranges, pregap, space outside the declared filesystem, and subchannel data retain separate hashes/metadata. Audio and subchannels were not decoded; sector ECC/EDC was not validated. The 876 ISO files yielded no recognized nested ZIP/SFX, 7z, RAR, CAB, gzip, SZDD, or tested InstallShield container signatures. This is bounded signature probing, not proof that opaque files cannot contain other formats.

There are **331 RNC-signature resource records** across the whole manifest, including 124 on the disc. They are explicitly marked compressed resources not decoded. Other proprietary resource formats are likewise inventoried at file level, not parsed into game records.

## Executable and variant comparison — observed

All four main binaries have MZ stubs whose actual new-header pointers lead to LE headers. Each has four LE objects. Header and object-table bounds pass; relocation semantics and successful loading are not established.

| Variant / member | Bytes | LE offset | Entry object + offset | Candidate virtual address |
|---|---:|---:|---:|---:|
| CD `GAME/GW.EXE;1` | 1,265,994 | `0x8F50` | `1 + 0x96E40` | `0xA6E40` |
| Smaller RIP `Gene_Wars_RePack/GW.EXE` | 1,265,994 | `0x8F50` | `1 + 0x96E40` | `0xA6E40` |
| Larger RIP `GW.EXE` | 5,184,949 | `0x8F78` | `1 + 0xB2438` | `0xC2438` |
| Patch payload `patch/GW.EXE` | 1,251,658 | `0x8F50` | `1 + 0x9F96C` | `0xAF96C` |

SHA-256 identities, respectively:

```text
CD:          4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba
Smaller RIP: 6a8c75b7de5f4e7f27c5378b712e04c24fd106778429f5765c49a641fac90e64
Larger RIP:  824bcf84a6bff6f3b98ec8eb94e376b37ee55eb0a2fa1dbbb3d4a3b6f3249d1e
Patch:       2a0a51a5a8170cb7555cc0a745c08ed371c1500503d5cec08d77190d39822a3f
```

The CD and smaller RIP executables differ at **12 bytes**. Six differences exactly match the bundled unofficial patcher's documented before/after values: `0x4E033`, `0x4E034`, `0x5619F`, `0xAE6AD`, `0x12CB20`, and `0x12CB21`. Six additional differences occur at `0x4DBB0` and `0x4EACF–0x4EAD3`. Their authorship, purpose, and gameplay effects are unverified. Exact byte values are in `variant-comparison.json`; do not attribute all modifications to the documented patcher.

The larger RIP's `file_id.diz` claims English version 1.0 and a `b1`/bad-version designation. This is an unverified packaging label. Its larger executable and different object layout establish a distinct binary, not its authenticity or quality. The patch's bundled prose claims multiplayer and speed-control changes; those behaviors were not tested.

| Package/tree | Files | Total file bytes | Signature-identified video files |
|---|---:|---:|---:|
| Smaller RIP | 341 | 38,152,732 | 2 |
| Larger RIP | 419 | 76,890,738 | 7 |
| CD `GAME/` | 490 | 186,368,166 | 9 |
| Patch payload only | 25 | 4,823,768 | 0 |

After removing package roots, folding case, and stripping ISO version suffixes, the smaller RIP and CD game tree share 305 paths: 300 have identical hashes; five differ (`DATA/CYAN.DAT`, `DATA/ICE.DAT`, `DATA/RED.DAT`, `GW.ERR`, `GW.EXE`). There are 36 smaller-RIP-only and 185 CD-only paths under that normalization. Language directories and installation layouts were not remapped, so these counts are not a direct missing-assets assessment. The CD contains six named speech-language directories; the smaller RIP has a flattened speech layout.

The CD's 37,660-byte `GAME/GENEWARS.EXE;1` is separately classified as a DOS MZ launcher/game-entry candidate; it is not the LE game binary. DOS extenders, sound setup, patchers, and the disc's DirectX2 installers/drivers are separate records. Across the entire inventory, 326 MZ-bearing records comprise 8 DOS MZ, 90 LE, 78 PE, and 150 NE files. Analyzing a setup program or MZ stub would not establish analysis of `GW.EXE`.

## Toolchain route — observed, documented, and untested

**Observed locally:** Node 24.14.0 (plus bundled Node 24.19.0) and npm 11.9.0; Java/javac 1.8.0_211, 64-bit. REA, Ghidra launchers, and Gradle were not found on PATH or in the reported bounded locations. This is not an exhaustive machine-wide absence claim. No REA/Ghidra MCP tools were exposed in this session. Filesystem admission checks requiring CIM were denied and remain unverified.

**Documented:** REA at commit `8fc2043eb8bcedbf34dce064f977e455438635d6` identifies package version 6.4.0. Its Node range accepts the observed Node versions; its Ghidra route uses 12.1.x and a full 64-bit JDK 21+. DOS admission rejects declared NE/LE/LX; Windows P0 admits native PE, not this DOS LE game. These are source-based conclusions, not observed local provider failures. Sources: [installation](https://github.com/morluto/rea/blob/8fc2043eb8bcedbf34dce064f977e455438635d6/docs/installation.md), [DOS admission](https://github.com/morluto/rea/blob/8fc2043eb8bcedbf34dce064f977e455438635d6/docs/ghidra-dos.md), [Windows P0](https://github.com/morluto/rea/blob/8fc2043eb8bcedbf34dce064f977e455438635d6/docs/windows-ghidra-p0.md).

**Fallback candidate:** standalone Ghidra **12.0.1**, full 64-bit **JDK 21+**, and loader **v12.0.1**, whose published extension asset is `ghidra_12.0.1_PUBLIC_20260129_ghidra-lx-loader.zip`. Release metadata provides an exact version pairing. Published asset hashes, sizes, URLs, and pinned source commits are recorded in `tooling-sources.json`; the installation archives were not downloaded. This Ghidra version is outside REA's documented 12.1.x range, and adding an extension would not establish REA admission. Sources: [loader release](https://github.com/yetmorecode/ghidra-lx-loader/releases/tag/v12.0.1), [Ghidra 12.0.1 properties](https://github.com/NationalSecurityAgency/ghidra/blob/Ghidra_12.0.1_build/Ghidra/application.properties).

The [GeneWars helper project](https://github.com/mefistotelis/genewars-re-helpers/blob/a85d5c517b524aa4906a720330ed1435bf8b65cf/README.md) documents LE loading and symbol imports. Its symbols have **not** been matched to these hashes or imported. Neither that workflow nor matching release metadata proves local loader success.

## Subsequent validation, not performed in this phase

1. Establish which supplied build should define intended behavior; preserve the CD label, modified RIP, different large binary, and patch payload as separate provenance. No canonical reference is selected by this report.
2. In a separately scoped setup phase, obtain and verify the pinned Ghidra/loader archives, provide a compatible JDK, and import a copy of the selected LE executable into a fresh standalone project.
3. Confirm the LE loader is used, objects/pages are mapped correctly, and entry coordinates agree with this inventory. Validate relocations and representative cross-object references before trusting decompilation. A raw import or analysis of only the MZ stub is insufficient.
4. Match any recovered symbols against the exact build, object layout, and representative address/function boundaries before import. Keep symbol-derived hypotheses separate from measured behavior.
5. Only then scope execution-based comparisons and asset decoding. Original execution, Rust implementation, and SFHS feasibility remain untested and outside this delivery.

## Reproduction and acceptance

From `C:\Users\fallo\Documents\ChatGPT\Freecad\genewars-preflight`, with the original archive still at its supplied path:

```powershell
python -B scripts/prepare_inputs.py
python -B scripts/disc_inventory.py
python -B scripts/build_manifest.py
python -B scripts/compare_variants.py
python -B scripts/verify_preflight.py
```

Only Python standard-library modules and the existing Windows `tar.exe` are needed for these inventory scripts. For a subsequent disc run, `disc_inventory.py --reuse-verified` reuses scratch files only after checking the source package and each cached member's size and SHA-256. Toolchain diagnostics are a dated snapshot; their exact read-only commands and upstream commit identities are preserved for repetition.

`verification.json` records **all 12 acceptance checks passing**, including seven synthetic parser/ancestry tests, successful rebuilds, byte-identical repeated manifest/comparison output, archive integrity, source-bound disc evidence, unique ancestry IDs, and exclusion of every retained source copy from Git. Synthetic tests cover truncated/overlapping LE structures, invalid pointers, invalid entry offsets, embedded ZIPs, duplicate nested names, corrupt ZIP status, and rejection of stale disc evidence.

**Implemented and tested:** archive/filesystem inventory, hashes, structural executable inspection, variant comparison, documented compatibility assessment, and reproducibility checks. **Still untested:** proprietary asset decoding, physical-disc authenticity/ECC, loader import/relocations, symbol compatibility, and all game behavior. The assessment is complete; those boundaries remain explicit prerequisites for later work.

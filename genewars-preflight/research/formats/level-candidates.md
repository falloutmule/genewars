# Level-resource candidates: manifest inventory and bounded sample

## Scope and confidence

This is a filename and byte-structure inventory with the now-confirmed first campaign and mission loader paths. Filenames are preserved as manifest paths; names such as `CONQ`, `HDR`, `PLR`, and `LEVSCR` are not treated as format or behavior evidence beyond the specific traced paths below. Only the bounded header title and the first mission loader's byte-copy extent are exposed; other fields remain opaque.

## Exact CD groups

`GAME/LEVELS/` in the CD manifest contains 92 files (manifest entries `root/2/disc/152–243`). The exact name groups and extents are:

| Filename family | Count | Size per file | CD manifest entries |
|---|---:|---:|---|
| `CONQ001–024.DAT`, `CONQ030.DAT`, `CONQ040.DAT` | 26 | 343,958 bytes | `root/2/disc/152–177` |
| `CUST001–012.DAT` | 12 | 343,958 bytes | `root/2/disc/178–189` |
| `HDRCO1–25.DAT`, `HDRCO30.DAT`, `HDRCO40.DAT` | 27 | 699 bytes | `root/2/disc/190–216` |
| `HDRCU1–12.DAT` | 12 | 699 bytes | `root/2/disc/217–228` |
| `PLR1–13.DAT`, `PLR23.DAT`, `PLR24.DAT` | 15 | 142 bytes | `root/2/disc/229–243` |

These files have no `RNC` signature and the manifest classifies them as `game_resource_not_decoded`. The 343,958-byte group is structurally distinct from the 699- and 142-byte groups by extent alone. Static traces establish the initial campaign path to `HDRCO1.DAT` and the initial mission path to `CONQ001.DAT`; they do not establish associations for every other numbered member.

Separate visual-resource candidates in `GAME/DATA` include `LEVSCR01–05.DAT` plus matching `.PAL` files (manifest entries `root/2/disc/93–102`; each decoded DAT is 307,200 bytes), `B32A–D.DAT` (entries `40–43`; each decodes to 163,840 bytes), and the twelve `.ANB` names `ALL1–3`, `DESERT1–3`, `GRASS1–3`, and `SNOW1–3` (entries `341–352`). These are listed as adjacent candidates only; there is no asserted link from them to a mission record in this note.

## One targeted raw-file inspection

The small sample set copied into ignored `private/assets/level-candidates/` is limited to `CONQ001.DAT` and `HDRCO1.DAT`. Both came from the hash-checked run-003 staged CD tree and match their manifest records exactly:

| File | Manifest entry | Bytes | SHA-256 |
|---|---|---:|---|
| `GAME/LEVELS/CONQ001.DAT` | `root/2/disc/152` | 343,958 | `72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2` |
| `GAME/LEVELS/HDRCO1.DAT` | `root/2/disc/190` | 699 | `5e1f382ea68f57f1cec03a3f7de2c4903ca127519481da98232a7f1c3a46aa11` |

The larger file is raw data, not an RNC stream. Its first little-endian 32-bit value is `0x55336` (348,982), which selects the traced fast path. The loader reads those first four bytes to state+0, then copies `0x3B55A` further bytes to state+4. Thus the exact copied source prefix is `[0, 0x3B55E)` (243,038 bytes), with source and destination offsets matching. The copy contains four initial player-record byte ranges at `0x14 + i*0x70D` for `i=0..3`. Cleanup callbacks normalize existing live player records before the overwrite; after the copy, another normalizer acts on the newly loaded records. The accessor exposes only initial file bytes, not final live player state or a field layout. The region `[0x355E,0x3B55E)` is traversed by a legacy path in 14-byte steps, but its fields remain opaque. The `ConqFastPrefix` accessor validates the tag and required prefix extent, exposes these bounded byte slices, and preserves all following bytes as an opaque tail. The first word's meaning beyond selecting that loader branch remains unknown. The file contains all 256 byte values and 259,370 zero bytes.

`HDRCO1.DAT` is exactly 699 bytes. The traced initial campaign path calls the common header loader for `LEVELS\\hdrco1.dat`; its consumer passes the bounded NUL string at file offset `0x20` to a briefing formatter. `CampaignHeader` exposes only this byte slice and preserves the full source bytes as opaque. All 39 manifest-matched `HDRCO*.DAT` and `HDRCU*.DAT` candidates were checked against the bounded parser: each is 699 bytes and has a NUL terminator at or after offset 32. Across these candidates the parsed string lengths range 3–9 bytes for HDRCO and 5–12 for HDRCU, with no empty strings. Only the HDRCO1 campaign path has confirmed title semantics; the HDRCU consumer is not traced. Fields following that string remain opaque, including numeric fields read at offsets `0x27E`, `0x280`, `0x29A`, and `0x29E` relative to the file.

The selected sample set does not include every file in either filename family. Original bytes remain private and are not embedded in test fixtures.

## Evidence and remaining limits

- Campaign path evidence: `private/ghidra/campaign-header/instructions-00015720.txt` and `private/ghidra/campaign-screen/instructions-00015b90.txt`. The initial single-player branch clears the progress byte, adds one for the first campaign header, and the common loader reads 699 bytes from `LEVELS\\hdrco1.dat`.
- Mission path evidence: `private/ghidra/campaign-level/instructions-000071b70.txt` and `private/ghidra/campaign-level/instructions-00007d090.txt`. The initial mission selection formats `levels\\conq%03d.dat` with progress+1, producing `CONQ001.DAT`; the `0x55336` tag selects the bounded prefix-copy branch described above.
- Header title evidence: the traced consumer at `17640/17850` receives file+`0x20`; the NUL search in `CampaignHeader` is bounded to `[32,699)`. The 666-byte maximum is a safe allocation bound, not evidence of a fixed-width title field. Empty strings are allowed by the observed string consumer.
- The metadata-only first-mission staging receipt is `research/integration/campaign-inputs.json`; `scripts/campaign_inputs.py` stages the exact CD entries for `HDRCO1.DAT`, `CONQ001.DAT`, `PL1.DAT`, `LEVSCR01.DAT/.PAL`, and `TOPLNT0.DAT/.PAL`. Compressed inputs reuse the passed hash-bound RNC corpus outputs. The receipt lists exact-source-hash aliases, but does not include resource bytes.
- Are the `GAME/DATA` candidate groups consumed during the same mission-entry path, or by separate screen/terrain loaders?
- Numbered `CONQ`, `CUST`, `HDRCU`, and `PLR` variants are not assigned broader semantic relationships by this trace.

No mission-level parser is implemented; the runtime normalization of player records and remaining live-state interpretation are outside this bounded resource accessor.

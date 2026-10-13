# Bounded source view for the CD mission file

`genewars-assets::ConqMissionSource` is a borrowed parser for the exact CD fast-path mission layout represented by `CONQ001.DAT`. It admits a 343,958-byte file with the little-endian `0x55336` tag and the four-byte `0x6BE` extension tag at byte 337,054. This is an explicit layout/version admission, not an assertion that other same-name mission files are corrupt.

The parser exposes the direct copied prefix `[0, 243038)`, four raw `(u16 x, u16 y)` words at `0x1C7E`, eight numbered record groups, the raw 52-byte tail globals, its two evidenced bytes (flags at `+0`, active-count at `+28`), a raw four-byte viewport snapshot, and the opaque extension payload. Record bytes, including byte `+6`, remain borrowed and unlabelled. It does not rebase pointers, create live entities, or apply runtime constructors.

The groups use only the measured numeric family/count/stride contracts:

| Family | File offset | Count | Packed stride | End offset |
|---:|---:|---:|---:|---:|
| 0 | 243,038 | 300 | 116 | 277,838 |
| 1 | 277,838 | 512 | 41 | 298,830 |
| 2 | 298,830 | 20 | 140 | 301,630 |
| 3 | 301,630 | 256 | 59 | 316,734 |
| 4 | 316,734 | 200 | 73 | 331,334 |
| 5 | 331,334 | 32 | 41 | 332,646 |
| 6 | 332,646 | 32 | 89 | 335,494 |
| 7 | 335,494 | 16 | 94 | 336,998 |

The groups exactly cover the interval from the fast prefix boundary to the tail globals. The 52 globals end at byte 337,050, the viewport snapshot ends at 337,054, and the `u32` extension tag plus 6,900 opaque payload bytes end at the exact file boundary. No extension payload schema is claimed.

Family 1 has a narrow accessor for source values read by the traced consumer: signed words at record offsets `+7` and `+9`; raw bytes at `+0x15` (flags), `+0x17` (shade), `+0x19` (animation selector), and `+0x1B` (frame selector); and the resource selector at `+0x23`, preserved both as its raw `u8` and as signed `i8`. Source byte `+0x26` is exposed under the destination-oriented name `native_flags_at_2e`, because the traced copy places it at native offset `+0x2E`. On the admitted source variant, the family-1 bytes occupy the same offsets in the 41-byte serialized record and the 49-byte native record; only native bytes `[31,39)` are absent from the serialized record. These accessors preserve source values; they do not apply initialization or claim live-state values, project positions, assign resource names, select a codec, or create drawable entities. The corresponding resource selector and foreground-consumer evidence are summarized in [`plant-color-bridge.md`](../ghidra/plant-color-bridge.md).

The fixed extent/tag was checked against five exact files from the CD/RIP family: `CONQ001.DAT` (manifest IDs `root/1/123`, `root/2/disc/152`), `CONQ002.DAT` (`root/1/117`, `root/2/disc/153`), `CONQ003.DAT` (`root/1/118`, `root/2/disc/154`), `CONQ024.DAT` (`root/1/41`, `root/2/disc/175`), and `CONQ040.DAT` (`root/1/72`, `root/2/disc/177`). Each is 343,958 bytes with the admitted tags and group extents. Per-record bytes differ; the parser therefore keeps their contents opaque.

The distinct `root/4` package lists same-name mission files at 375,830 bytes. Those files are outside this parser's admission. The parser returns `UnsupportedSourceVariantLength` for them; it does not call them malformed or attempt the CD offsets on them.

Format evidence is in [`mission-load-evidence.json`](../ghidra/mission-load-evidence.json); the instruction checks are reproducible with `python -B scripts/ghidra/validate_mission_load_bridge.py`. The source-only prefix contract is also summarized in [`campaign-entry-bridge.md`](../ghidra/campaign-entry-bridge.md). No executable is run by this parser.

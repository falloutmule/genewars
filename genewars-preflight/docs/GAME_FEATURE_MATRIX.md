# Reconstruction feature status

The supplied CD executable is the provisional analysis candidate, SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. It is not established as the canonical behavioral reference. One approved original startup reached DOS/4GW and stopped at missing config.ini; no gameplay was observed. “Implemented” below means a bounded component exists, not that GeneWars is playable.

Manual page citations and documented rules are in [game-system-matrix.md](game-system-matrix.md). Static function evidence is in [function-tranche.md](../research/ghidra/function-tranche.md). Actual command results are in [integration/verification.json](../research/integration/verification.json).

| Subsystem | Documented / static evidence | Rust implementation and tests | Remaining gate |
|---|---|---|---|
| Source and edition provenance | Four distinct LE builds; CD matches helper repository's September 1.0 Git blob metadata; QA disc label still applies | Hash-bound local source reader; dated preflight preserved | Establish coherent behavioral edition; no cross-build symbol import |
| LE analysis | Four mapped objects, entry `0xA6E40`, 244 pages and 26,883 fixup instances checked against Ghidra exports | Independent parser/replay and direct instruction reviews pass | Continue subsystem discovery; compiler/calling conventions require instruction review |
| Resource decompression | All 331 RNC records use method 1; 179 distinct inputs | Bounded RNC1 decoder, CRC and length checks, explicit prefix API; all 331 records pass, including two compound INSTALL resources | Method 2 unsupported; success is not a universal RNC compatibility claim |
| Palettes | 26 distinct decoded `.PAL` inputs, each 768 bytes, byte range 0–63 | Explicit-mode triplet parser; no implicit scaling | Trace actual channel order and display conversion |
| Images, animations, sprites | 128x480 GENEPANL and RGB6 MAIN.PAL binding verified; ANB sprite-data base18 and six-byte embedded table traced | Native panel rendering matches every pixel; strict bounds and explicit codecs accept260/263 records and10990sprites; all2673active packed4 creature sprites independently compared |BONES and GPF remain blocked; runtime creature palette/scale/lighting and animation timing are unresolved |
| Geometry arithmetic | CD VAs `0x73690`, `0x737F0`, `0x97910` directly reviewed | Wrapped squared distance, integer distance and floor square root; five tests plus independent static models | Coordinate gameplay units and subsystem meanings unknown |
| Record queries / grid flags | CD VAs `0x432C0`, `0x8D580`, `0x8D5F0` independently reviewed | Safe slice APIs and overflow/inclusive/bounds/spill/wrap tests pass | Entity/flag meaning remains unknown |
| Input and selection | Manual pages plus CD custom handler0x7E400 | Offset/selection arithmetic, packet latches, delay-one arrow capture and bounded list names implemented and reviewed | Host event retention, exact hit testing integration, timing and game commands; minifb polling loses quick clicks |
| Native host / interface | Exact original panel resource; minifb host | Real native asset window builds and presents three frames; panel export independently pixel-checked | Full menu/mission UI and actual gameplay; physical input test still pending |
| Terrain and plants | Manual PDF 12–13 / printed 20–22 | Not implemented | Level format, terrain codes, growth/harvest rules |
| Specialists and creatures | Manual PDF 6–10 / printed 8–17 | Not implemented | Record meaning, stats, actions, movement/combat |
| Buildings, power, GOOP economy | Manual PDF 9–11 / printed 15–18 | Not implemented | Costs, rates, transactions, build/upgrade state |
| Genetics and breeding | Manual PDF 8–12 / printed 13–20 | Not implemented | Research/stat formulas, compatibility, inheritance |
| Mission and campaign | Manual says 24 planets; first five named | Not implemented | Mission data, objectives, progression, win/loss |
| Ethereals and scoring | Manual describes approval and consequences | Not implemented | Numeric predicates and timers |
| AI and pathfinding | Manual only gives qualitative behavior | Not implemented | Executable algorithms and deterministic state |
| Saves and persistence | Manual options diagram | Not implemented | Schema, versioning, campaign state and round trips |
| Audio | Source assets inventoried | Not implemented | File/codec mapping, timing and mixer behavior |
| Multiplayer | Manual describes IPX and eight levels | Not implemented | Protocol and simulation synchronization |
| Behavioral fidelity | Static evidence plus one approved startup failure | DOS4GW1.97 and missing config.ini observed; source integrity preserved | Complete static config requirements before proposing another scoped session; no menu/mission parity |
| SFHS / phone | Optional additional distribution | Not attempted | Working native core and measured offline single-file build |

Commercial bytes, decoded resources, manual text/images and raw decompiler output remain in ignored `private/`. Synthetic fixtures are independently generated. Product-wide completion is **not achieved**.

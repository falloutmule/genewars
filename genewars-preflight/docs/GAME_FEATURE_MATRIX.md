# Reconstruction feature status

The supplied CD executable is the provisional analysis candidate, SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. It is not established as the canonical behavioral reference. Original run 001 stopped at missing config.ini. Run 002 reached intro/menu/options artwork, with missing text and unresolved guest motion; no gameplay was observed. “Implemented” below means a bounded component exists, not that GeneWars is playable.

Manual page citations and documented rules are in [game-system-matrix.md](game-system-matrix.md). Static function evidence is in [function-tranche.md](../research/ghidra/function-tranche.md). Actual command results are in [integration/verification.json](../research/integration/verification.json).

| Subsystem | Documented / static evidence | Rust implementation and tests | Remaining gate |
|---|---|---|---|
| Source and edition provenance | Four distinct LE builds; CD matches helper repository's September 1.0 Git blob metadata; QA disc label still applies | Hash-bound local source reader; dated preflight preserved | Establish coherent behavioral edition; no cross-build symbol import |
| LE analysis | Four mapped objects, entry `0xA6E40`, 244 pages and 26,883 fixup instances checked against Ghidra exports | Independent parser/replay and direct instruction reviews pass | Continue subsystem discovery; compiler/calling conventions require instruction review |
| Resource decompression | All 331 RNC records use method 1; 179 distinct inputs | Bounded RNC1 decoder, CRC and length checks, explicit prefix API; all 331 records pass, including two compound INSTALL resources | Method 2 unsupported; success is not a universal RNC compatibility claim |
| Palettes | 26 distinct decoded `.PAL` inputs, each 768 bytes, byte range 0–63 | Explicit-mode triplet parser; no implicit scaling | Trace actual channel order and display conversion |
| Images, animations, sprites | 128x480 GENEPANL and RGB6 MAIN.PAL binding verified; ANB sprite-data base18 and six-byte embedded table traced | Native panel rendering matches every pixel; strict bounds and explicit codecs accept 262/263 records and 11,001 sprites; all 2,673 active creature sprites and 10 active BONES sprites independently compared | GPF remains blocked; runtime creature palette/scale/lighting and animation timing are unresolved |
| Geometry arithmetic | CD VAs `0x73690`, `0x737F0`, `0x97910` directly reviewed | Wrapped squared distance, integer distance and floor square root; five tests plus independent static models | Coordinate gameplay units and subsystem meanings unknown |
| Record queries / grid flags | CD VAs `0x432C0`, `0x8D580`, `0x8D5F0` independently reviewed | Safe slice APIs and overflow/inclusive/bounds/spill/wrap tests pass | Entity/flag meaning remains unknown |
| Input and selection | Manual pages plus CD custom handler0x7E400 | Offset/selection arithmetic, packet latches, delay-one arrow capture and bounded list names implemented and reviewed | Ordered host events and exact arrow pixels integrated; original cadence and full game commands remain unproved |
| Native host / interface | Exact original panel and IFCUSTOM resources; minifb inspector plus winit/softbuffer controls | Both native windows pass automated checks; supervised clicks produced events 27/28, margin rejection and Escape exit | Full menu/mission UI and gameplay; held-state and focus-loss UI behavior remain unmeasured |
| Terrain and plants | Manual PDF 12–13 / printed 20–22; PL1–4 texture sampler statically traced | All 56 texture tiles decoded, with 622,592 pixel/opacity comparisons | Level format, terrain semantics, projection, growth/harvest rules |
| Specialists and creatures | Manual PDF 6–10 / printed 8–17 | Not implemented | Record meaning, stats, actions, movement/combat |
| Buildings, power, GOOP economy | Manual PDF 9–11 / printed 15–18; CD stage-three debit caller and 14 costs traced | Signed skill cost and wrapping two-pool debit helper; 3,870,016 independent transaction comparisons | Pool meanings, callbacks, cancellation, construction lifecycle, rates and upgrades |
| Genetics and breeding | Manual PDF 8–12 / printed 13–20 | Not implemented | Research/stat formulas, compatibility, inheritance |
| Mission and campaign | Manual says 24 planets; first five named | Not implemented | Mission data, objectives, progression, win/loss |
| Ethereals and scoring | Manual describes approval and consequences | Not implemented | Numeric predicates and timers |
| AI and pathfinding | Manual only gives qualitative behavior | Not implemented | Executable algorithms and deterministic state |
| Saves and persistence | Manual options diagram | Not implemented | Schema, versioning, campaign state and round trips |
| Audio | Source assets inventoried | Not implemented | File/codec mapping, timing and mixer behavior |
| Multiplayer | Manual describes IPX and eight levels | Not implemented | Protocol and simulation synchronization |
| Behavioral fidelity | Static evidence plus two supervised reference sessions | Config parser/writer traced; second session displayed intro/menu/options artwork; integrity checks passed | Missing labels and guest mouse motion unresolved; no mission/save/reload or gameplay parity |
| SFHS / phone | Optional additional distribution | Not attempted | Working native core and measured offline single-file build |

Commercial bytes, decoded resources, manual text/images and raw decompiler output remain in ignored `private/`. Synthetic fixtures are independently generated. Product-wide completion is **not achieved**.

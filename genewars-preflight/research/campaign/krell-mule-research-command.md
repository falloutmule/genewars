# Krell Mule qualification and ordinary production control

Reference: verified CD QA BETA V5, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. This packet closes two immediate integration questions; it does not count another mission as accepted.

**No new Geneticist research action is required to unlock Mule production on the current reference route.** Source constructor `80460` sets human-player research entry 4 (`player+62A`) to `0x11800` at `80546`. The preceding `804C1..80505` scan appends species 4 when absent from the list at `+6AE`, whose count is `+6E0`. The ordinary human branch is selected by player byte `+2 == 0`; the alternative AI values are not substituted here. Mission restoration `80590` copies the carried range `[61A,6F7)`, preserving research, list and count.

Observed current native fixture `private/review/krell-mule-callbacks-025/frame-660-slot-0-before/players.bin` has player 0 research entry 4 = 71680, list `[4]`, count 1. This is a native carried-route observation, not execution of the original program. Constructor, carry and producer source independently corroborate why this gate is already satisfied. Species list **index 0** selects species **4**. A Geneticist-to-neutral-Mule research-command reconstruction is therefore deferred until an actual later requirement needs it; its mutation/completion behavior is not claimed recovered by this packet.

The ordinary start producer remains `21E20` action 8, detailed in `research/variants/genepod-human-producer.md`. Its research requirement is merely nonzero. For an owned completed idle Genepod, selected species 4 and total funds at least 75, the producer emits opcode 12 with words `[species=4, selected building slot, start=1]`. The Genepod handler remains `814A0`. Select the actual building and use its actual panel state/list index; do not inject research, funds or a forced production job. Panel initialization copies building `+3C` into the list index. In the source-backed first-item state this is index 0; do not override an unrelated existing selection.

## Resource-bound input control

Control `106DBC`, action 8, flags `9128`, origin `(26,344)` has mapped width and height zero because startup fills them. **Its raw descriptor offset 6 means six-byte sprite descriptor index 1, not sprite index 6.**

The mapped resource record `105698` names `data/genepanl.anb` and has destination pointer `185EE7 = MAINPANL receiver 185DFC + EB`. The `154B0` resource iteration calls `9A09C`, whose allocation/load path writes the loaded pointer through record `+1C`. `2BD08..2BD22` calls `98A60` on that resource and stores its sprite table in receiver `+F3`; `2BD93..2BDA4` binds the list at `106D6C`. Binder `A8320` bit-20 branch adds the raw offset to the table pointer and copies descriptor width/height at `A8376..A838A`.

Exact CD `GENEPANL.ANB` (manifest `root/2/disc/85`) descriptor 1 is **29 × 29**. The nominal half-open control rectangle is `[26,55) × [344,373)`. `A86D0` tests that rectangle, then bit 20 routes to sprite opacity test `98B60`; transparent corners are not hits. The actual mask has 665 opaque pixels. Local `(14,14)`, hence control-space `(40,358)` with zero control-manager origin, is a source-validated hit; local `(0,0)` and `(28,28)` miss. A host with a nonzero manager origin must add that original offset, not silently reuse absolute coordinates.

`21E96..21F0A` clears the panel disabled bit only when building completion bit 1 is set, bit `40` is clear, and owner equals selected player. The earlier allied-selection admission does not enable this action for an ally. Additional action-8 animation, work, grade, funds, research and input gates remain as documented in the existing producer report. Native input must preserve the existing capture/event dispatcher and reach the original `19FF70` bit-8 condition with bits `8001` clear; this packet does not invent a shortcut or equate all mouse event masks.

## Reproduction and remaining gate

Run `python -B scripts/ghidra/mission_contract_krell_research.py`. It checks the current fixture and resource mapping, hashes the source listings and asset, derives the sprite mask, and compares eight interior/boundary points against literal `98B60` instructions. Result: **passed**. The JSON contains all pins, mask metadata and observations; no commercial bytes are tracked. No Rust, Cargo, emulator or original-game execution was used.

The next ordinary action is production through this control after the owned Genepod is completed and its panel is ready. Remaining acceptance work is the native producer/capture path and resulting source-backed production lifecycle, owned by the integration lanes. This research gate introduces no new Geneticist implementation blocker.

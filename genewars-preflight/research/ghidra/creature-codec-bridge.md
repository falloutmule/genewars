# Caller-selected creature packed-nibble sprites

This static tranche applies to provisional CD executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. No original execution occurred in this lane. `creature-evidence.json` and `scripts/ghidra/validate_creature_bridge.py` record reproducible instruction checks, listing hashes, exact source admission, and original synthetic vectors.

The earlier corpus-wide assumption that every ANB sprite uses indexed-byte literals is **superseded**. UI blitter `0xA7730` and standard blitters `0x9A660`/`0x9BC40` consume one byte per positive-control pixel. Creature blitters `0x8FBD0`/`0x93400` use packed nibbles. The application chooses these consumers through callers; there is no proven self-identifying codec flag in the six-byte sprite descriptor. Whole-file bounds allowed an incorrect consumer to traverse metadata. A passing whole-file parse is insufficient evidence of a sprite's validity.

Startup `0x1D9A3–0x1D9AF` passes object `0x180E20` to loader `0x6AC50`. That loader reads exactly 25 filename pointers from `0x104494`, loads the databases, passes them through `0x98A60`, and stores each animation-table base at object+0x64+4*index. Custom creature display `0x22FD0` reads those animation slots at `0x180E84`; its drawing calls `0x947E0`/`0x94BC0`, whose frame-component walkers invoke `0x8FBD0`/`0x93400`. This establishes an actual file-to-consumer bridge for the 25 admitted files, rather than identifying them by artwork or a filename alone.

The receipt admits 25 unique input hashes and their 50 exact aliases from CD and root/1 Gene_Wars_RePack. Root/4 is a different package with the larger executable and is not silently admitted by filename. BONES is not in the 25-name loader and remains a separate unresolved consumer. GPF's wrapper/CRC issue remains separate.

`0x8FBD0` takes x in EAX, y in EDX, sprite descriptor in EBX, palette base byte in CL, and an optional translation-table pointer as one stack argument. Descriptor layout remains u32 relative pixel offset, u8 width, u8 height; normal ANB pixel base is file+18. Signed controls have these meanings:

- Positive N: N opaque pixels occupy ceil(N/2) bytes; high nibble first, then low nibble. An odd final pixel uses the high nibble and ignores the low padding nibble. Nibble zero is opaque.
- Negative -1 through -128: skip that magnitude of transparent pixels; no literal payload.
- Zero: end the row. Early row termination leaves the remaining destination pixels transparent.

Without a translation table, the destination byte is `(u8_base + nibble) mod256`. With a table, the original indexes `table[usize(base)+nibble]`, **without** first wrapping to a byte; a full base-independent table requires 271 entries. A port offering that option must check the caller's table bounds. The base-only decoder may preserve raw nibble values with base zero; the actual palette-bank selection remains caller state and must be explicit.

Horizontal clipping changes writes but not run advancement: the original still consumes the entire packed run. It scans remaining controls to a zero for intermediate rows after width exhaustion. On the last visible row it may return as soon as width is exhausted, without reading that row's final zero. A safe complete-stream parser that requires exactly height zero terminators is a deliberately narrower structural contract, not an exact execution trace. Scaling, flip, clipping, lighting and animation scheduling are not claimed as implemented by a base-only full-raster decoder. `0x93400` dispatches scale>=256 to `0x8FBD0`; its smaller-scale path separately reads packed nibbles.

The urgent audit's 480 metadata crossings are therefore evidence that the wrong consumer was applied to creature sprites. Corpus repair must use the explicit admitted codec and preserve metadata boundary checks; it must never select a codec by whichever parse succeeds.

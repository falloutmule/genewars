# Independently reviewed opaque-record contracts

Scope: CD candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba` only. Reviewed direct disassembly exported by SOL-01, plus mapped object bytes to reconcile function hashes. No helper symbols, original execution, or entity interpretation was used. Synthetic expected vectors and reproduction are `primitive-contract-vectors.json` and `primitive_contract_checks.py`.

## Record-radius predicate, VA 0x432C0

**Observed ABI and iteration:** EAX points to a structure whose first byte is an unsigned group selector. EDX contains query X in its low word and query Y in its high word. Group address is `0x1645AE + u8(selector) * 0x10C0`. The function scans exactly 64 successive records, stride `0x43` (67 bytes), in ascending order, returning at the first match. The selector is reread on each record iteration; ordinary immutable slice semantics assume the original memory is not concurrently mutated. Original truth is low AL only; upper EAX is neither cleared nor a meaningful full-width boolean.

**Observed record fields:**

| Offset | Access | Test or interpretation |
|---|---|---|
| `+6` | unsigned byte | Must be nonzero, including values above 1 |
| `+7..+10` | packed dword | Little-endian X and Y signed words for the wrapped-distance helper |
| `+0x2D` | unsigned byte | Must equal 3 exactly |
| `+0x2E` | unsigned byte | Radius parameter, no observed range check |
| `+0x31` | low byte of word read | Bit 0 must be set; all other low-byte bits ignored |
| `+0x32` | high byte of same word read | Physically read, then cleared/ignored |

After the three qualification checks, the exact arithmetic is:

```text
difference(component) = sign_extend_i16(low16(2 * (record - query))) >> 1
d2 = difference(x)^2 + difference(y)^2
radius = 1536 + 768 * u32(record[0x2E])
threshold = low32(radius * radius)
qualifies iff d2 <= threshold, comparing as unsigned u32
```

`d2` is at most 536,870,912. Radius itself never exceeds 197,376, but its square can exceed u32. Use `radius.wrapping_mul(radius)`: ordinary debug multiplication can panic and saturating multiplication changes behavior. The comparison is inclusive and unsigned (`JA` branches to the next record); signed comparison changes results whenever the threshold's high bit is set. The helper at `0x73690` preserves ECX, so the squared threshold survives the call. Qualification failures short-circuit before coordinate/radius arithmetic.

**Implementable safe API:**

```text
record_radius_match(group_records: &[u8], query: [i16; 2]) -> Result<bool, BoundsError>
```

Require at least 4,288 bytes for one complete group **before** scanning; examine exactly the first 64 records and ignore any suffix. Read components using byte slices and `i16::from_le_bytes`, without pointer casts or alignment assumptions. Return `Ok(true)` at the first matching record and `Ok(false)` after all 64. Return a length error for a shorter group regardless of whether an early accessible record would match. That prevalidation is an explicit safety boundary, not a claim that original code checks bounds. An alternative typed input `&[[u8; 67]; 64]` makes this contract structural.

A selector wrapper can take `(groups: &[u8], selector: u8, query)` and safely select `[selector * 4288 .. (selector + 1) * 4288]` with checked arithmetic/slicing. It must reject an incomplete selected group. Original code does not validate the selector. Mapped object 3 has capacity for 66 full groups from this base, but mapped memory is not proof that all 66 groups are allocated or semantically valid. Do not introduce a gameplay group count from that observation. The last physical word read by the scan ends at group offset 4,272; requiring the full 4,288-byte record group intentionally avoids exposing partial-record acceptance.

**Representative synthetic results** (all qualification bytes set unless stated):

| Query | Record coordinate | Radius byte | Squared distance | Wrapped threshold | Result |
|---|---|---:|---:|---:|---|
| `(0,0)` | `(1536,0)` | 0 | 2,359,296 | 2,359,296 | true |
| `(0,0)` | `(1537,0)` | 0 | 2,362,369 | 2,359,296 | false |
| `(0,0)` | `(-32768,-32768)` | 0 | 0 | 2,359,296 | true |
| `(0,0)` | `(16384,16384)` | 83 | 536,870,912 | 4,261,478,400 | true |
| `(0,0)` | `(16384,16384)` | 84 | 536,870,912 | 67,371,008 | false |
| `(0,0)` | `(0,0)` | 254 | 0 | 0 | true |
| `(0,0)` | `(1,0)` | 254 | 1 | 0 | false |
| `(-1,0)` | `(32767,0)` | 254 | 0 | 0 | true |

Additional required slice tests: all-zero 64-record group returns false; only record 63 qualifies and returns true; nonzero marker 255 qualifies; byte `+0x32` alone cannot satisfy bit 0; qualification kind 2 fails; incomplete group returns a bounds error; a selector for a missing group returns a bounds error. These are synthetic tests of proved numeric/layout behavior, not game assets.

Unresolved: the pointed structure's meaning and length beyond its first byte; entity/record identity; intended radius-byte range; live group count; mutation/concurrency model; whether other editions use the same offsets or arithmetic.

## Grid bit helpers, VA 0x8D580 and 0x8D5F0

**Observed:** EAX packs X/Y signed words, DL is unsigned column count, BL is unsigned row count. Base is `0x11C2BE`; records are 14 bytes and bit `0x10` of byte `+11` is changed. Other bits and bytes are preserved. Both counts are bytes, so accepted raw range is 0..255. Zero rows or columns cause no writes. Set performs OR `0x10`; clear AND `0xEF`. No return-value meaning is established.

For row index `r` and column index `c`, the byte offset from grid base is exactly:

```text
x_index = (signed16(x) >> 8) & 127
y_index = (signed16(low16(y + r * 256)) >> 8) & 127
offset = (y_index * 128 + x_index + c) * 14 + 11
```

The signed projection equals `(u16_coordinate >> 8) & 127` for all 65,536 word patterns; the independent check found zero mismatches. Every new row recomputes wrapped Y. Columns advance linearly by 14 bytes and do **not** wrap X at 128. More than 128 rows revisits rows; OR/AND remain idempotent. Negative inputs and the high sign bit preserve the same modulo projection. The original `ESI` load includes unrelated upper stack bytes, but only SI influences row addressing, so those upper bytes do not affect writes.

**Implementable safe API:**

```text
set_grid_bit10(grid_bytes: &mut [u8], coordinate: [i16; 2], columns: u8, rows: u8)
    -> Result<(), BoundsError>
clear_grid_bit10(same arguments) -> Result<(), BoundsError>
```

If either count is zero, return success with no required bytes. Otherwise compute and validate all touched offsets against the supplied slice before any mutation; a too-short slice returns an error with no partial writes. Then perform the precise row/column addressing and byte operation. Requiring the entire 14-byte record instead of only the touched field is an optional stricter API boundary and must be documented. Passing a standard 229,376-byte 128-by-128 backing block is valid only if every touched offset fits. Do not silently wrap columns, clamp counts, or discard writes beyond that standard block: reject the span or accept a sufficiently large backing slice to preserve the observed linear addressing.

| Coordinate | Columns × rows | Touched byte offsets | Minimum byte-slice length |
|---|---|---|---:|
| `(0,0)` | 1 × 1 | 11 | 12 |
| `(32512,0)` | 2 × 1 | 1789, 1803 (crosses a linear row boundary) | 1804 |
| `(0,32512)` | 1 × 2 | 227595, 11 (Y wraps) | 227596 |
| `(-1,-1)` | 1 × 1 | 229373 | 229374 |
| `(-1,-1)` | 2 × 1 | 229373, 229387 (second outside standard block) | 229388 |
| `(-1,-1)` | 255 × 1 | 255 writes, first 229373, last 232929 | 232930 |

Byte-operation vectors: setting `0xA5` yields `0xB5`; clearing `0xB5` yields `0xA5`; setting an already-set bit and clearing an already-clear bit are idempotent. Seed unrelated bytes with distinct values and assert they remain unchanged; test bounds rejection leaves the entire slice unchanged.

Control-flow caveat: clear helper `0x8D5F0` jumps to the shared epilogue at `0x8D5E3`, preceding its own entry. Its 105-byte contiguous-region hash does not include every instruction executed by the clear function. The set helper's direct export includes that epilogue; both traces reconcile. This was sent to SOL-01 for the dossier. Default function extents must not be treated as an exact executed-byte boundary.

Unresolved: bit meaning, live backing allocation extent, intended caller span restrictions, and cross-edition behavior. The grid projection is observed numeric addressing; it does not establish visibility, passability or occupation.

## Actual checks

`python -B research/variants/primitive_contract_checks.py` exited 0. The three private mapped function-region hashes match SOL-01's `primitive-evidence.json`; no original bytes were exported. It generated 14 record vectors, eight grid-span vectors, and the exhaustive 65,536-word projection check. Results were saved to `primitive-contract-vectors.json`. Existing independent distance and square-root reviews remain in `independent-review.md`.

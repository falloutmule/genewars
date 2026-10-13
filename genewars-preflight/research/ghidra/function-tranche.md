# CD candidate: verified coordinate and integer arithmetic tranche

Source package is the user's `genewars.zip`, SHA-256 `8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906`. Analysis input is `GAME/GW.EXE;1`, LBA 501, 1,265,994 bytes, SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. This remains a **provisional CD analysis candidate**, not a canonical retail baseline. No original game or installer was executed. Names below are independent descriptive labels, not recovered symbols.

Confidence **Observed** means directly established by static instruction/register/width review. Whole-image page/fixup reconstruction supports address validity. It does not mean gameplay execution was observed. Inferred feature identities remain **Unknown**.

## Reusable arithmetic

| Independent name | Object 1 offset | Loaded VA | Bytes | Observed inputs/output |
|---|---|---|---|---|
| wrapped_squared_distance | `0x63690` | `0x73690` | 95 | EAX points to two little-endian 16-bit coordinates; EDX packs target X in low 16 bits and Y in high 16 bits. Returns unsigned squared distance in EAX. |
| wrapped_floor_distance | `0x637F0` | `0x737F0` | 102 | Same inputs; returns integer floor square root of the same squared metric in EAX. |
| floor_sqrt_u32 | `0x87910` | `0x97910` | 66 | EAX unsigned 32-bit input; EAX floor square root. Zero returns zero. |

For each coordinate, the squared-distance function doubles source and target, subtracts in the low 16 bits, arithmetic-shifts the signed 16-bit result right by one, sign-extends, squares in 32 bits, then adds both squares. An equivalent portable delta is `(target - source + 16384).rem_euclid(32768) - 16384`. Delta range is `[-16384,16383]`; the half-period tie chooses `-16384`. Maximum squared result is `536870912`, so there is no overflow in this function's result domain. The input storage uses 16-bit words; signed versus unsigned interpretation does not change the wrapped result. Input coordinate units are not yet identified.

The integer square-root routine reads a 32-entry estimate table at VA `0x97952`, indexed by the most significant set-bit position. The table equals `min(65535, floor(sqrt(2^(bit+1))))`. Unsigned division calculates input/estimate; while that quotient is smaller than the estimate, the estimate becomes the integer average of both. The final result is floor sqrt for the entire u32 domain. A Rust implementation can use an independently written exact integer-square-root algorithm; no copied binary table is required.

The default decompiler compiler convention is `borlandcpp` supplied by the loader; it is **not** an identified compiler. Its inferred signatures contain spurious parameters and mark `0x737F0` void. Direct disassembly proves the sqrt return remains in EAX across that function's epilogue. Register and instruction-width evidence takes precedence over inferred C types.

The squared helper has 13 direct caller functions. At VA `0x8CB00`, arithmetic converts a record address into X/Y, calls `0x73690` then `0x97910`. VA `0x31330` uses the squared helper to select grid records and invoke a callback. VA `0x432C0` uses it in the radius query below. Addresses, mapped-byte hashes, caller lists, and synthetic model-derived vectors are in `primitive-evidence.json`.

## Actual record predicates and grid operations

**VA `0x432C0`, object 1 offset `0x332C0`, 124 bytes — record_radius_predicate.** EAX points to a structure whose first byte selects a group; EDX is packed query X/Y. The group base is `0x1645AE + group * 0x10C0`. It scans exactly 64 records of stride `0x43`. A record qualifies when byte `+6` is nonzero, bit 0 of `+0x31` is set, byte `+0x2D` equals 3, and wrapped squared distance from the query to its packed coordinate at `+7` is at most the square of `1536 + 768 * u8(record+0x2E)`. The radius square is **wrapping u32**, and its comparison is unsigned/inclusive. Return truth is in low AL only; upper EAX is unspecified. Record/entity identity and the valid range of the radius byte are unknown. Do not assign a gameplay name or remove integer wrapping based on a guessed range.

**VA `0x8D580` / `0x8D5F0`, object 1 offsets `0x7D580` / `0x7D5F0` — set/clear bit `0x10` in grid byte `+0xB`.** EAX packs X/Y, DL supplies column count, BL supplies row count. Grid base is `0x11C2BE`; record stride is 14 bytes; row stride is `0x700` = 128 records. Starting grid indices are `(signed16(coordinate) >> 8) & 127`, which establishes a 128-by-128 coordinate projection. Each new row recalculates the wrapped Y index, and columns advance linearly by 14 bytes. The column loop does **not** wrap at 128: callers must enforce a valid span or preserve this exact behavior. Zero row/column counts produce no writes. The set operation ORs `0x10`; clear ANDs `0xEF`. Meaning of that bit is unknown. No safe replacement should silently treat it as visibility, occupation, passability, or ownership.

## Verification and use boundaries

`validate_le.py` independently parses the LE object/page/fixup structures and reconstructs all four loaded objects. It checks every byte against Ghidra dumps, all 244 pages, all 26,883 type-7 internal fixup records, and all 24,497 cross-object fixup instances. There are 26,860 unique fixup addresses because 23 records repeat a relocation spanning a page boundary. Targets exactly one-past an object are allowed and counted separately. Entry is object 1 + `0x96E40` = `0xA6E40`. Detailed safe evidence is in `loader-validation.json`.

`validate_primitives.py` compares an abstract model of the observed 16-bit instruction semantics against the portable wrapped formula over all 131,071 possible differences of two 16-bit inputs. It compares the observed sqrt model to Python's mathematical integer sqrt for 200,139 inputs, including square/bit boundaries and deterministic broad samples. These are **static semantics checks and synthetic vectors**, not original-program execution traces. Raw decompiled C, instruction exports, mapped bytes, and Ghidra databases remain in Git-ignored `private/ghidra/`.

The clear-grid routine's exported 105-byte contiguous region branches to the set routine's shared epilogue at `0x8D5E3`; that executed tail lies outside the clear region's digest. The radius predicate physically reads word +0x31 before clearing DH, so byte +0x32 is read but does not affect the flag test. Mapped group storage capacity is not proof of a live allocation count.

The renderer and sprite follow-up is documented in `renderer-bridge.md`. Neighboring coordinate/angle functions around `0x733A0–0x74D90` and the grid callback caller `0x31330` remain useful simulation work; identify caller structure fields before naming entity behaviors. Runtime comparison still requires separately authorized original execution.

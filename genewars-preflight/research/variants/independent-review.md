# SOL-02 independent review — 2026-10-12 local

Scope: read-only review of asset/core code, private static Ghidra exports, existing preflight metadata, and pinned primary source metadata. This lane owns only `research/variants/`. No original game was executed; no commercial remote executable, symbol map, or third-party implementation was copied into the project.

## RNC1 result and remaining checks

**Observed defect in the initial reader:** a Huffman probe for a long code could refill a physical word before a later short code matched. With depths `[2,1,2]`, one buffered zero bit, and symbol 1's one-bit code, symbol-index probing advances the physical cursor by two bytes unnecessarily. The synthetic reproduction is in `review_checks.py` and `review-results.json`. The cursor then points beyond the correct literal start. This was sent to LUNA-01 and root while the decoder was being repaired.

**Observed repaired implementation:** the revised reader distinguishes committed 16-bit control words from non-consuming lookahead and rebuilds lookahead after literal bytes. Probing by symbol index is safe with that reader; sorting entries is an alternative, not a required patch. Tree oversubscription is checked before canonical assignment, literals use checked slices, output allocation is fallible and caller-limited, and backreferences validate distance and length while preserving overlap. Packed/unpacked CRC and exact size checks remain present. My `python -B scripts/cargo.py test -p genewars-assets -p genewars-core` run passed 6 asset tests and the initial 3 core tests. Root separately recorded the larger real-data corpus: 329 of 331 resource records passed; two `INSTALL.DAT` files have data beyond their first declared RNC stream. Those are framing research, not grounds to relax strict complete-stream validation.

**Actionable remaining checks sent to LUNA-01:** complete the declared chunk grammar instead of breaking solely because expected output length has been reached; add synthetic word-boundary/literal-rebase, overlap, malformed-tree and early-output-completion regressions. Zero-count tables are safe when unused. Whether a zero count can reuse a prior table remains a compatibility question; the current implementation creates an empty table, while a primary reference leaves the prior table untouched. Do not assert universal semantics from corpus success alone. Method 2 remains unsupported.

Reference reviewed: [ScummVM RNC decoder](https://github.com/scummvm/scummvm/blob/master/common/compression/rnc_deco.cpp), file header GPL-3.0-or-later. Its physical-cursor, lookahead, literal-rebase and table-order behavior supplied an algorithm comparison only. No code copied. LUNA-01's separate format sources and provenance are recorded in `research/formats/rnc.md`; this review did not independently audit all of those third-party packages.

## Licenses and exact commits

Reproducible primary checks are in `verify_source_provenance.ps1` and `source-provenance.json`.

| Repository | Pinned commit | Actual license evidence | Reuse decision |
|---|---|---|---|
| `mefistotelis/genewars-re-helpers` | `a85d5c517b524aa4906a720330ed1435bf8b65cf` | GitHub API license null; complete 21-entry tree has no LICENSE/COPYING/COPYRIGHT/NOTICE filename; README does not provide a grant | Metadata only; no helper code/maps/XML copied or applied |
| `mefistotelis/genewars` | `72ec4c4908d26304dd373132cb3187c855d4bf58` | API license null and no license-named file in complete 396-entry tree, **but** checked `bio/src/cvector.cpp` and `bio/include/bio_ver.h` headers explicitly grant GPL-2.0-or-later | These two files have a grant; do not infer a uniform license for every file or choose the Rust product's distribution license implicitly. Current Rust remains independently implemented |

Primary links: [helper pinned tree](https://github.com/mefistotelis/genewars-re-helpers/tree/a85d5c517b524aa4906a720330ed1435bf8b65cf), [cvector license header](https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/src/cvector.cpp), [bio_ver license header](https://github.com/mefistotelis/genewars/blob/72ec4c4908d26304dd373132cb3187c855d4bf58/bio/include/bio_ver.h). A missing GitHub license classification alone does not prove absent licensing.

## Variant and symbol compatibility

**Observed:** local CD candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`, size 1,265,994, has computed Git blob SHA-1 `cb7ddad872b1c169c8b9657bb83b0e6aa6703c75`. It matches the pinned helper tree's metadata for `bin/gw-genewars-1_0-sep_1996.exe`. This was calculated from the user's local bytes using Git's `blob <length>\0` framing; no remote game bytes were fetched. It corroborates the repository's filename label, not independent retail authenticity. SHA-1 metadata matching is not a substitute for a published independent SHA-256/provenance record.

**Observed:** helper symbol metadata lists November 1.1 and July beta maps/XML; there is no September 1.0 map/XML in the pinned tree. Therefore even the strong binary metadata match does not justify importing those symbols into the CD program. The patch payload's file size matches the helper's November binary size, and the larger RIP matches its beta size; size alone does not establish identity. The small repack and CD share layout but differ in 12 measured bytes. Preserve separate hashes and behavior scope. `review-results.json` extracts all four executable layouts from the existing preflight without another extraction campaign.

Criteria for a **provisional analysis build**: exact local source chain/hash, validated LE object/page/entry/fixup mapping, real code decompilation with compiler/calling-convention uncertainty explicit, and enough locally corroborated resources to connect analysis to runtime data. The CD candidate meets the static-analysis criteria after SOL-01's validation. Criteria for a **behavioral reference build** additionally require corroborated edition/patch provenance, coherent assets and campaign content for that edition, documented handling of the QA disc label and repack patches, and approved reproducible reference runs before execution-verified behavior claims. That stronger designation remains unknown.

## LE loader and Rust primitives

Reviewed `scripts/ghidra/validate_le.py` and reran its actual assertions through `review_loader.py`, which removes only its final output write and print. All 244 pages, 26,883 internal 32-bit fixup instances, 26,860 unique fixup addresses and all bytes of all four mapped objects agree with the Ghidra exports. 24,497 fixup instances cross objects. Entry is object 1 + `0x96E40` = `0xA6E40`. This is a rerun of SOL-01's independently written validator, not a second independently written LE parser. The inspected parser supports and asserts the types found in this exact candidate; do not advertise it as a general-purpose arbitrary LE validator. Loader default compiler and object permissions remain separate concerns from byte/address correctness.

**Observed core correspondence:** direct instructions at VA `0x73690` / object 1 + `0x63690` perform low-word doubling/subtraction, signed word shift, sign extension, two squares and their sum. EAX points to two little-endian signed word components; EDX packs the other pair. Uninitialized high stack bits in the second-component load do not affect the low word used by the result. `distance_squared_wrapping_15` matches that arithmetic for every one of 65,536 possible difference residues. Each delta is in `[-16384,16383]`; the maximum sum 536,870,912 fits signed 32-bit arithmetic. This supports a numeric helper, not gameplay coordinate units or world topology.

**Observed square-root correspondence:** direct instructions at `0x97910` use BSR-indexed unsigned-word seeds, unsigned division, and signed quotient/seed comparison followed by integer averaging until the quotient reaches the estimate. I read the seed table privately and verified all 32 bins start at or above the maximum floor root in that bin. No table values were exported. A separate static instruction model agrees with Python `math.isqrt` for 206,606 values (every positive u16 square and its neighbors, extrema, plus deterministic random u32s), with zero mismatches. Signed comparison never changes the tested cases; the seed bounds also keep quotients far below signed overflow. The root's independently chosen bit-width Newton seed yields the same floor-root result with different timing. Its estimate/quotient sum remains bounded and nonzero division is guaranteed after the zero fast path.

**Observed composition:** direct instructions at `0x737F0` compute the same wrapped squared result and call `0x97910`; the caller preserves the returned EAX through its epilogue. The decompiler's default void return is not authoritative. `distance_wrapping_15` matches the returned arithmetic. Latest independent `python -B scripts/cargo.py test -p genewars-core` passed all 5 core tests. No original execution or cross-edition equivalence is claimed.

## Reproduce

From project root:

```text
python -B research/variants/review_checks.py
python -B research/variants/review_loader.py
python -B scripts/cargo.py test -p genewars-assets -p genewars-core
```

`verify_source_provenance.ps1` uses read-only network requests and writes only its own safe metadata summary. Initial sandbox DNS requests failed; an approved read-only network retry succeeded. No global configuration was changed. Python emits a pre-existing executable-location warning but these checks exited 0; Cargo emits filesystem hard-link/canonicalization warnings but the tests pass.

# RNC ProPack evidence and decoder

Status: method 1 is implemented and validated against private GeneWars samples. Method 2 is deliberately unsupported.

## Observed GeneWars files

The preflight manifest (`research/game-files-manifest.json`) records **331** files with `RNC` at byte zero; all have method byte `1`. The records span nested archive and disc trees. This establishes the method byte, not that every method-1 stream uses the same historical sub-variant.

The RNC header used by the validated samples is 18 bytes: `RNC`, method, big-endian unpacked size, big-endian packed size, big-endian unpacked CRC16, big-endian packed CRC16, and two encoder metadata bytes. Packed size excludes the header. CRC16 is reflected polynomial `0xA001`, initial value zero, with no final XOR. The three sample packed CRCs match this calculation.

Method 1 contains two interleaved streams. Control bits are consumed least-significant-bit first in 16-bit little-endian words; byte literals are read from the shared packed stream between Huffman control fields. Each top-level block carries three canonical Huffman tables (literal run, match distance, match length), a 16-bit subchunk count, then literal runs and overlapping LZ backreferences. Header byte 17 is a block-count hint; its nonzero value matches the number of parsed top-level blocks in all checked samples. The decoder enforces that hint when present and always checks the exact declared output length. The bit reader keeps a bounded non-consuming lookahead because Huffman probes may cross a 16-bit word boundary while the literal byte cursor must remain positioned after only the word actually loaded.

The decoder rejects unknown methods, nonzero locked/encrypted flags, truncated streams, oversubscribed trees, invalid codes/backreferences, impossible early output completion, declared output larger than the caller's limit, mismatched packed size, and either CRC mismatch. A zero-count Huffman table preserves its prior table state; this behavior is recorded in ScummVM's method-1 decoder at pinned commit `fb7095f5a4eee3c7b984b07f05ed25a4ec70e30f` (`common/compression/rnc_deco.cpp`, GPL-3.0-or-later; read for behavior only, no code reused). If no prior table exists, a use of the empty table fails with an invalid-code error. It permits only the two zero-padding bytes needed by the method-1 bit-word reader at end of stream; literal reads remain strictly in bounds. No decompressed original data is stored in the crate or source repository.

`decode` requires the whole input length to equal the RNC header plus declared packed length. `decode_prefix` explicitly decodes the first header-bounded stream and returns `(decoded_bytes, consumed_input_bytes)`, preserving any trailing bytes for the caller. It does not guess that a tail contains another RNC stream; each subsequent stream must be passed explicitly by the caller.

## Targeted private sample validation

The private inputs were copied by exact member name from the supplied archive's nested small-RIP ZIP; no archive-wide extraction was performed. Their hashes agree with the corresponding manifest entries. Output hashes below are evidence for the local validation run, not distributable assets.

| Inner member | Compressed bytes / SHA-256 | Declared decoded bytes | Decoded SHA-256 |
|---|---|---:|---|
| `DATA/PLANT/ALL1.ANB` | 4,377 / `df9180a4dace76f722637f6b35f00114ea6a183603fd57c32502957c37378faa` | 6,342 | `d3db5aa80fe306840bb3344e9e5f614bd57867a67078cb9e53a5090b4cad45ed` |
| `DATA/QUIT.PAL` | 718 / `4bcb55e695c4574733b4e56a3a68ce169563408148aed83fd1cdbdddb1aada88` | 768 | `79b73e262c97fc47a847da0ffbae9f86ce274e7a9fc88f66789cac37204e2f15` |
| `DATA/GENEPANL.DAT` | 16,204 / `8116b2b883320b8573c50496a570f0f3fa978e4c20b9c85a284339ff1f920e24` | 61,440 | `af9741b4545fb6f28a1dd0e50da0570e39992e75c872bc2249891f3330f59951` |

Each decode verified the header size, packed CRC, exact output length, and unpacked CRC. Synthetic tests cover a literal stream, word-boundary lookahead without committing speculative bytes, oversubscribed trees, overlapping backreferences, bounded output, and rejection of output that completes before its declared chunks finish. The three input and three output files remain under ignored `private/assets/`.

The manifest also contains two `LDATA/INSTALL.DAT` files whose first RNC stream is followed by more bytes. On the larger RIP input, the first stream consumes 114,753 of 165,357 bytes; the 50,604-byte tail is a second RNC stream that decodes to 66,552 bytes. Its decoded SHA-256 is `154574cc30f55e1a386cc005c7eaf74cd0933edcabd1e4b0368b6ce4a1c9946c`. That output byte-for-byte matches the 66,552-byte raw tail of the disc version after its first stream at offset 334,383. These results validate explicit prefix decoding and the relationship between the two installed variants; they do not establish broader container semantics.

## Research and provenance

- [PlayStation Specifications: Compression](https://psx-spx.consoledev.net/ps1/cdr/cdromfileformats/compression/) gives a format-oriented account of the 18-byte RNC header and method-1 bit/literal interleaving. It is used as a description, not copied code. The relevant method-1 behavior was cross-checked against decoded GeneWars files.
- [temisu/ancient](https://github.com/temisu/ancient), source version packaged as Debian `ancient 2.2.0-1`, `src/RNCDecompressor.cpp`; the repository declares BSD-2-Clause. Its independent decoder documents the RNC1 new/old ambiguity and describes the method-1 trees and subchunks. It was read as a reference; no source code was copied into this crate. The project uses a 12-byte header for its legacy “old” variant, which is distinct from the 18-byte CRC-bearing files validated here.
- [ScummVM](https://github.com/scummvm/scummvm/commit/fb7095f5a4eee3c7b984b07f05ed25a4ec70e30f), `common/compression/rnc_deco.cpp`; GPL-3.0-or-later. Its `makeHufftable` returns without changing the existing table when the encoded symbol count is zero. This was used to resolve table-state behavior only; no code was reused.
- [nviewer RNC method notes](https://dsll32.github.io/nviewer/wiki/compression/rnc.html) describe method 1 and method 2 as separate decoders and explicitly cover overlapping matches. The page's code license was not established, so its code was not reused.
- [lab313ru/rnc_propack_source](https://github.com/lab313ru/rnc_propack_source) is a decompiled ProPack reference implementation. No license file was established during this task; its code was not reused.

The Rust implementation is an independent std-only implementation of the documented format and was checked against the sample CRCs and lengths. Method 2 is not inferred from method 1 and remains an explicit `UnsupportedMethod(2)` error. Container parsing beyond the RNC layer, and the internal contents of `.ANB`, `.PAL`, and `.DAT`, remain separate work.

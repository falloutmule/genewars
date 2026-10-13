# Palette file evidence

The RNC corpus contains repeated `.PAL` files from the RIP and disc variants. Grouping successful manifest records by compressed input SHA-256 gives 26 unique palette resources. Every one decodes to exactly 768 bytes, and the byte values across all 26 decoded files range from 0 through 63; all 64 possible values occur.

Native disassembly provides the channel interpretation: VGA upload routine VA `0xA6D2C` writes each index to port `0x3C8`, then sends three consecutive source bytes unchanged to port `0x3C9`, repeating for all 256 indices. Its paired palette reader at VA `0xA6CE0` reads the same 768-byte triplet sequence. This establishes RGB byte order at the upload boundary. The values' 0..63 range agrees with the six-bit VGA DAC component format described by [FreeVGA](https://www.scs.stanford.edu/10wi-cs140/pintos/specs/freevga/vga/colorreg.htm). No 6-to-8-bit display scaling is performed by the observed game routine, so the library only preserves raw bytes.

Representative entries from the inventory:

| Member | Compressed SHA-256 | Decoded SHA-256 | Decoded length |
|---|---|---|---:|
| `DATA/QUIT.PAL` | `4bcb55e695c4574733b4e56a3a68ce169563408148aed83fd1cdbdddb1aada88` | `79b73e262c97fc47a847da0ffbae9f86ce274e7a9fc88f66789cac37204e2f15` | 768 |
| `DATA/CUSTOM.PAL` | `35e22d67449148678f428756aa42b60d4a8ed3c6cb4de48687554f45edc462de` | `738198fa93ed90d9e5f1326afa81b8644913f83cf2024ecb7e2eb18ab5f491a9` | 768 |
| `DATA/TITLESCR.PAL` | `d5ec91e731c2ff7e8d74dbbde6a3fe0410ec602ff0da52eb1e6f6bb1a44307c7` | `36ef1916917c118a8115c8550a0d2d684d9041743a9d7521414479504606e491` | 768 |

`genewars_assets::Palette256::parse(bytes, PaletteChannelMode)` is a bounded structural reader for exactly 256 triplets. Supply `Rgb` for GeneWars `.PAL` files; `Bgr` remains available for generic callers. The parser preserves source triplets exactly and performs no channel swap, scaling, gamma handling, or display conversion. Other lengths are rejected.

The inventory and hashes above are derived from `research/integration/rnc-corpus.json`, whose source records are checked against the preflight manifest and whose decoded outputs remain under ignored `private/integration/rnc-decoded/`. The disassembly evidence is stored under ignored `private/ghidra/vga/` (`instructions-000a6d2c.txt`, `decompile-000a6d2c.c`, `instructions-000a6ce0.txt`, `decompile-000a6ce0.c`). No original palette bytes are checked in.

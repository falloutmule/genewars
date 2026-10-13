# Font and text byte readers

## Proven contracts

The font table uses an opaque six-byte prefix followed by six-byte records. Each record is a little-endian 32-bit offset into the paired `.DAT` byte slice, then one-byte width and height. The font loader uses the DAT base directly; the ANB `+0x12` sprite-data bias does not apply. The text context begins at TAB offset `+6`; byte 32 (space) selects the first record there, and each later byte selects the next record. Thus code 65 selects record 33 at whole-TAB byte offset 204.

Glyph data uses the indexed sprite row-control grammar confirmed at the shared decoder: positive signed controls copy that many literal bytes, negative controls skip transparent pixels, and zero ends the row. A literal palette index zero is opaque. The traced sprite routine clips writes/skips at the remaining row width while consuming complete literal runs; the safe decoder follows that behavior and still reads a zero row terminator for every declared row, including the last. That final-row requirement is a stricter completeness check than the runtime path, which can return immediately after the visible last row. In the 194 FONT2 records, all 3,613 rows sum to no more than their declared widths (3,598 rows leave a transparent remainder; 15 fill the width exactly). The line branch advances by `glyph.width - 1`; space advances but is not painted. The safe renderer accepts bytes at least 32, rejects lower control bytes, and has no wrapping or line-break behavior. These are the boundaries implemented by `FontResource`, `TextStringTable`, `measure_text_line`, and `blit_text_line`.

The startup text index loop starts at `0x25c`, decrements before storing, and stops when the counter reaches `-1`, so it indexes 604 NUL-terminated byte strings at indices 0–603. The parser consumes those 604 entries and returns the remaining bytes separately. On the English CD resource the final byte is one additional NUL, which is retained as trailing data. The reader does not assign a character encoding.

## Input evidence and measurements

The staged English file is `DATA/TEXT/ENG/TEXT.DAT` from the CD image, manifest entry `root/2/disc/592`, 36,610 bytes, SHA-256 `23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786`. The exact run-002 nested file matches this manifest object. Parsing the first 604 terminators consumes 36,609 bytes, leaving one byte (`00`) in the returned tail.

All staged CD FONT0–FONT5 and FONT7 DAT/TAB inputs in run-002 were checked against their exact manifest entries (`root/2/disc/71–84`) by size and source SHA-256. For the compressed inputs, the output lengths and hashes below come from the corpus decoder receipt. FONT4 is raw in the CD image and its sizes/hashes are listed as source values.

| Pair | DAT source SHA-256 | DAT output bytes / SHA-256 | TAB source SHA-256 | TAB output bytes / SHA-256 |
|---|---|---|---|---|
| FONT0 | `ee30f263c450c2611d8bfc0b7d3511d532477d320b7fa245076db28b766209af` | 8,039 / `af89c4e4c02e394810b22ecf3d57211cdb56a0006071a88f43d0ae6fb370447d` | `f08fe66f5be70bde22e2a59d160d1a097cc36d0dbcb011f254a8220275cef161` | 1,170 / `321c49b302552e2bcf287bd20ef0954fb977dd66aef6e82101e3c0c64b5b0a17` |
| FONT1 | `50d2e0c756f15266af732d189d4513a8a0b97885adcb1f865dbfbed43ff9af95` | 8,999 / `6d84ed8ef3cfe7d72f5b01de9605743de123d489b782866844b0cf5e7e88792d` | `6ceee14277701550731d4bda0db32032ca8fa3bc1c1b185648036b2dd5b035a4` | 1,170 / `a84743fd3df9e66b4866059148d0b2ada1df81117cbd7bdf06f10ab16b4ebf0b` |
| FONT2 | `2f3d275f88f2ff6671562a819cbc7a791b16ad4510a4020f3b6078c917f60dba` | 44,647 / `2e1abcf8fb13cc306449f25bb2b0da64aa9627646ad4335910bdd0114a36b4fa` | `f54057e1b19cc1711a1596b6459cfdfb359a170f5ed739df71fd852befba4bab` | 1,170 / `e7a5299f511e76de4999cfde40b8852ed1d8da83406d5f0284886996e8fd87b1` |
| FONT3 | `098da83c84bbbcc11f1ffd966bd203f069866f4d273851bc2db0747d7afcc25a` | 2,722 / `43ffbdf85215dcd15451c2a448552d1fb8e152226b4cc3f86aba72cd52cddd6f` | `73349c1ecacd83a86f4701bcee28efc787d5deb3666ea9b3eed018574000c71e` | 162 / `c806188a8d3f53b4637fa5742ebbc7d2c92f6cf3bf5f8eb7cc24e039c9bf70d9` |
| FONT4 (raw) | `fc7c09bfa5bfba95ca2c22fe49faf9b32e4f3c2f31e18a1fd76a8ae81b75251f` | 6,452 / same as source | `b421792e9eeb38e8c1ff8cb97310e1d02e4f1bd2395bd77b54bb796c05270fde` | 1,170 / same as source |
| FONT5 | `a301b2028bb6637325048a27c00a9688a32f7c240744202cdddee1b3fc2c135b` | 10,349 / `fad7ee193853a3b850e3008dd7d651ba487831f19f537a5289f2ff6a23652a2f` | `8107e02da5491f7c825d09774b4b3eeea4613570e0870dca24c839f37f2dc6c1` | 1,170 / `22aa04e3e2dff8fd2b66e2d3b569bcf6cce6b3b3b5bf5a693111024df6f5abba` |
| FONT7 | `cd7ccbfb005ee78ef29e29d620a34cb9e275522c4c96b963f84c3abda5757326` | 14,994 / `60dbffae53efe67f5ce634c71c0e97b91a2d4514d8eaf7796d947fc106ad6e0e` | `62a8476b908544ee6c1d4c6a9c28b1c294c41cdd520f45cc589ed69f90221ff8` | 1,170 / `73d5433364795a4d2fabdab1489e130120483f30e3d07e13c5a75aab55b3102b` |

FONT2 is manifest entries `root/2/disc/75` (`FONT2.DAT;1`) and `root/2/disc/76` (`FONT2.TAB;1`). Their source hashes are `2f3d275f88f2ff6671562a819cbc7a791b16ad4510a4020f3b6078c917f60dba` and `f54057e1b19cc1711a1596b6459cfdfb359a170f5ed739df71fd852befba4bab`. Bounded RNC decode yields 44,647 DAT bytes (SHA-256 `2e1abcf8fb13cc306449f25bb2b0da64aa9627646ad4335910bdd0114a36b4fa`) and 1,170 TAB bytes (SHA-256 `e7a5299f511e76de4999cfde40b8852ed1d8da83406d5f0284886996e8fd87b1`). The table contains 194 records after its six-byte prefix; all 194 have nonzero dimensions. The 194 bounded glyph streams contain 34,881 literal pixels with 28 distinct source indices:

```text
13:36  17:613  24:1228  28:8131  36:11671  44:5991  54:426
57:613  61:378  63:364  82:97  85:530  89:13  96:370  100:147
108:323  111:271  127:15  144:209  147:595  149:133  150:37
152:56  153:2  157:199  175:323  185:1752  188:358
```

The traced menu font context at `0x18500A` selects `TITLESCR.PAL`. Its CD manifest entry is `root/2/disc/136`, source length 667, source SHA-256 `d5ec91e731c2ff7e8d74dbbde6a3fe0410ec602ff0da52eb1e6f6bb1a44307c7`; bounded RNC decode yields 768 bytes, SHA-256 `36ef1916917c118a8115c8550a0d2d684d9041743a9d7521414479504606e491`. Its VGA upload triplets are RGB six-bit values. Entries used by FONT2's literal pixels are listed below as raw triplets; the matching palette and static menu context are established, while this inventory alone does not establish the final presentation of every label.

| Index | RGB6 | Index | RGB6 | Index | RGB6 | Index | RGB6 |
|---:|:---:|---:|:---:|---:|:---:|---:|:---:|
| 13 | 8,8,9 | 17 | 8,7,10 | 24 | 8,7,12 | 28 | 12,11,14 |
| 36 | 14,13,16 | 44 | 12,11,18 | 54 | 20,20,21 | 57 | 19,18,22 |
| 61 | 15,14,22 | 63 | 16,13,19 | 82 | 24,24,25 | 85 | 22,21,27 |
| 89 | 18,17,26 | 96 | 25,24,28 | 100 | 25,24,26 | 108 | 28,27,34 |
| 111 | 26,25,32 | 127 | 23,21,35 | 144 | 28,27,30 | 147 | 30,29,36 |
| 149 | 32,31,38 | 150 | 33,30,40 | 152 | 37,34,42 | 153 | 38,37,45 |
| 157 | 46,46,47 | 175 | 26,25,36 | 185 | 40,39,39 | 188 | 34,32,35 |

## Implementation limits

`FontResource::parse(dat, tab)` validates the table extent. `record_for_byte` implements the traced codepoint mapping; `decode_glyph` returns indexed pixels, explicit opacity, and consumed stream bytes. `TextStringTable::parse(bytes)` requires the proven 604 terminators and exposes both consumed length and tail. `measure_text_line` and `blit_text_line` support only one unkerned line. Palette choice remains caller supplied, and display conversion follows the renderer's explicit six-to-eight-bit presentation policy rather than claiming VGA analog fidelity.

The disassembly evidence is private under `private/ghidra/font-text/` and `private/ghidra/display-geometry/instructions-000a7730.txt`; decoded inputs remain in the ignored `private/integration/rnc-decoded/` cache. No raw game strings or glyph images are included in this repository.

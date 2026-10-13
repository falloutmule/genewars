# ANB/TAB bounded structural measurements

These measurements use the successful decoded-file rows in `research/integration/rnc-corpus.json`. Rows are grouped by compressed input SHA-256 so repeated RIP/disc copies are counted once. They do not establish a general ANB pixel layout or prove filename-based TAB/ANB association.

There are 40 unique `.ANB` outputs, from 1,767 to 207,733 bytes. 23/40 have even lengths and 13/40 are multiples of four. There are 16 unique `.TAB` outputs, from 66 to 1,176 bytes; all 16 have even lengths and 5/16 are multiples of four. Thus an even-byte record view is possible for each observed TAB file, but the lengths alone do not identify entry types or widths.

Same-stem resources with both extensions have these decoded sizes:

| Stem | ANB size(s) | TAB size | Measurement if TAB is viewed as little-endian `u16` values |
|---|---:|---:|---|
| `IFEDITOR` | 31,413 | 264 | 132 values; range 0..29,056; all 132 values are below the ANB byte length |
| `LEVEL` | 75,922 | 216 | 108 values; range 0..44,681; all 108 values are below the ANB byte length |
| `IFCUSTOM` | 36,660 and 11,338 | 78 | 39 values; range 0..11,555; 39/39 are below the larger ANB size and 37/39 below the smaller one |

The `IFCUSTOM` variants come from distinct archive entries: the 36,660-byte decoded ANB is manifest entry `root/1/133` and is byte-identical to disc entry `root/2/disc/89`; the 11,338-byte ANB is `root/4/108` from `Genewars_DOS_EN_RIP-Version.zip`, paired by filename with its 78-byte TAB entry `root/4/110`. This is a smaller decoded resource from a distinct archive package; “small RIP” is not a reliable package label here. No same-stem TAB companion was found for the 36,660-byte `IFCUSTOM.ANB` variant in this inventory.

Sol01's disassembly of loader VA `0x98A60` establishes an 18-byte ANB header: a `u16` at byte 0 and four `u32` section offsets at bytes 2, 6, 10, and 14. The sprite-data base is ANB file offset `0x12`. Loader VA `0xA7618` walks six-byte table records: a little-endian `u32` offset from that sprite-data base, followed by one-byte width and height. Draw routine VA `0xA7730` consumes signed RLE controls: positive values copy that many literal indices, negative values skip that many pixels, and zero terminates a row. Width clipping consumes complete literal runs but writes only visible pixels; an early row terminator leaves the remainder transparent. Those details motivated the bounded `SpriteTable`/`IndexedSprite` parser in `genewars-assets`.

The library exposes `AnbSections::parse(&[u8])` as a borrowed checked view. It validates the 18-byte header, absolute section offsets in nondecreasing order and within the file, exposes the first `u16` as an opaque header word, and returns the exact byte slice `[offset0, offset1)` through `sprite_table_bytes()`. `sprite_database_prefix()` bounds the embedded sprite database to `[0, offset0)`. `SpriteTable::from_anb` parses that embedded first section and retains that database bound for later decode calls, so an RLE read cannot silently continue into section data. Generic `SpriteTable::parse(tab, database)` remains available for external tables and uses exactly the caller-supplied database slice. The generic section view leaves the first word uninterpreted; the narrower `ui_metadata()` accessor applies the loader-proven animation-count interpretation described below.

Comparing the first ANB section range `[offset0, offset1)` to external TAB bytes provides direct pairing evidence in two cases: IFEDITOR's 264-byte range at file offset 29,875 exactly equals its 264-byte TAB, and `IFCUSTOM` entry `root/4/108` has a 78-byte range at offset 10,912 exactly equal to TAB entry `root/4/110`. The LEVEL candidate differs (its first section is 240 bytes versus a 216-byte TAB and is not byte-identical); the 36,660-byte IFCUSTOM candidate also differs (222-byte first section versus the 78-byte TAB). Their same-stem RLE records fit bounds, but that does not prove those external tables are used with those ANBs.

The parser was run against candidate same-stem decoded pairs and verified all RLE bounds:

| Stem | Records | Non-empty | Decoded pixel slots | RLE bytes consumed |
|---|---:|---:|---:|---:|
| `IFEDITOR` | 44 | 43 | 28,922 | 29,812 |
| `LEVEL` | 36 | 35 | 79,210 | 52,910 |
| `IFCUSTOM` | 13 | 12 | 13,746 | 10,880 |

`IFCUSTOM` was checked against both the 36,660-byte repack/disc ANB and the 11,338-byte `root/4/108` ANB with TAB `root/4/110`; the runs fit both. This validates structural compatibility, not the actual runtime association in the 36,660-byte build. Sol01 is still tracing which caller selects each table/database pair. The old LE-u16 table statistics above remain only an initial numeric bound check, not a claim that TAB fields are 16-bit values.

For Sol01's next-stage inspection, representative decoded private paths are:

- `private/integration/rnc-decoded/fb0b03b7…bin` (IFEDITOR.ANB), `private/integration/rnc-decoded/99ea7243…bin` (IFEDITOR.TAB)
- `private/integration/rnc-decoded/36f2a862…bin` (LEVEL.ANB), `private/integration/rnc-decoded/52d72cb0…bin` (LEVEL.TAB)
- `private/integration/rnc-decoded/94610c45…bin` (`IFCUSTOM.ANB`, manifest `root/4/108`), `private/integration/rnc-decoded/460e2400…bin` (`IFCUSTOM.TAB`, manifest `root/4/110`)

The leading path segments above are shortened SHA-256 names; the manifest and corpus receipt contain the complete values and ancestry. Disassembly artifacts are in ignored `private/ghidra/sprite-functions/`. Private originals and decoded outputs are not checked into source.

## Broad-corpus exceptions

The exact-hash BONES.ANB input `2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f` occurs in both the RIP and disc, so the corpus has two records but one unique byte sequence. Its header is `(count=5, offsets=[3721,3787,3887,4027])`; the six-byte table has 11 records. Record 5 is `(offset=0x4e1,width=65,height=36)`, which points to file byte `0x4f3` under the observed `0x12 + offset` base. Bounded RLE parsing stops in row 31: control byte `0x54` at file byte 4057 begins an 84-byte literal run, but only 19 file bytes remain. That point is past the header's final section offset 4027, and the bytes there have a repeated small-record pattern rather than a complete literal run. The two source copies hash identically. This is recorded as a structural exception; the parser was not relaxed to accept it.

The separate `GPF.ANB` input `d5499fbc6989eaa87523a698bc9421af07884c3b4a6a27e70dd4d7e93fea9959` is 1,634 bytes and starts `DE AD BE EF 01`, followed by `RNC` at byte 5. The 18 bytes beginning at that embedded signature resemble an RNC method-1 header declaring 3,960 unpacked and 1,596 packed bytes, but strict prefix decode from byte 5 fails packed CRC (`expected 57328`, computed `4199`). Its declared packed stream plus header leaves 15 bytes in the 1,629-byte suffix. The file is therefore not accepted as a normal 18-byte ANB or a valid RNC stream at that offset; its wrapper semantics remain unknown.

These bounded failures are included in `research/integration/sprite-corpus.json` as blocked examples, not counted as evidence that the parser accepts or decodes their contents. The embedded-table API now keeps RLE reads inside `[0, offset0)`, and synthetic tests confirm a stream cannot cross that boundary even when valid-looking bytes follow in the next section. Private exact-hash copies and the GPF offset-5 diagnostic suffix are under `private/integration/sprites/`.

## Verified CD IFCUSTOM UI metadata

Sol01's analysis of the CD candidate ANB loader and the UI initializer establishes these record boundaries for the exact 36,660-byte IFCUSTOM resource: component section `[offset1,offset2)` uses 10-byte records; frame section `[offset2,offset3)` uses 14-byte records; the final section contains `header_word` 10-byte animation records. The loader rebases each animation's first two `u32` values to the frame section, each frame's first `u32` to the component section, and each component's first `u32` to the six-byte sprite table. The frame and component records contain signed `i16` x/y or dimensions that the consumer arithmetic-shifts right by one. The frame component count is byte +11. Unknown record fields are retained raw by the library.

`AnbSections::ui_metadata()` validates record-stride divisibility and animation-section length before exposing `AnbUiMetadata`. `animation(index)` checks both frame byte offsets for alignment and bounds. `frame_at(offset)` checks component start/count bounds. `frame_component(frame, local_index)` resolves within that frame's validated component span. `component(index)` checks the referenced sprite-table offset. These accessors do not assign animation timing, flag, or unknown-field behavior. Existing indexed-8 RLE decoding remains a separate explicit path.

For CD IFCUSTOM, the two arrow controls at `(455,229)` and `(455,261)` bind animations 6 and 7. Their first/last frames are respectively 12/13 and 14/15. Each frame has one component and a 19×31 logical frame bound; each referenced sprite is 19×32. The normal component is `(0,0)` and the selected component is `(0,1)` after the signed shift. `genewars-render::render_cd_ifcustom_arrows` composes only those two sprites on a black 640×480 RGB image; it does not draw the surrounding interface or model input or animation timing. `genewars-desktop render-ifcustom ANB MAIN.PAL normal|selected OUTPUT.BMP` writes a non-overwriting BMP for inspection.

The per-resource source and BMP hashes for the generated normal and selected render receipts are recorded in `private/integration/panel/ifcustom-arrows-receipt.json`. The BMPs are ignored private artifacts and contain no source ANB bytes.

## Explicit packed-four-bit sprite variant

Separate creature-consumer analysis confirmed a second run payload encoding: positive signed controls count pixels, followed by `ceil(count/2)` bytes, high nibble first and low nibble second. A negative control skips opaque output positions; zero ends a row. Both nibble values are opaque, including zero. With an explicit base, each output index is `base.wrapping_add(nibble)`. The consumer also has a separate optional translation-table path indexed by `base+nibble` before byte conversion; that path is not part of the base-only API.

`SpriteTable::decode_packed4(record, database, base)` exposes only the caller-selected base-only contract. It consumes a complete packed literal run even when destination width clips writes, and respects the same embedded `[0,offset0)` ANB database limit as indexed-8 decoding. It never guesses the encoding from data. Synthetic tests cover odd pixel counts/padding, high-nibble order, transparent skips, wrapping base, clipped-run consumption, and truncated packed bytes. The existing indexed-8 `decode` is unchanged.

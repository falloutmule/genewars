# Mission source records and initial target

This tranche binds the exact CD CONQ001 file to source-only records and a gated initial camera derivation. It does not emulate a complete live mission. The validator pins 37 private listings, resolves the eight tail readers through their constructed virtual tables, checks 1,368 serialized record slots and every final file boundary, and derives a camera for selected player 0.

The file is 343,958 bytes with layout tag `0x55336`. Its direct prefix ends at `0x3B55E`. Four packed target values begin at prefix offset `0x1C7E`, with four-byte stride, each containing x and y 16-bit coordinate words with eight fraction bits. CONQ001 values are `[0x35005100,0,0,0]`. The first is x=`0x5100`, y=`0x3500`: cell `(81,53)`, fractions `(0,0)`. All four raw loaded player records' byte+9 dwords are zero, illustrating why those fields alone do not establish the starting target.

`0x80460` reads the separate table into player+9 when initializing a player. After mission loading, `0x7D090` again unconditionally assigns each active player's target from `0x11A9DE+4*player` at `0x7D6E5..0x7D6EB`. Its loop first jumps forward through that initialization, then branches backward to the selected-player camera call at `0x7D5A2`. Selected-player restoration therefore cannot determine the final target. `0x80590` copies player ranges `[0,0x4A1)` and `[0x61A,0x6F7)`; neither reaches the separate table after all four players. The checked `0x80810` success stub performs no record writes.

The serialized tail consists of these fixed source groups. Family numbers are neutral identifiers, not entity-class names.

| Family | File start | Count | Packed stride | Native state start | Native stride |
|---|---:|---:|---:|---:|---:|
| 0 | `0x3B55E` | 300 | `0x74` | `0x3B55E` | `0x7C` |
| 1 | `0x43D4E` | 512 | `0x29` | `0x446AE` | `0x31` |
| 2 | `0x48F4E` | 20 | `0x8C` | `0x4A8AE` | `0xC8` |
| 3 | `0x49A3E` | 256 | `0x3B` | `0x4B84E` | `0x43` |
| 4 | `0x4D53E` | 200 | `0x49` | `0x4FB4E` | `0x51` |
| 5 | `0x50E46` | 32 | `0x29` | `0x53A96` | `0x31` |
| 6 | `0x51366` | 32 | `0x59` | `0x540B6` | `0x61` |
| 7 | `0x51E86` | 16 | `0x5E` | `0x54CD6` | `0x66` |

Each packed record has a 31-byte source header, copied to native `[0,0x1F)`, and a remaining payload copied to native offset `0x27`. Native resource pointer `[0x1F,0x23)` and virtual-table pointer `[0x23,0x27)` are never serialized. Seven families' payloads extend to their native stride; family 2 instead stops at native `0x94`, leaving `[0x94,0xC8)` for runtime fields. A safe parser borrows the two serialized slices and preserves bytes without inventing pointers or treating packed records as native structs.

The source byte+6 controls the post-read active-initializer call. CONQ001 has one nonzero slot in family 0, forty in family 1, and none in the other six families. Constructor `0x1E280` establishes the virtual tables; the receipt records every reader at virtual offset `0x3C` and initializer at `0x60`. All reviewed active initializers write only within the entity or a stack temporary; one clears native `[0x94,0xC0)`. They do not modify the prefix target table or map height. Base reader `0x6D490` modifies staging-input bytes+15/+17 after copying the header; a source-only parser preserves the original file bytes.

After these groups, file `[0x52466,0x5249A)` supplies 52 bytes to global `0x118770`. Preserve this block except the established flag byte at offset 0 and active-player-count byte at offset `0x1C`. CONQ001's flag is `0x38` and count is 1. The next four bytes at `0x5249A` supply a saved viewport snapshot. Extension tag `0x6BE` is at `0x5249E`; its 6,900-byte payload starts at `0x524A2` and ends exactly at file end. The extension is opaque here. The loader finally derives `data/b32a.dat` from prefix byte `0x1C5B=0`, and updates a different numeric field at `0x1C67`, preserving targets.

Campaign bit `0x20` in the loaded flag bypasses `0x8A9F0`, which otherwise permutes the target table. For selected player 0, the active-player loop therefore supplies target `(81,53)` to `0x82E20`. At that cell the preserved signed height is 3136, so the height helper returns 6272; the camera quotient is `trunc((6272>>6)/32)=3`. Byte-wrapped camera components become `81-3-12=66` and `53-3-2=48`.

This camera result requires successful loading of the hash-bound source, campaign flag 0x20, and selected player 0. The menu retains the current selected-player index; the receipt does not claim the result for arbitrary prior session state. It is a static derivation, not an observed mission screenshot, complete visibility state, entity simulation, or proof that all startup callbacks are ported.

# Campaign entry and first mission resources

This contract is tied to CD candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. It follows actual control, constructor, callback, and file-loader instructions. It does not establish a complete mission simulation or campaign completion policy.

Main-menu control ID 0 is gated by input-mask bit `0x08` at `0x19FF70` and enabled-state bit `0x80` at `0x118969`. Handler `0x7C870`, branch `0x7C899`, takes the campaign path only when both gates permit it. The clear-enabled-state branch calls exit routines; it must not be mislabeled Single Player merely because it shares switch ID 0. The enabled branch clears the selected player's `0x70D` bytes, calls initializer `0x80460`, then queues view `0x186F19` at `0x7C945`. Constructor `0x15B10`, called with object `0x186548`, uses the proven layout offset `0x9D1`; its initializer is `0x15B90`.

The cleared progress byte is player+`0x6E7`, global `0x11945B + owner*0x70D`. The adjacent system byte is player+`0x6E8`, global `0x11945C + owner*0x70D`. Initial progress and system are therefore zero.

## State-aware path selection

The original sign-extends progress as an 8-bit integer and divides by six with signed division. In the bounded admitted range 0 through 23, quotient and remainder match ordinary nonnegative division. It stores local planet `remainder+1` as a 16-bit value at campaign screen+`0x28`. Only when local planet equals one does it update the stored system byte to the quotient and play the numbered system movie. For other local planets the previous system byte is retained, even if inconsistent with progress.

| Resource | Source of number | Relative path |
|---|---|---|
| Campaign header | Progress+1, byte argument to `0x15720` | `LEVELS/hdrcoN.dat` |
| Mission | Sign-extended progress+1 | `levels/conqNNN.dat` |
| Texture atlas | Effective unsigned stored system+1 | `data/plN.dat` |
| Campaign background and palette | Independently recomputed progress/6+1 | `data/levscr0N.dat`, `data/levscr0N.pal` |
| System movie | Newly assigned system+1, only when local planet=1 | `anims/systemN.smk` |
| Transition background and palette | Effective unsigned stored system, without increment | `data/toPlntN.dat`, `data/toPlntN.pal` |

A safe resource-plan API accepts progress 0 through 23 and prior system 0 through 3, preserves the conditional assignment, and rejects other inputs. Those are explicit port restrictions supported by available resource names, not claims that the original rejects those inputs or that mission 24 ends the campaign. DOS path comparison is case-insensitive; portable paths may normalize separators while preserving the original spelling. The receipt contains all 96 progress/prior-system vectors and 72 exact CD source identities.

## Confirmation and mission load

Input callback `0x17E80` forms a confirmation request from its input gates. When requested, it compares screen+`0x9A9` as a signed byte against nine. At phase seven it instead begins an intermediate transition. At phase nine or higher it sets screen+`0x9B7` to one and queues view `0x1863B8`. Phase production, exact key meanings, and the full animation sequence are outside this receipt.

Cleanup callback `0x16170` sees the transition byte, clears bit zero at `0x118968`, and sets campaign bit `0x20` at `0x118770`. The target view belongs to object `0x1863B0` with layout offset eight. Its callback table is at `0x1127D0`; initializer `0x7CE60` loads the transition artwork, and callback `0x7D090` formats the actual map path. The adjusted thunk `0x8DE70` subtracts eight and reaches `0x7D090`. This callback tests campaign bit `0x20`, sign-extends progress, adds one, formats `levels/conq%03d.dat`, and calls `0x71B70`. Initial progress zero therefore selects `CONQ001.DAT`; it is not inferred from the filename inventory alone.

## Bounded header and direct map prefix

Header loader `0x15720` takes destination in EAX, number in DL, and category selector in BL. BL equal to one selects `u`; otherwise it selects `o` in `LEVELS/hdrc%c%d.dat`. Campaign initializer passes BL=0, DL=progress+1, destination screen+`0x61`. It requests `0x2BB` (699) bytes. The original only tests for a nonzero read, so requiring exact input size is a deliberate safe-parser improvement.

The campaign title consumer passes screen+`0x81`, equal to file+32, as a NUL string. A safe parser searches only `[32,699)`, permits an empty string, and rejects a missing terminator. This allocation bound is not proof of a fixed-width title field. The remaining header is structured data; it must not be exposed as one text block. Exact first header `root/2/disc/190` has SHA-256 `5e1f382ea68f57f1cec03a3f7de2c4903ca127519481da98232a7f1c3a46aa11`; its first title terminator is byte 40. Other fields remain opaque in the initial parser.

Exact first map `root/2/disc/152`, SHA-256 `72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2`, is 343,958 bytes. Its first little-endian u32 is `0x55336`. Loader `0x71B70` first reads four bytes into state+0. This tag selects a read of `0x3B55A` further bytes into state+4. Thus file prefix `[0,0x3B55E)` maps directly into the same state offsets, covering 243,038 bytes. Four initial player records begin at offset `0x14`, stride `0x70D`. The uninterpreted map region begins at `0x355E` and consists of 16,384 records of 14 bytes; it ends at the prefix boundary. The legacy branch independently corroborates that count and stride, but its overlapping 6-byte/9-byte reads must not be used as the fast format.

Cleanup routines `0x80E90` and `0x80820` operate on existing live player records before the prefix overwrite. After loading, `0x80810` returns success (`AL=1`) through alignment instructions and `RET` at `0x80820`, without writing the loaded record; the selected local record can also be restored through `0x80590`. The bounded read-only listing and mapped bytes corroborate that return despite the earlier function export omitting it. Accordingly, a parser exposes initial file bytes, not final player state. The 100,920-byte serialized tail has separate deserializers and is opaque in this tranche. Neither the tag nor total file length licenses casting the whole file to a live-state struct. Map coordinate dimensions, cell field meanings, palette binding, mission entities, and gameplay commands remain unresolved here.

Run `python -B scripts/ghidra/validate_campaign_entry_bridge.py` from the project directory. It verifies the executable hash, mapped tables and format strings, 14 private instruction contracts, all resource-plan source identities, and exact first header/map bounds. Raw listings and resources remain beneath `private/ghidra/campaign-*`; the safe receipt is `campaign-entry-evidence.json`. No original executable is run.

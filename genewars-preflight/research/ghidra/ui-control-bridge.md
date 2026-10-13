# Concrete custom-screen controls and byte-state transitions

This tranche applies only to CD candidate executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. It is static analysis, not an original execution trace. No `ifeditor` filename literal exists in this candidate file. IFEDITOR assets' presence and table equality therefore do not prove this executable uses them. The concrete bridge below uses the explicitly referenced CD `data/ifcustom.anb` instead. Its screen label is not inferred from asset artwork.

`ui-evidence.json` contains exact addresses, private listing hashes, CD asset provenance and synthetic vectors. `scripts/ghidra/validate_ui_bridge.py` reproduces its checks. Raw listings/decompiled code are private in `ui-custom`, `ui-controls`, `ui-input`, `ui-frame`, and `ui-objects`.

## Actual initialization and asset association

Static constructor instructions `0x8DC51–0x8DCC2` set the screen object at `0x185014` and its control pointer at `0x185018` to the list at data VA `0x107154`. This is an instruction range with known boundaries, not a separately named function. Its exported range begins after register values were prepared, so it is not a callable standalone constructor.

Descriptor `0x106530` names `data/ifcustom.anb` and has allocation destination `0x18501C`, exactly screen+8. Initializer `0x7E0B0` loads that descriptor, passes screen+8 through ANB loader `0x98A60`, stores the returned animation-table base at screen+0xC and the sprite-table base at screen+0x10, then calls UI initializer `0xA8320` with screen+4 controls, sprite table in EDX, animation table in EBX. It sets active list global `0x19FF64` to those controls. At `0x7E2E7` it sets the byte at screen+0x16 to zero. Its setup also establishes input-delay byte `0x10AAFD` as one. The initializer is conditional on its preceding resource setup succeeding; the port must report failure instead of claiming this state exists unconditionally.

The exact CD decoded IFCUSTOM database has SHA-256 `daf41bf104dd5d6436f7af23158a5d23d624c12780e3e66177b3b5aec2fbe006`, size 36,660. Compressed input SHA-256 is `1f69d8bca8bed9d9e15ee1cc79e0ce0856bb4cb6d7e3791e704b31d02b0006b4`. Header is 13 final animation records and section offsets 35,496 / 35,718 / 36,068 / 36,530. The CD source path and entry ID come from the corpus receipt in the JSON. This uses its embedded 222-byte sprite table, not the different external 78-byte IFCUSTOM.TAB from root/4 (the package with the larger 5,184,949-byte GW.EXE and the 11,338-byte IFCUSTOM resource).

UI records are 40 bytes, terminated by zero flags. For these controls the verified fields are u32 flags+0, signed16 x/y+4/+6, signed16 dimensions+8/+0xA, signed16 returned event ID+0xC, callback pointer+0xE, resource offset+0x12, capture-mask byte+0x26 and draw-state byte+0x27. Packed unaligned fields must be read explicitly.

| Control record | Event ID | Position | Flags | Animation | Normal / selected frame | Normal / selected sprite |
|---|---:|---|---|---:|---|---|
| `0x1072BC` | 27 | (455,229) | `0x48` | 6 | 12 / 13 | 13 / 14 |
| `0x1072E4` | 28 | (455,261) | `0x48` | 7 | 14 / 15 | 15 / 16 |

Both controls' normal components have local position (0,0); selected components have (0,1), after the loader's signed right shift by one. Their frame dimensions become 19 by 31, while each sprite raster is **19 by 32**. The differing dimensions are real asset data. The UI initializer derives the record dimensions from the first frame, but the `0x40` hit-test branch bypasses the rectangular test and delegates to component pixel hit testing. Do not use a 19-by-31 clipping rectangle for these buttons.

## Input and draw binding

Custom-screen input callback `0x7E400`, reached through slot `0x112874`, calls generic manager `0xA87D0` and stores its low 16-bit event ID in global `0x1169A2`. The manager reads active list `0x19FF64`, tests controls in list order, and invokes hit tester `0xA86D0`. For flags `0x48`, the latter takes the `0x40` branch to frame hit tester `0x98C40`, which walks frame components from last to first and calls `0x98B60` for RLE opaque-pixel testing. Transparent pixels do not hit; raw color index zero is not itself proof of transparency. Physical mouse labels are established in the later input-packet-bridge.md: these two controls use the left button.

The input packet pointer is global `0x118970`; signed16 coordinates are at +0/+2. With no capture, packet byte +8 bit0 produces trigger mask 8; mask16 can be produced by bit6. The mask is intersected with control flags. For these controls only mask8 is enabled. An opaque hit stores capture-mask8 at record+0x26, draw-state1 at +0x27, captures the record in global `0x19FF88`, and returns low16 sentinel -2. The initial countdown is one in this screen.

With this captured ordinary control and countdown <=1, packet byte+8 bit1 produces mask8; bit7 produces mask16, intersected with the stored capture mask. A nonzero result returns the captured record's event ID, clears capture and record+0x26/+0x27, and records the input mask in `0x19FF70`. Because these controls lack flag `0x1000`, this branch does not require another pixel hit on release. A packet with neither matching bit leaves capture pending. The later input-packet-bridge.md establishes these bits as left/right press and release events and records their latch lifetime. Other control flag combinations and countdown>1 are outside this two-control port.

Draw callback `0x7E9C0`, slot `0x112870`, calls generic UI drawer `0xA85D0`. It passes record+0x27 bit0 into `0xA84D0`; the flags0x40 path selects the normal/selected frame via `0x982E0`. The four concrete frame/sprite states above are therefore actual display bindings, not merely adjacent ANB entries.

## Portable state contract

The switch table at VA `0x7E3B0` maps event27 to `0x7E8DC`, event28 to `0x7E8EF`. The caller's object pointer was saved from EAX on function entry.

- Event27 reads u8 screen+0x16 and decrements it only when nonzero: `max(value-1,0)`.
- Event28 increments that byte with modulo-256 wrapping: `(value+1)&255`.
- Other event IDs do not change that byte through these cases.

There is no item-count clamp in these two branches. A compatible primitive must preserve the byte wrap even if a modern screen later imposes separate valid-range rules. Initialization sets the byte to zero. Draw callback `0x7E9C0` uses it as a list offset, establishing that role without naming a particular gameplay item type.

The subsequent list-selection block `0x7E8F5–0x7E98A` is also deterministic: when global byte `0x19CFB6` is nonzero, input event ID is -1, signed32 global x at `0x19CF9C` is in [180,454), and y at `0x19CFA0` is in [224,296), it writes selected byte screen+0x19 as `(offset + (y-224)/12) mod256`. The lower bounds are inclusive; the decompiler's rewritten `>` expressions obscure this. It then copies a string from screen+0x515+selected*0xE1 into screen+0x3D5. The unchecked original string copy and its allocation assumptions must not be ported: the safe primitive returns the selected index and the caller validates its record/string bounds. The later input-packet-bridge.md establishes this global as the left physical-held byte.

Validation exhausts all 256 starting byte values for the two offset events and verifies unchanged -1/-2 sentinels; nine selection vectors cover both inclusive lower/exclusive upper edges, event gating, flag gating, and 255+5 wrapping. Four frame/component/sprite bindings are checked against the exact decoded CD database. Jump-table targets, initializer assignments, control records and private instruction contracts are checked independently. No original code is executed by these tests.

Later tranches establish the mode-two input producer and per-call callback order in input-packet-bridge.md, and ten-record storage/count bounds in custom-list-layout.md. Wallclock cadence, animation timing/flags, incoming-record string termination/encoding and the full custom screen remain unresolved. IFEDITOR's relationship to this CD executable remains unproven.

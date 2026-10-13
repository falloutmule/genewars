# Main-menu font and static composition bridge

The exact CD candidate loads its labels from installed `DATA/TEXT.DAT`, not a runtime language subdirectory. The missing flat file observed in run-002 is an evidenced prerequisite failure; `english-install-bridge.md` proves the installer mapping and hashes. A fresh run can distinguish that staging problem from any remaining runtime issue. No font corruption or emulator fault is established by the earlier screenshot.

This bridge is limited to executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. `font-menu-evidence.json` records source hashes, addresses, layout facts and private listing hashes. The validator reads originals and cached data without executing the game. Raw glyphs, text and listings remain private.

## Font and text contract

Menu initializer `0x7C6A0` loads descriptor list `0x1061EC`. This binds IFTITLE to screen `0x184FE8`+8, TITLESCR.PAL to `0x18500A`, and FONT2.DAT/TAB to globals `0x118C94` / `0x118C8C` (TAB end `0x118C90`). The screen layout offset read through `0x111ED8` is 0x20; `0x7C6EB` therefore supplies palette pointer screen+0x20+2, exactly `0x18500A`, to font initializer `0x1F550`. Menu FONT2 uses TITLESCR.PAL. MAIN.PAL is not the palette selected by this binding.

`0x1F550` calls `0xA7618` to relocate each six-byte TAB record's u32 source offset by the DAT base. There is no ANB +18 bias. It then sets context.table = TABbase+6. For byte code c>=32, measurement/drawing uses context.table+(c-32)*6; the **absolute whole-TAB byte offset is `(c-31)*6`**. Space32 uses TAB+6; A65 uses TAB+204 (whole-table record34, post-prefix record33). On exact FONT2, A's raw tuple is offset3917, width30, height27. The native API must apply the prefix once.

For the unkerned one-line subset, advance is width-1 for every admitted byte. Space advances without drawing. Other bytes select width/height from +4/+5 and draw indexed RLE; a literal index zero is opaque. Center wrapper `0x1F730` computes left = center - arithmetic_shift_right(measured_width,1). Restrict native line input to bytes>=32 admitted by the paired table; original code has no table bounds. Control bytes, wrapping, kerning tables and styled alternate font path `0x1FF30` are outside this line API.

Initializer `0x7C7CE` sets context flag0x20, selecting shared blitter `0xA7730` through `0x1FAAC`. The ordinary no-flip/no-translation branch copies min(positive control, remaining width), then advances over the full literal run. Negative controls skip transparently. If width is exhausted and more visible rows remain, it scans remaining controls/literals to the row zero. It may return immediately after the last visible row, before that scan. The safe complete-stream decoder requires one zero per declared row, including the last; this is an explicit stricter validation/consumption policy. FONT2's 194 admitted glyph streams have 3,613 rows with no width overrun, as independently measured in `research/formats/font-text.md`.

Text startup `0x1DA48..0x1DA7D` builds **604** NUL-terminated strings (predecrement before store). English source entry592 consumes36,609 bytes for these strings and leaves one trailing NUL. Expose opaque bytes and trailing remainder; do not guess an encoding or use unchecked scanning. Startup separately changes the first found `|` to newline for indices221..254; the menu indices below are outside that mutation range.

## Exact normal three-button menu

Constructor range `0x8DC04..0x8DC57` binds the main screen's controls to data VA `0x10708C`. Four initial40-byte records have flags0x1048, x140, y85/156/227/298 and IDs0/1/3/2. Their animation offset at +0x12 is zero. The stored value36 at +0x16 is **not** the animation index and is not used by the flag0x40 path.

When global byte `0x118968` bit0x04 is clear, initializer `0x7C714..0x7C739` disables record1 and moves visible records0/2/3 to y115/186/257. The mapped initial byte is zero. A command-line network initialization branch can set bit0x04 after `0xA604C` returns1; complete network behavior is not part of this renderer.

Generic initializer `0xA8320` adds the animation base to control+0x12. Drawer `0xA85D0` passes draw-state bit0 to `0xA84D0`; its flags0x40 branch selects animation.first for normal or animation.last for selected and calls `0x982E0`. Exact IFTITLE has only animation0: normal frame0 and selected frame2, each346x75. Normal frame0 components are sprite1 at(0,0), sprite2 at(116,11), sprite3 at(236,3), raster sizes116x75,120x50,110x71. Selected frame2 uses sprites7/8/9 at the same positions and dimensions. All six component flags are zero. No animation timing is needed for this bounded normal-state composition.

`0x7CA70` copies TITLESCR.DAT, draws controls, then draws labels. The source decodes to307,200 unchanged index bytes. `0xAEA78` copies clip-width/4 dwords per row, advances destination by the framebuffer pitch, and runs logical-height rows. For the already-established full640x480 clip/pitch this is a640x480 row-major raster. Other clip/pitch states are not admitted by the static native composition. Use title palette's exact768 RGB6 bytes and the existing explicit RGB6 presentation policy.

| Text index | Center x | Three-button y | Measured width | Left |
|---|---:|---:|---:|---:|
|255|312|140|246|189|
|258|312|211|151|237|
|259|312|282|85|270|

Optional fourth-branch index257 measures178 (left223). Its y is control record1.y+25 when that branch is enabled. The native three-button composition must not draw it. Text contents are fetched from the user's exact resource and are not copied into repository metadata.

The native normal render requires full640x480 clipping, zero incoming global draw flags and UI origin(0,0). A84D0 adds signed16 UI origin globals0x1A011E/0x1A0120 to control positions; these four controls lack flag4 and do not themselves reset that origin. The component drawer preserves incoming flags; its zero component flags do not establish that every runtime caller cleared them. Runtime flag state, callback wallclock timing, hover/capture state and cursor motion are excluded from this static image contract.

## Reproduction and remaining runtime diagnosis

Run `python -B scripts/ghidra/validate_font_menu_bridge.py`:14 private instruction contracts, five exact CD decoded assets, four control records, six frame components,604 text entries and four centering widths. `scripts/ghidra/validate_english_install_bridge.py` separately binds the prerequisite install copy. Luna owns native parser/raster tests; Sol02 owns independent review.

If mouse motion still fails after text repair, the already-exported callback `0xA3524` supplies a focused diagnostic: movement is processed only when event bit0 is set, from signed SI/DI cumulative counters and deltas rather than absolute CX/DX. Buttons arriving do not prove that movement event or counters arrive. `0xA381A..0xA3837` additionally gates direct coordinate updates on globals0x19F350 and0x19D220 both zero. This is a static watchpoint suggestion, not a proved runtime cause or a request to change host configuration.

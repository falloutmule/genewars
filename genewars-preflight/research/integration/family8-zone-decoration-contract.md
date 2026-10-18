# Krell marker arrival decoration: bounded source contract

Status: explicit-context state11/12 renderer implemented and verified by152 full-buffer cases (47,276,864 bytes), including60 actual source-compared marker-motion records; nine invalid inputs reject atomically. Existing state10 regression passes44 cases. Live UI timing/host integration is a separate proof. No mission acceptance is added.

The verified CD `10EA0` selector2 branch uses ETHEREAL animation8. State10 bypasses decoration; state11 calls `108E0` with displacement `(((20-timer)*256 / 20) * height_pixels) >> 8`; state12 calls it with displacement0. `height_pixels` is signed `(physical-ground)>>6`. The radius arguments are30 and15. Shadow precedes decoration and the ordinary foreground image follows it. Integer division truncates toward zero; shifts are arithmetic. State13 uses a different displacement and is outside this packet.

## Required inputs

- Exact marker record, projected ground origin, source frame counter and physical/ground height.
- Its live19-byte objective-zone record: native `11BD58 + 19*marker[58]`, corresponding to mission offset2FF8. Type is byte4; progress is signed dword0E.
- Hash-bound PLGBL.ANB, rebased according to98A60. CD original size13697, SHA256 `cce0f7de88f2dc8ec251e70925de93d6e406c109d0bea9a82f4ee7f1592c9a30`; decoded size40414, SHA256 `a39cc8ac8c987de5c4865ec77a4459221f276286f254653e4a3f3231424f6444`. Existing private paths are `private/reference-run/run-006/drive-c/GAME/DATA/PLGBL.ANB` and `private/review/PLGBL-decoded.bin`.
- Seven live UI frame bytes at184D0D+5*i, plus frame184D05. Existing countdown requests name their descriptor/state pairs, but the host currently counts requests without maintaining these animation states. A constant frame0 is not an authentic substitute.
- The seven mapped color bytes at102E38, active-palette white index186F55, pinned FADE and MAIN.GLA tables, viewport and destination extent.

## Ordered draw operations

1. For zone type7..13, draw PLGBL animation`type+21`, frame from184D0D+5*(type-7), at ground origin using96870/95390 recoloring: only paletteCx becomes mapped color[type-7]+x; other literal indices are preserved. Flags are4 only when marker byte15 has bit10 and `(source_frame + slot)` is odd; otherwise0. Its flag4 uses destination-high GLA; the progress icons use source-high GLA.1094C reads only the frame low byte, so the existing u8 host frame preserves all relevant parity.
2. Draw `trunc(progress/1024)+1` PLGBL animation35 symbols using frame184D05 and97A90. First position is `(x-count*7+7,y-10)`; subsequent symbols advance14 pixels. Source truncates coordinates to signed16-bit. Nonpositive counts draw none. Preserve source order and bounds; do not cap counts silently.
3. Clear sprite flags. Build twelve20-byte geometry vertices from radii30/15, integer ellipse calculations and97910 integer square root. Translate by `(origin_x, origin_y-displacement)`. Source clamps either radius to at least1.
4. Fill five vertical rectangles from `origin_y-height_pixels` to their corresponding translated ellipse points, with colors FADE[28][white], FADE[30][white] and white. The980C0→B5C50 flag8 path blends as `GLA[(destination<<8)|color]`. This differs from flag4 sprite blending and must not reuse its operand order accidentally.
5. Configure triangle viewport usingE88A3, preserving the source sidebar offset. Draw six mode15 triangles in source order with the same three colors. Existing `prepare_mode0_15_triangle` / `draw_mode15_triangle` APIs provide the shared raster path; no new triangle algorithm is needed.
6. Emit the7A710 dirty-region obligation, or explicitly consume it through the host's full redraw policy. Then continue10EA0's ordinary ETHEREAL foreground draw.

## Integration boundary and next work

The bounded renderer receives an explicit `ZoneDecorationContext` with the current zone record, eight verified animation frames, palette-white index and source frame. It validates every asset/frame and destination before committing any pixels, then stages the complete shadow→decoration→foreground result. It must not advance gameplay or UI animation itself.

The host needs source-ordered UI animation state advancement before this context can be supplied correctly; the separately verified ui_animation module is now available for integration. Resource staging alone does not resolve that dependency. The required runtime requests already exist in `countdown_expiry::animation_requests`; recover their animation callback and state before integrating. Root owns host/runtime integration and staging. Presentation work can independently implement and compare explicit-state fixtures, but these do not prove runtime timing.

Independent validation compares full pitched buffers for captured state11/12 records, animation changes, positive/negative origins, clipping, blinking, varying progress counts, and transactionally rejected invalid frames/zone types. Preserve the existing state10 proof. Validation uses standalone rustc against pinned existing release libraries, no Cargo build or native replay. A first harness run exposed the shared triangle primitive's complete-pitch-row requirement; the transactional staging buffer now supplies private final-row padding while preserving the caller-visible framebuffer extent. Its failed private receipt remains preserved.

## Evidence

Private disassembly: `countdown-expiry/instructions-00010ea0.txt`, `terrain-viewport-refs/instructions-000108e0.txt`, `display-geometry/instructions-000b5c50.txt`, `portrait-detail/instructions-000980c0.txt`, `triangle-buffer-refs/instructions-000e88a3.txt`, and `display-geometry/instructions-0007a710.txt`, all under `private/ghidra/`.

The existing PLGBL identity/relocation evidence is `research/ghidra/building-plgbl-overlay-contract.json`; its older bounded building-overlay renderer admits only animations2/11/12/13 and cannot be substituted for the marker's animations28..35.

The production API is `ZoneDecorationDatabase::from_bytes`, `ZoneDecorationContext`, and `draw_family8_selector2_arrival`. `actor_scene::render_actor_world_with_selected_portrait_and_zone_ui` takes the existing render arguments plus eight live UI frame bytes; older wrappers explicitly lack this context. PLGBL origin is corpus decode `private/integration/rnc-decoded/cce0f7de88f2dc8ec251e70925de93d6e406c109d0bea9a82f4ee7f1592c9a30.bin`, manifest `root/2/disc/124`. Mapped colors102E38 are `[240,208,192,208,240,224,16]`. Active-palette715E0 white is a host input (main palette145), not a fixed renderer color. Actual state11/12 records carry flags0020 after source arrival clears4000; both that form and4020 diagnostic input are admitted, with optional blink bit10.

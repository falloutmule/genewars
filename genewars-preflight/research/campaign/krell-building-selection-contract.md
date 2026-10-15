# World click to Genepod panel

Reference is the pinned CD QA BETA V5 executable. This contract comes from source inspection and vtable resolution, not a new native acceptance run.

## Hit candidate and order

Original83140 queues projected visible entities and independently maintains the pointer under the mouse. Visibility byte17 must be nonzero, the family bit in viewport word1F must allow the entity, viewport flag1C bit1 must be set, and special family gates must pass. For a building, positive viewport byte11D6 suppresses picking. Use current visibility and frame; ownership is not a substitute for visibility.

83140's834AA..834F9 compares the entity's **ground projected Y** with previous winning Y at viewport11D3. A smaller Y cannot replace the winner; equal Y can, so the later eligible queue entry wins ties. Its virtual20 hit uses screen X including viewport panel margin, ground projected Y, and mouse coordinates118771/118773. On an opaque hit,836D2 stores the entity in viewport37 and records projected coordinates. Modifier keys19D00E/19D01A can suppress friendly candidates. 83930 publishes viewport37 into viewport18 (global16E0C0) at83AE5 and clears37; picking and input therefore have an explicit publication boundary.

Do not substitute ascending building-slot priority, nearest-center selection, terrain-cell selection, or sprite rectangles. For a bounded single Genepod, use its actual image opacity and reject ambiguous overlap with another eligible candidate until shared queue order is composed.

## Exact building alpha hit

Mapped vtable112120+20 resolves1F3C0. The newly exported private `genepod-building-hit.txt` establishes that this function calls virtual1C, then98C40, and accepts any component index other than-1. Virtual1C resolves1E840: read animation byte19, clamp current frame byte1B to animation count minus1, and use that14-byte frame record. It does not use the shadow image.

98C40 scans components in reverse order. For each nonnull sprite, subtract projected origin and signed component offsets; component flag1 flips X as width-1-X, flag2 flips Y as height-1-Y. Then98B60 checks sprite opacity. Palette color zero must remain opaque when encoded as a literal pixel. Frame/component metadata and decoded opacity already exist in `SourceBuildingResourceDatabase::animation`, `BuildingAnimation0Frame`, and each decoded component sprite. Existing `entity_view_position` supplies the same ground projection. The current desktop draw adds panel offset through its destination slice; the hit position must use absolute screen coordinates consistently.

## Opening the panel

MAINPANL's2C965 branch excludes event high-byte0A and118968 bit2. The no-right-release branch reaches2C9D0. With no active conflicting selection group, a different nonnull hovered entity16E0C0, nonzero visibility17,118770 bit4, and118968 bit2 clear,2CA51 dispatches by entity kind. Kind4 owned by the selected player and selector2D=0 calls21A10 on the Genepod subpanel (root module offsetE2). Friendly reciprocal-alliance cases also exist, but owned single-player is sufficient for the upcoming bounded path. Other selectors open their corresponding panels. This is view selection, not opcode12 production; actual production still requires the separate panel capture/ready contract.

## Smallest remaining integration

1. Build hit candidates using the actual rendered Genepod resource/current frame and visibility, with the shared projection and original opacity rule.
2. Preserve candidate ordering/publication and ordinary mouse-event gating; compose only the owned selector0 case initially, with explicit unsupported overlap rather than guessed priority.
3. Open the Genepod panel using actual building slot/byte3C and current player list; then ordinary control down/up reaches the existing producer.
4. Capture intermediate selected-building/panel state and resulting opcode12. No direct slot selection or preselected panel counts as native world-click acceptance.

The exact upstream population of118770 and whole mixed-family queue tie order remain integration evidence to bind. The recovered component hit and panel destination are sufficient to implement the bounded no-overlap path without inventing geometry.

## Implemented bounded hook

`support/building_pick.rs::hit_animation` now consumes the existing animation frame/component types and returns the hit component index. Root supplies the selected animation, current frame, absolute screen origin, mouse position, visibility and queue ordering. It rejects empty/malformed resource metadata and coordinates outside source signed-word input range. `scripts/ghidra/building_pick_review.py` passes672 comparisons against literal98C40/98B60, covering two overlapping components, all H/V-flip combinations, frame clamp0/1/255, transparent pixels and literal palette zero. The standalone harness uses fixture stand-in resource types while compiling the exact module; production integration and native acceptance remain separate.

The native Game now connects ordinary pointer/down/up to visible owned Genepod alpha selection, lazy catalog-bound decoded GENEPANL loading, the control capture state, and transactional `dispatch_genepod_mule_input`. It draws the original control face and reports the producer outcome. The frame and gameplay world commit together after rendering succeeds. Disabled controls do not poll. Escape closes the panel. Loading Exceland does not require the optional new UI asset; opening the Genepod does.

This remains bounded integration: multiple building hits return an explicit unsupported-overlap error; mixed-family queue priority is not yet wired, and BTURR hits are excluded from this Source-resource picker. A chosen acceptance click must separately establish no competing actor/creature/effect/plant hit or extend the shared queue. The native fixture test uses a directly created completed building to exercise UI dispatch, so it cannot count as mission or construction acceptance.

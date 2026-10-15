# Krell zone bootstrap: bounded source contract

Reference QA BETA V5; source hashes in adjacent JSON. The initial static audit is supplemented by the frozen-world comparison below.

`72AD0` first resets all 16384 map-cell visibility bytes (`cell+12`) to zero for ordinary initialization, or `3F` for its nonzero mode. It then scans `backing+2FF8`, 19-byte records, count byte `34B8`. At `72B54` it tests signed zone dword `+0E > 0`, irrespective of zone type. For every eligible zone it calls `79D50(zone,1)` (`72B5F..66`). For ordinary mode only, it samples `7B140(zone)` and invokes `101D0` with selector EAX=2, packed position EDX=zone first dword, EBX=height+`7D00`, CL=zone index (`72B72..95`). Then normal `8AAE0`, selected-player `75B00(...,4096)`, and `72960` tails run. Preserve this order.

The existing recovered Krell contract has one type-12 zone at `[256,24320]`, radius dword `+6=1024`, positive quota `+0E=1024`; therefore it takes this path. Quota is an eligibility predicate here, not a falloff multiplier.

## 79D50 numeric obligations

Scan inclusive radius bounding-square cell origins, Y outer/X inner, by incrementing the high byte of wrapped 16-bit coordinates. For this zone there are 9 by 9 = 81 visits: X grid coordinates -3 through5, Y91 through99. Map indexing masks each axis to127; do not clamp the west edge.

For each visited cell, sample height at its origin and the zone center through `7B140`. Squared distance combines source wrapped horizontal deltas and `(height_difference >> 3)^2` (`79ECE..79F1E`). Recompute the base signed shade word at cell+0 with `8C290(cell,-1)` before radial addition (`79F22`). That routine preserves ECX, which holds squared distance.

If distance squared is below `(radius-512)^2`, add `11` to the shade high byte, clamp resulting signed word above `3000`, and set visibility+12=`3F` (`79F35..52`). Otherwise, if below radius squared, call source integer square root `97910`, compute `radius-distance-1`, add its signed/truncating scaled `1100/512` contribution to the shade, clamp above `3000`, and maximize visibility with its scaled `64/512` contribution (`79F5A..79FBA`). Preserve source intermediate widths, wrapping and truncation; do not substitute float distance or a generic circle.

`8C290` is required even outside the radial falloff within the square. Its water branch uses signed global `11A9C3` and cell height with integer division (`8C29C..8C2C2`); land shade uses two `733A0` terrain normals and pinned lighting globals `106604/608/60C`. It is not a no-op or merely a display request.

## 84B40: numeric and presentation work

Every square visit calls `84B40(16E0A8,point)` at `79E57..60`. Its `847D0` calls update viewport invalidation/cache state; these can become explicit presentation requests only if the native renderer consumes equivalent updates. Separately it ORs map+11 with `0C` on four neighboring cells and calls `8B800` on their entity lists. `8B800` traverses via `6D390`, tests entity virtual6C and flags+15 bit2, then writes interpolated shade to entity+17 using `7B240`. Those entity/map mutations are numeric obligations, not a presentation port. Existing cell entities, including plants, must be considered; an empty-map assumption requires an actual fixture check.

## Zone marker and reuse

`101D0` allocates the first inactive family8 slot (native base `16DA36`, stride`66`, count`11C2B9`, cap16), invokes base initialization, inserts into the map with `8AC60`, configures selector2 and runs virtual60 binding before incrementing count. Existing `family8_creation.rs` and `ActorWorld::create_family8` already implement this factory shape: reuse them with owner input equal to zone index, selector2, zone position and initial height `height+32000`. Selector2 subsequently sets owner4. Existing API owner<4 is sufficient for this zone index0 but not a general claim about all possible zone indices. Its initialized target height is a distinct factory field; retain existing source plan rather than replacing it with initial height.

Allocation when already at cap returns null in source and the caller does not reject the zone. Invalid count/links or unsupported resource initialization is different from that supported full-pool outcome. Native acceptance must eventually render the actual marker and revealed zone; successful numeric bootstrap alone does not prove that presentation.

## Atomic boundary and next proof

Stage visibility reset, every zone shade/visibility update, neighbor map/entity updates, family8 allocation/map links/count, and existing bootstrap tails together. Any unsupported callback/resource/division precondition should return typed unsupported/error and roll back; never commit a partial reveal. Keep invalidation requests ordered.

Smallest next proof: literal `72AD0 ->79D50` against authentic CONQ002 map and current loaded entities, with existing proven terrain helpers; compare map and entity shade bytes after the 81 visits, then compare the existing selector2 factory afterimage and links. Keep renderer/cache operations explicit and confirm their consumed native effect separately. No broad zone architecture is required.

## Frozen loaded-world comparison

`zone-bootstrap-review.py/.json` passed on `private/review/krell-zone-bootstrap-001`: 80,299 interpreted source instructions, 81 vertex visits, one `101D0` factory, and 427,369 compared bytes without differences. The fixture is actual CONQ002 loaded through the campaign path with a **synthetic carried prior**, not a completed campaign transition. It begins after visibility reset and ends after the zone before later player reveal/global shade. The literal wrapper executes the relevant `79D50`, height sample and factory sequence; it does not execute all `72AD0` tails.

Map, all supplied native pools, serialized plant shade, players, marker count and RNG match. Only native pointer/animation bytes31..39 are normalized. The `847D0` renderer-cache return and `97910` integer square root are explicit ports; actual `8C290`, `84B40` numeric tail and `8B800` execute. One `7B240` invocation is the marker factory sample, so this fixture does not establish occupied-zone entity-shade behavior. Native renderer/cache acceptance remains separate. Review JSON pins all fixture files, source listings, original memory objects and harness.

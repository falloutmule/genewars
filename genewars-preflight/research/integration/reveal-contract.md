# Numeric landing reveal

Corroborated static scope: CD75B00 with radius4096 calls84B40 after each increased map factor. The scan covers33×33 byte-grid vertices in row-major order. Each source cell is visited once by this scan, so the factor plan may be prepared from an immutable map before committing its ordered changes. Heights are unchanged by the callback.

For each changed vertex, the numeric84B40 tail marks cell+11 mask0C and calls8B800 for four cells in this order: current, left, upper-left, upper. Coordinates wrap modulo128. The entity callback follows the current intrusive list and calls vtable+6C. The admitted kind1/3/5 tables select1F240 (false); the plant table selects1F280 (true). A true predicate with entity+15 mask2 clear writes7B240's interpolated shade to entity+17. Thus plants update immediately; actors and effects refresh shade during their later physics callbacks.

Observed Rust behavior: `ActorWorld::apply_reveal_4096` commits factor changes, numeric dirty bits and eligible plant shades atomically. Unknown family predicates, malformed links or stale data reject the transaction. Native actor/creature records, players, RNG and other packed bytes remain unchanged. Repeated identical reveal produces no further changes or callback requests.

An independent ordered model over exact CONQ001 compares32 target cases and11,007,040 afterimage bytes, including wrapped positions and occupied plant cells. Unit tests cover mask2 suppression and rollback after an unsupported linked entity is reached. `reveal-validation.json` pins the executable-derived listings, implementation and compared output. No original executable ran for this validation.

Untested presentation work:84B40's viewport-column invalidation, optional minimap update and original redraw scheduling. The receipt retains ordered vertex notifications for those services. The native development host currently redraws the whole scene and applies a separate camera-local factor plan to its rendering copy. Neither policy establishes the complete original visibility lifecycle or campaign entry.

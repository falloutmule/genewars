# Shared landing and effect updates

Observed implementation state: `ActorWorld` owns one map, native creature/actor/effect pools, players and a shared RNG. Plant passes borrow that same storage. Numeric family4 remains separate from the complete inter-family scheduler. No original executable was run for these comparisons.

The code31 factory writes its native81-byte record, map links, pool count and player+496 type5 reference in one transaction. Audio and reveal are explicit pending requests. `step_landing_phase0` samples actual map height/shade/corners/terrain class, applies bounded vertical physics, and either creates a code6 smoke effect or invokes associated player activation at contact. Airborne smoke uses three parent draws plus one successful factory draw. Contact itself draws no random values; qualifying actor placement consumes its own recorded draws.

Physics constants come from hash-bound mapped tables. The mission loader replaces serialized water2000 with twice signed prefix1C63:1632 on CONQ001. Gravity remains signed prefix1C5D:1024. The host passes these runtime values explicitly. Clear map byte11 mask1 proves the bounded collision query's NULL result; water and unimplemented collision paths reject.

`step_effect6` implements fractional motion, first contact, repeated grounded motion with the second actual terrain sample, animation and damping. Moving between cells unlinks and reinserts the same native record. Kind5 virtual88 is a proved no-op. Retirement preserves the dead record's raw link bytes, clears matching reference kinds across all five source pools, preserves reference index words, applies the kind1 flag cleanup, clears the active kind and decrements the effect count. UI pointer cleanup remains typed state plus explicit pending requests. The native-empty UI value is a port policy, not recovered original GUI memory.

`step_landing_effect_callbacks` follows the first two4E950 scans. It checks live slots in ascending order, skips newborns in the primary scan, then clears each admitted newborn flag before its same-frame callback. A bounded second scan rejects an inconsistent creation count. Its queue list is only a candidate list:83140 draw admission and984C0 palette work remain presentation tasks. Unknown selectors and phases reject the entire pass.

Phase1 now includes timer49's message/random choice and49 smoke emissions. Phase2 raises the object and emits29 more smoke records; phase3 accelerates its departure and emits19 code2F particles. Parent retirement clears only the player reference's kind byte and writes timer700, then performs common link/reference cleanup. Code2F uses EFF animation47 and its own no-damping/virtual70-false physics boundary. Pool-full NULL creation is handled without inventing factory RNG draws; unsupported replacement remains transactional.

The owned native route now executes these phases, renders the landing/particles/actors/Mule, and accepts development selection plus source terrain-targeted commands. Player-list actor callbacks execute bounded movement, arrival, idle and footsteps before plants and effects. Empty-building bookkeeping and the empty-family8 countdown are also connected. Numeric radius4096 reveal applies ordered factor changes, four-cell dirty bits and eligible plant shade updates. Actors/effects refresh shade on their later physics paths. Creature AI, remaining-family callbacks, audio, palette and original GUI behavior still prevent calling this a complete startup or game tick. The isolated450-pass family4 receipts below freeze earlier callbacks; their timer700 is immediately after retirement, before later bookkeeping decrements it.

Validation:

- Factory integration:128 after-images,47,562,880 bytes,112 nonempty old heads and128 repeat-rejection rollbacks.
- Composed airborne callbacks:128 after-images,47,562,880 bytes and512 draws compared against an independently written model.
- Actual-map family4 regression:five explicit seeds,2,250 passes,1,425 particles created and retired,5,610 parent/factory draws. Contact occurs at pass189 and parent retirement at389. These players have no actors; separate owned-scene tests prepare and activate two actors.
- Numeric reveal:32 actual-map target cases and11,007,040 afterimage bytes match an independent ordered model. Native creature/actor records, players and RNG remain unchanged; repeated reveal is idempotent. Unknown virtual predicates reject atomically.
- Targeted tests cover mixed native/packed reference cleanup, singleton null-index preservation, slot reuse, late cleanup/factory rejection and rollback of motion, RNG, pools and links.

Receipts: `landing-world-validation.json`, `landing-step-validation.json`, `landing-scheduler-validation.json`; component evidence is `../ghidra/landing-evidence.json`. The five fixture seeds do not establish the original live seed or preceding shared-RNG history.

# Shared landing and effect updates

Observed implementation state: `ActorWorld` owns one map, native actor/effect pools, players and a shared RNG. Plant passes borrow that same storage. Numeric family4 remains separate from the complete inter-family scheduler. No original executable was run for these comparisons.

The code31 factory writes its native81-byte record, map links, pool count and player+496 type5 reference in one transaction. Audio and reveal are explicit pending requests. `step_landing_phase0` samples actual map height/shade/corners/terrain class, applies bounded vertical physics, and either creates a code6 smoke effect or invokes associated player activation at contact. Airborne smoke uses three parent draws plus one successful factory draw. Contact itself draws no random values; qualifying actor placement consumes its own recorded draws.

Physics constants come from hash-bound mapped tables. The mission loader replaces serialized water2000 with twice signed prefix1C63:1632 on CONQ001. Gravity remains signed prefix1C5D:1024. The host passes these runtime values explicitly. Clear map byte11 mask1 proves the bounded collision query's NULL result; water and unimplemented collision paths reject.

`step_effect6` implements fractional motion, first contact, repeated grounded motion with the second actual terrain sample, animation and damping. Moving between cells unlinks and reinserts the same native record. Kind5 virtual88 is a proved no-op. Retirement preserves the dead record's raw link bytes, clears matching reference kinds across all five source pools, preserves reference index words, applies the kind1 flag cleanup, clears the active kind and decrements the effect count. UI pointer cleanup remains typed state plus explicit pending requests. The native-empty UI value is a port policy, not recovered original GUI memory.

`step_landing_effect_callbacks` follows the first two4E950 scans. It checks live slots in ascending order, skips newborns in the primary scan, then clears each admitted newborn flag before its same-frame callback. A bounded second scan rejects an inconsistent creation count. Its queue list is only a candidate list:83140 draw admission and984C0 palette work remain presentation tasks. Unknown selectors and phases reject the entire pass.

Integrated phase1 currently covers the stationary interval from timer150 through50, consuming no RNG. Additional phase1 planner APIs are being developed separately and are not yet used by this world adapter. The next timer49 callback therefore remains a deliberate transactional rejection. Earlier kind1 AI, actor movement/animation, subsequent landing phases, audio, reveal, palette and original GUI behavior prevent calling this a complete startup or game tick.

Validation:

- Factory integration:128 after-images,47,562,880 bytes,112 nonempty old heads and128 repeat-rejection rollbacks.
- Composed airborne callbacks:128 after-images,47,562,880 bytes and512 draws compared against an independently written model.
- Actual-map family4 regression:five explicit seeds,1,445 passes,940 smoke effects created and retired,3,760 parent/factory draws. Contact occurs at pass189 for these frozen-map fixtures. These players have no actors, so contact exercises the activation counter's zero-slot branch; separate tests exercise two associated actors.
- Targeted tests cover mixed native/packed reference cleanup, singleton null-index preservation, slot reuse, late cleanup/factory rejection and rollback of motion, RNG, pools and links.

Receipts: `landing-world-validation.json`, `landing-step-validation.json`, `landing-scheduler-validation.json`; component evidence is `../ghidra/landing-evidence.json`. The five fixture seeds do not establish the original live seed or preceding shared-RNG history.

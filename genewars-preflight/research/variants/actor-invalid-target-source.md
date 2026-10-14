# Actor invalid-target repair

Reference: GENEWARS QA BETA V5, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.

**Passed:** captured fresh mission frame2636 route prefix matches the corrected Rust implementation across all200 actor bytes and4 working-target bytes. Only native vtable and blueprint pointers are normalized. Source interpretation executes6747 instructions from6FB60 through virtualB4 steering, stopping before70603 physics. No gameplay function is ported in this comparison. All other bound actor/creature/building/effect/family8/plant records, full map and players remain unchanged; RNG draws are zero.

The captured current point(21641,13431) and requested target(21375,13695) resolve to(21420,13650).74A50 queries the requested point then resolved point, both with74270 mask1. The nominated building is null in this fixture.

Source6FEB7 tests74D90. On rejection,6FEC0 passes nominated building on stack,6FEC1 passes current actor+7,6FEC4 passes requested actor+B,6FEC9 passes actor pointer,6FECB passes actor53 terrain mask.6FECE calls74A50 and6FED3 writes only packed actor+B. No idle-relocation5B/5C/57 side effects apply.74B94 uses mask1;74C13..1B permits null or nominated obstacle. Terrain mask and8B130 blocker weight below10 remain required. Normal ray/steering and49&F3 processing follow.

Production admission deliberately remains bounded to49&3==0. Existing-waypoint invalid-target variants remain unsupported because6FE63..8A may bypass repair when the current waypoint path is blocked. Do not broaden that branch from this fixture.

Evidence: `private/review/actor-invalid-target-source/b99a2bed68f1405296d2beb6ca508dea/result.json`. Initial world: `private/review/actor-target-repair/efb427501a8544a79b35c811181983ee/2636`. Corrected route capture: `private/review/actor-target-repair/92fd780c78a44c4b9530b36bcbb4970e`. Numeric repair separately produced identical results with stack fill0 and255 (private source runs4ca5624542eb45888d8d378f0abdbbb2 and7d79a1751f0a42428e5e9f793b1b488c), covering eight wide adjacent stack reads.

This is route-prefix acceptance, not a full architect callback or playable-mission acceptance. Reproduction: `C:/Python314/python.exe research/variants/actor-invalid-target-source.py <before2636> --route --expected-route <corrected-capture>`.

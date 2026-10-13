# Owned actor world: bounded integration

The Rust runtime now owns a twenty-slot native actor pool, four player records,
the explicit live RNG state, map bytes and non-actor common link headers.
`ActorWorld::from_mission` admits only the pinned CD CONQ001 source. It takes
native prior bytes and handle bindings, player records and live RNG explicitly.
These inputs do not establish the original startup timeline.

## Corroborated contracts

- `ActorPool` reproduces the inactive `57540` split copy: packed `[0,31)` to
  native `[0,31)` and packed `[31,140)` to native `[39,148)`. Both native gaps
  remain caller supplied. Active serialized actors require a different route
  and are rejected. Initialization commits verify complete before-images,
  recomputed category selection/allocation, after-images and typed handles.
- `ActorWorld` resolves kind3 references through the native pool. Other
  kinds resolve bounded family arrays and their common first31 bytes.
  Reachable lists must have consistent ID/type/previous links and unique
  membership. Terminal null next references retain their unused index bytes;
  a head's kind0 previous reference must identify its owning cell. An accepted
  insertion also changes the actual old head's previous reference.
- Both `80AE0` activation branches scan the five-slot player partition in
  order. Later slots see earlier map, actor and blueprint changes. Every
  candidate consumes one multiplicative RNG draw, one ring step and one
  wrapping draw-count increment. The source readiness prefix and updated RNG
  are visible to the read-only environmental query. Player+49D increments
  once after the scan, including when no actor qualifies.
- With no associated type5 reference, an actor is placed at the accepted
  candidate around player+9. With a resolved type5 reference at player+496,
  the actor stays at the associated object's position and receives a pending
  move order toward the accepted candidate around that object. Map insertion
  uses the actor's actual position. The association remains intact. Actor
  height and shade are preserved in both branches.
- Associated activation admits the closed `74A50` resolver outcome. A
  nonnull additional building/obstacle query requires the unimplemented
  backward search. Flag71 mask10 skips the move setter but still permits
  placement, linking and metadata changes. Other reinitialization routes
  reject the transaction.
- `dispatch_ground_move` checks the ground-control packet, group, pause,
  portrait and selected actor gates before applying the bounded opcode3
  setter. The target is already projected by the caller. Pending commands
  change destination/order fields; they do not move an actor immediately.
- The plant subsystem borrows the same backing and native actor pool through
  `PlantStorage`. Its traversals and unlink writes resolve kind3 directly in
  native records. `admit_plants` attaches hash-bound parameters without a
  reload; `tick_plants` commits the complete world only after validation.
  No packed actor mirror or second persistent map is maintained.
- Owned terrain, plant-blocker and bounded obstacle queries now replace
  caller values in the `*_owned` activation/command APIs. Terrain uses the
  corrected eight-neighbor fallback. Blockers honor native actor ignore bits,
  typed category1 plant exemptions and the packed/native plant flag mapping.
  Map byte11 mask01 clear proves the obstacle query returns NULL; set bits
  reject until building footprints are implemented. Kind5's pure predicate
  is admitted; unclosed family predicates remain explicit failures.
- `MinimapContext::project` implements874B0 for its actual88x89 control.
  It does not project ordinary terrain clicks; that producer calls7B550.

The source listings are hash pinned in `actor-activation-evidence.json` and
`move-order-evidence.json`. Relevant direct ranges are80B6B–80C65 for associated
position/order/linking,80CD8–80D21 for readiness/reference/batch count, and the
`74A50` bounded resolver. These are static executable findings, not observed
original gameplay.

## Observed automated validation

`scripts/validate_actor_world.py` executes the production library over the
verified mission with64 explicitly synthetic player/native-gap/RNG/environment
fixtures. An independent Python model compares192 complete activation/command/
plant after-images,68,195,520 bytes. Each session runs96 plant passes after its
order, totaling6,144 passes. The probe also checks128 rollback cases. Real
mission bytes are used for map/family provenance; the fixture seed is never
presented as the live landing seed.

Runtime unit tests cover every player partition, repeated same-cell insertion,
nonnull heads from all seven other families, null-index preservation,
malformed/duplicate references, later-slot rejection, counter overflow,
associated-object placement/order separation, no-op precedence and controller
gates. The associated planner has a separate independent validator and receipt
at `associated-activation-validation.json`.

Mixed-list tests retire plants at head/interior/tail around native actors,
preserve all nonlink actor fields and handles, and reject a later callback
without retaining an earlier retirement. Environment and minimap comparisons
are recorded separately in `research/variants/actor-environment-*.json`.
The16 hash-bound actor databases admit349 animations,1,982 frames and2,108
components; the typed actor render adapter validates bindings and frame bounds.

## Remaining integration boundary

The normal startup path retains the code31 landing object as the associated
type5 reference. Calling the no-associated route would bypass that behavior.
Neither activation API alone proves that landing has occurred. The newer
owned factory/physics/contact adapter now invokes associated activation from
the actual contact transition. Smoke creation, lifetime and reference cleanup
share the same map and RNG. Earlier kind1 AI and the complete shared update
schedule remain unfinished. See `landing-world-contract.md`. The explicit
environment APIs remain useful for independent fixtures; owned queries use
the current world state.

The native application continues to use its plant-only `MissionWorld`.
`ActorWorld` now hosts shared plant/actor operations, but native scene startup
has not yet switched to it.
Its packed backing alone is not a save file: native actors/effects and players
live in owned overlays. Resource handles resolve against16 exact databases.
Ordinary terrain projection and actor composition now pass independent
comparisons. The desktop adapter draws actors using explicit caller ordering;
the original actor queue, full movement/path search, complete ordered updates
and default scene integration remain unfinished. No complete playable game or
milestoneC is claimed.

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

The source listings are hash pinned in `actor-activation-evidence.json` and
`move-order-evidence.json`. Relevant direct ranges are80B6B–80C65 for associated
position/order/linking,80CD8–80D21 for readiness/reference/batch count, and the
`74A50` bounded resolver. These are static executable findings, not observed
original gameplay.

## Observed automated validation

`scripts/validate_actor_world.py` executes the production library over the
verified mission with64 explicitly synthetic player/native-gap/RNG/environment
fixtures. An independent Python model compares128 complete activation/command
after-images,45,463,680 bytes. The probe also checks128 rollback cases. Real
mission bytes are used for map/family provenance; the fixture seed is never
presented as the live landing seed.

Runtime unit tests cover every player partition, repeated same-cell insertion,
nonnull heads from all seven other families, null-index preservation,
malformed/duplicate references, later-slot rejection, counter overflow,
associated-object placement/order separation, no-op precedence and controller
gates. The associated planner has a separate independent validator and receipt
at `associated-activation-validation.json`.

## Remaining integration boundary

The normal startup path retains the code31 landing object as the associated
type5 reference. Calling the no-associated route would bypass that behavior.
Neither activation API proves that landing has occurred. Factory creation,
physics/contact, earlier effect/family callbacks and the shared live RNG
schedule remain to be connected. Environmental results are explicit evidence
inputs; world terrain/blocker/obstacle algorithms are not yet substituted.

The native application continues to use its plant-only `MissionWorld`.
`ActorWorld` is a separate owned stage, not a concurrently synchronized copy.
Its packed backing alone is not a save file: native actors and players live in
separate overlays. Resource handles identify exact table slots, but the host
has not yet resolved those handles into the16 source resources. Screen target
projection, full movement/path search, unified updates and rendering of these
actors remain unfinished. No complete playable game or milestoneC is claimed.

# Child20 impact integration boundary

Reference: verified CD candidate, executable SHA-256
`4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.
This is a bounded runtime component, not a playable combat mission.

## Observed source behavior and implementation

`projectile_impact::plan_neutral_mule_impact` implements the admitted
`4F6A0 → 5B9F0 → 5C7E0` impact sequence. It accepts an unlinked neutral
idle Mule, positive health, and a building-origin code20 effect. It returns
an immutable before/after plan, RNG transition, typed audio requests, and an
optional player kill-credit request. Unknown recipients, effects and invalid
RNG are explicit errors. No actor-world callback or firing path uses it yet.

Incoming damage1 is reduced by source armor arithmetic but floored to1.
The source cancels current movement/work references, subtracts health,
emits a randomized hurt sound, and applies reaction state18 with timer64.
When health reaches zero, the source first invokes mortality and credits
the effect owner's counter at player offset0x6FD. The caller still invokes
reaction afterward: final health0/state18 is intentional source behavior.
The planner does not substitute immediate state24 or classify this as a
mission outcome.

The planner preserves all map links and opaque pointers. Factory insertion,
the two initial spatial-query RNG draws, the final effect sound, player
counter application and atomic shared-world commit remain caller duties.
Its single-recipient input does not establish traversal order or authorize
skipping unsupported recipients in a multi-object query.

## Verification

Run `python -B research/variants/turret-projectile-impact-review.py`.
The validator snapshots the Rust source into an ignored private directory,
compiles that snapshot with overflow checks, and compares it with an
independent interpreter of the exact executable's listings.

The source fixture executes the complete code20 factory, randomized query,
damage, mortality when applicable, and reaction. Audio execution is an
explicit typed port. The comparison covers64 fixtures, **470,784 bytes**,
and **133,140 source instructions**: full124-byte recipients, all four
1805-byte player blocks, RNG state and ordered audio. Cases vary health
1/2/100/32767, skill and flag extremes, effect slots0–7, owners0–4, seeds
and ring slots; kill-counter wrapping is included. No uninitialized stack
read occurs. The probe also rejects unsupported linked work and recipient
states. See `turret-projectile-impact-review.json` for pins and scope.

## Remaining work

- Compose actual projectile physics, randomized proximity traversal,
  child20 factory and retirement in one transactional world update.
- Recover other recipient states/species and linked-work cancellation;
  prove multi-recipient traversal and pool-saturation behavior.
- Integrate health0/state18 continuation with the creature scheduler.
  A private source-only continuation (`private/review/turret-projectile/impact-continuation.py`)
  observed frame2 retaining18, frame3 entering24, and frame9 entering25,
  then ran through frame69. This is not an independent Rust continuation proof.
- Enable firing only after those applicable world paths are supported.

Separate private live42 proximity work currently verifies128 one-neutral-
candidate cases through actual8D7B0/8D930/4F9F0, including wrapped coordinates,
height/distance boundaries, two RNG draws and excluded states24/25. Its
physics, child allocation and unlink remain explicit ports; it is not a
full projectile proof. Latest receipt:
`private/review/turret-projectile/f60416f332b6437eabf8592e9f58e703/source.json`.

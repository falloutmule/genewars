The bounded landing path is code **0x31** in the81-byte numeric-family4 pool. `0x7F3B0` requests it at the player target, at terrain height plus0x6400, with angles0/0x400 and speed0x440. A free-slot factory plan preserves prior unknown bytes, initializes vertical velocity-1088, binds EFF.ANB animation49, links the object and saves `{ID,kind5}` at player+0x496. The first factory record, current pool count, terrain samples, prior bytes and runtime resource availability remain explicit inputs. Full-pool replacement is excluded.

EFF animation49 has one frame and hold0. Fresh animation state `[0,0,1,0,0]` stays unchanged through the preceding `0x7AC00` call; byte+0x1E must be supplied, since the factory does not initialize it. The physical callback `0x6DF30` runs first. Its airborne vertical candidate decreases physical height by136 under the fresh velocity/gravity gates, but collision, terrain-normal/bounce and water callbacks prevent treating that scalar recurrence as the complete physics routine. The contact plan therefore takes explicit post-physics state, preserving its angles and other fields.

Phase0 compares signed terrain height with physical height+0x27. On contact it snaps that height to terrain, clears all three velocity words, advances phase to1 and sets timer150, then requests `0x80AE0` for the owner. Child kind4 updates and follow behavior, stopping audio50 and playing audio51 remain ordered explicit effects. The saved player landing reference stays present.

That type5 reference changes activation behavior. Each eligible type3 actor is made ready by the actual `0x80AE0` masks, then candidate points are generated around the landing object. The actor's position becomes the **landing center**, while an internal move-order setter requests the accepted candidate as its destination. An additional building lookup is required by the move resolver. With that lookup null, accepted terrain/plant predicates, and the no-reinitialization gates, the candidate is returned unchanged. Actor insertion occurs at the center cell; blueprint counters and shared cell heads must be updated in slot order. Player+0x49D increments once after all five slots, even if none qualifies. The separate no-associated-reference plan is not this startup path.

Before contact, phase0 consumes exactly three direct RNG draws for an effect0x06 request. The receipt gives the angle formulas, source byte widths and pending factory/audio effects. It does not discard those effects or claim their transitive RNG footprint is already modeled. Seed provenance is also conditional: after the mission tail, runtime mode flags can replace the seed from saved clock state, current DOS time or synchronized input. Blueprint scratch bytes+0x18/+0x1B lack a proved writer. Those inputs cannot be selected from CD bytes alone.

Run `python -B scripts/ghidra/validate_landing_bridge.py` for16 listing pins, actual EFF animation metadata,32 factory and96 contact vectors, plus an independent newly authored Rust oracle covering8192 contact afterimages and all65536 three-draw RNG inputs. Raw resources, instructions and fixtures remain private. The machine-readable fields, call ordering and unresolved callbacks are in `landing-evidence.json`.

The production module now computes the fresh first-impact `6DF30` branch. It takes the actual four signed map height words around the impact point, uses `733A0`'s triangle choice and integer normalization, applies the actual `8B520` surface coefficient at `104530`, and computes orientation through the bound 257-word `AE134` table. `plan_landing_vertical_contact` admits the same fresh vertical landing gates as the airborne plan and preserves the resulting angles for phase0. Collision-object and water gates remain explicit environment proofs.

The code6 request has a transitive factory draw: successful free-pool allocation consumes one shared RNG draw with journal `21D`, sets animation delay to `1 + high % 3`, and sets vertical acceleration to `-(gravity >> 6)`. The null-parent landing route excludes the alternate code11-parent factory branch. Therefore a successful noncontact landing update consumes four draws across phase0 and the code6 factory. Allocation failure consumes only the three direct phase0 draws; full-pool replacement remains unsupported.

`plan_effect6_flight` computes fractional XY motion, airborne and first-impact physics, sampled ground/shade, bounce, speed and angles. `effect6_physics_candidate` identifies the exact environment-query point. A later grounded contact, water callback or nonnull collision query rejects transactionally. A map-cell change still needs the host's unlink/relink and virtual88 handling. These constraints must be checked on actual trajectories before claiming the full scheduler.

EFF animation6 has thirteen frames, flags0, and hold0 on every frame. `plan_effect6_animation_after_physics` implements its delay1..3 progression and retirement predicate; surviving updates apply signed `(15 * velocity) >> 4`. The code6 row's child selector is FF, so retirement's `4B3B0` returns zero without another factory or RNG draw. Retirement still requires `49D30` map unlink, `724C0` reference cleanup, clearing type and decrementing the family4 counter. The counter VA `11C2B4` corresponds to backing offset3554 from mission globals base118D60.

The validator now compares shipping APIs with 32 factory, 8,192 phase0, all65,536 three-draw seed, 1,024 first-contact, 1,024 code6 factory, 2,048 fractional code6 flight and156 animation cases. It runs the five landing module tests. The finite actual-map scheduler, preceding families and live seed selection remain separate integration work.
# Finite departure and uphill contact evidence

The source-backed production planners now cover phases0..3, code6 and code2F
factory/physics/animation, native retirement and family4 primary/newborn passes.
Standalone phase1, lifecycle, departure and particle validators compare shipping
APIs against independently written source models. Root's separate actual-map
composition receipt records five explicit seeds across450 passes each; it is
conditional evidence with other actor families frozen, not full startup parity.

The actual-map regression found an uphill impact with post-acceleration VZ9.
6E3C5..6E3E3 requests the unsupported damage callback only when
((signedVZ>>6)/16)>0. Small positive impacts remain admitted and bounce. A public
numeric fixture reproduces slot32's position, fractions, velocities, heights and
corner words without copying asset bytes. All five450-pass schedules pass after
this correction; all particles retire. Horizontal squared speeds use widened
math and reject negative wrapped native speeds before indexing trig tables.

Full pool exhaustion is conditional: source priorities are code6=27,31=1,2F=35.
The common factory can replace an unmarked lower-priority incumbent. A full pool
containing only6/31 has no eligible replacement for6 and returnsNULL before its
factory RNG draw; direct phase draws and audio still execute. A full pool with
unmarked2F cannot be treated as unconditionalNULL.

Observed code0x30 water splash now binds EFF animation48 (12 hold0 frames),
uses signed animation delay2 and immobile flag16&8, and consumes no RNG.
Source491CA requests audio3B; generic retirement clears reference kinds and
unlinks the map record. The independent water validator compares1024 prior
record factory cases and all3072 frame/signed-counter combinations.

Observed type5 virtual90 is1E600, returning row104EB9. Rows6/2F/30 return0;
the source skips splash factories and type1/3-only zero-height damage for
these particles. Their former water-height rejection was overrestrictive
and is removed; code31 retains its separate true-virtual90 water boundary.

# Backward target search

Observed static evidence: CD routine `74A50` starts at a requested world point and searches back toward the current position. It derives an angle from wrapped doubled signed coordinates, obtains the fixed-point step from the pinned trig tables, and queries candidates in order. The signed15 distance threshold4096 snaps the candidate to the current point. The snapped point is still queried and returned even if rejected. This preserves the source's query ordering, including the terminal case.

Implemented: `target_resolver::resolve_target` owns the numeric loop and accepts an explicit environmental predicate. `ActorWorld::apply_searched_move_order_owned` supplies the admitted kind3 obstacle, terrain-mask and plant-blocker queries; original setter no-op/reinitialization gates run before any search. An error commits no actor state. The original bounded resolver API remains available for its existing independent fixtures. The native terrain command now uses the searched API.

Independent validation interprets the pinned74A50 listing, including literal angle/table arithmetic, and compares the returned point and every queried point across160 scenarios:59,744 bytes. Environmental helper calls are replaced by explicit fixtures; their implementations remain separately validated. Immediate acceptance, several rejected candidates, persistent rejection, geometry-dependent acceptance, radius boundaries, signed wrapping and varying stack fills are included.

Two persistent-rejection scenarios did not terminate within4096 queries. The port returns an explicit limit error and matches the source's first4096 points in both cases; it does not substitute a destination or claim that the original loop terminated. The158 terminating scenarios execute742,399 literal instructions. The4096 limit is a port bound.

This closes target-point arithmetic within the admitted query domain. It does not by itself establish a walkable route, flight behavior, active-building collision queries or full pathfinding. Actor waypoint movement is separately tracked in `research/ghidra/actor-waypoint-evidence.json`.

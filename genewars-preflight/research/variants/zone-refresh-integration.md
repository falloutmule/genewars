# Terrain-job retirement and occupied-zone refresh

Shared `update_terrain_zones(create_markers)` preserves the original per-zone numeric79D50 loop. Initialization still creates each marker immediately after its zone. `refresh_terrain_zones` invokes the same loop without a factory call. Radius/count admission and transactional rollback remain unchanged.

`step_terrain_jobs` now performs the source79273 post-pass refresh exactly when at least one job retired, after all jobs, over positive-quota zones in source order. The old blanket active-zone/job rejection is removed. Ordered zone viewport cells append to the last retired receipt; no marker is recreated and no RNG is invented. The older explicitly-no-height API retains its narrower domain.

Literal79D50 occupied-marker2/state12 proof passes427,369 bytes,79,869 instructions and81 vertex requests withzero factorycalls. Full79220 retiring-job composition passes431,769 bytes and89,713 instructions, including complete map/entity/player/RNG/job afterimages. The2x2 job uses current map heights and a retiring counter; it is a source fixture, not gameplay acceptance. The generated border table is pinned in the adjacent receipt. Focused owned-world test proves late unsupported-zone rollback also restores the job pool.

Original bootstrap evidence remains untouched. These proofs reuse the exact extracted numeric loop and test real initialization before occupied refresh; they do not separately re-run the old bootstrap source comparison. Presentation cachelookup and viewport notifications are explicit ports; source state is retained. Source and fixture hashes live in the adjacent receipts.

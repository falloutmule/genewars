# Genepod construction admission

The shared construction path now admits source selector0 through its first completed grade0 Genepod. This is fixture/source validation, not an ordinary-command Krell acceptance run. Completed Genepod production remains explicitly unsupported and rolls back the entire attempted building pass. No creatures are spawned by this packet.

The original104D1E health row is280/600/1000; only grade0 health280 is admitted here.104DE0[0] gives footprint3;104E1C[0] gives construction base cost600, reduced by twice the architect's signed skill. Source43F60 has special completion branches for selectors2,7,11, but not0. Genepod uses the common completed-count/owned-bit update, supply/max-Genepod-grade recomputation and creature-target cleanup. Its incomplete451F0 branch has no selector-specific audio stop. Completed451F0 production is a separate branch and has not been replaced with a no-op.

Narrow source guards were extended in actor motion, pending-task setup dispatch, stage4 factory request, stage5 work, progress/completion, ordinary blocked-site cancellation and post-callback bookkeeping. Selector1-only public APIs retain their selector1 checks. A separate Genepod-or-selector1 insufficient-funds planner preserves source51E80 cleanup, including the larger3-by3 cell reservation /4-by4 linear point footprint for Genepod. Source stage5 finished cleanup leaves reservations unchanged; it does not reuse stage3 cleanup.

Evidence:

- `genepod-construction-world.json`:64 owned-world comparisons,26,226,752 bytes, real source factory records and BGENE incomplete animation; all four owners/loading modes, mismatched-resource rollback and repeated-completion rejection.
- `../ghidra/genepod-progress-evidence.json`:64 literal441C0/43F60 cases, including whole player and creature records.
- `../ghidra/genepod-work-evidence.json`:384 literal534D0 stage5 work/timer cases.
- `../ghidra/genepod-stage4-evidence.json`:192 literal stage4 factory-NULL/non-NULL and animation cases, explicit factory/progress ports.
- `../ghidra/genepod-finished-evidence.json` and `genepod-insufficient-evidence.json`:48 cases each, full actor/map afterimages; formatting/audio/queue remain explicit ports.
- `genepod-motion.json`:64 queued-motion/arrival-prefix cases.
- `../ghidra/genepod-arrival-evidence.json`:two full arrival→setup callbacks with empty/full terrain-job pools. Common physics/queue are ports; mapped-prior border input is a diagnostic fixture, not current native terrain initialization.
- `genepod-bookkeeping.json`:256 literal47F80 post-callback cases. The callback is explicitly a no-mutation port; this does not prove Genepod production.
- `genepod-completed-boundary.json`:actual owned-world completed Genepod pass returns typed `BuildingCallback(UnsupportedBuilding)` and leaves the entire world unchanged.

Reproduce planner proofs with `python -B scripts/desktop/validate_genepod_construction.py PHASE`, where PHASE is progress, work, stage4, finished, insufficient, motion, bookkeeping or arrival. The driver preserves each transformed fixture driver privately and records both original and transformed hashes; it does not replace the original instruction interpreter or old evidence. The world validator is `validate_genepod_construction_world.py`; the expected completed boundary is `validate_genepod_completed_boundary.py` and uses the existing debug runtime library without rebuilding.

One focused runtime build was used for the owned-world fixtures. Production file line endings were subsequently restored to their existing repository conventions without token changes; earlier fixture pins intentionally identify their compiled snapshot. No native launch, original-game execution, forced resources/objectives or campaign acceptance is claimed. Next required behavior is the source Genepod completed callback and creature production, followed by ordinary-command integration on Krell's actual path.

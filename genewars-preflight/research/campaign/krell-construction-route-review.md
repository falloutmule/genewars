# Krell construction route and apparent stall

**The Extractor is not shown stalled by funds or power.** Synthetic construction captures at frames 785, 787 and 820 all have health 187/280, but the Engineer's stage-5 work timer decreases **43 → 41 → 8**. Initial delay is 79, matching `110 - skill31`. The actor remains action 15, stage 5, flags `70=81`. At frame 820 the player has primary funds 1412 and secondary 0.

Source-backed `construction_work::plan_incomplete_work` / `534D0` decrements this timer each admitted working callback and requests building progress only when it reaches zero. Health need not change on intervening frames. Eight more successful actor callbacks should reach the next pulse; a blocked world transaction must not be counted as a completed callback. The observed 35-frame health plateau is consistent with the original work cycle. It is not evidence of a new construction bug. Stage-3 payment has already deducted 588; stage-5 work does not repeatedly require that payment. Completed-building power cannot be required to finish this incomplete Extractor.

## Required ordinary construction sequence

The **accepted** frozen004 Exceland→Krell report initializes Krell at progress 1 with funds **2000**, no buildings and construction choices **[11]**. The accepted Exceland winning report has funds 510, but those are not the next mission's spending balance.

`80590` restores carried player ranges `[0,4A1)` and `[61A,6F7)`. It preserves blueprints, research, species list and campaign state; it does not copy funds `4A3/4A7`, completed building counts `600..60D`, or owned selector bits `60E`. `7D57F..7D589` sets mission starting primary funds from multiplier20×100. Krell starts with zero building counts/owned bits. Consequently Exceland's completed infrastructure does not bypass Krell's build-menu dependencies.

For this ordinary menu route, **Extractor11 → Distributor3 → Genepod0 is required**: selector3 depends on completed selector11, and selector0 depends on completed selector3. The retained-owned-bit exception in `43960` is not available at this startup. This is the minimum recovered build chain to enable the briefing's Genepod/Mule actions, not a claim that constructing these buildings alone satisfies the zone objective.

**Photon Processor6 is not needed for the Genepod unlock or to permit production.** Source Genepod work can proceed unpowered on frames divisible by six; power increases cadence. A new Geneticist action is also unnecessary because Mule research entry4 is already nonzero. Additional infrastructure must be justified by a later ordinary action rather than copied from Exceland's objective.

## Budget and evidence limits

The current synthetic probe's Engineer skill31 gives source costs `base - 2*skill`: Extractor588, Distributor338, Genepod538, total1464. Starting at2000 leaves536 before income and grade-0 Mule attempts at75 each. This budget does not establish that seven Mules are required, nor that every attempt succeeds. It only shows that the chosen chain does not immediately exhaust the synthetic starting funds.

The accepted frozen004 report independently establishes the actual entry budget and initial menu. This review did not extract a raw accepted-carried blueprint to establish its exact skill; **do not relabel the synthetic skill31 record as accepted campaign state**. Subsequent gameplay acceptance must start from that accepted carry, including its actual blueprints and RNG.

Reproduce with `python -B scripts/ghidra/krell_construction_route_review.py`. Adjacent JSON pins the fixtures, frozen reports and implementation/source evidence. No Rust changes, Cargo, game execution or new mission acceptance occurred. The next action is to continue valid world callbacks through the pending eight work ticks, then finish the ordinary dependency chain; the current creature callback boundary remains separate from construction progress.

# Krell: minimum next-mission contract

Reference: verified **GENEWARS QA BETA V5** CD executable, SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Progress1 selects `HDRCO2.DAT` and `CONQ002.DAT`; the header names **Krell**. Both resources match the original disc inventory. Retail and other-variant equivalence remain unestablished.

Inventory comparison: the root/1 RIP's second header and world are byte-identical to the CD resources. The root/4 RIP differs in both; its world is375830 bytes versus343958 for the CD. Identical resource bytes corroborate the root/1 data only, not executable behavior. No root/4 objective or campaign equivalence is claimed, and its data is not mixed into this contract.

**Next player-visible capability:** after the existing Extractor/Distributor infrastructure, build a **Genepod**, create owned Mules and direct them into the marked zone. This paraphrases the original English briefing. Genepod is selector0, unlocked at progress1, requires a completed selector3 Distributor (or its original retained ownership-bit exception), and costs600 before the shared Engineer skill discount. This mission does not establish a requirement for combat.

## Initialization and continuity

The world starts with one participating player, four neutral Mules,25 plants and no serialized actors, buildings, effects or family6/7/8 entities. Source7D090 initializes funds from the mission's multiplier20, giving2000;801C0 supplies target `[1536,15360]`. Actor mask7 adds the category gated until progress1. The loaded seed is47117 and draw count33620.

These are serialized and source-composed inputs, not an accepted second-mission startup. Unlike fresh Exceland,7D090 skips blueprint generation at progress1 when the carried player's blueprint count is nonzero. Reusing the first-mission fresh-player generator would therefore alter campaign state and RNG consumption. The result-transition/cleanup work must preserve the required carried state before preparing Krell.

## Actual objective

The mission contains one zone at world position `[256,24320]`, radius1024, selector12, with initial quota1024. The generic7A280 zone scheduler waits for countdown0; selector12 dispatches793C0 with species4. That callback scans on frames divisible by64 and checks active creatures, mortality/state gates, and wrapped distance.

The species table maps the Mule's primary and secondary components to4. Their source contributions are16 and8 respectively, capped separately by remaining zone quota. Owner4 neutral creatures contribute one quarter of each capped amount.792C0 subtracts the credited amount from the quota;80780 adds it to player score `+0xD` and the separately capped approval field `+0x11`. This is a reusable zone-scoring rule, not a fixed number-of-Mules win condition. The primitive's24 literal fixtures cover both owner0 and neutral4, small quotas and both weights. Audio and visual feedback are explicit unresolved ports in that numeric proof.

In the eligible72ED0 branch, nonzero terrain-object count enables victory when player score `+0xD` reaches the loaded threshold1000. The shared exhausted-zone/highest-score fallback also remains applicable. Target-count objective is0. Existing capability/elimination conditions apply; an unsupported callback must remain an error rather than being converted to defeat.

## Smallest remaining work

1. Admit the second mission's reference-bound resources and carried-state startup without regenerating existing blueprints.
2. Recover and implement the Genepod production path and the minimum research prerequisite for Mule creation. The briefing alone does not define those numeric rules.
3. Supply owned-creature selection/movement through shared gameplay commands and the native input path.
4. Compose793C0/792C0/80780 with the actual owned world, including ordered effect/audio requests and zone exhaustion. The score predicate is already represented by the shared mission outcome evaluator.
5. Reuse the source-backed result flow and prove the first-to-second mission transition separately.

Reproduce metadata and the bounded scoring proof with `python -B scripts/ghidra/mission_contract_second.py`. Proprietary bytes/listings remain private; the public receipt records hashes and derived facts.

**Original campaign missions completable:0 / denominator not yet established.** Krell has neither automated nor native interactive acceptance; the Exceland-to-Krell transition is not yet accepted.

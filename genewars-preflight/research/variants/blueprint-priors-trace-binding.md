# Exceland blueprint18 qualification: minimal trace binding

This read-only audit does **not** increase the accepted mission count. Historical zero blueprint18 remains unestablished. Existing source invariant18=1B after activation is valid, but the winning trace lacks the observations needed to bind that invariant across both active specialists.

## What is narrower now

The actual frozen `fresh-photon-chain-003/report.json` reaches4630 steps, no blocker, Won, with two ready actors. It records final states1/1 and per-step world/RGB hashes; it does not record per-actor internal dispatch, restore source56, blueprint bytes, or setup calls. Final state1 cannot exclude a transient state10 or prior restoration.

Current construction-aware scheduler admits source-proved architect construction only in category0; ordinary callback admission permits states1/8, with a separate canceled-architect9 path. State5 restoration is currently unsupported. Idle target finding is admitted only after the owned-world `actor_idle_scorer_empty` gate. A successful run against these exact guards would exclude unsupported state5/10 and autonomous category2 setup. However this is current-code admission evidence, not a source-pinned history of the frozen accepted binary. It cannot be used as if an internal dispatch trace already existed.

Source52780 directly reads blueprint17 and16, not18/1B. The apparent52793 field18 is actor owner, not a C0 blueprint dereference. A found target can set pending work: category2 writes7E=1 at52948, then51960. Its next setup selects category2 branch5144C (state11), which dispatches55200 at5829E. That path is outside the bounded idle construction closure. Therefore a category2 actor must either have a recorded empty scorer or bring that action into a separate audit; mask5 alone is insufficient.

Source57A10 state5 restores55 from56 at58260..58263 and goes directly to animation tail582D9. It does not redispatch in that callback. A restored10 becomes relevant on the next callback; direct56530 count plus source-equivalent selected-dispatch recording closes this gap without reverse-engineering all state5 behavior. Source5828A invokes56530 only for state10 and signed7E<=1; state10 task>1 invokes56090 instead and should also be classified outside this narrow accepted path. State11/12 calls55200/54E80 likewise remain outside it.

## Smallest useful instrumentation packet

Add a bounded audit observer to the *shared* actor callback execution, not an extra gameplay implementation. It should identify build/resource/source hashes and mission runtime context, then record aggregated counts with a minimal first offending fixture:

1. At activation bind actor ID/blueprint handle/category/style and record18=1B. At every active callback entry and exit assert the same equality and record any write to either byte. Cover both active actors, not selected actor0 only.
2. Record actual source-equivalent dispatch **after** pending507D0 processing, plus the chosen subcallback. Count56530,56090,55200,54E80 and unsupported paths; all must be zero for this bounded acceptance. When state5 is seen record56 and its subsequent callback dispatch. Never infer absence solely from callback-entry55.
3. For every setup attempt record category,7E,82,83 and source branch; require category0 construction/cancellation or a separately audited branch. For category2 idle record each empty-scorer discharge and flag any nonempty result before assignment. Include style3/player610 charge-gate evidence if that branch is admitted.
4. Preserve the ordinary-command outcome, intermediate native observations and final frames. An all-successful observer run binds the established source invariants to that particular trace; it still does not establish original scratch bytes or DOS-save identity.

No4630-step replay is needed during this audit. Attach the observer to the next meaningful Exceland regression/native acceptance pass. A separately frozen replay varying explicit18 priors across all256 values would be stronger but is not the minimum next action and should not displace Krell implementation. An observer with only zero56530 count is insufficient if it silently permits unreviewed dispatches or misses setup transitions.

No new blueprint18 consumer was established inside the directly inspected52780/category0construction/periodic slices. That statement is not a whole-program read-set proof: common virtual/physics helpers and unadmitted restored/action branches remain covered only by their existing separate source evidence. The remaining concrete gap is **trace-to-audited-closure binding**, not historical-byte recovery.

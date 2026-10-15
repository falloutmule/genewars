# Blueprint scratch priors: bounded Exceland audit

Reference: verified QA BETA V5 CD executable. Source listing SHA-256 pins are in the adjacent JSON. This is a static source audit, not a historical observation of stack bytes or a new gameplay acceptance run.

**Historical zero at blueprint offsets `0x18` and `0x1B` remains unestablished.** Generator `7F580 -> 7FAD0` preserves these scratch bytes. Do not call the explicitly zero-filled reconstruction an exact byte reconstruction of that original stack.

## Proven bounded behavior

* Preparation `4FD30` and resource binding `56E20` use blueprint identity/style/category. `56E75..56EA0` reads blueprint `15/16`; similarly numbered actor fields are different storage, not evidence that blueprint priors were initialized.
* Activation overwrites blueprint `1B` with `18`: `80C65..80C6E`; the other activation path does the same at `8190E..81918`. Thus the original arbitrary `1B` does not survive activation. `18` remains opaque.
* Periodic architect recovery `5838C..583D1` compares signed `1B` against signed `18` at `58395..5839B`, taking `JGE` before state/frame conditions or the increment. Equality therefore skips the increment for every one of the 256 possible byte values, including negative signed values. This is a direct branch invariant, not evidence that the values were zero.
* Selected-actor text/panel `265F0` reads the name, category `16`, and bars `17`, `19`, `1A` (`26737`, `2675C`, `26781`). Portrait selector `56F10` reads style `15` (`56F21`) and category `16` (`57029`). These visible paths do not expose `18/1B`.
* Construction setup `507D0` reads skill `17` for work/payment (`50D4C`, `50E53`, `512FA`, `5139B`, `518BA`, `518E3`). Construction callback `534D0` uses skill `17` (`53A7A`, `53E56`, `53FDD`, `5408A`). Its apparent `1B` writes at `53B53/54526` address animation/building state, not an actor-C0 blueprint dereference. No blueprint `18/1B` dependency was found in these bounded construction paths.

## Concrete limit, not a blanket harmlessness claim

`56530` reads **signed blueprint `18`** at `56A91..56A9A` and uses it in a threshold predicate at `56AA4..56AA6`. This is a genuine gameplay input. The architect dispatch `57F97..57FA9` indexes state minus one; state 10 reaches `5827B`, which can call `56530` at `5828A`. Therefore arbitrary actions, later missions, and full save-byte fidelity are not covered by the construction-path invariant.

The smallest remaining acceptance proof is to bind this audit to the accepted Exceland action trace: demonstrate that no relevant actor enters this dependent action (or another unreviewed consumer), and that `18 == 1B` remains true after activation. A replay with varied explicit priors, comparing ordinary-command outcomes and visible frames while excluding the documented opaque bytes from byte equality, would close this boundary more strongly. That replay was **not run** here. Do not upgrade this bounded audit to a complete first-mission invariance claim without that trace binding.

### Exact trace location

Checking `actor55 != 10` only at callback entry is **insufficient**. `57F62..57F8D` can call `507D0` before dispatch; that routine can write `actor55 = 10` at `5173B`. The source then reads the changed state at `57F97`. Observe the state at that internal dispatch, or directly count invocations of the reconstructed `56530` subcallback. Its exact gate is state 10 and signed `actor7E <= 1` (`5827B..5828A`). State 5 can restore `55` from `56` at `58263`, but immediately jumps to `582D9`, so it does not redispatch during that callback. The exported instruction corpus contains no other direct `CALL 56530` beyond this site (duplicate exports excluded); this is not a whole-binary indirect-call audit.

For the accepted trace, record activation identity and equality `blueprint18 == blueprint1B`, check equality before/after active callbacks, and record the internal selected subcallback. A zero `56530` invocation count excludes the identified numerical dependency even when transient actor states differ from callback boundaries. Equality at boundaries is sufficient only alongside the audited callback write paths; it is not a general argument against an unobserved transient mutation in an unknown callback.

### Construction route excludes the transient setup branch

The source setup dispatch at `50827..5083D` indexes **blueprint category `16`**, not task `82` or building selector `83`. The four little-endian pointers at original address `507BC` are `[5122F,51738,5144C,5177A]` for categories 0,1,2,3 respectively. Category0 enters `5122F` and unconditionally writes state9 at `51235`; category1 enters `51738` and writes state10 at `5173B`; categories2/3 write states11/12. Category1 then branches on `7E` (0,1,2) at `51743..51767`; its state10 write precedes that task dispatch. Thus no construction-selector value itself selects the state10 branch.

For the accepted architect construction packet, category0, `7E=0`, `82=1`, and selectors11/3/6 take `51235 ->5125A ->509E4`. Valid construction, blocked-site cancellation, and the already-proved route repair do not change blueprint category and cannot redirect this category dispatch to `51738`. The special `7E=14` path bypasses the category dispatch at `507F7..507FA` into entity-use state7 paths; it does not select state10. This establishes **path independence from blueprint18 for these architect setup paths**, combined with the construction/periodic audit above. It does not establish historical scratch bytes or saved-record identity.

For a whole-mission claim, other activated categories still matter. Source idle `52780` can set pending `actor57` bit1 at `528F0` after finding an entity and can call task assignment `51960` at `5292C`; merely listing the player's construction/move commands does not rule out autonomous pending work. The minimal acceptance binding is therefore either (a) evidence that every setup call in the accepted trajectory has category0 or the proven special path, plus no state10 restoration, or (b) the earlier internal-dispatch/subcallback observation for all active actors. An architect-only construction proof should not be extended silently to every specialist.

## RNG diagnostic ring

In the audited generator, the random value comes from seed `11878A`, with the multiply/store at `7FDF4..7FDFA` and extraction at `7FE02..7FE0D`. Ring index `11872C` advances at `7FE1F..7FE32`; `7FE38 AND ECX,FF` removes the ring-containing high byte before the difficulty calculation. Thus this generator's numerical output is independent of its diagnostic ring index. Ring/log memory identity and a complete serialized-state comparison are separate obligations; this bounded finding does not establish that every original binary consumer ignores the ring.

No production edits, original execution, or broad exports were performed for this audit.

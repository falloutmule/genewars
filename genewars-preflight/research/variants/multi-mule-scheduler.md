# Multiple neutral Mules: bounded scheduling contract

Reference: QA BETA V5 CD. This source audit supports generalizing creature identity, not admitting unproved interactions or reproduction. Adjacent JSON pins the original listing and reviewed implementation snapshot. No production changes or new execution tests were performed.

## Source 6A540

1. `6A545` loads signed live count from `11C2B0` **once** into ECX. `6A554..6A56B` clears participating player counters at `+616` using active-player global `11878C`.
2. Scan native slots ascending from `1542BE`, stride `7C`, exclusive bound `15D40E` (`6A56D..6A58C`). Stop when initial remaining count reaches zero or the bound is reached. Inactive records (`+6 == 0`) and newborn-marked records (`+15 & 1`) are skipped without decrement (`6A58E..6A5A2`).
3. Call each eligible record's virtual `68`; if its result is false and owner `+18 != 4`, increment that owner's `+616` (`6A5A4..6A5D5`). Neutral owner 4 skips the increment regardless of predicate result.
4. Call virtual `64` with EAX equal to the **current native record pointer** (`6A5DB..6A5E0`). Zero-extend returned AL and add to EBX; decrement ECX once per invoked callback (`6A5E3..6A5E9`). The count is not reloaded when a callback retires a creature or changes world count. Each subsequent callback sees prior callbacks' world and RNG mutations.
5. If accumulated EBX is nonzero, rescan ascending from the pool base. Find active records with newborn bit set, clear that bit, decrement EBX and invoke virtual `64` (`6A5F1..6A627`). Their callback returns do not accumulate another pass. The literal second scan lacks an explicit pool-end check: a bounded port should report inconsistency rather than walk outside memory if callback birth accounting is invalid.
6. Eight `7AC00` UI-animation calls follow in source order (`6A629..6A6BF`), including when the pool is empty.

For four existing neutral Mules with no newborns/creation, the minimum generalization is the ascending primary pass, retaining initial count and callback identity, with a whole-pass transaction. A precomputed active list is equivalent only if no callback changes eligibility of another slot or creates a record. Prefer inspecting current staged records at each scanned slot. Births must remain explicitly unsupported until their factory and second pass are implemented; do not silently discard a nonzero source callback return.

## Slot-zero assumptions to remove together

* `family_scheduler.rs`: `active > 1`, `slot != 0`, callback closure guard, one-callback branch, receipt slot and portrait identity.
* `creature_updates.rs`: record/binding/replace index 0 and rejection of every other active slot; route own-reference, common target acceptance, contact/waypoint source record, food-search self exclusion, cell-entry and neighborhood self exclusion, `initial_mule_accepts` source record and selected-plant-cell exemption. Each must receive the actual `[slot_lo, slot_hi, 1]` reference.
* Map migration: `unlink_mule` must validate/update the actual entity's previous and next references; `insert_mule_head` must replace the actual slot and put its reference in the map head and old head's previous link. Do not substitute slot 0 or infer the current identity from an unrelated nearby entity.
* `creature_retirement.rs`: stale-record check, linked identity, record replacement, unlink, cleanup reference, retirement slot, final readback and module cleanup request all hardcode 0. Preserve the record's source ID `+0F` for UI list cleanup; do not rewrite it to make a fixture pass.
* `creature_world.rs` numeric planners mostly take a record directly; common physics already uses source `+0F` in frame parity. Preserve that ID. Audit helpers called by the generalized adapter, rather than replacing all literal zeroes indiscriminately.

## Interactions remain a separate boundary

Existing cell-entry, neighborhood and food-search guards reject another Mule. Changing only their self-reference to the current slot is correct; removing the other-creature rejection is not proved. Multiple distant Mules can be admitted while encounters still return typed unsupported and roll back. The target-acceptance helper already treats type-1 entities separately from plant blockers, but this does not prove the callback interaction/event-search branches. Shared food reservations must remain visible in ascending callback order; no parallel per-creature clone commits or independently reset RNG.

## Smallest next proof

Run literal `6A540` with controlled virtual-call ports and compare ordered pointer identities, per-player counters, newborn-bit changes and UI requests against the scheduler. Include sparse nonzero slots, four neutrals, current-slot retirement, initially marked newborns and a callback producing one newborn; unsupported newborn cases must roll back explicitly in the bounded production path. Then replay the actual CONQ002 startup once, comparing each supported Mule callback and full linked map/RNG after the pass. Keep the first actual proximity/food/retirement failure as a fixture. This audit does not claim those new execution comparisons have passed.

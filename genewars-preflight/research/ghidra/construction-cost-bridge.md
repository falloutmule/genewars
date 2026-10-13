# Construction stage-three resource debit

This is a bounded numeric transaction from the exact CD candidate SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. It is suitable for a pure simulation helper after the caller establishes the state and callback prerequisite. It does not implement the entire construction command, movement, reservations, animation, cancellation, or building creation.

## Trigger and branch evidence

The entity update path tests pending bit 0 at entity+`0x57` at `0x57F72`, clears it at `0x57F7C`, and calls initializer `0x507D0` at `0x57F8D`. For profile byte `[entity+0xC0]+0x16 = 0`, initializer jump-table entry zero selects `0x5122F`, which sets dispatch byte entity+`0x55` to 9 at `0x51235`. Dispatch `0x57FA9`, table `0x579AC`, index 8 selects `0x58271`, which calls `0x534D0` at `0x58274`.

Within `0x534D0`, mode byte entity+`0x7E = 0` selects the stage switch. The unsigned 16-bit stage is entity+`0x88` (task record entity+`0x82`, field+6). Stage 3 selects `0x53A33` through table `0x534B0`. Earlier instruction `0x5397C` writes stage 3. The stage-three branch first calls vtable slot+`0xAC`, with the vtable pointer stored at entity+`0x23`; only nonzero AL proceeds to the debit. Callback meaning, physical input binding, and scheduling cadence are unresolved.

If the callback returns zero, the original follows `0x53982` and calls general update helper `0x526C0(entity,0)`. A pure helper may return `NotReady` without changing its pools or stage; that return does not reproduce the helper's other entity side effects.

## Exact numeric contract

Use a safely admitted selector 0 through 13, signed-byte skill, two signed 32-bit pools, a Boolean derived from a nonzero original flag byte, and callback readiness. Reject selectors outside 0 through 13 before indexing. The original debit branch sign-extends the selector byte and has no local array bounds guard; this safe admission is deliberate.

| Field | Original access |
|---|---|
| Owner | Unsigned byte entity+`0x18` |
| Selector | Signed byte entity+`0x83` |
| Skill | Signed byte `[entity+0xC0]+0x17` |
| Resource record | `0x119215 + owner*0x70D` |
| Primary pool | Signed 32-bit resource+2 |
| Secondary pool | Signed 32-bit resource+6 |
| Refill flag | Byte `0x118D76 + unsigned(resource[0])*0x70D`, tested for nonzero |
| Stage | Unsigned 16-bit entity+`0x88` |

The fourteen signed 32-bit base costs at `0x104E1C` are, in selector order:

`600, 750, 400, 400, 300, 600, 250, 1000, 300, 500, 900, 650, 600, 300`.

At `0x53A77..0x53A8A`, cost is `base_cost - 2*skill`. All arithmetic below wraps at 32 bits and comparisons interpret the result as signed. There is no clamp on adjusted cost or balances.

1. If signed `wrapping(primary + secondary) < cost`, leave both pools unchanged in the debit branch and return insufficient. The original sets entity+`0x7A = 8`, calls `0x51E80(entity,1)`, and returns; cancellation side effects remain outside this helper.
2. Compute signed `remainder = wrapping(cost - primary)`.
3. If remainder is nonpositive, set primary to `wrapping(-remainder)` and retain secondary. Otherwise set secondary to `wrapping(-wrapping(remainder - secondary))` and primary to zero.
4. If the refill flag is nonzero and signed `wrapping(new_primary + new_secondary) <= 1000`, set secondary to 1000 while retaining primary.
5. Return the updated pools and stage 4, matching the 16-bit store at `0x53B09`.

The refill rule sets the second pool to 1000, not the combined total. For base cost 600, skill 0, primary 700, secondary 0, it produces `(100,1000)`. At primary 1600 it produces `(1000,1000)`; at primary 1601 it produces `(1001,0)`. The flag's gameplay meaning is unresolved. The original uses resource byte 0 to find this flag, so equality between that byte and the entity owner must not be assumed without further evidence.

For selector 6 and skill 127 the cost is -4; zero pools become `(4,0)`. For primary `i32::MAX`, secondary 1, and cost 600, the wrapped sum is negative and the branch rejects payment. These cases constrain faithful integer behavior even though their live gameplay reachability is unknown.

## Validation and scope

Run `python -B scripts/ghidra/validate_construction_bridge.py` from the project directory. It verifies the candidate hash, mapped cost bytes, three jump-table links, four private instruction listings, all 3,584 selector/skill cost combinations, and 18 transaction vectors including signed overflow, negative adjusted cost, threshold refill, insufficient resources, and callback not-ready. `construction-evidence.json` contains safe vectors, hashes, addresses, and exclusions. These checks use static evidence and a synthetic behavior model; they do not execute original game code.

Raw instruction/decompiler exports remain under `private/ghidra/construction-*` and `private/ghidra/building-cost-refs`. Original temporary memory writes before NEG are collapsed to final values in the pure transaction. Do not infer building names, skill gameplay range, pool invariants, callback meaning, refill flag meaning, or complete world mutations from this contract. The manual's construction/resource questions guided discovery but do not establish these machine-level semantics.

# Mission countdown expiry: bounded implementation

The shared native route currently rejects countdown expiry and rolls back the frame. The longer combined probe reaches that boundary at step 6,001 for seeds 0 and 65,535. Default seed 5,428 first reaches a creature retry-reset branch at step 5,677. No expiry has been committed in the application.

## Observed source and independent checks

The provisional CD source function `0x13780` decrements the timer, then visits the configured players when it reaches zero. Byte `0x6E3` gates each request. A positive signed byte at `0x4A2` selects the position at `0x5FC`; otherwise it uses the position at `9`. Literal `0x7B140` samples ground height. The request to `0x101D0` uses selector 0, the player index, this position, and height plus 25,600. The standalone Rust caller planner matches 80 cases and 880 request bytes, using 93,768 interpreted instructions across three stack fills. Factory and audio/UI bodies are explicit ports in this caller test.

The separate family8 creation planner matches the complete literal `0x101D0` factory and its `0x1EE80`, `0x1EA90`, `0x10060`, height/shade and link helpers. The 48 comparisons cover all three selectors, four owner inputs, slots 0/1/7/15, signed position/initial-height boundaries, and empty, plant and Mule cell heads: 5,232 bytes and 26,220 interpreted instructions. Typed animation offsets replace machine pointers; pointer slots are normalized in the comparison. Source table `0x112D14` is the zero initial motion vector. Selector 2 forces record owner 4 and preserves its input owner in byte `0x58`.

These are independently written, unexported numeric planners. They do not allocate or own the family8 pool, commit map links, run same-frame callbacks, load its animation resources, consume presentation requests, or enable countdown expiry. Validation uses the existing literal instruction interpreter on private disassembly, never execution of the original program.

## Next integration requirements

- Admit all 16 native 102-byte records and their typed animation bindings while preserving packed mission provenance and counts.
- Compose ordered factory calls and shared-map insertion with complete frame rollback.
- Implement source `0x118E0` callbacks, including the same-frame pass over newly created records. Its selector-0 state 4 can create selector-1 children after the grounded predicate becomes true.
- Bind the resource table at `0x116820` and its source rendering path; numeric creation alone is insufficient for a rendered world step.
- Keep audio, narration and UI requests explicit until their consumers are implemented.

Receipts: `countdown-expiry-validation.json` and `family8-creation-validation.json`. Reproduce with `python -B scripts/validate_countdown_expiry.py` and `python -B scripts/validate_family8_creation.py`. The planners remain separate from the legacy empty-pool countdown gate.

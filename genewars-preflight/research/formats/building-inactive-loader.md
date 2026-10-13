# Inactive building-record loader proof

`python -B scripts/validate_building_inactive_loader.py` interprets the pinned
`0x45010` and `0x6D490` listings and compares every output byte for 1,024
patterned synthetic records and all 256 family-3 records in the hash-bound
CD `CONQ001.DAT` input. The compact, source-pinned result is
[`building-inactive-loader-proof.json`](building-inactive-loader-proof.json).

The admitted packed record is 59 bytes and the native record is 67 bytes.
`0x6D490` copies packed bytes `[0,31)` to native `[0,31)`. The outer
`0x45010` routine copies packed `[31,59)` to native `[39,67)`. Native prior
bytes `[31,39)` remain untouched. The prior record's other bytes are replaced
by the packed source, including its type byte; a nonzero prior type is not an
admission constraint.

`0x6D490` then changes its mutable staging input at packed `+0x15` by masking
with `0xE3`, and sets packed `+0x17` to `0x20`. Those mutations occur after the
31-byte copy, so native header bytes retain the pre-mutation source values.
All 256 actual packed records had type zero. For these real rows, the `+0x15`
mask write did not change the byte value, while the `+0x17` assignment changed
all 256 staging records. Synthetic cases exercise both mutations changing.

Active packed records reach a virtual initializer at `0x45048`, so the
bounded inactive loader API should reject them instead of pretending the
inactive copy path handles them. The typed pool invariant also rejects an
inactive record carrying a stale animation binding. This is an admission
boundary; it is not a claim that the original routine itself rejects active
records. The constructor at `0x1E3F0` only writes the vtable at native `+0x23`
and is separate from this inactive-copy proof.

The string-copy interpreter assumes the caller has direction flag clear;
these routines do not execute `CLD`. It executes only the pinned listing
opcodes and models `MOVSD.REP`/`MOVSB.REP` directly. It does not run the game
executable, interpret active virtual constructors, or write private record
bytes to the public receipt.

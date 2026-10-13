# Custom-screen list storage bounds

The provisional CD candidate's `0x7E0B0` initializer zeroes 0x8CA bytes at state+0x507, exactly ten 225-byte records, then clears count byte state+0xDD1. Draw callback `0x7E9C0` calls list maintainer `0x7EF30` with that storage base. The maintainer accepts a new incoming record only while count<10, walks existing records at stride0xE1, increments count on insertion, and decrements count when removing expired entries. This establishes a bounded ten-record storage region independently of the wrapped selection arithmetic.

Names begin at record+0xE, matching selection's state+0x515+selected*225. The maintainer copies 40 incoming header bytes to record+8 and 88 following bytes to record+0x30. Consequently a conservative safe name window is record[0xE:0x30], 34 bytes; require a NUL within that window and validate encoding/destination capacity. The original code does not establish that every incoming name is terminated. Do not reproduce its unbounded string comparison or copy.

A safe caller may accept a selected index only if count<=10 and selected<count. The offset increment/decrement primitive remains byte-accurate, including wrap; these storage checks occur separately. The original display loop checks displayed row count against count and six, without subtracting the starting offset. That unchecked traversal is excluded. Full incoming-record semantics, outer object allocation, and maintainer timing are not reconstructed.

`list-layout-evidence.json` records exact addresses, four private listing hashes and 2,816 synthetic count/index checks. Reproduce with `python -B scripts/ghidra/validate_list_layout.py`. These are static instruction/storage contracts and safe port bounds, not an original execution trace.

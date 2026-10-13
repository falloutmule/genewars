# Bounded neutral creature scheduling

Observed static evidence: CD routine `6A540` resets a dword at each current player's offset `616`, traverses the native creature pool, invokes virtual `68` before virtual `64`, handles newborn callbacks, then updates eight shared UI animations. The Mule's virtual `68` is the pure `state >= 24` predicate at `1EC50`; neutral owner4 bypasses the player-counter increment for either result.

Implemented admission: zero or one active creature, coherent declared count, and only unmarked slot0 with owner4 and the hash-bound Mule selector4. Other owners, slots, newborn marks and inconsistent counts reject. The callback, player reset, native records, links and RNG commit as one cloned-world transaction. The eight UI updates are returned as ordered resource/global descriptors and remain pending. This is not a general creature scheduler or a complete original frame.

Independent evidence: `validate_family_scheduler.py` interprets the pinned source listing and compares12 first-pass cases (four seeds and player counts0,1,4), all four player records, native Mule record, map, RNG and UI request descriptors:2,841,684 bytes and137,528 literal instructions. Only the eight scheduler UI callsites are deferred; the same animation helper remains interpreted inside the Mule callback. Resource/vtable pointer bytes are normalized to the owned typed bindings. Separate Rust tests cover partial-player reset, hidden kind6/7 members, inconsistent counts and rollback after a callback changes RNG/backing and fails. Sol03 reviewed the scheduler and oracle against the listing.

Source `14EC0` scans all32 kind7 records. Source `4F580` with count0 has no callbacks. The native route now explicitly requires empty activity bytes in both families and a zero kind6 count, rather than assuming they remain empty.

The current main native route still freezes creature AI. The combined integration probe uses this scheduler while hungry target acquisition and subsequent approach are being admitted. Source frame order, caller seed, zero native priors, draw order and pending presentation requests remain separate from a claim of original startup or gameplay parity.

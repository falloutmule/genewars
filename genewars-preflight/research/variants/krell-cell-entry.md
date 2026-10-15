# Krell frame8 cell entry

Actual blocked callback was slot1; slot0 already passed source comparison. Captured005 slot1 now matches all427,369 bytes across8buffers,3201 literal instructions and1 RNG draw. Receipt: `private/review/krell-mule-callback/25946367f3184fe3a7715e1dc13c9b8a/result.json`. This closes the captured blocker, not unsupported states or later cells.

Source common physics6E791..6E7BD detects the new cell, unlinks8ACC0, prepends8AC60, then invokes virtual88=5B260 with the new cell. Slot1 position becomes[31988,22528]. The querying creature is first in the new cell chain.8B780 traverses the entire chain, counts living kind1 records using virtual68, and returns the **last** living kind1.5B100 state8 at5B242..5B252 tests last!=self; if false it then evaluates8B7D0. Ownership does not affect this predicate. State1/state10 branches remain separate.

Virtual10 is mapped111FEC+10 to1EE20: exactly `(word49 & 0400)!=0`.5B260 skips when true; otherwise invokes virtual8C=5B100. False returns without RNG or mutation. True consumes one RNG draw; remainder2 chooses+256 when nonzero,-256 otherwise. Starting with signed heading4B, each of at most8 attempts advances by0x800+step and masks7FF.73F70 produces the candidate from current position, heading and radius256.74D90 receives DL=record53,ECX=self,EBX=associatedbuilding4D only when kind4, otherwise null. Stop at the first accepted point. If all fail, retain the eighth point instead of raising target-search failure.

Write candidate to45. Compare original goal B cell against supplied new cell. If equal:49low|=1,54=80,4A&=EF. Otherwise49word|=1001. B itself stays unchanged. Continue common physics and the ordinary callback. Atomic integration includes source-ordered map unlink/prepend, waypoint/flags,RNG and later health/lifecycle/timer changes.

Candidate query is74D90 **alone** at5B38A. Existing mule_accepts is usable under state8/53==1/4F==0; do not add74270 through common_move_point_accepted. Captured input has53=1,49=0010,4F=0.

73F70 doubles signed16 coordinates. Index=heading<512?heading+1536:heading-512. Double radius; X delta=(mapped1086E8[index]*unsignedlow16doubledradius)>>16; Y uses107EE8. Add to doubled coordinate with low16 wrapping, arithmetic-shift signed16 by1, mask7FFF. No floating-point approximation.

Read-only Ghidra export `private/ghidra/krell-cell-entry/instructions-00073f70.txt` covers73F70..7406A without new analysis or original execution. Actual source RNG29645/ring1/count33668 becomes35081/ring2/count33669; first candidate accepted, waypoint45=[31782,22375],49word=1011, two map bytes change. Initial source expected is retained at `private/review/krell-mule-callback/1f57ed6db7d44d71b7135c7e8af3275a`. Render enqueue and UI services remain explicit ports declared by the callback oracle. Listing hashes and production receipt are pinned in `krell-cell-entry.json`.

# Krell cleared food target at frame 587

The actual frozen frame587 slot1 target reference is zero after another Mule consumes its plant. This is an original supported callback input, not a reference to plant slot0. Literal full callback succeeds: 1,638 instructions, no random draws, 427,369-byte afterimage generated. It retains state1 and null target; waypoint timer54 decrements50 to49. Production comparison is not yet run.

Source phase is frame+signedID=588, so the health/AI body is skipped. Existing waypoint-wait motion still executes. Admission must not eagerly resolve the null food reference before this phase distinction.

An isolated sequential callback at frame588 (phase589) resolves target at6574F/65752, stores null at65757, branches6575F to65861, then65869 to65F04. Actual flags60=C8 has bit40, selecting66080: call5CC60 reset, draw once at66087..660FA, set state8 at66100, compute timer=random%20+10 at66104. Virtual8C at66112 checks the saved current cell. On this fixture it returns false, so6611A reaches65C57 and stores timer28. Flags60 become08; waypoint timer54 becomes48. The alternative true branch at66120 onward requires additional relocation and must not be silently admitted by this result.

The second callback executes2,407 literal instructions and one random draw. It is an isolated slot continuation, not a complete scheduler frame: other actors and callbacks between frames were not replayed. Both receipts include source/input hashes and existing explicit UI/render ports. Original runtime observation and native comparison remain not run. This contract supports a narrow null-target phase admission plus the recovered reset path; it does not support treating every absent or invalid target as harmless.


## Pure reset boundary

Private boundary records are saved in `private/review/krell-null-food-reset-boundary-001`, at source66080 (before reset),66112 (before cell predicate),65C5B (after timer store). These derive from the isolated next callback, not a new gameplay acceptance fixture. The reset timer is **29**, followed by normal callback-tail decrement to28. Do not use final callback timer28 as the pure reset output.

For the narrow state1/null-target, flags60 bit40 set and bit2 clear domain,5C920 takes its immediate no-action exit (5C929..5C93B).5CC60 then follows state1 null-plant cleanup without touching a plant: clear15 bit20; zero words33 and35; mask word60 withD71A; clear5D when original60 bit2 is absent; clear target kind57; zero timer6C.66080 continuation draws once, sets state8, calls5B100 on the saved current cell, and false result stores random%20+10 in6C. The actual pure delta is5E:1→8,60:C8→08,6C:0x7FB0→29. No arrival predicate guards the null-target branch. Parent AI parity is explicit5EE28..5EE32 frame lowbyte+ID lowbyte, odd enters5EED5; even skips this AI.

A pure helper may share reset arithmetic with existing retry-wait logic but must have a distinct source entry guard: retry-wait's state8 and49 bit100 requirements are not prerequisites of this state1 null-food branch. True current-cell predicate, action-association60 bit2, and non-null target variants remain separate required behavior.

Actual frame587 production comparison passed against capture016: all eight buffers, 427,369 bytes, 1,638 literal instructions, zero RNG draws. Receipt krell-null-food-target-native.json. This closes the even-phase null-target callback only; odd-phase reset remains separately pending production comparison.

Actual frame588 production comparison now passed against capture017: all eight buffers, 427,369 bytes, 2,407 literal instructions, one RNG draw. Receipt krell-null-food-reset-native.json. This validates the integrated odd-phase reset ordering on the actual gameplay snapshot, including the existing lifecycle/timer tail; the earlier isolated continuation remains separately labeled.

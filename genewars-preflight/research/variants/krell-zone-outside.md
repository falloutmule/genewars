# Krell terrain-zone no-change admission

Reference: verified QA BETA V5 CD executable, SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.

`ActorWorld::step_admitted_zone_callbacks(source_frame)` now admits the bounded source 7A280/793C0 paths that leave the world unchanged. Countdown and empty-list returns retain their original behavior. Selector 12 requires positive original/current quotas, a bounded nonnegative radius, consistent active creature count, and supported Mule records. Nonpositive signed zone byte 18 skips processing. With positive occupancy, frames not divisible by 64 skip the scan; scan frames are admitted only when no eligible live Mule lies within the wrapped radius. An eligible in-range Mule, exhaustion, or unsupported metadata remains an explicit error without mutation. No score, effect, or RNG behavior has been substituted.

## Literal evidence

`krell-zone-outside-review.json` records three literal-source comparisons, totaling 1,282,107 bytes across backing, native pools, players, and RNG. All compared unchanged. There were no RNG draws or external ports.

- Actual `private/review/krell-after-landing-001`, frame 256: countdown gate, 21 instructions.
- Labeled synthetic `private/review/krell-zone-outside-synthetic-001`, frame 1000: countdown set to zero and occupancy set to one; original startup positions retained. Non-scan gate, 77 instructions.
- Same synthetic fixture, frame 1024: all four Mules outside the zone; full scan, 331 instructions.

These prove literal source no-change behavior. This script does not run the native method. Native method unit tests were not run by this agent; Root owns coordinated compilation. Synthetic occupancy is exclusively test input, not an authentic gameplay mutation.

Source gates: 7A290..7A298 countdown; 7A2C2 current quota; 7A2CD..7A2D9 signed occupancy; 7A46A selector-12 dispatch with category 4; 793CF frame mask 63 without identity addition. The scan retains initial live count and traverses ascending slots, excluding virtual-68 dead states and state 15. Distance uses doubled signed-16 wrapping coordinate subtraction, arithmetic halving, and unsigned squared-radius comparison; equality qualifies. Selector-4 category pair [4,4] enters 792C0 scoring, which remains unsupported.

## Occupancy is not a missing bootstrap activation

Serialized Krell zone byte 18 is zero. Source 72AD0 positive-quota initialization calls 79D50 (72B66), height lookup 7B140 (72B83), and marker factory 101D0 (72B95). None writes zone byte 18. Marker wait states 10/11/12 do not write it either.

The visitor arrival path at 13265..1327E uses signed family-8 record byte 57 to select `(zone_index - 1) * 19`, then increments byte `[11BD6A + offset]`. The departure path 11690..116AD decrements the previous zone's byte when that index is positive. Thus the byte tracks assigned visitor occupancy. Preserving zero is source-backed; forcing activation would conceal missing visitor behavior.

Source listings: `private/ghidra/mission-array-refs/instructions-0007a280.txt`, `instructions-000793c0.txt`; `private/ghidra/countdown-expiry/instructions-00013108.txt`, `instructions-000114b0.txt`. Their hashes are recorded alongside this report. Upcoming required behavior includes visitor arrival/departure, qualifying-Mule scoring, and quota/effect processing; these are not established by this bounded admission.

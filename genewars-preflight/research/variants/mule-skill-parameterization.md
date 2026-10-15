# Selector4 Mule skill: bounded Krell parameterization

Reference QA BETA V5. Actual first-tick fixture `private/review/krell-first-tick-001` has skills `[50,50,50,25]`, warmth7B=255, owner4. Its serialized max62/max64 fields are respectively88/42 for skill50 and76/34 for skill25. Preserve these until the original callback recalculates them; do not replace initialization with a new formula.

## Shared scale and speed

`6B150` zero-extends skill76, clamps it to100, sign-extends minimum DL, and returns `minimum + trunc(skill*(100-minimum)/100)`. Callers sign-extend its low AL before multiplying the source parameter and dividing by100. Relevant minima30/40/75 remain positive and <=100. Keep intermediate division order.

`5A990` (kind1 virtual80) uses selector4 speed parameter704, minimum75, then multiplies the truncated scaled speed by `(unsigned warmth7B+1)` and arithmetic-shifts8. Warmth255 speeds are skill25=570, skill30=577, skill50=612. Replace577 at rechoice, plant acquisition, and approach-to-feed activation with this helper; do not alter already-running speed fields without a source virtual80 call.

## Odd callback hunger/lifecycle tail

After the earlier `5E0B0` update and action callbacks, `6A290` clears4A low two bits. It skips the following decrement/refresh for virtual68 true (state>=24) or state15 (`6A2A0..6A2B7`). Otherwise decrement signed word66 with16-bit wrapping at `6A2C0..6A2C9`, then compute:

* `L = trunc(19000*percent30/100)`; `R = L >> 3`; `T=L-R`.
* If signed energy66 <=T, hunger77=255. Preserve max62/max64. Set stage `trunc((energy-R)/trunc(T/3))+1` only when it differs from `abs(signed old79)`; store the low byte (`6A416..6A48F`).
* Otherwise hunger=`max(16,trunc((L-energy)*255/R))`, stored as a byte. max62=`(trunc(160*percent40/100)*(unsigned hunger+1)>>8)+32`; max64=`(trunc(80*percent30/100)*(unsigned hunger+1)>>8)+16`; store words (`6A336..6A40D`).

| Skill | L | R | T | Stage divisor | max62 scale | max64 scale | Speed at255 |
|---|---:|---:|---:|---:|---:|---:|---:|
|25|8930|1116|7814|2604|88|37|570|
|30|9690|1211|8479|2826|92|40|577|
|50|12350|1543|10807|3602|112|52|612|

The common positive timer6C decrement follows at6A492. Formula refresh itself draws no RNG.

## Mortality, feeding and admission limits

Earlier `5E0B0` decrements66 and dispatches5CDB0 when the resulting signed value is <=0 (`5E0E1..5E0FA`); it does not compare against a fixed skill30 lifetime. Existing mortality entry atenergy1 and odd (frame+record ID) phase is independent of skill, but keep all its other state/physics guards. Feeding uses stored max62, hunger77 and reserves/energy; the callback has no need to regenerate initial max fields. Its existing gained-energy/retirement boundaries remain separate from this parameterization.

The existing approach `5D360` no-op argument for owner4 extends only through skill<=90: `5D399..5D3A1` tests unsigned76>90. Do not replace all skill30 guards with unconstrained0..255 admission. For the required25/50 states, parameterize the scalar calculations and retain all other callback/neighborhood/state bounds. Other AI paths directly use skill for delay (`63D91` and`64C00`: random remainder plus trunc((110-skill)*40/100)); those paths have not been admitted by this audit.

Independently, multiple-slot health recovery needs source ID phasing: `5E13D..5E153` uses `(frame + signed record0F)` with unsigned division by selector4 period20 and remainder<=1. The former single-slot `frame%20<=1` shortcut is not valid for slots1..3. Audit the analogous reserve-empty branch before admitting it. This is a slot-identity issue, not a skill formula.

## Focused literal proof

`mule-skill-parameterization-review.py/.json` passed20 tail cases and12 speed cases,4776 interpreted instructions, skills25/30/50/90. It covers upper lifetime, either side of the hunger threshold, low energy/stage truncation, and warmth0/127/255. The tail starts at6A2BD with an explicit caller local and stops before6A492; it compares all124 record bytes to the equations. Speed runs actual5A990. Original mapped selector4 parameters are pinned. This is a formula/source comparison, **not** a full-callback or production comparison. Next gate is an actual first/odd Krell callback after the narrow parameterization, preserving map/RNG and other creatures.

## Multiple-creature phase audit

The outer `5ED20` callback tests `(frame low byte + ID low byte)&1` at `5EE28..5EE32`. Zero takes the animation/presentation-only path ending `5EED4`; nonzero reaches health `5EF22 -> 5E0B0`, then the timer/state dispatch `5EF3A` and lifecycle tail. For phase masks this is equivalent to `phase = frame.wrapping_add(sign_extended(record.word0F))`. Apply this outer phase to health, AI, entry into mortality, states24..27, and the adapter's bones retirement selection. In particular, Krell IDs1/3 run their numeric body on frame0; IDs0/2 do not. Admission energy thresholds must use the same phase.

Within health, recovery `5E13D..5E153` uses unsigned `phase % selector.period20 <= 1`; empty-reserve starvation `5E193..5E1A5` uses `phase&31 <= 1`; temperature `5E3C5..5E3D7` uses `phase&15 <= 1`. The preceding virtual predicate may skip temperature. These are wrapping32-bit additions of signed16-bit ID, not a new RNG input.

Do not replace timer-based cadence with phase: plant feeding `688B8..688BC` tests record6C low nibble zero; decaying states25/26 `6A0AA`/`6A117` test timer6C low5 bits zero; bones state27 `6A169` tests timer6C low4 bits zero. All run within the outer odd phase. Positive timer decrement remains the common6A492 tail. Moving reserve expenditure `5E390..5E396` also uses timer6C with terrain-selected mask63/127, not frame.

The source contains additional phase&3==3 gates at65C60/65F0E/661E6, but these belong to other target/action branches; this packet does not admit them. Neutral owner4 approach remains bounded by skill<=90 and existing AI-state guards. Common6FB60 route/physics gates have separate source semantics: do not globally replace their frame input.

This phase addendum is a bounded listing audit, not a new whole-callback comparison. Scalar proof counts above remain unchanged. Next required proof is the captured Krell frame0 callback for actual IDs0..3, followed by frame1 and food/mortality edge fixtures; preserve all pools, map, RNG and unsupported rollback.

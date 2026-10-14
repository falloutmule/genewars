# Exceland: minimum playable-mission contract

Reference: the exact CD build labelled **GENEWARS QA BETA V5**, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`. Retail equivalence and cross-variant campaign equivalence remain unknown. No original game was run for this contract.

**Observed static evidence:** zero campaign progress selects HDRCO1 and CONQ001 through the original entry code. The header names Exceland. The English briefing requests an Engineer-built Extractor, First Stage Distributor and three Photon Processors; Geneticist research of Mules is advice. These are paraphrased requirements, not invented milestone mechanics.

The native title renderer selects text index `139 + 3*selector + grade`, corroborating the building identities independently of filenames:

| Construction sequence | Selector | Resource | Grade-0 health | Base cost |
|---|---:|---|---:|---:|
| Extractor | 11 | BMINE | 280 | 650 |
| First Stage Distributor | 3 | BPOWE | 240 | 400 |
| Photon Processor, three completed | 6 | BSOLA | 80 | 250 |

Actual cost subtracts twice the signed Engineer skill byte. Source construction-menu predicate `43960` admits these at progress zero but requires a completed Extractor before Distributor and a completed Distributor before Photon Processor. Existing selector bits also permit rebuilding after prerequisite loss. The first actionable implementation target is therefore **select Engineer, open construction menu, place and complete an Extractor**, followed by the newly unlocked Distributor and three Processors. A turret is not an authentic first-mission substitute.

**Winning predicate:** in the selected-player eligible campaign branch of `72ED0`, progress zero wins when unsigned player byte `606` is at least three. Generic building completion `43F60` increments `player+600+selector` for initial-grade completion, so this is the completed Photon Processor count. Other loaded target-count and terrain-object-count objectives are zero. The predicate is not an Ethereal approval counter and is not a shortcut checking newly created test objects.

For selector 6, completion sets health 80, the completed flag, player count606 and ownership bit6, and timer3D to200. It suppresses completion sound4D and has no extra selector-specific completion branch. General aggregation, dependent cleanup and presentation requests still apply.

**Failure and result flow:** earlier eligibility and the existing `805E0` capability predicate can set player elimination6E3. A selected win sets runtime terminal bit40 and invokes `2F810` in campaign mode. That function prepares a result widget and text; the button/continue transition still needs recovery. Neither a synthetic loss screen nor automatic progress increment is established by this contract.

Known initial data: one active player, target cell(81,53), initial funds2000, no completed buildings, and an initially empty objective counter. Source postload countdown1000 and frame sequencing are separately pinned by the bootstrap contract. Live RNG provenance and full campaign startup remain unresolved.

Run `python -B scripts/ghidra/mission_contract_first.py` to regenerate the companion metadata from pinned input bytes and checked instruction contracts. This is a static-data validation, not a gameplay replay. **Accepted completable campaign missions: 0 / total not yet established.** Next evidence targets are the three required building callbacks and result continuation; unrelated alternate-seed relocation remains parallel work.

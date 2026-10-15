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

**Failure and result flow:** earlier eligibility and the existing `805E0` capability predicate can set player elimination6E3. A selected win sets runtime terminal bit40 and invokes `2F810` in campaign mode. That function prepares a result widget and text; Enter continuation is traced below; the destination view and progression update still need recovery. Neither a synthetic loss screen nor automatic progress increment is established by this contract.

Known initial data: one active player, target cell(81,53), initial funds2000, no completed buildings, and an initially empty objective counter. Source postload countdown1000 and frame sequencing are separately pinned by the bootstrap contract. Live RNG provenance and full campaign startup remain unresolved.

Run `python -B scripts/ghidra/mission_contract_first.py` to regenerate the companion metadata from pinned input bytes and checked instruction contracts. This is a static-data validation, not a gameplay replay. **Accepted completable campaign missions: 0 / total not yet established.** Next evidence targets are the three required building callbacks and result continuation; unrelated alternate-seed relocation remains parallel work.

## Implemented bounded predicates and Extractor behavior

- `building_availability::available` matches literal43960 over2,048 cases, including all14 selectors, signed progress, completed prerequisites and prior ownership bits. This is menu eligibility, not placement validation.
- `extractor_production::plan_completed_idle` matches96 whole451F0 callbacks, including power exhaustion, production cycle start and exact BMINE animation timing. It rejects powered harvest/contact frames7..10 before returning any mutation. Audio service queryfalse and queued audio/render requests are explicit ports in this comparison.
- `extractor_production::plan_harvest_slice` matches64 resource-harvest slices over complete building/player and map resource-byte afterimages. It chooses the first strictly richest cell in the wrapped7x7 neighborhood, consumes up to2*(grade+1), and credits twice that amount capped to source capacity. Depleted terrain cleanup, exhausted-field notification and viewport/audio requests remain typed outputs; the caller must resolve them before committing. The preceding random particle and contact-damage path is not covered by this slice.

`plan_contact_pass` additionally matches128 original contact/RNG slices, including three draws only on odd frame-plus-building identity, wrapped contact position and the exact smoke angles. It requests an effect6 factory and ordered foreign then own kind1 queries8B000 then8B080; kind1 protection/damage and allocation effects remain explicit world work.

`plan_completed_with_requests` composes these numeric steps and matches64 whole451F0 producing-frame cases with typed boundary ports. It returns building/player afterimages plus contact and harvest requests; it does not mutate the world. Empty-resource shutdown returns `EmptyResourceNoticeRequired`, not a gameplay loss. Depleted-cell terrain mutation8CBA0 must not be omitted: it changes corner terrain types and retessellates surrounding cells. All contact, terrain and presentation requests must resolve before committing an admitted callback.

Validators are `mission_contract_extractor_idle.py`, `mission_contract_extractor_slice.py`, `mission_contract_extractor_contact.py` and `mission_contract_extractor_producing.py`. These focused checks are not owned-world acceptance or mission completion. Scheduler47F80 includes completed enabled Extractor timer samples but excludes selectors3,6,8,9; this distinction matters when integrating Distributor and Photon Processor.

The traced result input path is Enter scan1C after terminalbit40, which creates packet21. Dispatch1C120 sends that packet to82590; campaign mode queues view1863DC. A winning objective must therefore display a result and accept the original continuation action, rather than silently loading the next map. The destination view and campaign-state update still need recovery.


Owned Extractor integration now has six successful whole451F0 source comparisons over2,530,422 bytes, including actual code6 smoke allocation, signed pitch, terrain depletion repair and protected/allied linked kind1 records. Three further fixtures assert transaction rollback for unsupported own/enemy contact damage and empty-resource shutdown. The adapter `ActorWorld::step_completed_extractor` is bounded; its presentation/audio requests still require consumption by the interface. This is not yet a whole mission replay. Evidence: `extractor-owned-integration.json`; reproduction: `mission_contract_extractor_owned.py`.

The ordinary construction path exposed an actual height-change requirement. New `terrain_height::plan_height_change` matches64 literal8C550/8BEE0 cases and preserves ordered mutations, RNG draws, shade/viewport refreshes and four linked-object refreshes. Wrapped map identities do not replace the original unwrapped viewport coordinates or8C290 linear index. World execution of those refresh ports is tracked separately; a pure plan alone does not clear the construction blocker.

### Startup table correction (2026-10-14)

Literal CD routine884D0 and an independent Rust corner-pattern generator agree on the complete8192-byte material-border table (SHA-256422f124eac806197ba95d2438516284c3bf828a1db8e8aabe330661fff23ef8a). The previously used mapped112D24 prior is8192zero bytes, not the initialized table. Earlier comparisons using that prior remain development-input comparisons; they do not establish original startup terrain behavior. Root is replacing this input and rerunning affected ordinary/native construction evidence. Generator inputs are640 reference-bound bytes at105060, SHA-256d848e423b7a76a2c2dbf517fc595e33a789df7382697b56f382cc7606b18939e. No proprietary bytes are embedded in production code.

The bounded proof covers four allocation/retention branches, explicit allocation/free ports, exact generated bytes, and absence of runtime-context or RNG mutations. It does not close all transitive startup effects. Receipt: `bootstrap-details.json`; replay: `python -B scripts/ghidra/mission_contract_bootstrap_details.py`.

CONQ001 serialized context additionally contains seed5428, RNG draw count16855, frame0 and frame-backup0. Ordinary mode0 preserves the serialized seed through the early7D090 setup; subsequent player preparation and RNG ring provenance remain to be established. These values must not be confused with a completed original-startup state.

## Accepted native mission path - frozen004

Exceland now passes ordinary-command and repeated native input/hit-testing acceptance, original static victory/result flow and carried Krell initialization on one source/validator/resource-pinned build. Full two-specialist dispatch traces and winning worlds match exactly across the paths. Acceptance: `research/integration/mission-001-acceptance-004.json`; independent closure review: `research/variants/blueprint-priors-headless004-review.md`. Original missions completable: **1/24 corroborated standard missions**. Historical blueprint priors, DOS-save identity and retail equivalence remain unresolved; unavailable movie/audio uses the source nonblocking fallback. This accepts the bounded first-mission path, not the complete game or all possible specialist actions.

# Rust-first runtime evidence — 2026-10-13

| Claim | Evidence and status | Boundary |
|---|---|---|
| Exact mission and plant parameters | Corroborated: `research/ghidra/plant-state-evidence.json`, pinned candidate file/mapped data; compiled `research/integration/runtime-resource-catalog.tsv` | CD is provisional; parameter table is commercial data and remains private |
| Owned plant updates match instructions | Corroborated: plant startup/state/step bridges, `research/variants/runtime-world-review.json`; 1,100 passes, 55 callbacks, 378,353,800 exact source-state bytes | Other-family updates and transitive cleanup suspended |
| First reproduction is rejected transactionally | Observed in Rust and independently compared: attempted pass1,101, record10; all bytes/cursor/step unchanged | RNG, placement and reproduction not implemented |
| Menu loads an updating world | Observed: `research/integration/runtime-validation.json`; menu packet path, loaded 40 plants, step2 changes source-derived pixels | Development direct-load route, not original campaign flow |
| Native controls work | Observed: `research/integration/runtime-ui.json`; menu→world, two manual steps, arrows, run/pause at52, options toggle, quit, exit0 | Historical tested binary/source hashes retained; window captures are private and not raw original frames |
| Opcode3 records a move order | Corroborated: `research/ghidra/move-order-evidence.json`, `research/variants/actor-command-review.json`; bounded Rust setter and resolver return-value helper | Caller must supply a bound actor and validated environment/resolved point; no complete command integration |
| Raw mission lacks owned actors | Observed source facts: `research/variants/first-world-command-review.json`; all41 active records have owner4, actor family empty | Does not describe the population after original initialization |
| Startup needs landing/activation | Corroborated: `research/ghidra/actor-activation-evidence.json` (9 listing pins, 4,096 trig entries, 65,536 seeds, 131,072 flag cases, 19 vectors) and `research/variants/actor-initialization-boundary.md` | Explicit seed/accepted point only; landing timeline, intervening RNG, blueprints/resources and player state must be integrated before gameplay milestone C |
| DOSBox is optional for this phase | Current work policy, not a technical compatibility claim | No new original execution in this tranche; prior runs remain dated evidence |

Milestones A/B are bounded successes. C (a player command acting on the loaded world) and D (complete systems) remain unfinished. Manual command descriptions remain documentation claims until their executable routes and initialized state are established. No emulator observation is inferred from the Rust tests.

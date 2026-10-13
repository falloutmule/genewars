# Rust mission runtime

`genewars-game` is a native development build with an owned plant route and a seeded landing route. It uses the user's exact privately staged CD resources. It does not run GW.EXE or DOSBox. Bounded actor movement works; the complete game remains unfinished.

From the `genewars-preflight` directory:

```powershell
python -B scripts/run_game.py
```

This builds an optimized executable with overflow checks using the existing project-local Rust toolchain in offline/locked mode, verifies/stages49 pinned resources, and opens the application. It installs nothing. Missing or altered resources cause an explicit error. The compiled resource catalog checks both size and SHA-256 at runtime; a different local manifest cannot silently change the admitted build.

- **Single Player:** shows Loading, then reads CONQ001 and its parameter table into owned state, validates map/entity links, and renders the world.
- **Space:** advances one plant subsystem step.
- **Enter:** starts or pauses automatic plant steps. The 10 Hz timer is a development policy, not established original timing. Focus loss pauses it.
- **Arrow keys:** move the development view. Plants are reprojected from world coordinates along with terrain.
- **Escape:** returns to the menu; from the menu, exits.
- **Options:** toggles automatic stepping for the next mission load. These are clearly labeled development options.
- **Quit:** closes the application.

## Owned landing route

```powershell
python -B scripts/run_game.py --landing-seed 5428
```

The seed is an explicit development input, not a recovered original live seed. This route loads the same exact mission into one shared map with native creature/actor/effect pools and fresh player records. Native prior bytes and blueprint scratch are explicitly zeroed by host policy. It generates the64-draw blueprints, prepares actors0/1, creates the normal associated landing object, and applies the numeric radius4096 reveal update.

Space/Enter advance actor callbacks, empty-building bookkeeping, neutral Mule callbacks, plants, the empty-family8 countdown and landing/effects in the admitted source order. Contact activates actors; the fixture reaches it on pass189. Subsequent phases complete departure and remove the landing object on pass389. Actors walk to their sampled destinations, arrive and idle; smoke and departure particles animate, move and retire. The neutral Mule walks, finds a plant, approaches it, feeds and returns to idle after plant retirement. Other creature kinds and remaining systems are not yet integrated. Audio, message and original UI/minimap requests are counted as pending.

The landing route includes neutral plant reproduction with its shared RNG and source newborn pass; the separate plant-only route remains bounded. Tab selects the next activated player actor and immediately shows its source-backed portrait, name, role and raw-field status bars. Right-click on terrain dispatches the source move-order transaction using actual terrain projection. Later steps consume the order and move the actor. The automatic seed5428 test reaches target[20840,13673] and returns to idle after the click at[320,240]. These selection controls and actor/Mule/effect drawing order are explicit development policies. The native host renders tentative updates before committing them, so an unsupported callback or render error preserves the prior world. The extended 16-case suite includes 2,400-step runs for seeds 0, 5428 and 65535, with 8, 9 and 8 plant births respectively. An exact repeat agrees on world state and pixels; the selected-panel window is tested automatically.

Replay commands `select 0` and `order 320 240` exercise the same selection/command methods. `--window-replay REPLAY --frames 3 --seconds 20` runs a script then presents the resulting scene in a bounded native window, allowing automatic presentation testing. This route rejects `--world-output`: packed backing alone omits owned native/player/RNG state and is not a complete save.

## Plant route

The native scene host uses the existing menu capture/release state machine and the original resource-backed artwork. Menu, loading, mission, error, options and exit use one event loop. The old `genewars-menu` and `genewars-terrain` inspectors remain available for earlier tests.

The world owns the complete packed mission source. It interprets only established fields, retaining other family records and unknown fields as opaque bytes. Current typed plant state, links, positions and growth parameters drive the update. Neighbors come from current map lists, not caller-supplied per-step snapshots. Retirement rewrites the exact original reference bytes and source counts; it preserves unused null index words and the retired record's own links.

Admitted callbacks include first and later counters, wrapping arithmetic, crowding, starvation, animation selection and delayed retirement. Updates commit only after a complete valid plant pass. The application also renders the tentative result before replacing the displayed world. Unsupported reproduction, owner bookkeeping or other branches stop with a visible error and preserve the previous state.

The renderer projects current world records every frame, orders them through the existing queue and composes their foreground/shadows with terrain. A view-only radius4096 factor plan, empty plant-only queue, full redraw and GENEPANL backing are explicit development policies. They do not establish the original visibility, panel or campaign-entry gates.

## Evidence and limits

**Observed in Rust:** actual menu clicks load 40 plants. Step1 retires records8/16 offscreen (40→38); step2 retires visible record32 (38→37) and changes the pixels. The supervised native run also exercised view movement/return, automatic updates to step52, Options toggle, menu return and Quit. Its 145.44-second process exited normally; input hashes stayed unchanged. Captures, process records and binaries remain private; `research/integration/runtime-ui.json` is the metadata audit.

**Independently compared:** 1,100 admitted passes, 55 callbacks and 378,353,800 world bytes match a separate source-derived model. Attempt1,101 reaches record10 reproduction/RNG and rolls back without changing bytes, cursor or step count. Headless tests cover scene routing, exact resource rejection, missing-resource recovery, deterministic replay and real pixel changes. These checks are repeatable through `scripts/verify.py`.

**Partial milestones:** A (native menu→loaded world) and B (admitted deterministic updates→state/pixel changes) are implemented within this plant subsystem. Other-family AI, original dirty rectangles, target-reference/UI/audio cleanup, whole-game scheduling remain suspended or pending in the legacy plant route. Neutral reproduction is separately enabled and verified in the landing route. The raw mission contains no player-owned actors. The direct load bypasses the original campaign movie/confirmation sequence.

**A bounded gameplay command cycle now works.** The seeded route dispatches terrain commands to landing-activated actors and executes their movement, arrival and idle callbacks. Admitted source-order actor, Mule, plant and effect callbacks share one RNG; broader pathfinding, creature interaction and original whole-frame ordering remain under integration; this does not establish complete gameplay or behavioral parity.

Owned fresh/restored player state, the20-record/64-draw blueprint generator and actor initialization now feed the seeded route. First-mission mask5 prepares actors0/1 from blueprints0/10 with ready word0x0400; landing contact performs associated-object activation and queues sampled destinations. Source-derived placement queries and shared link commits are implemented. Explicit fixtures do not establish the original native priors, initial live seed or complete campaign entry.

No original-game mission comparison, save/reload, combat, economy, objectives, general AI, genetics, audio or behavioral parity is claimed. DOSBox is optional for a specific unresolved observation, not a prerequisite for further Rust implementation.

## Repeatable headless flow

Create a private replay text file containing:

```text
click 312 140
load
tick 2
```

Run the executable with the private bundle and `--headless REPLAY --report NEW_JSON --bmp NEW_BMP`. Optional `--world-output NEW_BIN` exports the owned state privately for bytewise comparison. Output paths must be new. Coordinate events use the same menu packet path as the window; `load` completes the loading scene. The runtime never reads an externally authored GWPLANT1 snapshot as its world state.

## Owned actor integration library

`ActorPool` preserves the inactive loader's native gaps and validates startup transactions with typed handles. `ActorWorld` owns the pool, player records, map/common link headers and an explicit live RNG. It commits both bounded activation routes and ground-controller pending commands atomically. The associated-object route places an actor at the object and issues its sampled destination order; it does not move the actor immediately.

The library is wired into the seeded native screen. Its fixture, headless and bounded window tests are automatic. General movement/pathfinding, prior-family RNG consumers and full scheduling remain unfinished. The packed backing excludes native/player overlays and cannot be used alone as a save file. See `research/integration/actor-world-contract.md`, `landing-world-contract.md`, `shared-runtime-validation.json` and `reveal-validation.json`; receipts identify their dated compilation snapshots.

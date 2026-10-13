# Rust mission runtime

`genewars-game` is a native development build with a real, mutable plant world. It uses the user's exact privately staged CD resources. It does not run GW.EXE or DOSBox.

From the `genewars-preflight` directory:

```powershell
python -B scripts/run_game.py
```

This builds with the existing project-local Rust toolchain in offline/locked mode, verifies/stages 26 pinned resources, and opens the application. It installs nothing. Missing or altered resources cause an explicit error. The compiled resource catalog checks both size and SHA-256 at runtime; a different local manifest cannot silently change the admitted build.

- **Single Player:** shows Loading, then reads CONQ001 and its parameter table into owned state, validates map/entity links, and renders the world.
- **Space:** advances one plant subsystem step.
- **Enter:** starts or pauses automatic plant steps. The 10 Hz timer is a development policy, not established original timing. Focus loss pauses it.
- **Arrow keys:** move the development view. Plants are reprojected from world coordinates along with terrain.
- **Escape:** returns to the menu; from the menu, exits.
- **Options:** toggles automatic stepping for the next mission load. These are clearly labeled development options.
- **Quit:** closes the application.

## What is implemented

The native scene host uses the existing menu capture/release state machine and the original resource-backed artwork. Menu, loading, mission, error, options and exit use one event loop. The old `genewars-menu` and `genewars-terrain` inspectors remain available for earlier tests.

The world owns the complete packed mission source. It interprets only established fields, retaining other family records and unknown fields as opaque bytes. Current typed plant state, links, positions and growth parameters drive the update. Neighbors come from current map lists, not caller-supplied per-step snapshots. Retirement rewrites the exact original reference bytes and source counts; it preserves unused null index words and the retired record's own links.

Admitted callbacks include first and later counters, wrapping arithmetic, crowding, starvation, animation selection and delayed retirement. Updates commit only after a complete valid plant pass. The application also renders the tentative result before replacing the displayed world. Unsupported reproduction, owner bookkeeping or other branches stop with a visible error and preserve the previous state.

The renderer projects current world records every frame, orders them through the existing queue and composes their foreground/shadows with terrain. A view-only radius4096 factor plan, empty plant-only queue, full redraw and GENEPANL backing are explicit development policies. They do not establish the original visibility, panel or campaign-entry gates.

## Evidence and limits

**Observed in Rust:** actual menu clicks load 40 plants. Step1 retires records8/16 offscreen (40→38); step2 retires visible record32 (38→37) and changes the pixels. The supervised native run also exercised view movement/return, automatic updates to step52, Options toggle, menu return and Quit. Its 145.44-second process exited normally; input hashes stayed unchanged. Captures, process records and binaries remain private; `research/integration/runtime-ui.json` is the metadata audit.

**Independently compared:** 1,100 admitted passes, 55 callbacks and 378,353,800 world bytes match a separate source-derived model. Attempt1,101 reaches record10 reproduction/RNG and rolls back without changing bytes, cursor or step count. Headless tests cover scene routing, exact resource rejection, missing-resource recovery, deterministic replay and real pixel changes. These checks are repeatable through `scripts/verify.py`.

**Partial milestones:** A (native menu→loaded world) and B (admitted deterministic updates→state/pixel changes) are implemented within this plant subsystem. Other-family AI, original dirty rectangles, target-reference/UI/audio cleanup, generation/reproduction and whole-game scheduling remain suspended or pending. The raw mission contains no player-owned actors. The direct load bypasses the original campaign movie/confirmation sequence.

**Gameplay milestone C remains open.** A bounded move-order setter and target return-value helper exist as explicit-input primitives. The player/actor startup helpers below still need to join the native world, command eligibility and map queries. Landing and intervening RNG consumption must be integrated before a user command can act on a valid player actor. Camera movement, plant stepping and UI selection are not player gameplay commands.

The runtime library now implements the next startup prerequisites: owned fresh/restored player state, the 20-record/64-draw blueprint generator, actor initialization with typed resource bindings, and a transaction that prepares the player record plus its five actor slots. Under the fresh first-mission inputs it prepares actors 0/1 from blueprints 0/10. Their ready word remains 0x0400. The per-slot activation planner is also implemented and independently checked, with explicit RNG/environment and resolved map-head inputs. These helpers are not yet dispatched by the native Single Player route. The landing/effect schedule, native pool gaps, opaque generator scratch bytes, resource loading and complete map commit remain explicit integration work.

No original-game mission comparison, save/reload, combat, economy, objectives, AI, genetics, audio or behavioral parity is claimed. DOSBox is optional for a specific unresolved observation, not a prerequisite for further Rust implementation.

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

This library is not yet wired into the native screen. Its fixture tests are automatic and do not require a user gameplay test. Factory/landing scheduling, prior RNG consumers, actual environment predicates, resource resolution, screen target projection and movement remain unfinished. The packed backing excludes native actor/player overlays and cannot be used alone as a save file. See `research/integration/actor-world-contract.md`, `actor-world-validation.json` and `associated-activation-validation.json`.

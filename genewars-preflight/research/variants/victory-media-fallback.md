# Victory media fallback

Reference: verified CD **GENEWARS QA BETA V5**, executable SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.

**Observed source behavior:** BIOWIN open/allocation failure is nonblocking. `A4FF8` branches from `A5051` to its return at `A51BE` on open failure. `A51C8` similarly returns through `A5322` on open failure; allocation failure closes the movie and returns zero at `A523D..A5245`. `1BE50` ignores the playback return at `1BF5D` and unconditionally clears input edges through `715A0` at `1BF62`. Victory entry `7DBC0` also ignores the return from `1BE50`. Statistics and Continue remain reachable.

Source entry order:

1. Load original WINSCR palette/background, clear flag4, prepare display/audio and close old widgets.
2. Set stage5, save current frame and set modebit2.
3. Attempt BIOWIN, accepting the original nonblocking failure result, then clear consumed input edges.
4. Lock/copy/unlock framebuffer where available. Lock failure at `7DCA7` also falls through.
5. Reset frame0 and increment selected-player progress at `7DD0A`.
6. Set drawing bounds through `1F490`, derive eight palette mappings through `715E0`, then reset service `A2FB8`.
7. Render original statistics/feedback; accept a fresh input edge through `7DDD0`; retain `72E50` cleanup on departure.

For atomic native integration, prepare original static resources, palette mappings, bounds, input reset and an explicit movie result before committing entry progress. Distinguish **MovieUnavailable** from **MoviePlayed**. A missing decoder is a documented presentation limitation; it does not establish that the original asset failed. Do not fabricate modebit8 (which independently skips playback at `1BE5F`) and do not claim successful animation or complete presentation fidelity. Full-game movie support remains unfinished.

Scope: read-only inspection of already-exported listings; no new tests, exports or original execution. Exact listing hashes and structured findings are in `victory-media-fallback.json`.

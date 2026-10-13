# Independent integration diff review

**Corroborated within the stated scope:** no blocker found in the root-owned integration changes since checkpoint `750b9cd`. The review covers construction exports, separate BONES admission/accounting, verification wiring, and the project-private Rust/LLVM build environment. Related core, native-control, dependencies, BONES and planet-texture reviews have their own exact source/evidence receipts.

The sprite integration retains the ANB metadata boundary and selects packed4 only through exact source-hash and provenance admissions. The 25 creature hashes remain separate from the single BONES hash. The updated expected totals—262 passing records out of 263, 26 packed4 inputs, 52 packed4 source aliases, and 11,001 sprite records—agree with the reviewed corpus. GPF remains explicitly blocked; the verifier does not turn that failure into successful decoding.

The verification script builds the native tools before corpus refresh and independent byte reviews. Static and independent construction, atlas, packet/name and dependency checks are wired into the aggregate. Its two native presentation smokes intentionally open windows; this reviewer did not run that full script under root's no-GUI instruction. The aggregate receipt read during this review was the older 01:02 UTC result, so it cannot validate the newly added checks. Root should refresh full verification before a new checkpoint.

Actual headless `python -B scripts/cargo.py test --workspace --offline --locked` passed 84 Rust tests: assets 29, core 23, controls host 11, desktop 10, renderer 11. No GUI or original program was launched by these unit tests. All Python sources parsed. Cached LLVM archive bytes match the pinned checksum; every selected member matches its receipt and archive payload, the dlltool alias is byte-identical to llvm-ar, and its two runtime DLL copies match their sources. The build environment changes are process-local and use private tool/cache paths. This review neither downloaded nor installed tools.

All hashes listed by the supervised native-control observation receipt matched their current/private artifacts, including the captured tested executable and source. This corroborates receipt provenance; it is not an independent physical interaction test or original-game parity result.

The run002 checker now reports `checker_grants_execution_authorization:false` and does not assert that later human permission is absent. Historical receipts and the current approval record remain unchanged. Synthetic fixtures verify that the checker accepts observed nonzero exits and released-but-unconfirmed exits without assuming game success, preserves all file deltas, and refuses an audit without a final-process receipt.

`integration-diff-review.json` pins reviewed source hashes and headless results. Whole-game behavior, original timing, dynamic text rendering, guest cursor behavior, missions and saving remain outside this review.

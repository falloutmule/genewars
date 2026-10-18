# Captured blocked Photon callback comparison

Reference: verified CD GENEWARS QA BETA V5, GW.EXE SHA-256 `4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba`.

The ordinary fresh mission capture at frame 2523 reaches the original `74E80` invalid-site class 1 for queued Photon placement. The literal `57A10` wrapper, `507D0` setup, `51E80` cancellation and callback tail were interpreted against that world. The virtual `94` movement call at `57A3E` is explicitly supplied from a cloned production post-motion capture, not independently proven by this test. Audio, formatting, panel and queue services are explicit no-gameplay-state ports.

Initial comparison: **error**, two actor bytes differ from the prototype. `509C2` clears actor byte `81` during setup task-reference normalization; `51235` sets actor state byte `55` to 9 before the site query. Rejection clears the task but does not restore those two bytes. The prototype instead retained byte `81` = 4 and state 1. The remaining 198 actor bytes and all 36 blueprint bytes match. This identifies a setup-prefix omission; it does not authorize a mission-completion claim.

Evidence: `private/review/blocked-photon-source/74bae7229fdc44b08748bd7970ad69a8/result.json`. The before world is `private/review/blocked-photon-site/2ced9c6f6b4a4484bc21d23a523bb133/2523`; the explicit movement result is `private/review/blocked-photon-site/30ec25edea1d412c8fbd6f9ad90c0af6/2523-postmotion`.

The next whole-frame map differs at two bytes and RNG differs by four draws. Those results include unrelated callbacks and are not attributed to cancellation. An isolated callback afterimage is required before reporting map/RNG equality. Run `C:/Python314/python.exe research/variants/blocked-photon-source.py --expected <isolated-after-folder>` once available. Each invocation preserves a new private result directory and listing hashes.

The following callback sees task byte 58 = 0 and state 9, causing the original `57AAC..57B8F` normalization and RNG path. Moving that normalization into the rejection frame changes source timing.

## Repaired isolated callback result

**Passed.** `private/review/blocked-photon-source/e7b1b7ec6240478f96eac54d8f383092/result.json` compares the repaired isolated callback at `private/review/blocked-photon-site/7855fd5421dc49388fdc6e3d60245d1f/2523-callback`: all 200 actor bytes, 229,376 map bytes, 7 RNG bytes and 36 blueprint bytes match exactly (229,619 bytes total). Source execution used 933 interpreted instructions. No gameplay-state differences remain in this bounded comparison.

Movement remains an explicit captured production port at `57A3E`/`6FB60`; audio, formatting, panel and queue services remain explicit ports. This proves the rejection wrapper/setup/cancellation/tail around that movement result, not movement independently, complete mission acceptance, audio playback or UI usability. The original failing result is retained above.

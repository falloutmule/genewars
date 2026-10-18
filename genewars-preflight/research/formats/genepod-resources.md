# Genepod resource admission

The verified QA BETA V5 CD selector0 binds `BGENE.ANB`. This is corroborated by mapped descriptor `0x10393C`, whose destination is `0x116824`, and the existing loader chain through the relocated selector table `0x11685C`. CD manifest member `root/2/disc/301` has 62,062 bytes and SHA-256 `a71f03d3cce3d36798cd7f8ad578d665e65883bfe9b541ba93d0ad9eff814cab`.

The ANB contains 15 sprite-table records, 52 components, 26 frames and five animations. Section offsets are 61038, 61128, 61648 and 62012. Animation0 contains three construction-grade frames. Animation1 contains four frames, with hold bytes `[0,0,3,0]`. The source callback at `47F08..47F13` selects grade+1 for a completed ordinary building, establishing animation1 for grade0. The incomplete branch selects animation0; flag40 selects animation4 separately. These observations do not establish the Genepod's production callback or when production advances its frames.

`SourceBuildingBinding::CdBgeneSelector0` admits only the exact CD bytes. The existing common-image renderer accepts animations0/1 and rejects upgraded Genepod grades. Animations2/3 and special animation4 remain outside this admission. Existing BTURR and other building bindings retain their checks.

Reproduce static evidence with `python -B scripts/desktop/genepod_resource_evidence.py`; add `--test` to run only the resource module's focused Rust tests. The independent Python ANB parser produces a 117,413-byte metadata/pixel/opacity stream. Fourteen complete padded framebuffer hashes cover each admitted frame at ordinary and clipped origins. The adjacent JSON distinguishes static evidence from whether Rust tests have actually run. No proprietary bytes are tracked.

Host integration requires the catalog row in that JSON, adding `buildings/BGENE.ANB` to private missing-resource staging, and binding selector0 to `CdBgeneSelector0`. This work does not implement Genepod construction, production, shadow rendering, or mission acceptance.

# Independent panel and palette review

Observed static result, 2026-10-12 local: no blocking defect found in the bounded Rust panel renderer or its evidence bridge. This is a static resource export, not observed original gameplay or complete screen rendering.

## Direct evidence reviewed

Reviewed the exported instructions for panel copier VA `0x24AD0`, DAC uploader `0xA6D2C`, display setup `0x1CAA0`, wrapper `0x979A0`, mode initializer `0xA13F0`, and panel-region caller `0x82BD0`, together with the private mapped mode table. These establish 32 DWORDs (128 bytes) copied per source row, destination advance `(pitch >> 2) * 4`, and signed height <= 0 as a no-op. Normal initialization supplies width 640, height 480, mode index 13 and palette global `0x115E80`. The selected mode table independently contains 640x480 for mode 13. The 61,440-byte decoded panel therefore fits the instruction-derived 128x480 source layout.

The alternative path sets the same logical 640/480 globals after selecting physical 320x200 mode 1. Its later screen transformation is not established by this review. Do not generalize the logical buffer dimensions to every physical display path. Likewise, direct copier bounds depend on caller storage; the portable renderer's exact source-length check is a new safe API boundary.

The descriptor and loader dossier links `data/main.pal` to global `0x115E80`, which display setup passes to the DAC uploader. The uploader emits each index and then three successive bytes without arithmetic scaling. Hardware RGB channel order and six-bit DAC components are corroborated by the primary [FreeVGA DAC reference](https://www.scs.stanford.edu/10wi-cs140/pintos/specs/freevga/vga/colorreg.htm). No source-to-display transfer function for an original monitor is measured here.

## Actual outputs independently checked

`renderer_review_checks.py` independently verifies the staged exact CD `MAIN.PAL`, source index hash, BMP header, every source index/palette lookup, every BGR triplet and bottom-up row address. It does not reuse the integration BMP parser. All 61,440 pixels / 184,320 channels passed.

| Artifact | SHA-256 |
|---|---|
| CD GENEPANL compressed input (`root/2/disc/86`) | `8116b2b883320b8573c50496a570f0f3fa978e4c20b9c85a284339ff1f920e24` |
| Decoded panel indices | `af9741b4545fb6f28a1dd0e50da0570e39992e75c872bc2249891f3330f59951` |
| CD raw MAIN.PAL (`root/2/disc/108`), 768 bytes | `8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05` |
| BMP, 184,374 bytes | `af218cc65e0abb410242cd73440b53d4c18f4d8cd3f03f1f7542c4bb1c70809b` |
| Top-down RGB24 pixels | `bb135e29eb2e6dacf17ab8273e57df185fe24b82320ab5b296fbf74a6c11444d` |

MAIN.PAL contains only 0..63 components. Another 26 unique corpus palette inputs independently have 768 decoded bytes and collectively cover all 64 component values; this supports representation, not universal palette association. No alternate palette was substituted for the reviewed panel.

Reviewed `genewars-render/src/lib.rs`: RGB mode and component-range validation, exact index count, checked image sizes, fallible allocation, BGR output, DWORD row padding and positive BMP height are consistent with this export. The declared modern conversion is normalized floor `v*255/63`. Synthetic examples include 32->129, 47->190 and 63->255; bit replication instead maps 32->130. These are deliberately different policies, and the chosen policy must remain explicit. The current Rust test includes 32/47.

Unknowns remain original overlay composition, palette transitions, fallback display conversion, sprites, actual DAC/monitor response and observed interactive behavior. The CD candidate's retail authenticity remains provisional. Results are in `renderer-review-results.json`; raw proprietary evidence and images remain private.

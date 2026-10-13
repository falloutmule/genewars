# Independent explicit shade and fade review

No defect found. Actual native output for all 16,777,216 signed16-light/unsigned8-factor combinations matches an instruction-sequence model. The source shifts the signed operand by8, multiplies by the unsigned factor, applies its sign/carry truncation correction and shifts by6, then adds32768. Products are always divisible256, so this is exactly4×light×factor+32768. All source intermediates fit signed32; output ranges from-33390592 to33455108.

All16384 lookups in the exact64×256 FADE table match the decoded source bytes. Every unsupported row64..255 rejects at both palette-index endpoints; lengths0,1,16383,16385 reject. Source index0 is mapped rather than skipped: shade63 produces148. No automatic shade-row conversion, clamping, lighting-factor meaning or visibility source is inferred. The API requires the caller to supply an admitted row explicitly.

Reproduce with `python -B research/variants/terrain-shade-review.py`; the run takes only a few seconds on this host. Native buffers and decoded bytes remain private. The receipt binds the production module and exact instruction/asset evidence. This does not establish gradients, clipping, triangle coverage or original frame fidelity, and opens no GUI or original.

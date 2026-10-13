# Independent prepared mode-five span review

No implementation defect found in7,979 actual native spans and583275 emitted pixels. The oracle concatenates C32,D32,v8 into one72-bit value and performs one modular72-bit addition per pixel. This independently models the literal ADD/ADC/ADC chain; it does not advance independent fixed-point U/V/shade values after packing. Full32 wrapping clipping occurs before packing.

The review pins the exact source bytes83FD00730149. CMP against unsigned zero never borrows, so JNC always skips DEC of gradientU, including negative shade gradients. The deliberate negative-shade fixture therefore changes integerU despite dU=0. Initial D clears its middle byte; gradient D preserves the corresponding rotated-dV byte. Pre-add U/V/shade select the sample before advancing state.

Coverage tests include all widths1..640, signed-floor fractional edges, i32 extrema, negative-left advances and7,000 seeded carry/bounds cases. Eight complete synthetic and four exact CD atlas/FADE fixtures agree. Texture lookup uses byteU/V and full256-byte pitch; no compact32×32 bounds or coordinate clamp is introduced. Source index0 and output index0 are emitted.

Every complete output, prefix and tail sentinel is compared. Late shade/source rejection, address overflow, short destination and negative width leave all output unchanged. Two-pass validation is safe because both source slices and state are immutable between validation and writing. The input's fixed-point high words bound any covered count to32767, making destination count arithmetic safe.

A source-domain caveat was identified and corrected in the evidence/docs: original negative-left coverage checks right>0 before clamping to clipwidth and has no post-clamp empty check. With zero width it can enter the count0 unroll and write16 pixels. Rust deliberately returns an empty span; positive clip width is required for original coverage equivalence. Negative widths reject. This safe deviation must remain explicit.

Reproduce with `python -B research/variants/mode5-span-review.py`. Receipt and native/private output hashes bind the reviewed code. This is a prepared row primitive; triangle edges, gradient derivation, vertical coverage, culling and complete original frames remain outside scope. No original or GUI is executed.

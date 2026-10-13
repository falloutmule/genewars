# Independent renderer terrain primitives

No defect found. The private native harness checks all 65,536 signed height values, all 256 cell flag bytes against the mapped UV table at1060EC, every orientation admission value,154368 strip vertices and six overflow boundaries. It compiles the production module directly; no GUI, original, module export or crate edit is performed by the reviewer.

The strip derivation uses column c, n=floor(c/2), p=c&1. Prime A=(camera_x+n,camera_y-n-1-p) has X=32(c+1), baseline Y=-16(1+p)-top. B=A+(0,1) has X=32c and Y=A_y+16; C=A+(1,1) has right X and Y=B_y+16; D=A+(1,2) has left X and Y=C_y+16. Substitution into the helper's unwrapped map/camera deltas reproduces all four points for both parities. Each vertex subtracts2*(signed_height>>6). The independent oracle steps these strip points rather than recomputing the helper's delta expression.

First raster call7BB8E uses A,C,B and UV entries0,2,3 for cellA. Second call7BD9D uses B,C,D and entries0,1,2 for adjacent cellB. The exported triangle patterns therefore describe the two cell-relative halves; the original traversal does not issue both successively for one fixed quad. This is an equivalence of7B7F0's strip path, not7AF00's separately narrowed point projection.

Checked intermediate overflow is an explicit safe-port restriction; the original uses32-bit wrapping. Some large placements whose final mathematical sum fits can still reject because an intermediate does not fit. Normal bounded cell/camera differences and the tested signed-height rounding remain exact. No startup camera, full triangle coverage, clipping, shading, opacity, overlay or original terrain-frame parity is established.

Reproduce with `python -B research/variants/terrain-vertex-review.py`. The JSON pins production source, exact strip listing, native harness and evidence receipt. SOL-01 independently supplied the parity/corner derivation; this reviewer checked the listed instructions and native outputs. The earlier terrain-review covers complete data views and interpolated heights separately.

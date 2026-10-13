# Independent explicit-target camera review

No defect found. Actual native camera words match475136 real-coordinate targets and917504 synthetic targets. The latter cover all65536 x/y high-byte pairs at fourteen constant height levels, including negative division thresholds and extrema. Short/long map extents reject.

The oracle independently applies arithmetic height shift6, then the source's sign/carry subtraction correction before shift5. For negative values this adds31 and therefore truncates division32 toward zero. It models byte additions of12 and2, byte subtraction from coordinate high bytes, and zero-extended word stores. Returned cells0..255 are separate from source-map indices modulo128; replacing byte wrap with map wrap would change the camera words.

The helper takes explicit target coordinates. No raw-file player field is presumed to be the live startup target, and no viewport/minimap/sound/redraw side effects are implemented. Initial mission camera and loaded-state mutations remain separate evidence. Reproduce with `python -B research/variants/camera-target-review.py`; all source/listing/native harness hashes are in the adjacent JSON. No GUI or original is executed.

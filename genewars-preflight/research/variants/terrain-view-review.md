# Independent full-viewport assembly

The shared viewport renderer passed 24 native cases: 19 complete 640×480 indexed frames, 5,836,800 final index pixels, 8,294,400 ordered writes and 5,760 ordered dirty requests. The oracle composes this lane's independent cached strip traversal, repeated triangle stepping, packed mode-5 carry model and mode-specific sampling. It does not import the Ghidra validators' mathematical models. See [the receipt](terrain-view-review.json).

Cases cover exact CONQ001 snapshots at three cameras, an explicitly applied pure factor plan, generated debug flags and a mode-0→6→15 fixture. All twenty columns clip to the complete frame. Base and optional calls preserve strip order, mode 6 retains prior pixels for transparent samples, and mode 15 reads the destination produced by preceding draws. Dirty requests retain their order and inputs remain unchanged.

Missing MAIN.GLA rejects when mode 15 is requested, and is accepted when no such request occurs. Unsupported texture selectors reject textured mode 5, while geometry-only mode 0 requires no texture access. Invalid cameras and atlas size reject. The function owns its newly allocated frame and returns no partial frame on error. No further defect was found.

The full 640×480 extent, twenty-column interval, supplied cameras, factor snapshots and context flags are explicit host policies. These frame comparisons do not establish the original mission's initial viewport, live callbacks or runtime scene parity. No original executable or GUI ran.

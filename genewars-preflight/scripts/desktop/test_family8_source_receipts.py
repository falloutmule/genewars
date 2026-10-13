#!/usr/bin/env python3
"""Pure regression checks for stable/canonical family-8 receipt validation."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
from family8_source_receipts import (  # noqa: E402
    validate_onward_source, validate_state5_source, validate_target_source,
)


def load(relative: str):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


def must_reject(call, label: str) -> None:
    try:
        call()
    except (ValueError, OSError):
        return
    raise AssertionError(f"accepted tampered canonical source receipt: {label}")


def main() -> int:
    s5 = load("private/review/family8-state4/source.json")
    s5_review = load("research/variants/family8-state4-review.json")
    onward = load("private/review/family8-onward/source.json")
    target = load("research/variants/family8-target-review.json")
    validate_state5_source(ROOT, s5, s5_review)
    validate_onward_source(ROOT, onward)
    validate_target_source(ROOT, target)

    # Regeneration changes volatile pins/timestamps. Canonical trace facts and
    # the raw-input/model pins still validate, so harmless receipt churn passes.
    changed = copy.deepcopy(target)
    changed["utc"] = "2099-01-01T00:00:00Z"
    changed["source_pins"].append({"path": "volatile/irrelevant", "sha256": "0" * 64})
    changed["input_and_listing_pins"].append({"path": "volatile/irrelevant", "sha256": "1" * 64})
    validate_target_source(ROOT, changed)
    changed_s5 = copy.deepcopy(s5)
    changed_s5["pins"]["volatile/irrelevant"] = "2" * 64
    validate_state5_source(ROOT, changed_s5, s5_review)

    bad_target = copy.deepcopy(target)
    next(entry for entry in bad_target["input_and_listing_pins"]
         if entry["path"] == "private/review/family8-onward/child-after6143.bin")["sha256"] = "0" * 64
    must_reject(lambda: validate_target_source(ROOT, bad_target), "target raw child hash")
    bad_onward = copy.deepcopy(onward)
    bad_onward["timeline"]["child"][-1]["state"] = 5
    must_reject(lambda: validate_onward_source(ROOT, bad_onward), "onward endpoint")
    bad_s5 = copy.deepcopy(s5)
    next(row for row in bad_s5["timeline"] if row.get("first_frame") == 6002 and
         row.get("first_seed") == 5428 and row.get("frame") == 6047)["state"] = 4
    must_reject(lambda: validate_state5_source(ROOT, bad_s5, s5_review), "state5 endpoint")
    print("family8 source receipt canonicalization passed (volatile metadata tolerated; pinned identity tampering rejected)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

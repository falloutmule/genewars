#!/usr/bin/env python3
"""Pure regression checks for stable/canonical family-8 receipt validation."""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(__file__).resolve().parent))
from family8_source_receipts import (  # noqa: E402
    validate_onward_source, validate_state5_source, validate_target_source,
)


def load(relative: str):
    return json.loads((ROOT / relative).read_text(encoding="utf-8"))


class Family8SourceReceiptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.state5 = load("private/review/family8-state4/source.json")
        cls.state5_review = load("research/variants/family8-state4-review.json")
        cls.onward = load("private/review/family8-onward/source.json")
        cls.target = load("research/variants/family8-target-review.json")

    def test_current_canonical_sources_validate(self):
        validate_state5_source(ROOT, self.state5, self.state5_review)
        validate_onward_source(ROOT, self.onward)
        validate_target_source(ROOT, self.target)

    def test_volatile_regeneration_metadata_is_tolerated(self):
        # Regeneration changes timestamps and whole-workspace pin lists.
        # Canonical facts remain tied to raw inputs and model source hashes.
        changed = copy.deepcopy(self.target)
        changed["utc"] = "2099-01-01T00:00:00Z"
        changed["source_pins"].append({"path": "volatile/irrelevant", "sha256": "0" * 64})
        changed["input_and_listing_pins"].append({"path": "volatile/irrelevant", "sha256": "1" * 64})
        validate_target_source(ROOT, changed)
        changed_s5 = copy.deepcopy(self.state5)
        changed_s5["pins"]["volatile/irrelevant"] = "2" * 64
        validate_state5_source(ROOT, changed_s5, self.state5_review)

    def test_raw_target_fixture_tampering_is_rejected(self):
        bad = copy.deepcopy(self.target)
        next(entry for entry in bad["input_and_listing_pins"]
             if entry["path"] == "private/review/family8-onward/child-after6143.bin")["sha256"] = "0" * 64
        with self.assertRaises((ValueError, OSError)):
            validate_target_source(ROOT, bad)

    def test_source_endpoints_tampering_is_rejected(self):
        bad_onward = copy.deepcopy(self.onward)
        bad_onward["timeline"]["child"][-1]["state"] = 5
        with self.assertRaises((ValueError, OSError)):
            validate_onward_source(ROOT, bad_onward)
        bad_state5 = copy.deepcopy(self.state5)
        row = next(row for row in bad_state5["timeline"] if row.get("first_frame") == 6002 and
                   row.get("first_seed") == 5428 and row.get("frame") == 6047)
        row["state"] = 4
        with self.assertRaises((ValueError, OSError)):
            validate_state5_source(ROOT, bad_state5, self.state5_review)


if __name__ == "__main__":
    unittest.main()

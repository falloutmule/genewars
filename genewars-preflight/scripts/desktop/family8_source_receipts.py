"""Stable source-identity checks for regenerated family-8 review receipts.

The JSON receipts contain volatile whole-workspace pins and (for some reports)
timestamps. These helpers validate only the canonical trace facts used by the
pixel probes, while binding those facts to the exact raw inputs and model source.
"""
from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ValueError(message)


def _mapping_pin(root: Path, mapping: dict[str, str], relative: str, expected: str) -> None:
    _require(mapping.get(relative) == expected, f"source receipt lacks stable pin for {relative}")
    _require(sha((root / relative).read_bytes()) == expected, f"source input identity changed: {relative}")


def validate_state5_source(root: Path, source: dict[str, Any],
                           review: dict[str, Any]) -> dict[str, Any]:
    _require(source.get("status") == "literal-source-fixtures", "state5 source fixture status changed")
    _require(source.get("cases") == 276 and source.get("original_execution") is False,
             "state5 source fixture admission changed")
    pins = source.get("pins")
    _require(isinstance(pins, dict), "state5 source pins are malformed")
    _mapping_pin(root, pins, "private/integration/game-runtime/terrain/CONQ001.DAT",
                 "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2")
    model = "research/variants/family8-state4-source.py"
    model_sha = pins.get(model)
    _require(isinstance(model_sha, str) and len(model_sha) == 64, "state4 source model pin is missing")
    _mapping_pin(root, pins, model, model_sha)
    rows = source.get("timeline")
    _require(isinstance(rows, list), "state5 source timeline is malformed")
    final = [row for row in rows if row.get("first_frame") == 6002 and
             row.get("first_seed") == 5428 and row.get("frame") == 6047]
    _require(len(final) == 1, "state5 source timeline lacks unique seed-5428 terminal row")
    row = final[0]
    _require((row.get("state"), row.get("animation"), row.get("animation_frame"),
              row.get("height"), row.get("speed")) == (5, 0, 6, 22302, 0),
             "state5 source terminal fields changed")
    _require(row.get("requests", {}).get("child") is True,
             "state5 source terminal callback boundary changed")
    _require(review.get("status") == "passed-bounded-family8-selector0-state4" and
             review.get("callbacks") == 276,
             "state5 reviewed source contract changed")
    return {"model_path": model, "model_sha256": model_sha,
            "raw_input_path": "private/integration/game-runtime/terrain/CONQ001.DAT",
            "raw_input_sha256": pins["private/integration/game-runtime/terrain/CONQ001.DAT"],
            "terminal": {key: row[key] for key in ("first_frame", "first_seed", "frame", "height",
                                                       "speed", "state", "animation", "animation_frame")}}


def validate_onward_source(root: Path, source: dict[str, Any]) -> dict[str, Any]:
    _require(source.get("status") == "literal-onward-fixtures" and
             source.get("original_execution") is False,
             "onward source fixture status changed")
    pins = source.get("pins")
    _require(isinstance(pins, dict), "onward source pins are malformed")
    input_path = "private/integration/game-runtime/terrain/CONQ001.DAT"
    _mapping_pin(root, pins, input_path, "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2")
    model = "research/variants/family8-onward-source.py"
    model_sha = pins.get(model)
    _require(isinstance(model_sha, str) and len(model_sha) == 64, "onward source model pin is missing")
    _mapping_pin(root, pins, model, model_sha)
    timeline = source.get("timeline")
    _require(isinstance(timeline, dict), "onward source timeline is malformed")
    child = timeline.get("child")
    _require(isinstance(child, list) and len(child) == 98, "onward child trace extent changed")
    first, last = child[0], child[-1]
    _require((first.get("frame"), first.get("state"), first.get("animation"),
              first.get("animation_frame")) == (6046, 0, 4, 0),
             "onward child trace start changed")
    _require((last.get("frame"), last.get("state"), last.get("animation"),
              last.get("animation_frame")) == (6143, 1, 4, 0),
             "onward child trace endpoint changed")
    return {"model_path": model, "model_sha256": model_sha,
            "raw_input_path": input_path, "raw_input_sha256": pins[input_path],
            "child_extent": len(child), "child_start": {k: first[k] for k in ("frame", "state", "animation", "animation_frame")},
            "child_end": {k: last[k] for k in ("frame", "state", "animation", "animation_frame")}}


def validate_target_source(root: Path, report: dict[str, Any]) -> dict[str, Any]:
    _require(report.get("status") == "passed-bounded-family8-initial-target" and
             report.get("cases") == 64 and report.get("original_execution") is False and
             report.get("gui_execution") is False,
             "target source review status/scope changed")
    inputs = report.get("input_and_listing_pins")
    _require(isinstance(inputs, list), "target input pins are malformed")
    input_pins = {entry.get("path"): entry.get("sha256") for entry in inputs}
    expected_inputs = {
        "private/integration/game-runtime/terrain/CONQ001.DAT":
            "72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2",
        "research/variants/family8-state4-source.py":
            "cc0d5a38b1c0c2d6369811328c7685818ceb0f15152b5dfae83b753839f7392a",
        "research/variants/family8-target-review.py":
            "8d54794003d621600ec80088bae898efcf6eba4cd272f119c8cb841511241734",
        "research/variants/family8-target-review.rs":
            "bd7364fd5671a8c7b4763d85044c08617fece08cdeb0fe2b06e255ea00816f98",
        "private/review/family8-onward/child-after6143.bin":
            "51bd5831820154fb508cc18f16dd5f2177a2e6d2fad3b24694593e38119f777a",
    }
    for path, digest in expected_inputs.items():
        _mapping_pin(root, input_pins, path, digest)
    outputs = report.get("outputs")
    _require(isinstance(outputs, list), "target output pins are malformed")
    output_pins = {entry.get("path"): entry.get("sha256") for entry in outputs}
    input_sha = "57c37e848d7a719baa265f7b7bb4a0f46f24a49a054367f1f7a7d4037c8a32b3"
    expected_sha = "0fc3988c7dcc8b4393b5c328bafce434e2268c522e974bf9b740eaddaf28552f"
    _require(output_pins.get("private/review/family8-target/input.bin") == input_sha and
             output_pins.get("private/review/family8-target/expected.bin") == expected_sha and
             output_pins.get("private/review/family8-target/actual.bin") == expected_sha,
             "target source review raw-output identity changed")
    return {"status": report["status"], "cases": report["cases"],
            "source_inputs": expected_inputs, "target_fixture": {
                "input_sha256": input_sha, "expected_sha256": expected_sha,
                "actual_sha256": output_pins["private/review/family8-target/actual.bin"]}}

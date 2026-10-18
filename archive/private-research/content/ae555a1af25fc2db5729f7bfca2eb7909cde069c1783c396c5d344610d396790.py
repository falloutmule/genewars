#!/usr/bin/env python3
"""Run the admitted EFF26 full-buffer foreground/shadow oracle and pin its results."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
from pixel_receipts import write_current_receipt

ROOT = Path(__file__).resolve().parents[2]
IMPLEMENTATION = ROOT / "rust/crates/genewars-desktop/src/bin/support/effect_scene.rs"
RESOURCE_CATALOG = ROOT / "research/integration/runtime-resource-catalog.tsv"
CONTRACT = ROOT / "research/ghidra/effect-draw-contract.json"
RESOURCE = ROOT / "private/integration/game-runtime/effects/EFF.ANB"
EXPECTED_RESOURCE_SHA256 = "f45f1201e50ce67199f1e1e8448d250f221cad71f1328ee44b7bacf9ee62c253"
EXPECTED_RESOURCE_SIZE = 186_726


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def main() -> int:
    resource = RESOURCE.read_bytes()
    if len(resource) != EXPECTED_RESOURCE_SIZE or sha(resource) != EXPECTED_RESOURCE_SHA256:
        raise ValueError("EFF.ANB does not match the pinned CD resource")
    catalog_line = next(line for line in RESOURCE_CATALOG.read_text(encoding="utf-8").splitlines()
                        if line.startswith("effects/EFF.ANB\t"))
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    source_pins = {"implementation": sha(IMPLEMENTATION.read_bytes()),
                   "validator": sha(Path(__file__).read_bytes()),
                   "contract": sha(CONTRACT.read_bytes()), "catalog_line": sha(catalog_line.encode())}
    args = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "test", "-p", "genewars-desktop",
            "--bin", "genewars-game", "effect_scene::tests::code0x1a_uses_its_five_pinned_frames_on_the_generic_shadow_and_main_path",
            "--", "--nocapture"]
    result = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=180)
    if result.returncode:
        raise RuntimeError(f"EFF26 pixel oracle failed ({result.returncode}):\n{result.stdout}\n{result.stderr}")
    cases = []
    for line in (result.stdout + "\n" + result.stderr).splitlines():
        match = re.fullmatch(r"EFF26_ORACLE frame=(\d+) sha256=([0-9a-f]{64}) shadow=(\d+) shadow_clipped=(\d+) color=(\d+) color_clipped=(\d+) effective_shade=(\d+) color_origin_y=(-?\d+)", line.strip())
        if match:
            names = ("frame", "sha256", "shadow_pixels", "shadow_clipped", "color_pixels",
                     "color_clipped", "effective_shade", "color_origin_y")
            case = {key: (int(value) if key != "sha256" else value)
                    for key, value in zip(names, match.groups())}
            cases.append(case)
    if [case["frame"] for case in cases] != [0, 2, 3, 4]:
        raise RuntimeError(f"expected four exact EFF26 frame oracle records, got {cases!r}")
    current_catalog_line = next(line for line in RESOURCE_CATALOG.read_text(encoding="utf-8").splitlines()
                                if line.startswith("effects/EFF.ANB\t"))
    if source_pins != {"implementation": sha(IMPLEMENTATION.read_bytes()),
                       "validator": sha(Path(__file__).read_bytes()),
                       "contract": sha(CONTRACT.read_bytes()), "catalog_line": sha(current_catalog_line.encode())}:
        raise RuntimeError("EFF26 source or resource pin changed while the Rust test was running")
    payload = {
        "schema_version": 1,
        "scope": "EFF effect code 0x1A, binding animation 26, four representative frames. Rust test compares complete 512x480 indexed framebuffers against a separately walked component/shadow/color model; no original executable run.",
        "resource": {"path": "effects/EFF.ANB", "sha256": EXPECTED_RESOURCE_SHA256,
                     "bytes": EXPECTED_RESOURCE_SIZE, "catalog_line_sha256": source_pins["catalog_line"]},
        "source_contract_sha256": source_pins["contract"],
        "implementation_sha256": source_pins["implementation"],
        "animation": {"index": 26, "first_frame_offset": 2030, "last_frame_offset": 2086,
                      "frame_count": 5, "hold_bytes": 1, "component_flags_by_frame": [0, 0, 0, 132, 4]},
        "draw_path": "generic code-6 shadow then normal indexed8 frame; ordinary FADE row 7; frame component flag 0x04 uses the proved MAIN.GLA destination blend; no special code-31 layer",
        "framebuffer": {"width": 512, "height": 480, "pitch": 512, "initial_index": 23},
        "pixel_oracle": {"full_buffer_cases": cases, "all_passed": True},
        "command": "python -B scripts/cargo.py test -p genewars-desktop --bin genewars-game effect_scene::tests::code0x1a_uses_its_five_pinned_frames_on_the_generic_shadow_and_main_path -- --nocapture",
    }
    receipt_name = sys.argv[1] if len(sys.argv) > 1 else "eff26-pixel-validation-current.json"
    if len(sys.argv) > 2 or Path(receipt_name).name != receipt_name or not receipt_name.endswith(".json"):
        raise ValueError("optional receipt name must be one plain .json filename")
    receipt = ROOT / "research/integration" / receipt_name
    write_current_receipt(receipt, payload)
    print(json.dumps({"receipt": str(receipt), "cases": cases}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

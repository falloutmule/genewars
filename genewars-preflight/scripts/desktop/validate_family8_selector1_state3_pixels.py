#!/usr/bin/env python3
"""Independent pixel oracle for a source-captured selector-1/state-3 frame."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import uuid

from pixel_receipts import write_current_receipt
from family8_source_receipts import sha

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/desktop"))
import validate_family8_state4_pixels as pixel_model  # noqa: E402

RESOURCE = ROOT / "private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
TRAJECTORY = ROOT / "private/review/family8-trajectory/e1ec8cf35b6e4f8fbaa276e4a0fee8f1"
POOL = TRAJECTORY / "frame6289/after/family8.bin"
RECORD = ROOT / "private/review/family8-return/actual-state3-slot6-before6290.bin"
RESULT = TRAJECTORY / "result.json"
DRAW_LISTING = ROOT / "private/ghidra/countdown-expiry/instructions-00010ea0.txt"
ORIENTATION_LISTING = ROOT / "private/ghidra/initial-creature/instructions-0001f060.txt"
HARNESS = ROOT / "scripts/desktop/family8_selector1_state3_harness.rs"
IMPLEMENTATION = ROOT / "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"
PINS = {
    RESOURCE: ("318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4", 91_123),
    FADE: ("b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392", 16_384),
    GLA: ("bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3", 65_536),
    POOL: ("ccef7dc9dc2adc993ebc3ef638c7c907401d7df9f3ad81fbac4a332140a79bd8", 16 * 102),
    RECORD: ("38ddbfa05597908a4b784b8bee0462eaf3747065c379ca5123a61ef19c68d6a9", 102),
}
LISTINGS = {
    "private/ghidra/countdown-expiry/instructions-00010ea0.txt": "639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb",
    "private/ghidra/initial-creature/instructions-0001f060.txt": "0554747598bbe72a344acf4e107f79f9e398a8db0f91ea27208f3bd3486a92d7",
}
WIDTH, HEIGHT, PITCH, FILL = 640, 480, 648, 0x35


def checked(path: Path, expected: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != expected or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def run_harness(run_dir: Path, record_path: Path) -> tuple[dict[str, dict[str, str]], Path]:
    run_dir.mkdir(parents=True, exist_ok=False)
    dep_root = (ROOT / "rust/crates").as_posix()
    manifest = run_dir / "Cargo.toml"
    manifest.write_text(
        "[package]\nname=\"family8-state3-pixel-probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
        f"\n[[bin]]\nname=\"family8-state3-pixel-probe\"\npath=\"{HARNESS.as_posix()}\"\n"
        "[dependencies]\n"
        f"genewars-assets={{path=\"{dep_root}/genewars-assets\"}}\n"
        f"genewars-render={{path=\"{dep_root}/genewars-render\"}}\n"
        f"genewars-runtime={{path=\"{dep_root}/genewars-runtime\"}}\n[workspace]\n", encoding="utf-8")
    out = run_dir / "pixels"
    args = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
            "--manifest-path", str(manifest), "--", str(RESOURCE), str(FADE), str(GLA), str(record_path), str(out)]
    proc = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=180)
    if proc.returncode:
        raise RuntimeError(f"state3 Rust harness failed ({proc.returncode}):\n{proc.stdout}\n{proc.stderr}")
    reports: dict[str, dict[str, str]] = {}
    for line in proc.stdout.splitlines():
        words = line.split()
        if words:
            reports[words[0]] = dict(token.split("=", 1) for token in words[1:])
    if set(reports) != {"state3_clipped", "state3_visible"}:
        raise RuntimeError(f"unexpected Rust harness output: {proc.stdout!r}")
    return reports, out


def main() -> int:
    inputs = {p: checked(p, digest, size) for p, (digest, size) in PINS.items()}
    result = json.loads(RESULT.read_text(encoding="utf-8"))
    expected_capture = PINS[POOL][0]
    if (result.get("status") != "boundary" or result.get("completed_frames") != 6289 or
            result.get("seed") != 5428 or result.get("original_execution") is not False or
            result.get("captures", {}).get("frame6289/after/family8.bin") != expected_capture):
        raise ValueError("state3 source trajectory identity/scope changed")
    for rel, digest in LISTINGS.items():
        checked(ROOT / rel, digest)
    source_pins = {"implementation": sha(IMPLEMENTATION.read_bytes()),
                   "harness": sha(HARNESS.read_bytes()), "validator": sha(Path(__file__).read_bytes())}
    pool = inputs[POOL]
    record = inputs[RECORD]
    if record != pool[6 * 102:7 * 102]:
        raise ValueError("captured return record differs from combined trajectory slot 6")
    if (record[6], record[0x55], record[0x56], record[0x15:0x17], record[0x19], record[0x1b]) != \
            (8, 1, 3, b"\x20\x00", 7, 0):
        raise ValueError("captured slot-6 state3 record does not match admitted source case")
    run_dir = ROOT / "private/integration/family8-state3-pixels" / uuid.uuid4().hex
    actual_reports, out = run_harness(run_dir, RECORD)
    summaries = []
    fade = inputs[FADE]
    gla = inputs[GLA]
    anb = inputs[RESOURCE]
    virtual = int.from_bytes(record[0x27:0x2b], "little", signed=True) - int.from_bytes(
        record[0x11:0x15], "little", signed=True)
    for name, origin in (("state3_clipped", (280, 170)), ("state3_visible", (280, 350))):
        # The independent common-path oracle is invoked with selector1/state0:
        # 0x10EA0's ordinary shadow/height/color route is textually complete and
        # returns before its later selector/state decoration logic.
        case = (f"selector1_direction{record[0x19]}", 1, 0, 0, origin, record[0x17], virtual)
        expected = pixel_model.oracle(anb, fade, gla, case)
        actual = (out / f"{name}.raw").read_bytes()
        if actual != expected["pixels"]:
            raise AssertionError(f"full {name} state3 framebuffer differs from independent common-path oracle")
        meta = actual_reports[name]
        for field in ("sha256", "shadow", "shadow_clipped", "color", "color_clipped", "virtual_height",
                      "effective_shade", "color_y", "global_animation"):
            if meta.get(field) != str(expected[field]):
                raise AssertionError(f"{name} {field}: Rust={meta.get(field)} Python={expected[field]}")
        summaries.append({"name": name, "origin": list(origin), "sha256": sha(actual),
                          "full_bytes_equal": True, "metadata": meta})
    if source_pins != {"implementation": sha(IMPLEMENTATION.read_bytes()),
                       "harness": sha(HARNESS.read_bytes()), "validator": sha(Path(__file__).read_bytes())}:
        raise RuntimeError("state3 pixel source changed while Cargo probe ran")
    payload = {
        "schema_version": 1,
        "scope": "Source-captured selector1/state3 slot6 record from the combined local trajectory at frame6289; the common 0x10EA0 indexed sprite path is compared to an independent full-buffer oracle. Diagnostic origins only; no original executable or GUI.",
        "trajectory": {"source_result": "private/review/family8-trajectory/e1ec8cf35b6e4f8fbaa276e4a0fee8f1/result.json",
                       "source_result_canonical": {"status": result["status"], "seed": result["seed"],
                                                   "completed_frames": result["completed_frames"],
                                                   "original_execution": False},
                       "captured_family8_sha256": PINS[POOL][0], "slot": 6,
                       "captured_record_path": "private/review/family8-return/actual-state3-slot6-before6290.bin",
                       "record_sha256": sha(record), "record_bytes": len(record),
                       "record_fields": {"kind": record[6], "selector": record[0x55], "state": record[0x56],
                                         "flags": int.from_bytes(record[0x15:0x17], "little"),
                                         "animation": record[0x19], "frame": record[0x1b],
                                         "ground": int.from_bytes(record[0x11:0x15], "little", signed=True),
                                         "physical": int.from_bytes(record[0x27:0x2b], "little", signed=True)}},
        "inputs": {"ethereal_sha256": PINS[RESOURCE][0], "fade_sha256": PINS[FADE][0],
                   "main_gla_sha256": PINS[GLA][0], "instruction_pins": LISTINGS},
        "implementation_sha256": source_pins["implementation"], "harness_sha256": source_pins["harness"],
        "validator_sha256": source_pins["validator"],
        "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH, "fill": FILL,
                         "bytes": (HEIGHT - 1) * PITCH + WIDTH},
        "cases": summaries,
    }
    receipt = ROOT / "research/integration/family8-selector1-state3-pixels-current.json"
    write_current_receipt(receipt, payload)
    print(json.dumps({"receipt": str(receipt), "full_bytes_equal": True, "cases": summaries}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

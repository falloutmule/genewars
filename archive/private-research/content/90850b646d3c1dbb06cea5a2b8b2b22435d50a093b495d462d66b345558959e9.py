#!/usr/bin/env python3
"""Full-frame oracle for the captured selector-1/state-3 return substate."""
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
RECORD = ROOT / "private/review/family8-return/actual6290/source-after6329.bin"
DRAW_LISTING = ROOT / "private/ghidra/countdown-expiry/instructions-00010ea0.txt"
ORIENTATION_LISTING = ROOT / "private/ghidra/initial-creature/instructions-0001f060.txt"
HARNESS = ROOT / "scripts/desktop/family8_selector1_state3_harness.rs"
IMPLEMENTATION = ROOT / "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"
PINS = {
    RESOURCE: ("318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4", 91_123),
    FADE: ("b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392", 16_384),
    GLA: ("bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3", 65_536),
    RECORD: ("7882526bfd84c88178fb5229e28decb8feb914082aa8e4f999ff73c1820bbd15", 102),
}
LISTINGS = {
    "private/ghidra/countdown-expiry/instructions-00010ea0.txt": "639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb",
    "private/ghidra/initial-creature/instructions-0001f060.txt": "0554747598bbe72a344acf4e107f79f9e398a8db0f91ea27208f3bd3486a92d7",
}
WIDTH, HEIGHT, PITCH, FILL = 640, 480, 648, 0x35
ORIGINS = ((280, 170), (280, 350))


def checked(path: Path, expected: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != expected or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def invoke(run_dir: Path, record_path: Path) -> tuple[dict[str, dict[str, str]], Path]:
    run_dir.mkdir(parents=True, exist_ok=False)
    dep_root = (ROOT / "rust/crates").as_posix()
    manifest = run_dir / "Cargo.toml"
    manifest.write_text(
        "[package]\nname=\"family8-state3-return-pixel-probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
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
        raise RuntimeError(f"return-substate Rust pixel probe failed ({proc.returncode}):\n{proc.stdout}\n{proc.stderr}")
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
    for rel, digest in LISTINGS.items():
        checked(ROOT / rel, digest)
    original = inputs[RECORD]
    if (original[6], original[0x55], original[0x56], original[0x15:0x17], original[0x19], original[0x1b]) != \
            (8, 1, 3, b"\x20\x40", 7, 0):
        raise ValueError("captured return-substate record no longer matches source evidence")
    pins_before = {"implementation": sha(IMPLEMENTATION.read_bytes()),
                   "harness": sha(HARNESS.read_bytes()), "validator": sha(Path(__file__).read_bytes())}
    run_dir = ROOT / "private/integration/family8-state3-return-pixels" / uuid.uuid4().hex
    run_dir.mkdir(parents=True, exist_ok=False)
    # The captured record is direction 7. Other directions are covered by the
    # module's synthetic equivalence test; this oracle stays source-captured.
    cases = []
    reports, out = invoke(run_dir / "captured-direction-7", RECORD)
    for name, origin in zip(("state3_clipped", "state3_visible"), ORIGINS):
        virtual = int.from_bytes(original[0x27:0x2b], "little", signed=True) - int.from_bytes(
            original[0x11:0x15], "little", signed=True)
        expected = pixel_model.oracle(inputs[RESOURCE], inputs[FADE], inputs[GLA],
            ("selector1_direction7", 1, 0, 0, origin, original[0x17], virtual))
        actual = (out / f"{name}.raw").read_bytes()
        if actual != expected["pixels"]:
            raise AssertionError(f"full state3 return framebuffer mismatch: case={name}")
        meta = reports[name]
        for field in ("sha256", "shadow", "shadow_clipped", "color", "color_clipped", "virtual_height",
                      "effective_shade", "color_y", "global_animation"):
            if meta.get(field) != str(expected[field]):
                raise AssertionError(f"{name} {field}: Rust={meta.get(field)} Python={expected[field]}")
        cases.append({"direction": 7, "case": name, "origin": list(origin),
                      "sha256": sha(actual), "full_bytes_equal": True,
                      "record_is_captured": True, "metadata": meta})
    if pins_before != {"implementation": sha(IMPLEMENTATION.read_bytes()),
                       "harness": sha(HARNESS.read_bytes()), "validator": sha(Path(__file__).read_bytes())}:
        raise RuntimeError("state3 return pixel source changed while Cargo probe ran")
    payload = {
        "schema_version": 1,
        "scope": "Captured selector1/state3/substate2 flags 0x4020 record at source after6329. Two 640x480 full framebuffers are compared byte-for-byte with an independent 0x10EA0 common-path oracle; origins are diagnostic only. The Rust module separately checks flag equivalence across all eight direction indices.",
        "captured_record": {"path": "private/review/family8-return/actual6290/source-after6329.bin",
                            "sha256": PINS[RECORD][0], "bytes": len(original), "kind": original[6],
                            "selector": original[0x55], "state": original[0x56],
                            "flags": int.from_bytes(original[0x15:0x17], "little"),
                            "direction": original[0x19], "frame": original[0x1b]},
        "inputs": {"ethereal_sha256": PINS[RESOURCE][0], "fade_sha256": PINS[FADE][0],
                   "main_gla_sha256": PINS[GLA][0], "instruction_pins": LISTINGS},
        "implementation_sha256": pins_before["implementation"], "harness_sha256": pins_before["harness"],
        "validator_sha256": pins_before["validator"],
        "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH, "fill": FILL,
                        "bytes": (HEIGHT - 1) * PITCH + WIDTH},
        "cases": cases,
    }
    receipt = ROOT / "research/integration/family8-selector1-state3-return-pixels-current.json"
    write_current_receipt(receipt, payload)
    print(json.dumps({"receipt": str(receipt), "full_bytes_equal": True, "case_count": len(cases),
                      "captured_direction": 7, "captured_flags": "0x4020"}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

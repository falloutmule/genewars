#!/usr/bin/env python3
"""Full-buffer oracle for an onward selector-1/state-0 direction-4 frame."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import uuid

from pixel_receipts import write_current_receipt
from family8_source_receipts import validate_onward_source

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/desktop"))
import validate_family8_state4_pixels as pixel_model  # noqa: E402

ETHEREAL = ROOT / "private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
RECORD = ROOT / "private/review/family8-onward/child-after6046.bin"
TIMELINE = ROOT / "private/review/family8-onward/source.json"
SOURCE_MODEL = ROOT / "research/variants/family8-onward-source.py"
ORIENTATION_LISTING = ROOT / "private/ghidra/initial-creature/instructions-0001f060.txt"
DRAW_LISTING = ROOT / "private/ghidra/countdown-expiry/instructions-00010ea0.txt"
HARNESS = ROOT / "scripts/desktop/family8_selector1_direction4_harness.rs"
IMPLEMENTATION = ROOT / "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"
PINS = {
    ETHEREAL: ("318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4", 91_123),
    FADE: ("b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392", 16_384),
    GLA: ("bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3", 65_536),
    RECORD: ("ad599c9be03f6adf54625c3431ed88b1a15fa8cba93970dbbc3c1ad804238a23", 102),
}
OTHER_PINS = {
    SOURCE_MODEL: "dc4ca79d79222e27cbe945f7aa87d80c28ea50a65922c1648f49e23518199ab2",
    ORIENTATION_LISTING: "0554747598bbe72a344acf4e107f79f9e398a8db0f91ea27208f3bd3486a92d7",
    DRAW_LISTING: "639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb",
}
WIDTH, HEIGHT, PITCH, FILL = 640, 480, 648, 0x35


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def checked(path: Path, expected_sha: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != expected_sha or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def run_probe(run_dir: Path) -> tuple[dict[str, dict[str, str]], Path]:
    run_dir.mkdir(parents=True, exist_ok=False)
    dep_root = (ROOT / "rust/crates").as_posix()
    manifest = run_dir / "Cargo.toml"
    manifest.write_text(
        "[package]\nname=\"family8-selector1-direction4-probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
        f"[[bin]]\nname=\"family8-selector1-direction4-probe\"\npath=\"{HARNESS.as_posix()}\"\n"
        "[dependencies]\n"
        f"genewars-assets={{path=\"{dep_root}/genewars-assets\"}}\n"
        f"genewars-render={{path=\"{dep_root}/genewars-render\"}}\n"
        f"genewars-runtime={{path=\"{dep_root}/genewars-runtime\"}}\n[workspace]\n", encoding="utf-8")
    out = run_dir / "pixels"
    args = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
            "--manifest-path", str(manifest), "--", str(ETHEREAL), str(FADE), str(GLA), str(RECORD), str(out)]
    proc = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, timeout=180)
    if proc.returncode:
        raise RuntimeError(f"direction4 Rust probe failed ({proc.returncode}):\n{proc.stdout}\n{proc.stderr}")
    metadata = {}
    for line in proc.stdout.splitlines():
        words = line.split()
        if words:
            metadata[words[0]] = dict(word.split("=", 1) for word in words[1:])
    if set(metadata) != {"direction4", "direction4_visible"}:
        raise RuntimeError(f"unexpected probe receipt: {proc.stdout!r}")
    return metadata, out


def main() -> int:
    inputs = {path: checked(path, digest, size) for path, (digest, size) in PINS.items()}
    other = {path: checked(path, digest) for path, digest in OTHER_PINS.items()}
    timeline = json.loads(TIMELINE.read_text(encoding="utf-8"))
    source_identity = validate_onward_source(ROOT, timeline)
    record = inputs[RECORD]
    fields = (record[6], record[0x55], record[0x56], record[0x19], record[0x1b],
              int.from_bytes(record[0x15:0x17], "little"))
    if fields != (8, 1, 0, 4, 0, 0x4020):
        raise ValueError(f"literal onward selector1 state0 record changed: {fields!r}")
    source_pins = {"implementation": sha(IMPLEMENTATION.read_bytes()),
                   "oracle": sha(Path(__file__).read_bytes()), "harness": sha(HARNESS.read_bytes())}
    physical = int.from_bytes(record[0x27:0x2b], "little", signed=True)
    ground = int.from_bytes(record[0x11:0x15], "little", signed=True)
    run_dir = ROOT / "private/integration/family8-selector1-direction4" / uuid.uuid4().hex
    actual_metadata, out_dir = run_probe(run_dir)
    summaries = []
    for name, origin in (("direction4", (120, 115)), ("direction4_visible", (120, 350))):
        case = ("selector1_direction4", 1, 0, 0, origin, record[0x17], physical - ground)
        expected = pixel_model.oracle(inputs[ETHEREAL], inputs[FADE], inputs[GLA], case)
        actual = (out_dir / f"{name}.raw").read_bytes()
        actual_meta = actual_metadata[name]
        if actual != expected["pixels"]: raise AssertionError(f"full {name} framebuffer differs from independent oracle")
        for key in ("sha256", "shadow", "shadow_clipped", "color", "color_clipped", "virtual_height",
                    "effective_shade", "color_y", "global_animation"):
            if actual_meta.get(key) != str(expected[key]):
                raise AssertionError(f"{name} {key}: Rust={actual_meta.get(key)} Python={expected[key]}")
        summaries.append({"name": name, "origin": list(origin), "full_bytes_equal": True,
                          "sha256": sha(actual), "metadata": actual_meta})
    if source_pins != {"implementation": sha(IMPLEMENTATION.read_bytes()),
                       "oracle": sha(Path(__file__).read_bytes()), "harness": sha(HARNESS.read_bytes())}:
        raise RuntimeError("pixel-validator source changed while the Rust harness was running")
    payload = {
        "schema_version": 1,
        "scope": "Exact literal selector-1 state-0 child record at local animation 4/frame 0; source-mapped global animation 4; independent complete indexed framebuffer oracle. Diagnostic placement [120,115] is explicit and not a world projection claim.",
        "record_sha256": PINS[RECORD][0], "record_bytes": 102, "fields": {
            "kind": record[6], "selector": record[0x55], "state": record[0x56],
            "entity_flags": int.from_bytes(record[0x15:0x17], "little"),
            "local_animation": record[0x19], "frame": record[0x1b], "physical_height": physical,
            "ground_height": ground, "raw_shade": record[0x17]},
        "source_timeline": source_identity, "source_model_sha256": OTHER_PINS[SOURCE_MODEL],
        "instruction_pins": {str(path.relative_to(ROOT)): digest for path, digest in OTHER_PINS.items()
                             if path in (ORIENTATION_LISTING, DRAW_LISTING)},
        "resource_sha256": PINS[ETHEREAL][0], "fade_sha256": PINS[FADE][0], "main_gla_sha256": PINS[GLA][0],
        "implementation_sha256": source_pins["implementation"], "harness_sha256": source_pins["harness"],
        "oracle_sha256": source_pins["oracle"],
        "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH, "fill": FILL, "bytes": len(actual)},
        "result": {"full_bytes_equal": True, "cases": summaries},
    }
    receipt_name = sys.argv[1] if len(sys.argv) > 1 else "family8-selector1-direction4-pixel-validation-current.json"
    if len(sys.argv) > 2 or Path(receipt_name).name != receipt_name or not receipt_name.endswith(".json"):
        raise ValueError("optional receipt name must be one plain .json filename")
    receipt = ROOT / "research/integration" / receipt_name
    write_current_receipt(receipt, payload)
    print(json.dumps({"receipt": str(receipt), "run_directory": str(run_dir), "full_bytes_equal": True,
                      "cases": summaries}, indent=2))
    return 0


if __name__ == "__main__": raise SystemExit(main())

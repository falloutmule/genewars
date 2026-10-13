#!/usr/bin/env python3
"""Independent framebuffer oracle for one literal selector-0/state-5 record."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import uuid

from pixel_receipts import write_current_receipt
from family8_source_receipts import validate_state5_source

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/desktop"))
import validate_family8_state4_pixels as base  # noqa: E402

ETHEREAL = ROOT / "private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
RECORD = ROOT / "private/review/family8-state4/state5-start6002-seed5428.bin"
SOURCE_TIMELINE = ROOT / "private/review/family8-state4/source.json"
STATE5_REVIEW = ROOT / "research/variants/family8-state4-review.json"
LISTING = ROOT / "private/ghidra/countdown-expiry/instructions-00010ea0.txt"
HARNESS = ROOT / "scripts/desktop/family8_state5_pixel_harness.rs"
IMPLEMENTATION = ROOT / "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"
EXPECTED_HASHES = {
    ETHEREAL: ("318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4", 91_123),
    FADE: ("b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392", 16_384),
    GLA: ("bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3", 65_536),
    RECORD: ("f54530d8bb8e04aa74347e346d60f9d08eb3abe72e5b21afed1af4ffeee130b0", 102),
}
LISTING_SHA = "639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb"
WIDTH, HEIGHT, PITCH, FILL = 640, 480, 648, 0x35


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path, expected_sha: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != expected_sha or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def run_probe(run_dir: Path, paths: list[Path]) -> tuple[dict[str, str], bytes]:
    run_dir.mkdir(parents=True, exist_ok=False)
    dep_root = (ROOT / "rust/crates").as_posix()
    source = HARNESS.as_posix()
    manifest = run_dir / "Cargo.toml"
    manifest.write_text(
        "[package]\nname=\"family8-state5-pixel-probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
        f"[[bin]]\nname=\"family8-state5-pixel-probe\"\npath=\"{source}\"\n"
        "[dependencies]\n"
        f"genewars-assets={{path=\"{dep_root}/genewars-assets\"}}\n"
        f"genewars-render={{path=\"{dep_root}/genewars-render\"}}\n"
        f"genewars-runtime={{path=\"{dep_root}/genewars-runtime\"}}\n[workspace]\n",
        encoding="utf-8")
    out = run_dir / "pixels"
    command = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
               "--manifest-path", str(manifest), "--", *(str(p) for p in paths), str(out)]
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
    if result.returncode:
        raise RuntimeError(f"state5 Rust harness failed ({result.returncode}):\n{result.stdout}\n{result.stderr}")
    words = result.stdout.strip().split()
    if not words or words[0] != "state5":
        raise RuntimeError(f"unexpected harness receipt: {result.stdout!r}")
    metadata = dict(token.split("=", 1) for token in words[1:])
    return metadata, (out / "state5.raw").read_bytes()


def main() -> int:
    for path, (digest, size) in EXPECTED_HASHES.items():
        pin(path, digest, size)
    pin(LISTING, LISTING_SHA)
    timeline = json.loads(SOURCE_TIMELINE.read_text(encoding="utf-8"))
    review = json.loads(STATE5_REVIEW.read_text(encoding="utf-8"))
    source_identity = validate_state5_source(ROOT, timeline, review)
    record = RECORD.read_bytes()
    if (record[6], record[0x55], record[0x56], record[0x19], record[0x1b],
            int.from_bytes(record[0x15:0x17], "little")) != (8, 0, 5, 0, 6, 0x20):
        raise ValueError("literal state5 record fields changed")
    ground = int.from_bytes(record[0x11:0x15], "little", signed=True)
    physical = int.from_bytes(record[0x27:0x2b], "little", signed=True)
    virtual = physical - ground
    raw_shade = record[0x17]
    # State 5 uses the common ordinary wrapper and animation-8 frame table.
    # Adapt to the state-4-independent pixel model, which exercises the same
    # wrapper math while this check separately pins the state-5 admission bytes.
    case = ("state5", 0, 4, record[0x1b], (280, 170), raw_shade, virtual)
    expected = base.oracle(pin(ETHEREAL, EXPECTED_HASHES[ETHEREAL][0], 91_123),
                           pin(FADE, EXPECTED_HASHES[FADE][0], 16_384),
                           pin(GLA, EXPECTED_HASHES[GLA][0], 65_536), case)
    source_pins = {"implementation": sha(IMPLEMENTATION.read_bytes()),
                   "oracle": sha(Path(__file__).read_bytes()), "harness": sha(HARNESS.read_bytes())}
    run_dir = ROOT / "private/integration/family8-state5-pixels" / uuid.uuid4().hex
    actual_meta, actual = run_probe(run_dir, [ETHEREAL, FADE, GLA, RECORD])
    if actual != expected["pixels"]:
        raise AssertionError("complete state5 framebuffer differs from independent byte oracle")
    for key in ("sha256", "shadow", "shadow_clipped", "color", "color_clipped",
                "virtual_height", "effective_shade", "color_y", "global_animation"):
        if actual_meta.get(key) != str(expected[key]):
            raise AssertionError(f"state5 {key}: Rust={actual_meta.get(key)} Python={expected[key]}")
    if source_pins != {"implementation": sha(IMPLEMENTATION.read_bytes()),
                       "oracle": sha(Path(__file__).read_bytes()), "harness": sha(HARNESS.read_bytes())}:
        raise RuntimeError("pixel-validator source changed while the Rust harness was running")
    payload = {
        "schema_version": 1,
        "scope": "Literal selector-0/state-5 source record with explicit diagnostic origin; independent full framebuffer oracle for the common 0x10EA0 wrapper. No original code executed.",
        "source_record": {"sha256": EXPECTED_HASHES[RECORD][0], "bytes": 102,
                          "fields": {"kind": 8, "selector": 0, "state": 5, "flags": 32,
                                     "animation": 0, "frame": 6, "physical_height": physical,
                                     "ground_height": ground}},
        "source_replay": {"canonical_identity": source_identity, "review_callbacks": 276,
                          "review_status": review["status"], "literal_callbacks": 276},
        "resource_sha256": EXPECTED_HASHES[ETHEREAL][0],
        "fade_sha256": EXPECTED_HASHES[FADE][0], "main_gla_sha256": EXPECTED_HASHES[GLA][0],
        "instruction_pins": {"private/ghidra/countdown-expiry/instructions-00010ea0.txt": LISTING_SHA},
        "implementation_sha256": source_pins["implementation"],
        "harness_sha256": source_pins["harness"], "oracle_sha256": source_pins["oracle"],
        "origin": [280, 170], "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH,
                                                  "fill": FILL, "bytes": len(actual)},
        "comparison": {"sha256": sha(actual), "full_bytes_equal": True,
                       "metadata": actual_meta},
    }
    receipt_name = sys.argv[1] if len(sys.argv) > 1 else "family8-state5-pixel-validation-current.json"
    if len(sys.argv) > 2 or Path(receipt_name).name != receipt_name or not receipt_name.endswith(".json"):
        raise ValueError("optional receipt name must be one plain .json filename")
    receipt = ROOT / "research/integration" / receipt_name
    write_current_receipt(receipt, payload)
    print(json.dumps({"receipt": str(receipt), "run_directory": str(run_dir),
                      "full_bytes_equal": True, "metadata": actual_meta}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

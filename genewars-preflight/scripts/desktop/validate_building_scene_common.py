#!/usr/bin/env python3
"""Independent full-buffer check for the desktop BTURR main-image adapter."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/desktop"))
import validate_bturr_common_pixels as oracle_module  # noqa: E402
from pixel_receipts import write_current_receipt  # noqa: E402

ANB = ROOT / "private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
HARNESS = ROOT / "scripts/desktop/building_scene_pixel_harness.rs"
ORACLE_SOURCE = ROOT / "scripts/desktop/validate_bturr_common_pixels.py"
ADAPTER = ROOT / "rust/crates/genewars-desktop/src/bin/support/building_scene.rs"
RESOURCE = ROOT / "rust/crates/genewars-runtime/src/building_resources.rs"
ACTOR_RENDER = ROOT / "rust/crates/genewars-runtime/src/actor_render.rs"
INPUTS = {
    ANB: ("c26df60af3a3166aec54112f197a4334966eb214eeaa48e0d9284e0510b8cd20", 68_030),
    FADE: ("b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392", 16_384),
    GLA: ("bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3", 65_536),
}
LISTINGS = {
    "private/ghidra/building-draw/instructions-00043a40.txt": "62a4377cb232402be9cf5a1fb0c7da61d29cc582632c60c217b0357e2961ff1b",
    "private/ghidra/actor-draw/instructions-00095390.txt": "8ae130718f8bb1c699ccfc7e44a6a83fc9f41c10770fb5b9b4050efc96bfcfa5",
    "private/ghidra/entity-plant/instructions-0009a660.txt": "f6e10c86aa6ce6cf6b7906b26279ffc9baf4142fd81b0f99779f58b40b3001ee",
}
WIDTH, HEIGHT, PITCH, FILL = 180, 180, 192, 0x35
oracle_module.WIDTH, oracle_module.HEIGHT, oracle_module.PITCH, oracle_module.FILL = WIDTH, HEIGHT, PITCH, FILL


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def check(path: Path, expected: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != expected or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def main() -> int:
    raw = {path: check(path, digest, size) for path, (digest, size) in INPUTS.items()}
    for relative, digest in LISTINGS.items():
        check(ROOT / relative, digest)
    source_paths = [ADAPTER, RESOURCE, ACTOR_RENDER, HARNESS, ORACLE_SOURCE, Path(__file__)]
    source_pins = {path.relative_to(ROOT).as_posix(): sha(path.read_bytes()) for path in source_paths}
    run_dir = ROOT / "private/integration/building-scene-common" / uuid.uuid4().hex
    run_dir.mkdir(parents=True, exist_ok=False)
    crates = (ROOT / "rust/crates").as_posix()
    manifest = run_dir / "Cargo.toml"
    manifest.write_text(
        "[package]\nname=\"building-scene-pixel-probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
        f"\n[[bin]]\nname=\"building-scene-pixel-probe\"\npath=\"{HARNESS.as_posix()}\"\n"
        "[dependencies]\n"
        f"genewars-render={{path=\"{crates}/genewars-render\"}}\n"
        f"genewars-runtime={{path=\"{crates}/genewars-runtime\"}}\n[workspace]\n",
        encoding="utf-8",
    )
    output = run_dir / "frames"
    command = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
               "--manifest-path", str(manifest), "--", str(ANB), str(output)]
    process = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
    if process.returncode:
        raise RuntimeError(f"building scene harness failed ({process.returncode}):\n{process.stdout}\n{process.stderr}")
    reports = {}
    for line in process.stdout.splitlines():
        fields = line.split()
        if fields:
            reports[int(fields[0])] = dict(item.split("=", 1) for item in fields[1:])
    if set(reports) != {0, 1, 2}:
        raise AssertionError(f"expected three frame receipts, got {reports}")

    summaries = []
    for frame in range(3):
        case = {"id": frame, "frame": frame, "owner": 2, "route": "direct_30", "shade": 30,
                "actor_flags": 0, "global_flags": 0, "origin_name": "common", "origin": [2, 3], "phase": 0}
        expected = oracle_module.oracle(raw[ANB], raw[FADE], raw[GLA], case)
        actual = (output / f"frame{frame}.raw").read_bytes()
        report = reports[frame]
        if actual != expected["pixels"]:
            raise AssertionError(f"frame {frame} complete framebuffer differs from independent BTURR oracle")
        for key, expected_value in {
            "sha": expected["sha256"], "written": expected["written"], "clipped": expected["clipped"],
            "opaque": expected["opaque"], "x": expected["draw_origin"][0], "y": expected["draw_origin"][1],
            "shadow": "false", "post_overlays": "false",
        }.items():
            if report.get(key) != str(expected_value):
                raise AssertionError(f"frame {frame} {key}: wrapper={report.get(key)} oracle/receipt={expected_value}")
        summaries.append({"frame": frame, "hash": expected["sha256"], "components": expected["components"],
                          "opaque": expected["opaque"], "written": expected["written"], "clipped": expected["clipped"],
                          "full_addressed_buffer_equal": True, "omissions": {"shadow_pass_included": False,
                          "post_image_overlays_included": False}})

    if source_pins != {path.relative_to(ROOT).as_posix(): sha(path.read_bytes()) for path in source_paths}:
        raise RuntimeError("building-scene code changed while the harness ran")
    receipt = {
        "schema_version": 1,
        "status": "passed-independent-building-scene-main-image-oracle",
        "scope": "The desktop adapter forwards the three hash-bound BTURR animation-0 frames through the runtime main-image helper. It does not claim shadow, selection, owner, or post-image UI overlays; each omission is explicit in the returned receipt.",
        "input_sha256": {path.relative_to(ROOT).as_posix(): digest for path, (digest, _size) in INPUTS.items()},
        "instruction_pins": LISTINGS,
        "code_pins": source_pins,
        "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH, "bytes_per_frame": (HEIGHT - 1) * PITCH + WIDTH},
        "frames": summaries,
    }
    write_current_receipt(ROOT / "research/integration/building-scene-common-pixels-current.json", receipt)
    print(json.dumps({"status": receipt["status"], "frames": len(summaries),
                      "frame_hashes": [item["hash"] for item in summaries],
                      "omissions_explicit": True,
                      "receipt": "research/integration/building-scene-common-pixels-current.json"}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

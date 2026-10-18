"""Independent BONES selector-4 owner-4 framebuffer oracle.

The Python side parses the pinned ANB/RLE bytes and models the common
59470/94F30/95100/93F80/937A0 paths independently. It compares complete
pitched destination buffers emitted by the bounded Rust adapter; this is not
an original-execution screenshot comparison.
"""
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))
from cargo import environment

MULE_ORACLE_PATH = ROOT / "scripts" / "validate_mule_scene.py"
spec = importlib.util.spec_from_file_location("mule_oracle", MULE_ORACLE_PATH)
mule_oracle = importlib.util.module_from_spec(spec)
assert spec and spec.loader
spec.loader.exec_module(mule_oracle)

SHA = lambda data: hashlib.sha256(data).hexdigest()
BONES_SHA = "2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f"
FADE_SHA = "b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392"
GLA_SHA = "bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3"


def pin(path: Path) -> dict[str, object]:
    data = path.read_bytes()
    return {"path": path.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": SHA(data)}


def main() -> None:
    resources = ROOT / "private" / "integration" / "game-runtime"
    anb_path = resources / "creatures" / "BONES.ANB"
    fade_path = resources / "terrain" / "FADE.DAT"
    gla_path = resources / "terrain" / "MAIN.GLA"
    anb, fade, gla = anb_path.read_bytes(), fade_path.read_bytes(), gla_path.read_bytes()
    assert len(anb) == 4077 and SHA(anb) == BONES_SHA
    assert len(fade) == 64 * 256 and SHA(fade) == FADE_SHA
    assert len(gla) == 256 * 256 and SHA(gla) == GLA_SHA

    sources = [
        ROOT / "scripts" / "desktop" / "validate_bones_pixels.py",
        MULE_ORACLE_PATH,
        ROOT / "rust" / "crates" / "genewars-desktop" / "src" / "bin" / "support" / "creature_scene.rs",
        ROOT / "rust" / "crates" / "genewars-desktop" / "src" / "bin" / "support" / "runtime_resources.rs",
        ROOT / "research" / "ghidra" / "bones-evidence.json",
        ROOT / "research" / "formats" / "mule-render-contract.md",
        ROOT / "research" / "formats" / "bones-render-contract.md",
        ROOT / "private" / "ghidra" / "terrain-viewport" / "instructions-00059470.txt",
        ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00094f30.txt",
        ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00095100.txt",
        ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00093f80.txt",
        ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-000937a0.txt",
        ROOT / "private" / "ghidra" / "display-geometry" / "instructions-0008f8f0.txt",
        ROOT / "private" / "integration" / "game-runtime" / "creatures" / "BONES.ANB",
        ROOT / "private" / "integration" / "game-runtime" / "terrain" / "FADE.DAT",
        ROOT / "private" / "integration" / "game-runtime" / "terrain" / "MAIN.GLA",
        ROOT / "private" / "review" / "mule-bones" / "expected.bin",
        ROOT / "private" / "review" / "mule-bones" / "wrapper-trace.json",
    ]
    pins_before = [pin(path) for path in sources]

    scratch = ROOT / "private" / "review" / f"bones-pixel-oracle-{uuid.uuid4().hex}"
    scratch.mkdir(parents=True, exist_ok=False)
    rust_output = scratch / "rust.jsonl"
    env = os.environ.copy()
    env["GENEWARS_BONES_ORACLE_OUT"] = str(rust_output)
    command = [sys.executable, str(ROOT / "scripts" / "cargo.py"), "test",
               "--offline", "--locked", "-p", "genewars-desktop", "--bin",
               "genewars-game", "creature_scene::tests::export_source_bones_pixel_oracle_cases_when_requested"]
    result = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
    if result.returncode:
        raise RuntimeError(f"Rust BONES fixture rendering failed:\n{result.stdout[-3500:]}\n{result.stderr[-3500:]}")
    actual_lines = rust_output.read_text(encoding="utf-8").splitlines()
    if len(actual_lines) != 2:
        raise AssertionError(f"expected two BONES wrapper cases, got {len(actual_lines)}")

    total_bytes = 0
    foreground_pixels = shadow_pixels = 0
    for line in actual_lines:
        case = json.loads(line)
        actual = bytes.fromhex(case.pop("pixels"))
        expected = mule_oracle.make_surface(case, anb, fade, gla)
        if actual != expected:
            mismatch = next((i for i, (got, want) in enumerate(zip(actual, expected)) if got != want), None)
            raise AssertionError(f"BONES state{case['state']} frame{case['frame']} mismatch at {mismatch}")
        total_bytes += len(expected)
        foreground_pixels += case["fg_written"]
        shadow_pixels += case["sh_written"]

    pins = [pin(path) for path in sources]
    assert pins_before == pins, "a pinned input changed during the pixel comparison"
    receipt = {
        "status": "passed_bones_owner4_wrapper_pixel_oracle",
        "scope": "Independent Python ANB/RLE, scale, flip, fade, and GLA model compared against complete Rust destination buffers; not original-execution parity.",
        "cases": 2,
        "states_frames": [[26, 0], [27, 1]],
        "framebuffer_bytes_compared": total_bytes,
        "foreground_pixels_reported": foreground_pixels,
        "shadow_pixels_reported": shadow_pixels,
        "resources": {"bones_sha256": BONES_SHA, "fade_sha256": FADE_SHA, "main_gla_sha256": GLA_SHA},
        "source_pins": pins,
        "private_rust_output": pin(rust_output),
        "limitations": [
            "Owner 4 only: this matches the 59470 branch to 94F30; owner 0..3 use 94D70 and remain unsupported.",
            "Caller projection and global wrapper inputs are explicit; no full visibility, queue, or scene-order claim.",
        ],
    }
    output = ROOT / "research" / "integration" / "bones-pixel-validation-current.json"
    temporary = output.with_suffix(".json.tmp")
    temporary.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    temporary.replace(output)
    print(json.dumps({key: receipt[key] for key in ["status", "cases", "framebuffer_bytes_compared", "foreground_pixels_reported", "shadow_pixels_reported"]}))


if __name__ == "__main__":
    main()

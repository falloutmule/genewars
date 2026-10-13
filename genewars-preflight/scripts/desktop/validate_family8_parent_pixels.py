#!/usr/bin/env python3
"""Compare selector-zero parent state 8/9 draws with an independent byte oracle."""
from __future__ import annotations

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import uuid

from pixel_receipts import write_current_receipt

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts/desktop"))
import validate_family8_state4_pixels as oracle_module  # noqa: E402

ETHEREAL = ROOT / "private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
STATE8 = ROOT / "private/review/family8-parent/state8-after-notification6442.bin"
STATE9 = ROOT / "private/review/family8-parent/state9-after-group-ready6443.bin"
LISTING = ROOT / "private/ghidra/countdown-expiry/instructions-00010ea0.txt"
HARNESS = ROOT / "scripts/desktop/family8_parent_pixel_harness.rs"
IMPLEMENTATION = ROOT / "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs"
REVIEW = ROOT / "research/variants/family8-parent-review.json"
RESOURCE_SHA = "318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4"
FADE_SHA = "b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392"
GLA_SHA = "bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3"
RECORD_HASHES = {
    STATE8: "d9ecf37055c79073615d0660dd7cd455b3e8ff4cb22d70263121b4286442b5e4",
    STATE9: "7f27f8f734bef035382c08163b5e402875992f231bec2c154617bf6d8a444e87",
}
LISTING_SHA = "639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb"
ORIGIN = (280, 430)
WIDTH, HEIGHT, PITCH = 640, 480, 648


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_pin(path: Path, expected: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != expected or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def run_probe(run_dir: Path, paths: list[Path]) -> tuple[dict[str, str], dict[str, bytes]]:
    run_dir.mkdir(parents=True, exist_ok=False)
    dep_root = (ROOT / "rust/crates").as_posix()
    manifest = run_dir / "Cargo.toml"
    manifest.write_text(
        "[package]\nname=\"family8-parent-pixel-probe\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
        f"\n[[bin]]\nname=\"family8-parent-pixel-probe\"\npath=\"{HARNESS.as_posix()}\"\n"
        "[dependencies]\n"
        f"genewars-assets={{path=\"{dep_root}/genewars-assets\"}}\n"
        f"genewars-render={{path=\"{dep_root}/genewars-render\"}}\n"
        f"genewars-runtime={{path=\"{dep_root}/genewars-runtime\"}}\n[workspace]\n",
        encoding="utf-8",
    )
    out = run_dir / "pixels"
    result = subprocess.run(
        [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
         "--manifest-path", str(manifest), "--", *(str(path) for path in paths), str(out)],
        cwd=ROOT, capture_output=True, text=True, timeout=180,
    )
    if result.returncode:
        raise RuntimeError(f"parent pixel Rust harness failed ({result.returncode}):\n{result.stdout}\n{result.stderr}")
    words = result.stdout.strip().split()
    metadata: dict[str, str] = {}
    for line in result.stdout.splitlines():
        items = line.split()
        if not items:
            continue
        metadata[items[0]] = dict(token.split("=", 1) for token in items[1:])
    if set(metadata) != {"state8", "state9"} or not words:
        raise RuntimeError(f"unexpected Rust harness receipt: {result.stdout!r}")
    return metadata, {name: (out / f"{name}.raw").read_bytes() for name in metadata}


def main() -> int:
    resource = read_pin(ETHEREAL, RESOURCE_SHA, 91_123)
    fade = read_pin(FADE, FADE_SHA, 16_384)
    gla = read_pin(GLA, GLA_SHA, 65_536)
    records: dict[str, tuple[bytes, dict[str, int]]] = {}
    for path, expected_state in ((STATE8, 8), (STATE9, 9)):
        record = read_pin(path, RECORD_HASHES[path], 102)
        fields = {
            "kind": record[6], "substate": record[0x55], "state": record[0x56],
            "flags": int.from_bytes(record[0x15:0x17], "little"), "selector": record[0x2d],
            "animation": record[0x19], "frame": record[0x1b], "shade": record[0x17],
            "physical_height": int.from_bytes(record[0x27:0x2b], "little", signed=True),
            "ground_height": int.from_bytes(record[0x11:0x15], "little", signed=True),
        }
        if (fields["kind"], fields["substate"], fields["state"], fields["flags"], fields["selector"],
            fields["animation"], fields["frame"]) != (8, 0, expected_state, 0x0020, 0, 0, 2):
            raise ValueError(f"source state {expected_state} no longer matches narrow parent draw admission")
        records[f"state{expected_state}"] = (record, fields)

    source_paths = [IMPLEMENTATION, HARNESS, Path(__file__), LISTING, REVIEW, STATE8, STATE9, ETHEREAL, FADE, GLA]
    source_pins = {path.relative_to(ROOT).as_posix(): sha(path.read_bytes()) for path in source_paths}
    if source_pins[LISTING.relative_to(ROOT).as_posix()] != LISTING_SHA:
        raise ValueError("0x10EA0 draw listing pin mismatch")

    expected: dict[str, dict] = {}
    for name, (record, fields) in records.items():
        # The independent state-4 oracle's state predicate is intentionally
        # bypassed by aliasing this case to its common wrapper arithmetic. The
        # direct 0x10EA0 listing sends state8/9 with substate0 to that same path.
        common_case = (name, 0, 4, fields["frame"], ORIGIN, fields["shade"],
                       fields["physical_height"] - fields["ground_height"])
        expected[name] = oracle_module.oracle(resource, fade, gla, common_case)

    run_dir = ROOT / "private/integration/family8-parent-pixels" / uuid.uuid4().hex
    actual_meta, actual_pixels = run_probe(run_dir, [ETHEREAL, FADE, GLA, STATE8, STATE9])
    if source_pins != {path.relative_to(ROOT).as_posix(): sha(path.read_bytes()) for path in source_paths}:
        raise RuntimeError("pixel-validation input or source changed while the harness was running")

    comparisons = {}
    for name in ("state8", "state9"):
        expected_pixels = expected[name]["pixels"]
        if actual_pixels[name] != expected_pixels:
            raise AssertionError(f"{name} production full framebuffer differs from independent common-path oracle")
        oracle = expected[name]
        metadata = actual_meta[name]
        for key, expected_value in {
            "sha256": sha(expected_pixels), "shadow": oracle["shadow"],
            "shadow_clipped": oracle["shadow_clipped"], "color": oracle["color"],
            "color_clipped": oracle["color_clipped"], "virtual_height": oracle["virtual_height"],
            "effective_shade": oracle["effective_shade"], "color_y": oracle["color_y"],
            "global_animation": oracle["global_animation"],
        }.items():
            if metadata.get(key) != str(expected_value):
                raise AssertionError(f"{name} {key}: Rust={metadata.get(key)} Python={expected_value}")
        comparisons[name] = {
            "record_sha256": RECORD_HASHES[STATE8 if name == "state8" else STATE9],
            "draw_fields": records[name][1], "explicit_origin": list(ORIGIN),
            "framebuffer_sha256": sha(actual_pixels[name]), "full_addressed_bytes_equal": True,
            "metadata": {key: int(value) if value.lstrip("-").isdigit() else value for key, value in metadata.items()},
        }

    post_pins = {path.relative_to(ROOT).as_posix(): sha(path.read_bytes()) for path in source_paths}
    if source_pins != post_pins:
        raise RuntimeError("pixel-validation pins changed after harness run")
    receipt = {
        "schema_version": 1,
        "status": "passed",
        "scope": "Two actual selector-0 parent records on the ordinary 0x10EA0 shadow/color wrapper, at an explicit diagnostic origin. No original executable, game timing, caller order, or parent-state callback is claimed.",
        "source_pins": source_pins,
        "source_record_origin": "private/review/family8-parent/source.json; state8 from combined frame6442, state9 from explicit all-parent-ready fixture6443",
        "resource_sha256": RESOURCE_SHA, "fade_sha256": FADE_SHA, "main_gla_sha256": GLA_SHA,
        "instruction_contract": "private/ghidra/countdown-expiry/instructions-00010ea0.txt",
        "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH,
                        "addressed_bytes": (HEIGHT - 1) * PITCH + WIDTH},
        "comparisons": comparisons,
    }
    from pixel_receipts import write_current_receipt
    write_current_receipt(ROOT / "research/integration/family8-parent-pixels-current.json", receipt)
    print(json.dumps({"receipt": "research/integration/family8-parent-pixels-current.json",
                      "state8_sha256": comparisons["state8"]["framebuffer_sha256"],
                      "state9_sha256": comparisons["state9"]["framebuffer_sha256"],
                      "full_buffers_equal": True}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

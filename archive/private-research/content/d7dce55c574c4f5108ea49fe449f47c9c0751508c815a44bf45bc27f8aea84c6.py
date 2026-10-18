#!/usr/bin/env python3
"""Independent byte/pixel oracle for the selector-0 state-4 ETHEREAL path.

The Python renderer reads hash-pinned private decoded inputs, reconstructs ANB
animation 8/indexed8 masks, and independently applies the 0x96AC0 shadow and
fade/direct color passes. It compares complete row-pitched output bytes and
draw receipts from the Rust helper compiled in a temporary project-private
Cargo package. It does not launch the game or execute original code.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import uuid

from pixel_receipts import write_current_receipt

ROOT = Path(__file__).resolve().parents[2]
ETHEREAL = ROOT / "private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
ETHEREAL_SHA = "318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4"
FADE_SHA = "b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392"
GLA_SHA = "bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3"
LISTING_PINS = {
    "private/ghidra/countdown-expiry/instructions-00010ea0.txt": "639fdb8a92c1342f206b3ddd7ebf563618988b4943ff53ca3c9bec9aed881dfb",
    "private/ghidra/mission-tail-init/instructions-00010060.txt": "518f7e316012183b083623aca3e4b01b8feedee77b0476c56e24d6f76a7d441d",
}
CASES = [
    ("fade0", 0, 4, 0, (320, 240), 0, 0),
    ("fade7_clip", 0, 4, 4, (1, 1), 7, 0),
    ("direct_clip", 0, 4, 10, (637, 475), 200, 16_000),
    ("factory_height", 0, 4, 0, (320, 380), 0, 16_000),
    ("selector1_state0", 1, 0, 0, (72, 90), 0, 4_800),
]
WIDTH, HEIGHT, PITCH, FILL = 640, 480, 648, 0x35


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def checked_bytes(path: Path, expected_sha: str, expected_size: int) -> bytes:
    data = path.read_bytes()
    if len(data) != expected_size or sha(data) != expected_sha:
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def i16(value: int) -> int:
    return ((value + 0x8000) & 0xffff) - 0x8000


def decode_resource(data: bytes, global_animation: int, relative_frame: int):
    if len(data) != 91_123 or struct.unpack_from("<H", data)[0] != 9:
        raise ValueError("ETHEREAL top-level identity mismatch")
    offsets = struct.unpack_from("<4I", data, 2)
    if offsets != (90_475, 90_577, 90_767, 91_033):
        raise ValueError("ETHEREAL ANB section offsets mismatch")
    sprite_start, component_start, frame_start, animation_start = offsets
    animation = struct.unpack_from("<IIH", data, animation_start + global_animation * 10)
    if global_animation == 8:
        expected_animation = (112, 252, 267)
        valid_frame = relative_frame in range(11)
    elif global_animation == 0:
        expected_animation = (0, 0, 1)
        valid_frame = relative_frame == 0
    elif global_animation == 4:
        expected_animation = (56, 56, 1)
        valid_frame = relative_frame == 0
    elif global_animation == 7:
        expected_animation = (98, 98, 1)
        valid_frame = relative_frame == 0
    else:
        raise ValueError("unadmitted global animation")
    if animation != expected_animation or not valid_frame:
        raise ValueError("selector/global-animation frame admission failed")
    frame_offset = animation[0] + relative_frame * 14
    frame_abs = frame_start + frame_offset
    component_offset, frame_w, frame_h = struct.unpack_from("<Ihh", data, frame_abs)
    component_count = data[frame_abs + 11]
    if component_count != 1 or component_offset + 10 > frame_start - component_start:
        raise ValueError("state-4 frame component extent mismatch")
    component = data[component_start + component_offset:component_start + component_offset + 10]
    sprite_ref, raw_x, raw_y, flags, _extra = struct.unpack("<IhhBB", component)
    if flags & ~1 or sprite_ref % 6:
        raise ValueError("unsupported state-4 component flags/reference")
    descriptor = data[sprite_start + sprite_ref:sprite_start + sprite_ref + 6]
    if len(descriptor) != 6:
        raise ValueError("truncated state-4 sprite descriptor")
    data_offset, sprite_w, sprite_h = struct.unpack("<IBB", descriptor)
    source = 18 + data_offset
    indices = [0] * (sprite_w * sprite_h)
    opaque = [False] * len(indices)
    cursor = source
    for y in range(sprite_h):
        x = 0
        while True:
            if cursor >= len(data):
                raise ValueError("truncated sprite RLE control")
            control = struct.unpack_from("<b", data, cursor)[0]
            cursor += 1
            if control == 0:
                break
            run = abs(control)
            if control > 0:
                end = cursor + run
                if end > len(data):
                    raise ValueError("truncated sprite literal run")
                visible = min(run, max(sprite_w - x, 0))
                for column in range(visible):
                    at = y * sprite_w + x + column
                    indices[at] = data[cursor + column]
                    opaque[at] = True
                cursor = end
            x = min(sprite_w, x + run)
    if flags & 1:
        for row in range(sprite_h):
            start = row * sprite_w
            indices[start:start + sprite_w] = reversed(indices[start:start + sprite_w])
            opaque[start:start + sprite_w] = reversed(opaque[start:start + sprite_w])
    return (frame_w, frame_h, sprite_w, sprite_h, i16(raw_x >> 1), i16(raw_y >> 1), indices, opaque)


def oracle(data: bytes, fade: bytes, gla: bytes, case):
    name, selector, state, frame_index, origin, raw_shade, virtual_height = case
    if (selector, state) not in ((0, 4), (0, 5), (1, 0)):
        raise ValueError("unadmitted selector/state")
    if selector == 0:
        global_animation = 8
    elif "direction" in name:
        suffix = name.rsplit("direction", 1)[1]
        global_animation = int(suffix.split("_", 1)[0])
    else:
        global_animation = 0
    frame_w, frame_h, sw, sh, local_x, local_y, indices, opacity = decode_resource(data, global_animation, frame_index)
    pixels = bytearray([FILL]) * ((HEIGHT - 1) * PITCH + WIDTH)
    shadow_written = shadow_clipped = 0
    shadow_x = i16(origin[0]) + local_x
    shadow_y = i16(origin[1]) + int(local_y / 2)
    for row in range(sh // 2):
        source_y = row * 2 + 1
        y = shadow_y + row
        for x in range(sw):
            if not opacity[source_y * sw + x]:
                continue
            screen_x = shadow_x + x
            if 0 <= screen_x < WIDTH and 0 <= y < HEIGHT:
                at = y * PITCH + screen_x
                pixels[at] = gla[pixels[at]]
                shadow_written += 1
            else:
                shadow_clipped += 1

    shade = 0 if raw_shade == 0 else raw_shade + (virtual_height >> 7)
    if shade < 0:
        raise ValueError("negative source fade row is outside safe admission")
    direct = shade >= 30  # record flags are exactly the factory word 0x4020
    color_y = i16(origin[1] - (virtual_height >> 6))
    left = i16(i16(origin[0]) + local_x)
    top = i16(color_y + local_y)
    color_written = color_clipped = 0
    for y in range(sh):
        py = top + y
        for x in range(sw):
            src = y * sw + x
            if not opacity[src]:
                continue
            px = left + x
            if 0 <= px < WIDTH and 0 <= py < HEIGHT:
                color = indices[src] if direct else fade[shade * 256 + indices[src]]
                pixels[py * PITCH + px] = color
                color_written += 1
            else:
                color_clipped += 1
    return {
        "name": name, "sha256": sha(pixels), "shadow": shadow_written,
        "shadow_clipped": shadow_clipped, "color": color_written,
        "color_clipped": color_clipped, "virtual_height": virtual_height,
        "effective_shade": shade, "color_y": color_y, "global_animation": global_animation, "pixels": pixels,
        "frame_width": frame_w, "frame_height": frame_h,
    }


def run_rust_harness(run_dir: Path, input_paths: tuple[Path, Path, Path]) -> dict[str, dict[str, str]]:
    run_dir.mkdir(parents=True, exist_ok=False)
    manifest = run_dir / "Cargo.toml"
    source = (ROOT / "scripts/desktop/family8_pixel_harness.rs").as_posix()
    dep_root = (ROOT / "rust/crates").as_posix()
    manifest.write_text(
        "[package]\nname = \"family8-pixel-harness\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
        "[[bin]]\nname = \"family8-pixel-harness\"\npath = \"" + source + "\"\n"
        "[dependencies]\n"
        f"genewars-assets = {{ path = \"{dep_root}/genewars-assets\" }}\n"
        f"genewars-render = {{ path = \"{dep_root}/genewars-render\" }}\n"
        f"genewars-runtime = {{ path = \"{dep_root}/genewars-runtime\" }}\n"
        "[workspace]\n", encoding="utf-8")
    outdir = run_dir / "pixels"
    command = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
               "--manifest-path", str(manifest), "--", *(str(p) for p in input_paths), str(outdir)]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, timeout=180)
    if result.returncode:
        raise RuntimeError(f"Rust probe failed ({result.returncode}):\n{result.stdout}\n{result.stderr}")
    lines = {}
    for line in result.stdout.splitlines():
        words = line.split()
        if not words:
            continue
        lines[words[0]] = dict(word.split("=", 1) for word in words[1:])
    return lines, outdir


def main() -> int:
    anb = checked_bytes(ETHEREAL, ETHEREAL_SHA, 91_123)
    fade = checked_bytes(FADE, FADE_SHA, 16_384)
    gla = checked_bytes(GLA, GLA_SHA, 65_536)
    for rel, expected in LISTING_PINS.items():
        if sha((ROOT / rel).read_bytes()) != expected:
            raise ValueError(f"instruction pin mismatch: {rel}")
    expected = [oracle(anb, fade, gla, case) for case in CASES]
    run_id = uuid.uuid4().hex
    run_dir = ROOT / "private/integration/family8-state4-pixels" / run_id
    source_paths = {
        "implementation": ROOT / "rust/crates/genewars-desktop/src/bin/support/family8_scene.rs",
        "oracle": Path(__file__),
        "harness": ROOT / "scripts/desktop/family8_pixel_harness.rs",
    }
    source_pins = {key: sha(path.read_bytes()) for key, path in source_paths.items()}
    rust, outdir = run_rust_harness(run_dir, (ETHEREAL, FADE, GLA))
    summaries = []
    for result in expected:
        name = result["name"]
        actual_bytes = (outdir / f"{name}.raw").read_bytes()
        if actual_bytes != result["pixels"]:
            raise AssertionError(f"complete framebuffer mismatch for {name}")
        actual_meta = rust.get(name)
        if not actual_meta:
            raise AssertionError(f"Rust helper omitted case {name}")
        for field in ("sha256", "shadow", "shadow_clipped", "color", "color_clipped", "virtual_height", "effective_shade", "color_y", "global_animation"):
            if actual_meta.get(field) != str(result[field]):
                raise AssertionError(f"{name} {field}: Rust={actual_meta.get(field)} Python={result[field]}")
        summaries.append({k: v for k, v in result.items() if k != "pixels"})

    if any(sha(path.read_bytes()) != source_pins[key] for key, path in source_paths.items()):
        raise RuntimeError("pixel-validator source changed while the Rust harness was running")

    payload = {
        "schema_version": 1,
        "scope": "Independent Python byte/pixel oracle compared against Rust family8 selector0/state4 and selector1/state0 helpers; no original code executed.",
        "resource": {"sha256": ETHEREAL_SHA, "decoded_bytes": len(anb), "manifest_entry": "root/2/disc/60", "selector_global_animation": {"0": 8, "1": 0}},
        "fade_sha256": FADE_SHA, "main_gla_sha256": GLA_SHA,
        "instruction_pins": LISTING_PINS,
        "implementation_sha256": source_pins["implementation"],
        "oracle_sha256": source_pins["oracle"],
        "harness_sha256": source_pins["harness"],
        "cases": summaries,
        "result": {"full_framebuffer_bytes_matched": True, "cases": len(summaries)},
    }
    receipt = ROOT / "research/integration/family8-initial-selectors-pixel-validation-current.json"
    write_current_receipt(receipt, payload)
    print(json.dumps({"receipt": str(receipt), "run_dir": str(run_dir), "cases": summaries}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

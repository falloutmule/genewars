#!/usr/bin/env python3
"""Independent full-frame oracle for the BTURR selector-1 common image pass."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

from pixel_receipts import write_current_receipt

ROOT = Path(__file__).resolve().parents[2]
ANB = ROOT / "private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB"
FADE = ROOT / "private/integration/game-runtime/terrain/FADE.DAT"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
HARNESS = ROOT / "scripts/desktop/bturr_common_pixel_harness.rs"
IMPLEMENTATION = ROOT / "rust/crates/genewars-runtime/src/building_resources.rs"
ACTOR_RENDER = ROOT / "rust/crates/genewars-runtime/src/actor_render.rs"
ASSET_PARSER = ROOT / "rust/crates/genewars-assets/src/lib.rs"
SCRIPT = Path(__file__).resolve()
INPUTS = {
    ANB: ("c26df60af3a3166aec54112f197a4334966eb214eeaa48e0d9284e0510b8cd20", 68_030),
    FADE: ("b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392", 16_384),
    GLA: ("bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3", 65_536),
}
LISTINGS = {
    "private/ghidra/building-draw/instructions-00043a40.txt": "62a4377cb232402be9cf5a1fb0c7da61d29cc582632c60c217b0357e2961ff1b",
    "private/ghidra/actor-draw/instructions-00095390.txt": "8ae130718f8bb1c699ccfc7e44a6a83fc9f41c10770fb5b9b4050efc96bfcfa5",
    "private/ghidra/actor-draw/instructions-00095e40.txt": "3982c16e3623879cc29ee2e5851cf429c760a57d79add30bbcd262103dedfeb5",
    "private/ghidra/actor-draw/instructions-00096870.txt": "3f9eb98b5a042f83896da8afca1b68fc8b7a5681ec4668ef4a6ccce568a2bc21",
    "private/ghidra/actor-draw/instructions-00096990.txt": "318a4d5f9fea633f859a291e7449239393e54f7c889eeb970cf9b4d3c229398c",
    "private/ghidra/entity-plant/instructions-0009a660.txt": "f6e10c86aa6ce6cf6b7906b26279ffc9baf4142fd81b0f99779f58b40b3001ee",
    "private/ghidra/entity-plant/instructions-0009b0d0.txt": "b8516427e8601213d500caa4fdf5fcfd0dbba6885f73a0f1890a6dbb6d2f26a0",
}
WIDTH, HEIGHT, PITCH, FILL = 640, 480, 648, 0x35
CASES = []
for frame in range(3):
    for owner in range(4):
        for label, shade, actor_flags in (("fade_zero", 0, 0), ("fade_17", 17, 0),
                                          ("direct_30", 30, 0), ("direct_flag", 11, 2)):
            for global_flags in (0, 4):
                for origin_name, x, y, phase in (("center", 320, 240, 0), ("top_left", 1, 1, 0),
                                                 ("edge", 637, 475, 2), ("signed", -32768, 32767, 3)):
                    CASES.append({"id": len(CASES), "frame": frame, "owner": owner,
                                  "route": label, "shade": shade, "actor_flags": actor_flags,
                                  "global_flags": global_flags, "origin_name": origin_name,
                                  "origin": [x, y], "phase": phase})


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def check(path: Path, digest: str, size: int | None = None) -> bytes:
    data = path.read_bytes()
    if sha(data) != digest or (size is not None and len(data) != size):
        raise ValueError(f"pinned input mismatch: {path}")
    return data


def i16(value: int) -> int:
    return ((value + 0x8000) & 0xffff) - 0x8000


def s16(data: bytes, pos: int) -> int:
    return int.from_bytes(data[pos:pos + 2], "little", signed=True)


def decode_frame(anb: bytes, frame_index: int):
    count = int.from_bytes(anb[:2], "little")
    sections = tuple(int.from_bytes(anb[2 + 4 * i:6 + 4 * i], "little") for i in range(4))
    if count != 5 or sections != (66_562, 67_000, 67_560, 67_980):
        raise ValueError("BTURR ANB section signature changed")
    sprite_start, component_start, frame_start, animation_start = sections
    first = int.from_bytes(anb[animation_start:animation_start + 4], "little")
    last = int.from_bytes(anb[animation_start + 4:animation_start + 8], "little")
    trailing = int.from_bytes(anb[animation_start + 8:animation_start + 10], "little")
    if (first, last, trailing) != (0, 28, 3):
        raise ValueError("animation-0 source range changed")
    frame_pos = frame_start + frame_index * 14
    component_offset = int.from_bytes(anb[frame_pos:frame_pos + 4], "little")
    component_count = anb[frame_pos + 11]
    width_raw = s16(anb, frame_pos + 4)
    height_raw = s16(anb, frame_pos + 6)
    if component_count != (2 if frame_index == 2 else 1):
        raise ValueError("expected source component cardinality differs")
    components = []
    stats = []
    for component_index in range(component_count):
        cp = component_start + component_offset + component_index * 10
        sprite_offset = int.from_bytes(anb[cp:cp + 4], "little")
        local = [s16(anb, cp + 4) >> 1, s16(anb, cp + 6) >> 1]
        flags = anb[cp + 8]
        if flags != 0 or sprite_offset % 6:
            raise ValueError("unadmitted BTURR component flags/reference")
        desc = sprite_start + sprite_offset
        data_offset = int.from_bytes(anb[desc:desc + 4], "little")
        sw, sh = anb[desc + 4], anb[desc + 5]
        cursor = 18 + data_offset
        indices = bytearray(sw * sh)
        opacity = bytearray(sw * sh)
        overshoot = 0
        for y in range(sh):
            x = 0
            while True:
                if cursor >= sprite_start:
                    raise ValueError("BTURR sprite RLE crossed the metadata boundary")
                control = int.from_bytes(anb[cursor:cursor + 1], "little", signed=True)
                cursor += 1
                if control == 0:
                    break
                if control > 0:
                    if cursor + control > sprite_start:
                        raise ValueError("BTURR literal run crossed the metadata boundary")
                    run = anb[cursor:cursor + control]
                    cursor += control
                    overshoot += max(0, x + control - sw)
                    for j, value in enumerate(run):
                        sx = x + j
                        if sx < sw:
                            at = y * sw + sx
                            indices[at] = value
                            opacity[at] = 1
                    x += control
                else:
                    x -= control
                    overshoot += max(0, x - sw) - max(0, x + control - sw) if x > sw else 0
        components.append({"sprite_width": sw, "sprite_height": sh, "local": local,
                           "indices": indices, "opacity": opacity})
        stats.append({"component": component_index, "sprite_record": sprite_offset // 6,
                      "width": sw, "height": sh, "opaque": sum(opacity),
                      "over_width_literal_pixels": overshoot, "local": local})
    return {"display_dimensions_raw": [width_raw, height_raw], "components": components, "stats": stats}


def oracle(anb: bytes, fade: bytes, gla: bytes, case: dict) -> dict:
    frame = decode_frame(anb, case["frame"])
    output = bytearray([FILL]) * ((HEIGHT - 1) * PITCH + WIDTH)
    phase = case["phase"]
    adjust = (phase & 1) if phase > 1 else 0
    origin_x = i16(case["origin"][0] - adjust)
    origin_y = case["origin"][1]
    written = clipped = opaque_count = 0
    direct = (case["actor_flags"] & 2) != 0 or case["shade"] >= 30
    fade_row = None if direct else fade[case["shade"] * 256:(case["shade"] + 1) * 256]
    for component in frame["components"]:
        sw, sh = component["sprite_width"], component["sprite_height"]
        left, top = origin_x + component["local"][0], origin_y + component["local"][1]
        for sy in range(sh):
            for sx in range(sw):
                at_src = sy * sw + sx
                if not component["opacity"][at_src]:
                    continue
                opaque_count += 1
                dx, dy = left + sx, top + sy
                if not (0 <= dx < WIDTH and 0 <= dy < HEIGHT):
                    clipped += 1
                    continue
                mapped = component["indices"][at_src]
                if 0xC0 <= mapped <= 0xCF:
                    mapped = (case["owner"] * 16 - 64 + (mapped & 0x0F)) & 0xFF
                if fade_row is not None:
                    mapped = fade_row[mapped]
                if case["global_flags"] & 0x0C:
                    mapped = gla[output[dy * PITCH + dx] * 256 + mapped]
                output[dy * PITCH + dx] = mapped
                written += 1
    return {"sha256": sha(output), "pixels": output, "opaque": opaque_count,
            "written": written, "clipped": clipped, "draw_origin": [origin_x, origin_y],
            "components": len(frame["components"]), "component_stats": frame["stats"],
            "display_dimensions_raw": frame["display_dimensions_raw"], "direct": direct}


def main() -> int:
    raw_inputs = {path: check(path, digest, size) for path, (digest, size) in INPUTS.items()}
    for rel, digest in LISTINGS.items():
        check(ROOT / rel, digest)
    pins_before = {"harness": sha(HARNESS.read_bytes()), "script": sha(SCRIPT.read_bytes()),
                   "building_resources": sha(IMPLEMENTATION.read_bytes()),
                   "actor_render": sha(ACTOR_RENDER.read_bytes()), "assets_parser": sha(ASSET_PARSER.read_bytes())}
    anb, fade, gla = raw_inputs[ANB], raw_inputs[FADE], raw_inputs[GLA]
    scratch = ROOT / "private/integration/bturr-common-pixels" / uuid.uuid4().hex
    scratch.mkdir(parents=True, exist_ok=False)
    cases_path = scratch / "cases.txt"
    case_lines = []
    expected = []
    for case in CASES:
        case_lines.append(" ".join(map(str, (case["id"], case["frame"], case["owner"], case["shade"],
            case["actor_flags"], case["global_flags"], *case["origin"], case["phase"]))))
        expected.append(oracle(anb, fade, gla, case))
    cases_path.write_text("\n".join(case_lines) + "\n", encoding="ascii")
    manifest = scratch / "Cargo.toml"
    crates = (ROOT / "rust/crates").as_posix()
    source = HARNESS.as_posix()
    manifest.write_text(
        "[package]\nname=\"bturr-common-pixel-oracle\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"
        f"\n[[bin]]\nname=\"bturr-common-pixel-oracle\"\npath=\"{source}\"\n"
        "[dependencies]\n"
        f"genewars-render={{path=\"{crates}/genewars-render\"}}\n"
        f"genewars-runtime={{path=\"{crates}/genewars-runtime\"}}\n[workspace]\n", encoding="utf-8")
    command = [sys.executable, "-B", str(ROOT / "scripts/cargo.py"), "run", "--offline", "--quiet",
               "--manifest-path", str(manifest), "--", str(ANB), str(FADE), str(GLA), str(cases_path)]
    proc = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=180)
    if proc.returncode:
        raise RuntimeError(f"BTURR Rust pixel probe failed ({proc.returncode}):\n{proc.stdout}\n{proc.stderr}")
    reports = {}
    rejected = {}
    for line in proc.stdout.splitlines():
        words = line.split()
        if not words:
            continue
        if words[0].startswith("reject_"):
            label, value = words[0].split("=", 1)
            rejected[label[7:]] = value
        else:
            reports[int(words[0])] = dict(token.split("=", 1) for token in words[1:])
    if len(reports) != len(CASES) or rejected != {"destruction": "atomic", "blink": "atomic", "global": "atomic",
            "frame": "atomic", "fade": "atomic", "blend": "atomic"}:
        raise AssertionError(f"probe case/rejection count mismatch: {len(reports)}, {rejected}")
    summaries = []
    for case, wanted in zip(CASES, expected):
        got = reports[case["id"]]
        for key in ("sha", "written", "clipped", "opaque", "x", "y"):
            expected_value = {"sha": wanted["sha256"], "written": wanted["written"],
                              "clipped": wanted["clipped"], "opaque": wanted["opaque"],
                              "x": wanted["draw_origin"][0], "y": wanted["draw_origin"][1]}[key]
            if got.get(key) != str(expected_value):
                raise AssertionError(f"BTURR case={case} {key}: Rust={got.get(key)} oracle={expected_value}")
        summaries.append({"id": case["id"], "frame": case["frame"], "owner": case["owner"],
                          "route": case["route"], "global_flags": case["global_flags"],
                          "input_origin": case["origin"], "phase": case["phase"],
                          "draw_origin": wanted["draw_origin"], "components": wanted["components"],
                          "component_stats": wanted["component_stats"], "display_dimensions_raw": wanted["display_dimensions_raw"],
                          "frame_sha256": wanted["sha256"], "full_frame_sha256_equal": True,
                          "opaque": wanted["opaque"], "written": wanted["written"], "clipped": wanted["clipped"]})
    pins_after = {"harness": sha(HARNESS.read_bytes()), "script": sha(SCRIPT.read_bytes()),
                  "building_resources": sha(IMPLEMENTATION.read_bytes()),
                  "actor_render": sha(ACTOR_RENDER.read_bytes()), "assets_parser": sha(ASSET_PARSER.read_bytes())}
    if pins_before != pins_after:
        raise RuntimeError("BTURR source changed during Rust comparison")
    report = {
        "schema_version": 1,
        "status": "passed-independent-bturr-common-image-pixel-oracle",
        "scope": "Exact hash-bound BTURR animation 0 frames 0–2, owner recolor and indexed8 masks compared against an independent Python ANB/RLE/color model for all four owners, fade/direct dispatch, global GLA mode 0/4, signed/clipped origins, and wrapper +0x30 X adjustment. The production Rust helper is exercised; six unsupported branches must reject before destination writes.",
        "inputs": {"bturr_sha256": INPUTS[ANB][0], "fade_sha256": INPUTS[FADE][0], "main_gla_sha256": INPUTS[GLA][0],
                   "source_listings": LISTINGS},
        "code_pins": pins_before,
        "framebuffer": {"width": WIDTH, "height": HEIGHT, "pitch": PITCH, "fill_index": FILL,
                        "bytes_compared_per_case": (HEIGHT - 1) * PITCH + WIDTH},
        "case_count": len(summaries), "rejections_atomic": rejected,
        "frames": {str(i): {"components": next(s["components"] for s in summaries if s["frame"] == i),
                             "raw_dimensions": next(s["display_dimensions_raw"] for s in summaries if s["frame"] == i),
                             "component_stats": next(s["component_stats"] for s in summaries if s["frame"] == i)} for i in range(3)},
        "cases": summaries,
    }
    receipt = ROOT / "research/integration/bturr-common-pixels-current.json"
    write_current_receipt(receipt, report)
    shutil.rmtree(scratch)
    print(json.dumps({"status": report["status"], "case_count": len(summaries),
                      "frames": report["frames"], "rejections_atomic": rejected,
                      "receipt": receipt.relative_to(ROOT).as_posix()}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

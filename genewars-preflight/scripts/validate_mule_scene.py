"""Independent source-derived pixel oracle for the bounded MULE draw wrapper.

The script asks the private Rust test harness to render a fixed set of real
MULE frames, then independently parses the hash-pinned ANB/RLE streams and
models the 59470/94F30/95100/93F80/937A0 paths. Raw pixels stay in private
scratch; the public receipt contains hashes and comparison counts only.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]
PRIVATE = ROOT / "private" / "integration" / "game-runtime"
MULE_SHA = "f6c62221b52a62088f91ae8d00becfef513282cbf98a5990c1a0859cd8b2d346"
FADE_SHA = "b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392"
GLA_SHA = "bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3"
sha = lambda b: hashlib.sha256(b).hexdigest()


def read_inputs():
    anb = (PRIVATE / "creatures" / "MULE.ANB").read_bytes()
    fade = (PRIVATE / "terrain" / "FADE.DAT").read_bytes()
    gla = (PRIVATE / "terrain" / "MAIN.GLA").read_bytes()
    assert sha(anb) == MULE_SHA and len(anb) == 57038
    assert sha(fade) == FADE_SHA and len(fade) == 64 * 256
    assert sha(gla) == GLA_SHA and len(gla) == 256 * 256
    return anb, fade, gla


def u16(b, p): return int.from_bytes(b[p:p + 2], "little")
def i16(b, p): return int.from_bytes(b[p:p + 2], "little", signed=True)
def u32(b, p): return int.from_bytes(b[p:p + 4], "little")


def parse_frame(anb, animation, frame_index):
    if len(anb) < 18:
        raise ValueError("short ANB header")
    offsets = [u32(anb, 2 + 4 * i) for i in range(4)]
    s0, s1, s2, s3 = offsets
    if not (18 <= s0 <= s1 <= s2 <= s3 <= len(anb)):
        raise ValueError("bad ANB section bounds")
    if (s1 - s0) % 6 or (s2 - s1) % 10 or (s3 - s2) % 14 or (len(anb) - s3) % 10:
        raise ValueError("bad ANB record section size")
    animation_count = u16(anb, 0)
    if animation >= animation_count:
        raise ValueError("animation out of range")
    ar = s3 + animation * 10
    first, last = u32(anb, ar), u32(anb, ar + 4)
    if first > last or (last - first) % 14:
        raise ValueError("bad animation frame interval")
    if frame_index > (last - first) // 14:
        raise ValueError("frame out of range")
    fr = s2 + first + frame_index * 14
    component_rel = u32(anb, fr)
    component_count = anb[fr + 11]
    if component_rel % 10 or component_rel + component_count * 10 > s2 - s1:
        raise ValueError("frame component range out of bounds")
    components = []
    for i in range(component_count):
        cr = s1 + component_rel + i * 10
        sprite_offset = u32(anb, cr)
        if sprite_offset % 6 or sprite_offset + 6 > s1 - s0:
            raise ValueError("component sprite reference out of bounds")
        sr = s0 + sprite_offset
        data_offset, width, height = u32(anb, sr), anb[sr + 4], anb[sr + 5]
        components.append({
            "x": i16(anb, cr + 4) >> 1,
            "y": i16(anb, cr + 6) >> 1,
            "flags": anb[cr + 8],
            "record": sprite_offset // 6,
            "sprite": decode_packed4(anb, data_offset, width, height),
        })
    return components


def decode_packed4(anb, offset, width, height):
    values = [0] * (width * height)
    opaque = [False] * (width * height)
    if width == 0 or height == 0:
        return {"width": width, "height": height, "pixels": values, "opaque": opaque}
    cursor = 18 + offset
    if cursor >= len(anb):
        raise ValueError("sprite stream outside ANB")
    for y in range(height):
        x = 0
        while True:
            if cursor >= len(anb):
                raise ValueError("truncated sprite row")
            control = int.from_bytes(anb[cursor:cursor + 1], "little", signed=True)
            cursor += 1
            if control == 0:
                break
            count = abs(control)
            if control < 0:
                x = min(width, x + count)
                continue
            byte_count = (count + 1) // 2
            payload = anb[cursor:cursor + byte_count]
            if len(payload) != byte_count:
                raise ValueError("truncated packed4 payload")
            for j in range(min(count, width - x)):
                byte = payload[j // 2]
                values[y * width + x + j] = (byte >> 4) if j % 2 == 0 else (byte & 15)
                opaque[y * width + x + j] = True
            cursor += byte_count
            x = min(width, x + count)
    return {"width": width, "height": height, "pixels": values, "opaque": opaque}


def make_surface(case, anb, fade, gla):
    width, height, pitch = 64, 48, 68
    dst = bytearray([0xA5] * (pitch * height + 4))
    comps = parse_frame(anb, case["animation"], case["frame"])
    a, b = case["scale_a"], case["scale_b"]
    factor = 256 if b == 255 and a >= 100 else min(256, max(150, (a * b) // 100))
    record_state = case.get("state", 8)
    animation = case["animation"]
    flags = case["inherited"]
    if not (animation < 0x10 and record_state < 0x1A):
        sector = case["yaw"] >> 7
        if 6 <= sector <= 13:
            flags = 1
    origin_x, origin_y = case["origin"]
    shadow_mode = case["shadow_mode"]
    raw_shade = case["raw_shade"]
    effective_shade = 0 if raw_shade == 0 else min(31, raw_shade + (case["virtual_height"] >> 7))
    if effective_shade < 0:
        raise ValueError("negative shade rows are rejected by the Rust adapter")
    assert case["effective_shade"] == effective_shade
    if shadow_mode == 0:
        shadow_base_y = origin_y + (factor >> 5)
        for c in comps:
            s = c["sprite"]
            sw, sh = s["width"], s["height"]
            out_w = (sw * factor) >> 8
            out_h_half = ((sh >> 1) * factor) >> 8
            full_out_h = (sh * factor) >> 8
            x_scaled = (c["x"] * factor) >> 8
            y_scaled = (c["y"] * factor) >> 9
            xoff = -x_scaled - out_w if flags & 1 else x_scaled
            yoff = -y_scaled - full_out_h if flags & 2 else y_scaled
            cflags = (c["flags"] | (flags & ~3)) ^ (flags & 3)
            draw_shadow(dst, width, height, pitch, s, [origin_x + xoff, shadow_base_y + yoff],
                        factor, cflags, case["shadow_color"], gla)
    main_y = origin_y if shadow_mode else origin_y - (case["virtual_height"] >> 6)
    for c in comps:
        s = c["sprite"]
        sw, sh = s["width"], s["height"]
        out_w = (sw * factor) >> 8
        out_h = (sh * factor) >> 8
        x_scaled = (c["x"] * factor) >> 8
        y_scaled = (c["y"] * factor) >> 8
        xoff = -x_scaled - out_w if flags & 1 else x_scaled
        yoff = -y_scaled - out_h if flags & 2 else y_scaled
        cflags = (c["flags"] | (flags & ~3)) ^ (flags & 3)
        draw_color(dst, width, height, pitch, s, [origin_x + xoff, main_y + yoff],
                   factor, cflags, case["clip_bit"], fade[effective_shade * 256:(effective_shade + 1) * 256])
    return bytes(dst)


def draw_shadow(dst, width, height, pitch, sprite, origin, factor, flags, color, gla):
    sw, sh = sprite["width"], sprite["height"] >> 1
    out_w, out_h = (sw * factor) >> 8, (sh * factor) >> 8
    if out_w == 0 or out_h == 0:
        return
    xstep, ystep = (sw << 16) // out_w, (sh << 16) // out_h
    for dy in range(out_h):
        sy = min(sh - 1, (dy * ystep) >> 16)
        y = origin[1] + (out_h - 1 - dy if flags & 2 else dy)
        for dx in range(out_w):
            sx = min(sw - 1, (dx * xstep) >> 16)
            x = origin[0] + (out_w - 1 - dx if flags & 1 else dx)
            if not (0 <= x < width and 0 <= y < height):
                continue
            source = sy * sprite["width"] + sx
            if sprite["opaque"][source]:
                at = y * pitch + x
                dst[at] = gla[color * 256 + dst[at]]


def draw_color(dst, width, height, pitch, sprite, origin, factor, flags, clip_bit, fade_row):
    sw, sh = sprite["width"], sprite["height"]
    out_w, out_h = (sw * factor) >> 8, (sh * factor) >> 8
    if out_w == 0 or out_h == 0:
        return
    xstep, ystep = (sw << 16) // out_w, (sh << 16) // out_h
    for dy in range(out_h):
        sy = min(sh - 1, (dy * ystep) >> 16)
        y = origin[1] + (out_h - 1 - dy if flags & 2 else dy)
        for dx in range(out_w):
            sx = min(sw - 1, (dx * xstep) >> 16)
            x = origin[0] + (out_w - 1 - dx if flags & 1 else dx)
            if not (0 <= x < width and 0 <= y < height):
                continue
            source = sy * sw + sx
            if not sprite["opaque"][source]:
                continue
            at = y * pitch + x
            alternate = clip_bit ^ (((at & 0xff) ^ (origin[0] & 0xff)) & 1)
            mapped = (208 if alternate else 192) + sprite["pixels"][source]
            dst[at] = fade_row[mapped]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--input", type=Path, help="existing private Rust JSONL; skips Cargo")
    parser.add_argument("--receipt", type=Path, default=None,
                        help="fresh receipt path; omitted updates the standard receipt")
    args = parser.parse_args()
    if args.receipt is not None and args.receipt.exists():
        raise FileExistsError(f"receipt exists; choose a fresh --receipt path: {args.receipt}")
    anb, fade, gla = read_inputs()
    if args.input is None:
        scratch = ROOT / "private" / "review" / f"mule-pixel-oracle-{uuid.uuid4().hex}"
        scratch.mkdir(parents=True, exist_ok=False)
        rust_output = scratch / "rust.jsonl"
        env = os.environ.copy()
        env["GENEWARS_MULE_ORACLE_OUT"] = str(rust_output)
        command = [sys.executable, str(ROOT / "scripts" / "cargo.py"), "test",
                   "--offline", "--locked",
                   "-p", "genewars-desktop", "--bin", "genewars-game",
                   "creature_scene::tests::export_source_mule_pixel_oracle_cases_when_requested"]
        run = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=300)
        if run.returncode:
            raise RuntimeError(f"Rust fixture render failed: {run.stdout[-4000:]}\n{run.stderr[-4000:]}")
    else:
        rust_output = args.input
    actual_lines = rust_output.read_text(encoding="utf-8").splitlines()
    if len(actual_lines) != 164:
        raise AssertionError(f"expected 164 Rust oracle cases, got {len(actual_lines)}")
    compared = 0
    pixels = 0
    for line in actual_lines:
        case = json.loads(line)
        actual = bytes.fromhex(case.pop("pixels"))
        expected = make_surface(case, anb, fade, gla)
        if actual != expected:
            for i, (a, e) in enumerate(zip(actual, expected)):
                if a != e:
                    raise AssertionError(f"MULE case {case['animation']}/{case['frame']} first mismatch at {i}: Rust={a}, Python={e}, case={case}")
            raise AssertionError(f"MULE framebuffer length differs: {len(actual)} vs {len(expected)}")
        compared += 1
        pixels += len(expected)
    pins = {}
    for path in [ROOT / "scripts" / "validate_mule_scene.py",
                 ROOT / "rust" / "crates" / "genewars-desktop" / "src" / "bin" / "support" / "creature_scene.rs",
                 ROOT / "rust" / "crates" / "genewars-desktop" / "src" / "bin" / "support" / "actor_scene.rs",
                 ROOT / "rust" / "crates" / "genewars-desktop" / "src" / "bin" / "support" / "runtime_resources.rs",
                 ROOT / "research" / "ghidra" / "creature-evidence.json",
                 ROOT / "research" / "ghidra" / "creature-codec-bridge.md",
                 ROOT / "private" / "ghidra" / "terrain-viewport" / "instructions-00059470.txt",
                 ROOT / "private" / "integration" / "combined-before-mule366" / "source-after366.bin",
                 ROOT / "private" / "integration" / "combined-before-mule375" / "source-after375.bin",
                 ROOT / "private" / "integration" / "combined-before-mule385" / "source-after385.bin",
                 ROOT / "private" / "integration" / "combined-before-mule415" / "source-after415.bin",
                 ROOT / "private" / "review" / "mule-mortality" / "frame9053" / "source-after9053.bin",
                 ROOT / "private" / "review" / "mule-mortality" / "frame9053" / "source-trace9053.json",
                 ROOT / "private" / "review" / "mule-mortality" / "frame9058" / "source-after9058.bin",
                 ROOT / "private" / "review" / "mule-mortality" / "frame9058" / "source-trace9058.json",
                 ROOT / "private" / "review" / "mule-mortality" / "frame9059" / "source-after9059.bin",
                 ROOT / "private" / "review" / "mule-mortality" / "frame9059" / "source-trace9059.json",
                 ROOT / "private" / "ghidra" / "actor-activation" / "instructions-0001f260.txt",
                 ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00094f30.txt",
                 ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00094a60.txt",
                 ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00095100.txt",
                 ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-000937a0.txt",
                 ROOT / "private" / "ghidra" / "entity-draw-refs" / "instructions-00093f80.txt",
                 ROOT / "private" / "ghidra" / "display-geometry" / "instructions-0008f8f0.txt",
                 ROOT / "research" / "formats" / "mule-render-contract.md"]:
        raw = path.read_bytes()
        pins[path.relative_to(ROOT).as_posix()] = {"bytes": len(raw), "sha256": sha(raw)}
    receipt = {
        "scope": "Independent Python source-derived framebuffer comparison; not original-execution parity",
        "cases": compared,
        "framebuffer_bytes_compared": pixels,
        "mule_input": {"bytes": len(anb), "sha256": sha(anb)},
        "fade_input": {"bytes": len(fade), "sha256": sha(fade)},
        "main_gla_input": {"bytes": len(gla), "sha256": sha(gla)},
        "ordinary_shade_cases": 3,
        "mortality_animation_frames": 10,
        "mortality_scope": "state 24 / animation 17 (7 metadata frames) and state 25 / animation 18 (3 metadata frames); only unmodified frame 9053 and 9059 snapshots are source-traced records, other frame-byte selections are codec-domain tests",
        "code_pins": pins,
        "comparison": "all Rust framebuffer bytes equal the independently parsed ANB/RLE/scale/flip/fade/GLA model",
        "shade_contract": "raw zero selects row 0; nonzero adds arithmetic virtual_height >> 7 and caps at 31; negative rows and unresolved parity-special dispatch reject atomically",
    }
    out = args.receipt
    with out.open("x", encoding="utf-8") as file:
        file.write(json.dumps(receipt, indent=2) + "\n")
    print(json.dumps(receipt, indent=2))


if __name__ == "__main__":
    main()

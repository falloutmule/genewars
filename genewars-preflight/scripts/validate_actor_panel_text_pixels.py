#!/usr/bin/env python3
"""Independent FONT0/TEXT selected-actor panel pixel oracle.

The Rust harness renders private, generated blueprint records. This script
decodes the source-bound FONT0 RLE and text strings independently, compares
all 921,600 RGB bytes for four category records, and writes metadata only.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import tempfile
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
FONT_DAT = ROOT / "private/review/font-menu/font0.dat"
FONT_TAB = ROOT / "private/review/font-menu/font0.tab"
TEXT = ROOT / "private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT"
PALETTE = ROOT / "private/integration/terrain-inputs/MAIN.PAL"
EXPECTED = {
    FONT_DAT: "af89c4e4c02e394810b22ecf3d57211cdb56a0006071a88f43d0ae6fb370447d",
    FONT_TAB: "321c49b302552e2bcf287bd20ef0954fb977dd66aef6e82101e3c0c64b5b0a17",
    TEXT: "23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786",
    PALETTE: "8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05",
}
WIDTH, HEIGHT = 640, 480
CLIP = (84, 382, 78, 21)
BLACK_MARKERS = {2, 4, 12, 16, 20, 24, 32, 36, 40, 48, 52, 56, 60}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_pinned(path: Path) -> bytes:
    data = path.read_bytes()
    if sha(data) != EXPECTED[path]:
        raise SystemExit(f"pinned input changed: {path.relative_to(ROOT)}")
    return data


def text_strings(data: bytes) -> list[bytes]:
    result, start = [], 0
    for _ in range(604):
        end = data.find(b"\0", start)
        if end < 0:
            raise ValueError("TEXT is missing one of the 604 bounded strings")
        result.append(data[start:end])
        start = end + 1
    return result


def font_records(tab: bytes) -> list[tuple[int, int, int]]:
    if len(tab) < 6 or (len(tab) - 6) % 6:
        raise ValueError("bad six-byte FONT TAB record layout")
    return [(int.from_bytes(tab[p:p + 4], "little"), tab[p + 4], tab[p + 5])
            for p in range(6, len(tab), 6)]


def decode_glyph(dat: bytes, records: list[tuple[int, int, int]], code: int):
    if code < 32 or code - 32 >= len(records):
        raise ValueError(f"unsupported glyph byte {code}")
    offset, width, height = records[code - 32]
    indices = [0] * (width * height)
    opaque = [False] * (width * height)
    if not width or not height:
        return width, height, indices, opaque
    cursor = offset
    for y in range(height):
        x = 0
        while True:
            if cursor >= len(dat):
                raise ValueError(f"truncated glyph row for byte {code}")
            signed = dat[cursor] if dat[cursor] < 128 else dat[cursor] - 256
            cursor += 1
            if signed == 0:
                break
            count = abs(signed)
            if signed > 0:
                if cursor + count > len(dat):
                    raise ValueError(f"truncated literal run for byte {code}")
                visible = min(count, max(0, width - x))
                for k in range(visible):
                    indices[y * width + x + k] = dat[cursor + k]
                    opaque[y * width + x + k] = True
                cursor += count
            x = min(width, x + count)
    return width, height, indices, opaque


def render_line(frame: bytearray, dat: bytes, records, palette: bytes,
                text: bytes, x0: int, y0: int) -> int:
    x = x0
    for code in text:
        width, height, indices, mask = decode_glyph(dat, records, code)
        if code != 32:
            for gy in range(height):
                for gx in range(width):
                    if not mask[gy * width + gx]:
                        continue
                    px, py = x + gx, y0 + gy
                    if px < 84 or px >= 84 + 78 or py < 382 or py >= 382 + 21:
                        continue
                    literal = indices[gy * width + gx]
                    color = nearest_palette_index(palette, (63, 63, 63)) if literal == 1 else (
                        nearest_palette_index(palette, (0, 0, 0)) if literal in BLACK_MARKERS else literal)
                    rgb = [palette[color * 3 + c] * 255 // 63 for c in range(3)]
                    at = (py * WIDTH + px) * 3
                    frame[at:at + 3] = bytes(rgb)
        x += width - 1
    return x - x0


def nearest_palette_index(palette: bytes, rgb6: tuple[int, int, int]) -> int:
    # 0xA3F54 scans palette entries in ascending order, keeps the first
    # minimum-distance match, and returns immediately for exact distance 0.
    return min(range(256), key=lambda i: (
        sum((palette[i * 3 + c] - rgb6[c]) ** 2 for c in range(3)), i))


def render_raw_bars(frame: bytearray, record: bytes, palette: bytes) -> None:
    # Exact call order and rectangle arguments from 0x265F0 / 0x2FF50.
    for field, x, y, color in ((0x17, 116, 415, 0xE8),
                               (0x19, 116, 435, 0xC4),
                               (0x1A, 116, 435, 0xC8)):
        value = record[field] if record[field] < 128 else record[field] - 256
        if value <= 0:
            continue
        width = max(1, (45 * value) // 100)
        rgb = bytes(palette[color * 3 + c] * 255 // 63 for c in range(3))
        for py in range(y, min(y + 6, HEIGHT)):
            for px in range(x, min(x + width, WIDTH)):
                at = (py * WIDTH + px) * 3
                frame[at:at + 3] = rgb


def main() -> None:
    dat, tab, text_data, palette = (read_pinned(p) for p in EXPECTED)
    strings = text_strings(text_data)
    records = font_records(tab)
    if len(records) != 194 or records[0][2] != 8:
        raise SystemExit(f"unexpected FONT0 records/space height: {len(records)}/{records[0][2]}")

    run_id = str(uuid.uuid4())
    out = ROOT / "private/review/actor-panel-text" / f"pixels-{run_id}"
    with tempfile.TemporaryDirectory(prefix=f"actor-panel-text-{run_id}-", dir=ROOT / "private/review") as temp:
        temp_dir = Path(temp)
        manifest = temp_dir / "Cargo.toml"
        manifest.write_text(
            "[package]\nname = \"actor-panel-text-pixel-probe\"\nversion = \"0.1.0\"\n"
            "edition = \"2021\"\npublish = false\n\n[workspace]\n\n"
            "[[bin]]\nname = \"actor-panel-text-probe\"\npath = \"../../../scripts/actor_panel_text_probe.rs\"\n\n"
            "[dependencies]\n"
            "genewars-assets = { path = \"../../../rust/crates/genewars-assets\" }\n"
            "genewars-render = { path = \"../../../rust/crates/genewars-render\" }\n"
            "genewars-runtime = { path = \"../../../rust/crates/genewars-runtime\" }\n",
            encoding="utf-8")
        relative_manifest = Path(os.path.relpath(manifest, ROOT / "rust"))
        subprocess.run(["python", "-B", str(ROOT / "scripts/cargo.py"), "run", "--manifest-path",
                        str(relative_manifest), "--offline", "--", str(ROOT), str(out)],
                       cwd=ROOT, check=True)

    metadata = []
    for category in range(4):
        actual = (out / f"category-{category}.rgb").read_bytes()
        record = (out / f"category-{category}.record").read_bytes()
        if len(actual) != WIDTH * HEIGHT * 3 or len(record) != 36:
            raise SystemExit("harness output has an unexpected extent")
        nul = record[:20].find(b"\0")
        if nul < 1 or record[0x16] != category:
            raise SystemExit("generated blueprint name/category is not in the expected bounded fields")
        name, category_text = record[:nul], strings[128 + category]
        background = bytearray(WIDTH * HEIGHT * 3)
        for pixel in range(WIDTH * HEIGHT):
            background[pixel * 3:pixel * 3 + 3] = bytes((pixel % 251, (pixel // WIDTH) % 239, 37))
        expected = bytearray(background)
        name_width = render_line(expected, dat, records, palette, name, 84, 382)
        category_width = render_line(expected, dat, records, palette, category_text, 84, 390)
        render_raw_bars(expected, record, palette)
        if actual != expected:
            first = next(i for i, (a, b) in enumerate(zip(actual, expected)) if a != b)
            raise SystemExit(f"category {category} pixel mismatch at byte {first}: {actual[first]} != {expected[first]}")
        x, y, w, h = CLIP
        changed = sum(actual[i] != background[i] for py in range(y, y + h)
                      for px in range(x, x + w) for i in range((py * WIDTH + px) * 3, (py * WIDTH + px) * 3 + 3))
        metadata.append({"category": category, "category_text_index": 128 + category,
                         "name_width": name_width, "category_width": category_width,
                         "changed_channel_values_in_clip": changed, "rgb_sha256": sha(actual)})

    receipt = {
        "schema": "genewars-actor-panel-text-pixels-v1",
        "inputs": {str(path.relative_to(ROOT)): {"sha256": EXPECTED[path], "bytes": len(read_pinned(path))}
                   for path in EXPECTED},
        "font_records": len(records), "space_height": records[0][2],
        "viewport": {"origin": [84, 382], "size": [78, 21], "line_advance": 8},
        "source_evidence": {"path": "research/ghidra/actor-panel-text-evidence.json",
                            "sha256": sha((ROOT / "research/ghidra/actor-panel-text-evidence.json").read_bytes())},
        "font_color_control": {"white_rgb6": [63, 63, 63], "black_rgb6": [0, 0, 0],
                               "white_palette_index": nearest_palette_index(palette, (63, 63, 63)),
                               "black_palette_index": nearest_palette_index(palette, (0, 0, 0)),
                               "black_marker_literals": sorted(BLACK_MARKERS),
                               "evidence": "0x1D770 initializes FONT0 context color slots from 0x186F55/54; 0x715E0 resolves white/black against active palette; 0x1FF30 maps selected-panel glyph marker bytes"},
        "comparison": "independent Python FONT0 RLE/TEXT/palette/font-marker/bar model matched every byte of each Rust 640x480 RGB frame over a patterned background",
        "bars": {"raw_fields": [23, 25, 26], "origins": [[116, 415], [116, 435], [116, 435]],
                 "height": 6, "palette_indices": [232, 196, 200], "meaning_labels": None},
        "categories": metadata,
        "private_run_id": run_id,
        "probe_source": {"path": "scripts/actor_panel_text_probe.rs",
                         "sha256": sha((ROOT / "scripts/actor_panel_text_probe.rs").read_bytes())},
    }
    target = ROOT / "research/integration/actor-panel-text-pixels.json"
    target.write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(f"PASS actor panel pixel oracle: 4 categories, {4 * WIDTH * HEIGHT * 3:,} RGB bytes")


if __name__ == "__main__":
    main()

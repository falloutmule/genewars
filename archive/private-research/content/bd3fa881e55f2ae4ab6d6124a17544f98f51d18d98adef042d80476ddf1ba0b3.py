#!/usr/bin/env python3
"""Validate bounded selector-1 BTURR animation-0 ANB metadata."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import os
import uuid

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB"
LOADER = ROOT / "research/ghidra/building-loader-contract.json"
FLAGS_READER = ROOT / "rust/crates/genewars-runtime/src/actor_world/actor_updates.rs"
SCRIPT = Path(__file__).resolve()
OUT = ROOT / "research/formats/bturr-selector1-animation0.json"
EXPECTED_SHA256 = "c26df60af3a3166aec54112f197a4334966eb214eeaa48e0d9284e0510b8cd20"
EXPECTED_BYTES = 68_030


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def pin(path: Path) -> dict[str, str]:
    return {"path": path.relative_to(ROOT).as_posix(), "sha256": sha(path.read_bytes())}


def main() -> None:
    pins = [pin(p) for p in (RESOURCE, LOADER, FLAGS_READER, SCRIPT)]
    data = RESOURCE.read_bytes()
    if len(data) != EXPECTED_BYTES or sha(data) != EXPECTED_SHA256:
        raise SystemExit("BTURR resource identity mismatch")
    count = int.from_bytes(data[0:2], "little")
    offsets = [int.from_bytes(data[2 + 4*i:6 + 4*i], "little") for i in range(4)]
    if count != 5 or offsets != [66562, 67000, 67560, 67980]:
        raise SystemExit(f"BTURR ANB section layout changed: count={count}, offsets={offsets}")
    if [offsets[1]-offsets[0], offsets[2]-offsets[1], offsets[3]-offsets[2], len(data)-offsets[3]] != [438,560,420,50]:
        raise SystemExit("BTURR ANB section extents changed")
    # Loader contract binds selector 1 to this exact file/destination.
    loader = json.loads(LOADER.read_text(encoding="utf-8"))
    matches = [row for row in loader["building_descriptors"] if row.get("selector") == 1]
    if len(matches) != 1 or matches[0].get("filename", "").lower() != "data/building/bturr.anb" or matches[0].get("destination_va") != "0x116828":
        raise SystemExit("building-loader contract no longer binds selector 1 to BTURR at 0x116828")

    first = int.from_bytes(data[offsets[3]:offsets[3]+4], "little")
    last = int.from_bytes(data[offsets[3]+4:offsets[3]+8], "little")
    trailing = int.from_bytes(data[offsets[3]+8:offsets[3]+10], "little")
    frame_bytes = offsets[3] - offsets[2]
    if first % 14 or last % 14 or first > last or last + 14 > frame_bytes:
        raise SystemExit("BTURR animation 0 frame range is invalid")
    frames = []
    for frame_index, rel in enumerate(range(first, last + 1, 14)):
        p = offsets[2] + rel
        comp_rel = int.from_bytes(data[p:p+4], "little")
        width = int.from_bytes(data[p+4:p+6], "little", signed=True)
        height = int.from_bytes(data[p+6:p+8], "little", signed=True)
        hold = data[p+10]
        component_count = data[p+11]
        if comp_rel % 10 or comp_rel + component_count * 10 > offsets[2] - offsets[1]:
            raise SystemExit(f"BTURR frame {frame_index} component range invalid")
        frames.append({"frame_index": frame_index, "frame_section_offset": rel,
                       "width_raw": width, "height_raw": height, "hold_byte": hold,
                       "component_count": component_count})

    if (first, last, trailing) != (0, 28, 3) or frames != [
        {"frame_index": 0, "frame_section_offset": 0, "width_raw": 66, "height_raw": 212, "hold_byte": 0, "component_count": 1},
        {"frame_index": 1, "frame_section_offset": 14, "width_raw": 74, "height_raw": 228, "hold_byte": 0, "component_count": 1},
        {"frame_index": 2, "frame_section_offset": 28, "width_raw": 204, "height_raw": 320, "hold_byte": 0, "component_count": 2},
    ]:
        raise SystemExit("BTURR animation-0 metadata differs from pinned values")
    if [pin(p) for p in (RESOURCE, LOADER, FLAGS_READER, SCRIPT)] != pins:
        raise SystemExit("BTURR source pins changed during validation")
    report = {
        "schema_version": 1,
        "status": "passed-bturr-selector1-animation0-metadata",
        "binding": {"selector": 1, "filename": "data/building/BTURR.ANB", "descriptor_va": "0x103a9c", "destination_va": "0x116828"},
        "resource": {"private_source": RESOURCE.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)},
        "anb_layout": {"animation_count": count, "offsets": offsets, "section_lengths": [438,560,420,50]},
        "animation0": {"first_frame_section_offset": first, "last_frame_section_offset": last,
                       "frame_count": len(frames), "raw_trailing_word": trailing,
                       "flags_byte_high": (trailing >> 8) & 0xff, "trailing_low_byte": trailing & 0xff,
                       "flags_byte_extraction": "high byte of animation trailing word, as used by the shared animation updater",
                       "trailing_word_semantics": "low byte remains uninterpreted; high byte exposed as flags input, no ping-pong behavior inferred here",
                       "frames": frames},
        "source_pins": pins,
        "original_executable_run": False,
        "private_asset_bytes_written_to_report": False,
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    temp = OUT.with_name(f".{OUT.name}.{uuid.uuid4().hex}.tmp")
    try:
        temp.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8", newline="\n")
        os.replace(temp, OUT)
    finally:
        if temp.exists(): temp.unlink()
    print(json.dumps({"status": report["status"], "bytes": len(data), "animation0_frames": len(frames),
                      "holds": [frame["hold_byte"] for frame in frames],
                      "output": OUT.relative_to(ROOT).as_posix()}, indent=2))


if __name__ == "__main__":
    main()

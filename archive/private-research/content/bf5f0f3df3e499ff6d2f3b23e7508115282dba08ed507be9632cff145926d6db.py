#!/usr/bin/env python3
"""Hash-check and summarize category-0 architect ANB animation metadata.

The script reads only the four already-staged category-0 actor databases. It
emits offsets, counts, hold bytes, and component flags, never sprite payloads.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import uuid

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "research/ghidra/blueprint-init-evidence.json"
SPECS = ROOT / "rust/crates/genewars-runtime/src/actor_resources.rs"
ASSET_PARSER = ROOT / "rust/crates/genewars-assets/src/lib.rs"
ACTOR_ANIMATION_EVIDENCE = ROOT / "research/ghidra/actor-animation-evidence.json"
ACTOR_ANIMATION_SOURCE = ROOT / "rust/crates/genewars-runtime/src/actor_motion.rs"
INIT_LISTING = ROOT / "private/ghidra/actor-panel-font-color/instructions-0001d770.txt"
BUILDING_HOOK = ROOT / "private/ghidra/construction-refs/module-range.txt"
BUILDING_REFERENCE_GRAPH = ROOT / "private/ghidra/building-loader-refs/reference-graph.json"
BUILDING_CONTRACT = ROOT / "research/ghidra/building-loader-contract.json"
BUILDING_IMAGE = ROOT / "private/ghidra/actor-activation/memory-.object3.bin"
OUT = ROOT / "research/formats/category0-construction-animation-metadata.json"
ANIMATIONS = (13, 14, 15)
EXPECTED_ANIM_COUNTS = {0: 18, 4: 18, 8: 18, 12: 18}


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def source_pin(path: Path) -> dict[str, str]:
    return {"path": path.relative_to(ROOT).as_posix(), "sha256": sha(path.read_bytes())}


def parse_animation(data: bytes, offsets: list[int], animation_index: int) -> dict:
    component_start, frame_start, animation_start, animation_end = offsets
    entry = animation_start + animation_index * 10
    first = int.from_bytes(data[entry:entry + 4], "little")
    last = int.from_bytes(data[entry + 4:entry + 8], "little")
    raw_word = int.from_bytes(data[entry + 8:entry + 10], "little")
    frame_bytes_len = animation_start - frame_start
    if last < first or (last - first) % 14 or last + 14 > frame_bytes_len:
        raise ValueError(f"animation {animation_index} has an invalid frame range")
    frame_count = (last - first) // 14 + 1
    frames = []
    component_bytes_len = frame_start - component_start
    for index in range(frame_count):
        frame_offset = frame_start + first + index * 14
        component_offset = int.from_bytes(data[frame_offset:frame_offset + 4], "little")
        component_count = data[frame_offset + 11]
        component_end = component_offset + component_count * 10
        if component_offset % 10 or component_end > component_bytes_len:
            raise ValueError(f"animation {animation_index} frame {index} has an invalid component range")
        flags = [data[component_start + component_offset + component * 10 + 8]
                 for component in range(component_count)]
        frames.append({
            "index": index,
            "frame_section_offset": first + index * 14,
            "hold_byte_at_frame_plus_10": data[frame_offset + 10],
            "component_count": component_count,
            "component_flags": flags,
        })
    flags = (raw_word >> 8) & 0xFF
    return {
        "animation_index": animation_index,
        "first_frame_index": first // 14,
        "last_frame_index": last // 14,
        "first_frame_section_offset": first,
        "last_frame_section_offset": last,
        "frame_count": frame_count,
        "raw_trailing_word": raw_word,
        "animation_flags_high_byte": flags,
        "ping_pong_flag_bit0": bool(flags & 1),
        "frames": frames,
    }


def decode_summary(data: bytes, entry: dict, resource_path: Path) -> dict:
    if len(data) != entry["decoded_size"] or sha(data) != entry["decoded_sha256"]:
        raise ValueError(f"staged actor resource mismatch for {entry['source_name']}")
    if len(data) < 18:
        raise ValueError("truncated ANB header")
    header_count = int.from_bytes(data[:2], "little")
    offsets = [int.from_bytes(data[2 + 4 * index:6 + 4 * index], "little")
               for index in range(4)]
    if (header_count != entry["animation_count"] or offsets[0] < 18
            or offsets != sorted(offsets) or offsets[3] > len(data)):
        raise ValueError(f"ANB section/header mismatch for {entry['source_name']}")
    component_bytes, frame_bytes = offsets[2] - offsets[1], offsets[3] - offsets[2]
    animation_start = offsets[3]
    if (component_bytes % 10 or frame_bytes % 14
            or animation_start + header_count * 10 != len(data)):
        raise ValueError(f"ANB metadata sections are misaligned for {entry['source_name']}")
    # Sprite table begins at offsets[0], followed by components, frames, and
    # animation records. Metadata parsers receive the exact component/frame/
    # animation boundaries plus EOF as the final bound.
    metadata_offsets = [offsets[1], offsets[2], offsets[3], len(data)]
    animations = []
    for animation_index in ANIMATIONS:
        animations.append(parse_animation(data, metadata_offsets, animation_index))
    return {
        "style": entry["style"],
        "category": entry["category"],
        "resource_key": entry["key"],
        "source_name": entry["source_name"],
        "manifest_entry": entry["entry_id"],
        "decoded_sha256": sha(data),
        "decoded_bytes": len(data),
        "private_staged_path": resource_path.relative_to(ROOT).as_posix(),
        "anb_animation_count": header_count,
        "animations": animations,
    }


def building_selector1_status() -> dict:
    image = BUILDING_IMAGE.read_bytes()
    references = json.loads(BUILDING_REFERENCE_GRAPH.read_text(encoding="utf-8"))
    contract = json.loads(BUILDING_CONTRACT.read_text(encoding="utf-8"))
    # The 14-entry descriptor-pointer table in the loaded image is zero-filled.
    offset = 0x116824 - 0x100000
    if offset < 0 or offset + 14 * 4 > len(image):
        raise ValueError("building descriptor table address is outside the captured object")
    pointer_words = [int.from_bytes(image[offset + index * 4:offset + index * 4 + 4], "little")
                     for index in range(14)]
    selector1 = next((row for row in contract["building_descriptors"] if row["selector"] == 1), None)
    if selector1 != {"descriptor_va": "0x103a9c", "filename": "data/building/BTURR.ANB",
                     "destination_va": "0x116828", "selector": 1}:
        raise ValueError("selector-1 descriptor mapping no longer matches the pinned contract")
    return {
        "status": "proved",
        "source_name": selector1["filename"],
        "selector": 1,
        "descriptor_va": selector1["descriptor_va"],
        "destination_va": selector1["destination_va"],
        "loader_contract": "startup conditionally loads nonzero descriptor pointers from 0x116824 + 4*i into 0x11685c + 4*i; building hook indexes loaded table by signed selector byte",
        "indirect_writer_contract": "154b0 walks the 44-byte descriptor list at 0x10388c; 9a09c writes the allocated file pointer through descriptor+0x1c; descriptor 0x103a9c therefore fills 0x116828",
        "selector_hook_va": "0x44ff0",
        "selector_table_va": "0x11685c",
        "descriptor_pointer_table_va": "0x116824",
        "captured_static_pointer_words_all_zero": all(word == 0 for word in pointer_words),
        "captured_static_pointer_word_count": len(pointer_words),
        "direct_reference_graph_sites": [
            {"address": ref["from"], "type": ref["type"], "function": ref.get("function")}
            for ref in references if ref["target"] in {"00116824", "0011685c"}
        ],
        "note": "Captured object3 pointer words are zero before runtime loading. The descriptor-list path writes them indirectly through descriptor+0x1c; selector1 is BTURR.ANB, not the neighboring BFARM descriptor.",
    }


def main() -> None:
    evidence = json.loads(EVIDENCE.read_text(encoding="utf-8"))
    entries = {entry["key"]: entry for entry in evidence["resources"]
               if entry["category"] == 0 and entry["style"] in range(4)}
    keys = (0, 4, 8, 12)
    if set(entries) != set(keys):
        raise ValueError(f"expected four category-0 actor resources, got {sorted(entries)}")
    pins = [source_pin(path) for path in (EVIDENCE, SPECS, ASSET_PARSER,
                                           ACTOR_ANIMATION_EVIDENCE, ACTOR_ANIMATION_SOURCE,
                                           INIT_LISTING, BUILDING_HOOK, BUILDING_REFERENCE_GRAPH,
                                           BUILDING_CONTRACT,
                                           ROOT / "private/ghidra/descriptor-loader/instructions-0009a09c.txt",
                                           ROOT / "private/ghidra/planets-load/instructions-000154b0.txt",
                                           ROOT / "private/ghidra/animal-loader/instructions-0006ac50.txt",
                                           BUILDING_IMAGE)]
    actors = []
    for key in keys:
        entry = entries[key]
        if entry["animation_count"] != EXPECTED_ANIM_COUNTS[key]:
            raise ValueError(f"unexpected animation count for resource key {key}")
        path = ROOT / "private/integration/game-runtime/actors" / f"key-{key:02}.anb"
        actors.append(decode_summary(path.read_bytes(), entry, path))
    output = {
        "schema_version": 1,
        "scope": "Metadata-only summary of category-0 actor ANBs and animations 13–15. No sprite payload bytes are emitted and no construction label/visual meaning is inferred.",
        "actor_resource_table": {"evidence": "research/ghidra/blueprint-init-evidence.json", "key_rule": "4*style+category", "category": 0},
        "animations": actors,
        "building_selector1": building_selector1_status(),
        "source_pins": pins,
    }
    current_pins = [source_pin(path) for path in (EVIDENCE, SPECS, ASSET_PARSER,
                                                    ACTOR_ANIMATION_EVIDENCE, ACTOR_ANIMATION_SOURCE,
                                                    INIT_LISTING, BUILDING_HOOK, BUILDING_REFERENCE_GRAPH,
                                                    BUILDING_CONTRACT,
                                                    ROOT / "private/ghidra/descriptor-loader/instructions-0009a09c.txt",
                                                    ROOT / "private/ghidra/planets-load/instructions-000154b0.txt",
                                                    ROOT / "private/ghidra/animal-loader/instructions-0006ac50.txt",
                                                    BUILDING_IMAGE)]
    if current_pins != pins:
        raise RuntimeError("a pinned source changed while metadata was being read")
    OUT.parent.mkdir(parents=True, exist_ok=True)
    encoded = json.dumps(output, indent=2, sort_keys=True) + "\n"
    temp = OUT.with_name(f".{OUT.name}.{uuid.uuid4().hex}.tmp")
    try:
        temp.write_text(encoded, encoding="utf-8", newline="\n")
        os.replace(temp, OUT)
    finally:
        if temp.exists():
            temp.unlink()
    print(json.dumps({"output": OUT.relative_to(ROOT).as_posix(), "actor_resources": len(actors),
                      "animations": sum(len(actor["animations"]) for actor in actors),
                      "building_selector1": output["building_selector1"]["source_name"]}, indent=2))


if __name__ == "__main__":
    main()

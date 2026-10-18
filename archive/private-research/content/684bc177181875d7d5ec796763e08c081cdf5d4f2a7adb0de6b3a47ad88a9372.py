#!/usr/bin/env python3
"""Independent PLGBL post-image overlay pixel oracle for BTURR selector 1.

The script reads the staged private PLGBL.ANB and MAIN.GLA, pins the four
source listings, independently decodes the four referenced one-frame ANB
images, and models the bounded 43A40 -> 982E0 -> A7730 path. It does not
claim a building shadow pass, scene ordering, or complete 43A40 behavior.
"""
from __future__ import annotations

import hashlib
import json
import subprocess
import struct
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PLGBL = ROOT / "private/reference-run/run-006/drive-c/GAME/DATA/PLGBL.ANB"
DECODED_PLGBL = ROOT / "private/review/PLGBL-decoded.bin"
GLA = ROOT / "private/integration/game-runtime/terrain/MAIN.GLA"
PLGBL_IDENTITY = ROOT / "research/ghidra/building-plgbl-overlay-contract.json"
LISTINGS = {
    "43a40": ROOT / "private/ghidra/building-draw/instructions-00043a40.txt",
    "982e0": ROOT / "private/ghidra/building-draw/instructions-000982e0.txt",
    "a7730": ROOT / "private/ghidra/display-geometry/instructions-000a7730.txt",
    "98a60": ROOT / "private/ghidra/sprite-functions/instructions-00098a60.txt",
}
OUT = ROOT / "research/integration/bturr-status-overlay-pixels-current.json"
RUST_HARNESS = ROOT / "scripts/desktop/bturr_status_overlay_harness.rs"
RUST_SCRATCH = ROOT / "private/integration/bturr-status-overlay-rust"
PITCH, WIDTH, HEIGHT = 648, 640, 480
PINNED_ASSET_SHA = {
    "plgbl_raw": "CCE0F7DE88F2DC8EC251E70925DE93D6E406C109D0BEA9A82F4EE7F1592C9A30",
    "plgbl_decoded": "A39CC8AC8C987DE5C4865EC77A4459221F276286F254653E4A3F3231424F6444",
    "main_gla": "BF841A1C4FCA2A80FBEF3C2FD6B502C4AF34EDE8E67B41FF69F64836726EA5B3",
}
PINNED_LISTING_SHA = {
    "43a40": "62A4377CB232402BE9CF5A1FB0C7DA61D29CC582632C60C217B0357E2961FF1B",
    "982e0": "2F34698B16267DA198CE40F56B82157367C539A43A8A15F1C4DAC7001D94C5B3",
    "a7730": "AA63D9CAF20FBCD13F8D139958DF0E542A2CA45437A42B7249C87ADBE9A60F00",
    "98a60": "75B266F751D4C7D29DC4D5BF83493B821CDB38F7E10838403DAA29AC1FDC4A8A",
}
PINNED_PLGBL_IDENTITY_SHA = "9C85D349C6296182732387A15FFC04BB976E9A1909D848139419A0EB14944A58"


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest().upper()


def u16(data: bytes, at: int) -> int:
    return struct.unpack_from("<H", data, at)[0]


def i16(data: bytes, at: int) -> int:
    return struct.unpack_from("<h", data, at)[0]


def u32(data: bytes, at: int) -> int:
    return struct.unpack_from("<I", data, at)[0]


def anb_sections(data: bytes):
    if len(data) < 18:
        raise ValueError("truncated ANB header")
    count = u16(data, 0)
    off = tuple(u32(data, 2 + 4 * i) for i in range(4))
    if not (18 <= off[0] <= off[1] <= off[2] <= off[3] <= len(data)):
        raise ValueError("invalid ANB section offsets")
    if (off[1] - off[0]) % 6 or (off[2] - off[1]) % 10 or (off[3] - off[2]) % 14:
        raise ValueError("unexpected PLGBL table record sizes")
    if len(data) - off[3] != count * 10:
        raise ValueError("unexpected animation table extent")
    return count, off


def sprite_from_table(data: bytes, offsets, record_index: int):
    table_start, table_end = offsets[:2]
    count = (table_end - table_start) // 6
    if not 0 <= record_index < count:
        raise ValueError("sprite record outside table")
    at = table_start + record_index * 6
    data_offset, width, height = struct.unpack_from("<IBB", data, at)
    start = 0x12 + data_offset
    if width == 0 or height == 0:
        return width, height, [0] * (width * height), [False] * (width * height), 0
    if not 0x12 <= start < table_start:
        raise ValueError("sprite data outside bounded database prefix")
    pixels = [0] * (width * height)
    opaque = [False] * (width * height)
    cur = start
    for y in range(height):
        x = 0
        while True:
            if cur >= table_start:
                raise ValueError("truncated RLE control")
            control = struct.unpack_from("b", data, cur)[0]
            cur += 1
            if control == 0:
                break
            if control > 0:
                end = cur + control
                if end > table_start or x + control > width:
                    raise ValueError("literal RLE run exceeds sprite/database")
                for source in data[cur:end]:
                    index = y * width + x
                    pixels[index] = source
                    opaque[index] = True
                    x += 1
                cur = end
            else:
                x += -control
                if x > width:
                    raise ValueError("transparent RLE run exceeds sprite width")
    return width, height, pixels, opaque, cur - start


def overlay_sprite(data: bytes, offsets, animation_index: int):
    _, _, _, animation_start = offsets
    if not 0 <= animation_index < 44:
        raise ValueError("PLGBL animation outside admitted table")
    entry = animation_start + animation_index * 10
    first, last, trailing = struct.unpack_from("<IIH", data, entry)
    if first != last or first % 14:
        raise ValueError("overlay animation is not the pinned one-frame form")
    frame_idx = first // 14
    frame_at = offsets[2] + first
    component_off = u32(data, frame_at)
    raw_w, raw_h = i16(data, frame_at + 4), i16(data, frame_at + 6)
    component_count = data[frame_at + 11]
    if component_count != 1:
        raise ValueError("expected one proved overlay component")
    comp_at = offsets[1] + component_off
    sprite_off = u32(data, comp_at)
    if sprite_off % 6:
        raise ValueError("unaligned component sprite reference")
    sprite_index = sprite_off // 6
    x_raw, y_raw = i16(data, comp_at + 4), i16(data, comp_at + 6)
    comp_flags = data[comp_at + 8]
    width, height, pixels, opaque, consumed = sprite_from_table(data, offsets, sprite_index)
    frame = {
        "animation": animation_index,
        "animation_entry_offset": entry - animation_start,
        "first_frame_offset": first,
        "last_frame_offset": last,
        "trailing_word": trailing,
        "frame_index": frame_idx,
        "frame_dimensions": [raw_w >> 1, raw_h >> 1],
        "component_count": component_count,
        "component_sprite_index": sprite_index,
        "component_offset_raw": [x_raw, y_raw],
        "component_flags": comp_flags,
        "sprite_dimensions": [width, height],
        "sprite_consumed_bytes": consumed,
        "opaque_pixels": sum(opaque),
        "opaque_zero_pixels": sum(1 for p, m in zip(pixels, opaque) if m and p == 0),
    }
    return frame, (width, height, pixels, opaque, x_raw >> 1, y_raw >> 1, comp_flags)


def draw_sprite(surface: bytearray, sprite, origin, gla: bytes | None, flags: int):
    sw, sh, pix, mask, sx, sy, comp_flags = sprite
    full_flags = flags | comp_flags
    if full_flags & ~0x04:
        raise ValueError("overlay oracle admits only source-backed base flags 0 or 4")
    left, top = origin[0] + sx, origin[1] + sy
    written = clipped = 0
    for yy in range(sh):
        dy = top + yy
        for xx in range(sw):
            src_i = yy * sw + xx
            if not mask[src_i]:
                continue
            dx = left + xx
            if dx < 0 or dx >= WIDTH or dy < 0 or dy >= HEIGHT:
                clipped += 1
                continue
            at = dy * PITCH + dx
            src = pix[src_i]
            if full_flags & 0x04:
                if gla is None or len(gla) != 65536:
                    raise ValueError("GLA required for flag-4 remap")
                surface[at] = gla[(src << 8) | surface[at]]
            else:
                surface[at] = src
            written += 1
    return written, clipped


def draw_sprite_atomic(surface: bytearray, sprite, origin, gla: bytes | None, flags: int):
    if len(surface) < PITCH * (HEIGHT - 1) + WIDTH:
        raise ValueError("destination extent is too short")
    if flags not in (0, 4) or (flags == 4 and (gla is None or len(gla) != 65536)):
        raise ValueError("unadmitted overlay blitter inputs")
    staged = bytearray(surface)
    receipt = draw_sprite(staged, sprite, origin, gla, flags)
    surface[:] = staged
    return receipt


def select_overlay(record: bytes, origin, *, selected_owner: int, global_frame: int,
                   completed_threshold: int | None = None):
    """43A40 post-image overlay selector; threshold is explicit fixture input."""
    if len(record) != 67:
        raise ValueError("BTURR record must be 67 bytes")
    if record[0x18] != (selected_owner & 0xFF) or record[0x17] == 0 or (global_frame & 0x3F) < 0x20:
        return None
    flags = u16(record, 0x31)
    if flags & 0x0100:
        anim = 11
    elif not flags & 1:
        if record[0x2E] == record[0x2F]:
            return None
        anim = 12
    else:
        if completed_threshold is None:
            raise ValueError("completed overlay requires explicit threshold input")
        branch_a = (flags & 0x04) == 0
        branch_b = i16(record, 0x2B) < completed_threshold
        if branch_a and branch_b:
            if (global_frame & 0x7F) < 0x40:
                branch_b = False
            else:
                branch_a = False
        if branch_a:
            anim = 2
        elif branch_b:
            anim = 13
        else:
            return None
    # 43A40 adjusts X by byte+0x30 parity and wraps signed-word Y-10.
    x = origin[0] - (record[0x30] & 1 if record[0x30] > 1 else 0)
    y = struct.unpack("<h", struct.pack("<H", ((origin[1] - 10) & 0xFFFF)))[0]
    return {"animation": anim, "origin": [x, y], "base_frame": global_frame & 0xFF}


def make_record(*, owner=2, shade=1, flags=0, grade=0, compare_grade=1, health=1, selector=1, parity=2):
    rec = bytearray(67)
    rec[0x18] = owner & 0xFF
    rec[0x17] = shade & 0xFF
    rec[0x2D] = selector & 0xFF
    rec[0x2E] = grade & 0xFF
    rec[0x2F] = compare_grade & 0xFF
    rec[0x2B:0x2D] = struct.pack("<h", health)
    rec[0x30] = parity & 0xFF
    rec[0x31:0x33] = struct.pack("<H", flags)
    return bytes(rec)


def main():
    raw = PLGBL.read_bytes()
    decoded = DECODED_PLGBL.read_bytes()
    gla = GLA.read_bytes()
    identity_bytes = PLGBL_IDENTITY.read_bytes()
    assert sha(identity_bytes) == PINNED_PLGBL_IDENTITY_SHA, "PLGBL loader identity receipt changed"
    identity = json.loads(identity_bytes)
    assert identity["descriptor"] == {
        "va": "103D88", "stride": 44, "filename": "data/plgbl.anb",
        "destination_at_plus1c": "116808",
    }
    asset_hashes = {"plgbl_raw": sha(raw), "plgbl_decoded": sha(decoded), "main_gla": sha(gla)}
    assert asset_hashes == PINNED_ASSET_SHA, "asset input changed; refresh evidence intentionally"
    count, offsets = anb_sections(decoded)
    assert count == 44 and offsets == (35670, 36300, 37860, 39974)
    assert len(gla) == 65536
    frames = {}
    sprites = {}
    for anim in (2, 11, 12, 13):
        frames[str(anim)], sprites[anim] = overlay_sprite(decoded, offsets, anim)
    # These are the four selector-immediate animation-table entries used by 43A40.
    expected = {2: 0x14, 11: 0x6E, 12: 0x78, 13: 0x82}
    for anim, offset in expected.items():
        assert offset == anim * 10
        assert frames[str(anim)]["component_flags"] == 0
        assert frames[str(anim)]["component_offset_raw"] == [-22, -22]
        assert frames[str(anim)]["sprite_dimensions"] == [23, 23]

    # Check the instruction path that gives the frames meaning and documents
    # why this component renderer has no shadow pass.
    listing_text = {name: path.read_text(encoding="utf-8") for name, path in LISTINGS.items()}
    listing_pins = {name: sha(path.read_bytes()) for name, path in LISTINGS.items()}
    assert listing_pins == PINNED_LISTING_SHA, "instruction listing changed; refresh evidence intentionally"
    assert "[0x0011680c]" in listing_text["43a40"] and "CALL 0x000982e0" in listing_text["43a40"]
    assert all(f"[EBX + 0x{off:x}]" in listing_text["43a40"] for off in (0x14, 0x6E, 0x78, 0x82))
    assert "CALL 0x000a7730" in listing_text["982e0"]
    assert "CALL 0x00096ac0" not in listing_text["982e0"]
    assert "CALL 0x00096ac0" not in listing_text["a7730"]
    assert "CALL 0x00096ac0" not in listing_text["43a40"]
    assert "MOV EAX,dword ptr [ECX + 0xe]" in listing_text["98a60"]

    # Branch vectors cover each overlay plus the source gates and completed flicker split.
    cases = [
        ("high_flag", make_record(flags=0x0100), 0x20, 20),
        ("incomplete_grade", make_record(flags=0, grade=0, compare_grade=1), 0x20, None),
        ("completed_flag4_clear_pre_phase", make_record(flags=1, health=5), 0x20, 10),
        ("completed_flag4_clear_post_phase", make_record(flags=1, health=5), 0x60, 10),
        ("completed_flag4_set_low_health", make_record(flags=5, health=5), 0x60, 10),
        ("completed_flag4_set_high_health", make_record(flags=5, health=20), 0x60, 10),
        ("odd_parity_origin_adjust", make_record(flags=0x0100, parity=3), 0x20, 20),
    ]
    result_cases = []
    whole_hashes = {}
    expected_frames = {}
    for name, record, frame, threshold in cases:
        placement = select_overlay(record, [123, 90], selected_owner=2, global_frame=frame,
                                   completed_threshold=threshold)
        if name == "high_flag":
            assert placement and placement["animation"] == 11
        if name == "incomplete_grade":
            assert placement and placement["animation"] == 12
        if name == "completed_flag4_clear_pre_phase":
            assert placement and placement["animation"] == 2
        if name in ("completed_flag4_clear_post_phase", "completed_flag4_set_low_health"):
            assert placement and placement["animation"] == 13
        if name == "completed_flag4_set_high_health":
            assert placement is None
        if name == "odd_parity_origin_adjust":
            assert placement and placement["origin"] == [122, 80]
        screen = bytearray(((i * 17 + 9) & 0xFF) for i in range(PITCH * HEIGHT))
        # This adapter preserves row padding; it is not a full 43A40 renderer.
        written = clipped = 0
        if placement is not None:
            written, clipped = draw_sprite(screen, sprites[placement["animation"]],
                                           placement["origin"], gla, 0)
        whole_hashes[name] = sha(bytes(screen))
        expected_frames[name] = bytes(screen)
        assert all(screen[row * PITCH + WIDTH:(row + 1) * PITCH] == bytes(((i * 17 + 9) & 0xFF) for i in range(row * PITCH + WIDTH, (row + 1) * PITCH)) for row in range(HEIGHT))
        result_cases.append({"name": name, "selected_overlay": placement,
                             "pixels_written": written, "pixels_clipped": clipped,
                             "framebuffer_sha256": whole_hashes[name]})

    # Exercise both admitted global-color paths over complete padded frames.
    flag4_cases = []
    for flags in (0, 4):
        record = make_record(flags=0x0100)
        placement = select_overlay(record, [320, 240], selected_owner=2, global_frame=0x20)
        assert placement is not None and placement["animation"] == 11
        surface = bytearray(((i * 29 + 7) & 0xFF) for i in range(PITCH * HEIGHT))
        written, clipped = draw_sprite_atomic(surface, sprites[11], placement["origin"],
                                               gla if flags == 4 else None, flags)
        flag4_cases.append({"global_flags": flags, "written": written, "clipped": clipped,
                            "framebuffer_sha256": sha(bytes(surface))})
        expected_frames[f"flag_path_{flags}"] = bytes(surface)
        assert written > 0
        assert all(surface[row * PITCH + WIDTH:(row + 1) * PITCH] == bytes(((i * 29 + 7) & 0xFF) for i in range(row * PITCH + WIDTH, (row + 1) * PITCH)) for row in range(HEIGHT))

    # Invalid late table/bounds inputs are rejected transactionally.
    atomic_surface = bytearray([0xC3] * (PITCH * HEIGHT))
    atomic_before = bytes(atomic_surface)
    try:
        draw_sprite_atomic(atomic_surface, sprites[11], [20, 20], b"short", 4)
    except ValueError:
        pass
    else:
        raise AssertionError("short GLA should be rejected")
    assert bytes(atomic_surface) == atomic_before
    try:
        draw_sprite_atomic(atomic_surface, sprites[11], [20, 20], None, 1)
    except ValueError:
        pass
    else:
        raise AssertionError("unsupported global flags should be rejected")
    assert bytes(atomic_surface) == atomic_before
    short_surface = bytearray([0xC3] * (PITCH * (HEIGHT - 1) + WIDTH - 1))
    short_before = bytes(short_surface)
    try:
        draw_sprite_atomic(short_surface, sprites[11], [20, 20], None, 0)
    except ValueError:
        pass
    else:
        raise AssertionError("short destination should be rejected")
    assert bytes(short_surface) == short_before

    # Gate and clipped-origin adversarial examples.
    assert select_overlay(make_record(owner=1), [0, 0], selected_owner=2, global_frame=0x20) is None
    assert select_overlay(make_record(shade=0), [0, 0], selected_owner=2, global_frame=0x20) is None
    assert select_overlay(make_record(), [0, 0], selected_owner=2, global_frame=0x1F) is None
    clip_record = make_record(flags=0x0100)
    clip = select_overlay(clip_record, [-6, 4], selected_owner=2, global_frame=0x20)
    assert clip and clip["origin"] == [-6, -6]
    clip_surface = bytearray([0xA5] * (PITCH * HEIGHT))
    clipped_written, clipped_count = draw_sprite(clip_surface, sprites[clip["animation"]],
                                                  clip["origin"], None, 0)
    assert clipped_written == 10 and clipped_count == 399
    expected_frames["clipping"] = bytes(clip_surface)

    # Independent Rust decoder/drawer must produce byte-identical full padded
    # frames for the same source-derived cases. The Rust source is a standalone
    # proof harness and does not link the production renderer.
    sys.path.insert(0, str(ROOT / "scripts"))
    from cargo import environment as cargo_environment
    rustc = ROOT / "private/tools/rust/cargo/bin/rustc.exe"
    if not rustc.is_file():
        raise RuntimeError("project-local rustc is required for the cross-language pixel check")
    rust_env = cargo_environment()
    RUST_SCRATCH.mkdir(parents=True, exist_ok=True)
    exe = RUST_SCRATCH / "bturr_status_overlay_harness.exe"
    run_dir = RUST_SCRATCH / "frames"
    run_dir.mkdir(parents=True, exist_ok=True)
    harness_source_hash = sha(RUST_HARNESS.read_bytes())
    subprocess.run([str(rustc), "--edition=2021", "-O", str(RUST_HARNESS), "-o", str(exe)],
                   check=True, cwd=ROOT, env=rust_env, capture_output=True, text=True)
    subprocess.run([str(exe), str(DECODED_PLGBL), str(GLA), str(run_dir)],
                   check=True, cwd=ROOT, env=rust_env, capture_output=True, text=True)
    rust_checks = []
    for name, expected_bytes in expected_frames.items():
        actual = (run_dir / f"{name}.bin").read_bytes()
        if actual != expected_bytes:
            raise AssertionError(f"Rust full-frame mismatch: {name}")
        rust_checks.append({"case": name, "bytes": len(actual), "sha256": sha(actual),
                            "full_byte_equality": True})
    if sha(RUST_HARNESS.read_bytes()) != harness_source_hash:
        raise RuntimeError("Rust harness changed during validation")
    if ({"plgbl_raw": sha(PLGBL.read_bytes()),
         "plgbl_decoded": sha(DECODED_PLGBL.read_bytes()),
         "main_gla": sha(GLA.read_bytes())} != asset_hashes
            or {name: sha(path.read_bytes()) for name, path in LISTINGS.items()} != listing_pins
            or sha(PLGBL_IDENTITY.read_bytes()) != PINNED_PLGBL_IDENTITY_SHA):
        raise RuntimeError("source or asset pins changed during validation")

    source_pins = listing_pins
    report = {
        "schema": 1,
        "claim": "bounded PLGBL post-image overlay pixel oracle; no shadow or complete-scene claim",
        "assets": {
            "plgbl_source_sha256": sha(raw), "plgbl_source_bytes": len(raw),
            "plgbl_decoded_sha256": sha(decoded), "plgbl_decoded_bytes": len(decoded),
            "main_gla_sha256": sha(gla), "main_gla_bytes": len(gla),
            "anb_offsets": list(offsets), "animation_count": count,
        },
        "plgbl_loader_identity_receipt_sha256": sha(identity_bytes),
        "source_listing_sha256": source_pins,
        "source_pins_unchanged_after_render": True,
        "rust_harness_sha256": harness_source_hash,
        "overlay_frames": frames,
        "pixel_policy": {
            "base_global_flags": [0, 4], "component_flags": 0,
            "flags4_lookup": "MAIN.GLA[(source << 8) | destination]",
            "transparent_runs_preserve_destination": True,
            "literal_zero_pixels_remain_opaque": True,
            "row_padding_preserved": True,
            "completed_health_threshold": "explicit test/caller value; threshold-table resource not loaded by this oracle",
        },
        "flag_path_cases": flag4_cases,
        "cross_language_pixel_checks": rust_checks,
        "atomic_rejections": ["short GLA with flag 4", "unsupported global flags", "destination too short"],
        "cases": result_cases,
        "clipping_case": {"origin": clip["origin"], "written": clipped_written,
                          "clipped": clipped_count, "framebuffer_sha256": sha(bytes(clip_surface))},
        "gates": {"owner_mismatch": "rejected", "shade_zero": "rejected",
                  "frame_phase_below_32": "rejected"},
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "PASS", "source_listings": source_pins,
                      "overlay_cases": len(result_cases), "report": str(OUT.relative_to(ROOT))}, indent=2))


if __name__ == "__main__":
    main()

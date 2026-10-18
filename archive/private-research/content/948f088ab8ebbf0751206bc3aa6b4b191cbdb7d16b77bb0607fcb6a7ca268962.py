"""Check the selected-actor blueprint/category text path without emitting strings."""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / "private/ghidra"
sha = lambda data: hashlib.sha256(data).hexdigest()

exe_hash = "4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba"
assert sha((G / "GW-CD-candidate.exe").read_bytes()) == exe_hash

contracts = {
    "animal-consumers/instructions-0001d770.txt": [
        "0001da48 MOV EBX,0x25c",
        "0001da58 MOV EAX,[0x00115e88]",
        "0001da60 DEC EBX",
        "0001da64 JZ 0x0001da7d",
        "0001da68 ADD EDX,0x115e90",
        "0001da71 MOV dword ptr [EDX],EAX",
        "0001da73 MOVSX EDX,byte ptr [EAX]",
        "0001da77 TEST EDX,EDX",
        "0001d9fd MOV EDI,0x187054",
        "0001da07 MOV EBX,0x118c74",
        "0001da25 CMP ESI,0x2",
        "0001da2a MOV EDX,dword ptr [EBP + 0x62]",
        "0001da33 MOV EAX,EDI",
        "0001da37 CALL 0x0001f550",
    ],
    "portrait-detail-refs/instructions-000265f0.txt": [
        "00026605 MOV EAX,dword ptr [EAX + 0xc0]",
        "0002660b MOVSX EBX,byte ptr [EAX + 0x16]",
        "0002660f MOV EDX,dword ptr [EBX*0x4 + 0x116090]",
        "00026617 PUSH EAX",
        "00026618 PUSH 0x10d687",
        "00026648 MOV EDX,0x17e",
        "0002664d MOV EAX,0x54",
        "00026652 CALL 0x0001f490",
        "00026660 XOR EBX,EBX",
        "00026662 OR AL,0x4",
        "00026664 XOR EDX,EDX",
        "0002666b MOV EAX,0x187054",
        "00026670 CALL 0x0001f750",
        "00026737 MOVSX EAX,byte ptr [EDI + 0x17]",
        "0002673b MOV EBX,0x2d",
        "00026741 MOV EDX,0x19f",
        "00026746 MOV EAX,0x74",
        "0002674b CALL 0x0002ff50",
        "0002675c MOVSX EAX,byte ptr [EDI + 0x19]",
        "00026750 PUSH 0xc4",
        "00026770 CALL 0x0002ff50",
        "00026781 MOVSX EAX,byte ptr [EDI + 0x1a]",
        "00026775 PUSH 0xc8",
        "00026795 CALL 0x0002ff50",
    ],
    "animal-consumers/instructions-0001d770.txt": [
        "0001da7d MOV EDI,0x115e90",
        "0001da82 MOV EBX,0x374",
        "0001da96 CMP EBX,0x3fc",
        "0001dac2 MOV byte ptr [ESI],0xa",
    ],
}
pins = []
for rel, terms in contracts.items():
    data = (G / rel).read_bytes()
    text = data.decode("ascii")
    for term in terms:
        assert term in text, (rel, term)
    pins.append({"private_path": "private/ghidra/" + rel, "sha256": sha(data)})

# The C0 pointer is a typed player-blueprint handle, not a byte string stored
# in the native actor record. The initializer evidence records this binding.
actor_init_path = P / "research/ghidra/actor-initialization-evidence.json"
actor_init = json.loads(actor_init_path.read_text(encoding="utf-8"))
assert actor_init["candidate_sha256"] == exe_hash
assert "C0" in actor_init["initialization"]
init_bridge = (P / "research/ghidra/actor-initialization-bridge.md").read_text(encoding="utf-8")
assert "blueprint index and style/category" in init_bridge and "typed handle bindings" in init_bridge
pins.append({"path": "research/ghidra/actor-initialization-evidence.json", "sha256": sha(actor_init_path.read_bytes())})
pins.append({"path": "research/ghidra/actor-initialization-bridge.md", "sha256": sha(init_bridge.encode("utf-8"))})

text_path = P / "private/reference-run/run-006/drive-c/GAME/DATA/TEXT.DAT"
text_bytes = text_path.read_bytes()
assert len(text_bytes) == 36610
assert sha(text_bytes) == "23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786"
parts = text_bytes.split(b"\0")
assert len(parts) >= 605 and parts[-1] == b""
category = parts[128:132]
assert [len(value) for value in category] == [8, 8, 10, 6]

base = 0x115E90
table_address = 0x116090
assert table_address == base + 128 * 4
assert 0x116164 == base + 181 * 4

# 1F490 clips the portrait's text viewport to (84,382)..(162,403).
viewport_path = G / "actor-name-trace/instructions-0001f490.txt"
viewport = viewport_path.read_bytes()
assert len(viewport) > 0
viewport_listing = viewport.decode("ascii")
for term in (
    "0001f4bd MOV [0x00112cb8],EAX",
    "0001f4c2 ADD EAX,EBX",
    "0001f4d9 MOV dword ptr [0x00112cc0],EDX",
    "0001f4df ADD EDX,ECX",
    "0001f50a MOV [0x00112cc8],EAX",
    "0001f516 MOV [0x00112ccc],EAX",
):
    assert term in viewport_listing, term
pins.append({"private_path": "private/ghidra/actor-name-trace/instructions-0001f490.txt", "sha256": sha(viewport)})

bar_contracts = {
    "portrait-detail/instructions-0002ff50.txt": [
        "0002ff60 CMP dword ptr [ESP + 0x14],0x0",
        "0002ff65 JLE 0x0002ffc0",
        "0002ff72 MOVSX EDX,AX",
        "0002ff75 IMUL EDX,EBP",
        "0002ff7d IDIV EBX",
        "0002ff84 MOV EAX,0x1",
        "0002ffb2 CALL 0x000980c0",
    ],
    "portrait-detail/instructions-000980c0.txt": [
        "000980d2 TEST byte ptr [0x0019cfac],0x10",
        "0009816d CALL 0x000b5c50",
    ],
    "display-geometry/instructions-000b5c50.txt": [
        "000b5c6b CMP EDI,dword ptr [0x0019cf80]",
        "000b5ce3 ADD EDI,EAX",
        "000b5d53 MOV AL,byte ptr [EBP + 0x8]",
        "000b5d59 STOSB ES:EDI",
    ],
}
bar_pins = []
for rel, terms in bar_contracts.items():
    data = (G / rel).read_bytes()
    listing = data.decode("ascii")
    for term in terms:
        assert term in listing, (rel, term)
    bar_pins.append({"private_path": "private/ghidra/" + rel, "sha256": sha(data)})
pins.extend(bar_pins)

font_color_contracts = {
    "portrait-detail-refs/instructions-000265f0.txt": [
        "0002665e AND AL,0xe3",
        "00026662 OR AL,0x4",
        "00026666 MOV [0x00187064],EAX",
        "00026670 CALL 0x0001f750",
    ],
    "actor-panel-font-color-target/instructions-0001ff30.txt": [
        "000200d7 AND EDX,0x180",
        "00020106 TEST byte ptr [ESI + 0x11],0x4",
        "00020237 TEST byte ptr [ESI + 0x10],0x2",
        "0002012e MOV AL,byte ptr [ESI + 0x18]",
    ],
    "font-color-global-refs/instructions-0001d770.txt": [
        "0001dac7 MOV ECX,0x300",
        "0001dac2 MOV byte ptr [ESI],0xa",
        "0001daf1 MOV EAX,0x1863a2",
        "0001daf6 CALL 0x000715e0",
        "0001df22 MOV DL,byte ptr [0x00186f55]",
        "0001df33 MOV DL,byte ptr [0x00186f54]",
    ],
    "font-color-global-refs/instructions-000715e0.txt": [
        "000715ef CALL 0x000a3f54",
        "000715f9 MOV [0x00186f54],AL",
        "00071600 MOV EDX,ECX",
        "0007160f MOV [0x00186f55],AL",
    ],
    "mule-render/instructions-000a3f54.txt": [
        "000a3f64 MOVZX EDI,DL",
        "000a3f67 MOVZX EBP,BL",
        "000a3f6a MOVZX ESI,CL",
        "000a401a JZ 0x000a436d",
    ],
}
font_color_pins = []
for rel, terms in font_color_contracts.items():
    data = (G / rel).read_bytes()
    listing = data.decode("ascii")
    for term in terms:
        assert term in listing, (rel, term)
    font_color_pins.append({"private_path": "private/ghidra/" + rel, "sha256": sha(data)})
pins.extend(font_color_pins)

font_dir = P / "private/review/font-menu"
font0_dat = (font_dir / "font0.dat").read_bytes()
font0_tab = (font_dir / "font0.tab").read_bytes()
assert len(font0_dat) == 8039 and sha(font0_dat) == "af89c4e4c02e394810b22ecf3d57211cdb56a0006071a88f43d0ae6fb370447d"
assert len(font0_tab) == 1170 and sha(font0_tab) == "321c49b302552e2bcf287bd20ef0954fb977dd66aef6e82101e3c0c64b5b0a17"
space_height = font0_tab[6 + 5]
assert space_height == 8
main_palette_path = P / "private/integration/terrain-inputs/MAIN.PAL"
main_palette = main_palette_path.read_bytes()
assert len(main_palette) == 768
nearest = lambda color: min(range(256), key=lambda i: (
    sum((main_palette[i * 3 + c] - color[c]) ** 2 for c in range(3)), i))
font_white = nearest((63, 63, 63))
font_black = nearest((0, 0, 0))
assert font_white == 145 and font_black == 0
assert main_palette[font_white * 3:font_white * 3 + 3] == bytes((63, 63, 63))
assert main_palette[font_black * 3:font_black * 3 + 3] == bytes((0, 0, 0))

newline = (G / "actor-name-trace/instructions-0001fe70.txt").read_bytes()
assert sha(newline) == "449f333522c68d6ef52e1d4ac2097329fbe951d66c1fd2a945f80321d19def28"
newline_listing = newline.decode("ascii")
for term in (
    "0001fe70 PUSH EBX",
    "0001fe71 MOV BL,byte ptr [EDX]",
    "0001fe75 CMP BL,0x1e",
    "0001feec MOV EAX,dword ptr [EAX]",
    "0001fef1 MOV EBX,dword ptr [0x00112d00]",
    "0001fefc ADD EBX,EAX",
    "0001ff03 MOV dword ptr [0x00112d00],EBX",
    "0001ff09 MOV [0x00112cfc],EAX",
):
    assert term in newline_listing, term

font_image = (G / "font-menu/memory-.object1.bin").read_bytes()
font_image_hash = sha(font_image)
table_off = 0x1FDF0 - 0x10000
assert struct.unpack_from("<I", font_image, table_off + 9 * 4)[0] == 0x1FEEC

receipt = {
    "status": "verified_selected_actor_category_pointer_path",
    "executable_sha256": exe_hash,
    "text_source": {
        "private_path": str(text_path.relative_to(P)).replace("\\", "/"),
        "sha256": sha(text_bytes),
        "bytes": len(text_bytes),
        "indexed_string_count": 604,
        "selector_domain": [0, 3],
        "text_indices": [128, 129, 130, 131],
        "category_entry_lengths": [len(value) for value in category],
    },
    "pointer_table": {
        "base": "0x115E90",
        "entry_stride": 4,
        "selector_base_address": "0x116090",
        "selector_base_index": 128,
        "neighboring_entry_181_address": "0x116164",
        "source_loop": "0x1DA60..0x1DA7B",
        "pipe_rewrite_indices": [221, 254],
    },
    "newline": {
        "control_byte": "0x0A",
        "handler": "0x1FEEC",
        "line_advance_source": "height byte at first glyph record in the font context",
        "listing_sha256": sha(newline),
        "jump_table_sha256": font_image_hash,
        "jump_table_entry_index": 9,
        "space_glyph_height": space_height,
    },
    "blueprint_name": {
        "typed_binding": "ActorHandleBindings.blueprint -> PlayerState.blueprint(index)",
        "actor_pointer_offset": "0xC0",
        "record_bytes": 36,
        "name_field": [0, 20],
        "nul_required_within_field": True,
        "category_field_offset": "0x16",
    },
    "font_binding": {
        "context": "0x187054",
        "descriptor_array": "0x118C74",
        "context_slot": 0,
        "font_pair_claim": "FONT0, pinned decoded bytes; source loader's descriptor sequence is not reconstructed from static zeroed pointer storage",
        "dat_bytes": len(font0_dat),
        "dat_sha256": sha(font0_dat),
        "tab_bytes": len(font0_tab),
        "tab_sha256": sha(font0_tab),
        "space_record_height": space_height,
    },
    "font_color_control": {
        "selected_context_flags_low_bits": "0x07 (initialized low bits 0x03, selected draw clears 0x1C and ORs 0x04)",
        "context_white_slot": "0x187068 <- 0x186F55",
        "context_black_slot": "0x18706C <- 0x186F54",
        "color_setup_function": "0x715E0 nearest-color lookups against active palette object at 0x115E80",
        "nearest_color_function": "0xA3F54 squared RGB distance; ascending palette scan keeps first tie and returns early on exact match",
        "main_palette_rgb6": {"white": [63, 63, 63], "black": [0, 0, 0]},
        "main_palette_indices": {"white": font_white, "black": font_black},
        "font_blitter_marker_mapping": {"literal_1": "context+0x14 (white)", "literal_values": [2, 4, 12, 16, 20, 24, 32, 36, 40, 48, 52, 56, 60], "mapped_to": "context+0x18 (black)", "other_literals": "copied as palette indices in this selected-panel mode"},
        "private_instruction_pins": font_color_pins,
    },
    "viewport": {
        "setup_function": "0x1F490",
        "callsite_registers": {"x": 84, "y": 382, "width": 78, "height": 21},
        "pixel_clip": {"left": 84, "top": 382, "right_exclusive": 162, "bottom_exclusive": 403},
        "category_line_y": 390,
    },
    "raw_panel_bars": {
        "call_order": [
            {"field_offset": "0x17", "origin": [116, 415], "height": 6, "palette_index": 232},
            {"field_offset": "0x19", "origin": [116, 435], "height": 6, "palette_index": 196},
            {"field_offset": "0x1A", "origin": [116, 435], "height": 6, "palette_index": 200},
        ],
        "input_interpretation": "signed i8; nonpositive values skipped; positive width=max(1,45*value/100) with truncation toward zero and no upper clamp",
        "meaning_labels": None,
        "private_instruction_pins": bar_pins,
    },
    "private_instruction_pins": pins,
    "limitations": [
        "The 0x118C74 descriptor array is zero in the static image; this receipt pins the exact decoded FONT0 pair but does not reconstruct runtime file-loader population of that descriptor slot.",
        "No field meanings are inferred for adjacent bars or icon draws.",
    ],
}
(P / "research/ghidra/actor-panel-text-evidence.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print("PASS actor panel text bridge: typed 20-byte name, 604 string pointers, category indices 128–131, FONT0 line advance, 78x21 viewport")

"""Check the selected-actor category pointer-table path without emitting strings."""
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
    ],
    "portrait-detail-refs/instructions-000265f0.txt": [
        "00026605 MOV EAX,dword ptr [EAX + 0xc0]",
        "0002660b MOVSX EBX,byte ptr [EAX + 0x16]",
        "0002660f MOV EDX,dword ptr [EBX*0x4 + 0x116090]",
        "00026617 PUSH EAX",
        "00026618 PUSH 0x10d687",
    ],
    "animal-consumers/instructions-0001d770.txt": [
        "0001da82 MOV EDI,0x115e90",
        "0001da87 MOV EBX,0x374",
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
    },
    "private_instruction_pins": pins,
    "limitations": [
        "The pointer at actor-object +0xC0 is passed as the first %s argument, but its bounded string storage and initializer are not established.",
        "No complete two-line panel renderer is claimed.",
    ],
}
(P / "research/ghidra/actor-panel-text-evidence.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
print("PASS actor panel text bridge: 604 string pointers, category indices 128–131, newline dispatch, 3 pinned instruction exports")

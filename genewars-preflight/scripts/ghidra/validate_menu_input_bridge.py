"""Validate exact CD main-menu capture and normal-frame opaque hit contracts."""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G / 'GW-CD-candidate.exe').read_bytes()) == candidate
memory = (G / 'validated/memory-.object3.bin').read_bytes()
controls = []
for i, (ident, y) in enumerate([(0, 115), (1, 156), (3, 186), (2, 257)]):
    off = 0x708c + 40 * i
    assert struct.unpack_from('<I', memory, off)[0] == 0x1048
    assert struct.unpack_from('<h', memory, off + 12)[0] == ident
    assert struct.unpack_from('<I', memory, off + 14)[0] == 0
    assert struct.unpack_from('<I', memory, off + 34)[0] == 0
    controls.append({'id': ident, 'record_va': hex(0x100000 + off), 'x': 140, 'three_button_y': y if i != 1 else None, 'enabled_in_three_button': i != 1, 'flags': '0x1048', 'callback': 0})

contracts = {
 'ui-input/instructions-000a87d0.txt': ['000a8805 MOV word ptr [0x001a0120],DX', '000a880f MOV word ptr [0x001a011e],DX', '000a8868 CMP EAX,0x1', '000a8871 TEST byte ptr [ECX + 0x8],0x2', '000a88a7 AND EAX,EDX', '000a88db MOV DL,byte ptr [EBX + 0x1]', '000a88ea TEST DL,0x10', '000a88fa CALL 0x000a86d0', '000a890a MOVSX ESI,word ptr [ESI + 0xc]', '000a8910 MOV ESI,0xffffffff', '000a8a25 MOV byte ptr [EAX + 0x26],0x0', '000a8a2e MOV dword ptr [0x0019ff88],ECX', '000a8a54 MOV byte ptr [EBP + 0x27],0x0', '000a8a7d TEST byte ptr [EDX + 0x8],0x4', '000a8a92 TEST byte ptr [EDX + 0x9],0x1', '000a8aac MOV dword ptr [0x0019ff70],EBX', '000a8b32 CMP AX,DI', '000a8b37 MOV SI,word ptr [EDX + 0xc]', '000a8b40 MOV dword ptr [0x0019ff70],0x1', '000a8b52 MOV [0x001a011c],AX', '000a8be2 MOV byte ptr [EAX + 0x27],DL', '000a8d89 MOV byte ptr [ECX + 0x27],AH', '000a8d93 MOV byte ptr [ECX + 0x26],AL', '000a8d96 TEST DL,0x14', '000a8d9f MOV EAX,0x1', '000a8da4 MOV dword ptr [0x0019ff88],ECX', '000a8daa MOV [0x001a011c],AX', '000a8e24 MOV dword ptr [0x0019ff70],EDX', '000a8f8a MOV SI,word ptr [ECX + 0xc]'],
 'ui-controls/instructions-000a86d0.txt': ['000a8739 TEST byte ptr [EAX],0x40', '000a8744 MOV EAX,dword ptr [EAX + 0x12]', '000a874b MOV EAX,dword ptr [EAX]', '000a8752 CALL 0x00098c40', '000a8757 CMP EAX,-0x1'],
 'ui-frame/instructions-00098c40.txt': ['00098c53 MOVZX ESI,byte ptr [EAX + 0xb]', '00098c57 DEC ESI', '00098c77 MOVSX EBP,word ptr [ECX + 0x4]', '00098c98 TEST byte ptr [ECX + 0x8],0x1', '00098cab LEA EDI,[EBX + -0x1]', '00098cae TEST byte ptr [EDX + 0x8],0x2', '00098cc1 LEA EAX,[EBX + -0x1]', '00098cd0 CALL 0x00098b60', '00098cd9 TEST AL,AL', '00098ce1 CMP ESI,-0x1'],
 'sprite-scaled/instructions-00098b60.txt': ['00098b69 JL 0x00098be0', '00098b75 JGE 0x00098be0', '00098b80 JL 0x00098be0', '00098b8c JGE 0x00098be0', '00098b95 MOVSX EBX,byte ptr [EAX]', '00098bbf SUB EDX,ECX', '00098bc6 JG 0x00098bd0', '00098bc8 MOV EAX,0x1', '00098bdc ADD EDX,EBX'],
 'ui-controls/instructions-000a84d0.txt': ['000a8567 TEST EBX,EBX', '000a856e MOV EBX,dword ptr [EBX + 0x4]', '000a8576 MOV EBX,dword ptr [EBX]', '000a857e CALL 0x000982e0'],
 'font-menu/instructions-0007c870.txt': ['0007c877 CALL 0x000a87d0', '0007c87c MOV [0x001169a2],AX', '0007c899 TEST byte ptr [0x0019ff70],0x8', '0007c8a6 TEST byte ptr [0x00118969],0x80'],
}
listings = []
for path, lines in contracts.items():
    raw = (G / path).read_bytes()
    for line in lines:
        assert line in raw.decode(), (path, line)
    listings.append({'private_path': 'private/ghidra/' + path, 'sha256': sha(raw)})

corpus = json.loads((P / 'research/integration/rnc-corpus.json').read_text())['results']
entry = next(r for r in corpus if r['entry_id'] == 'root/2/disc/92' and r['status'] == 'passed')
anb = (P / entry['private_output']).read_bytes()
assert sha(anb) == entry['output_sha256']
n, table, components, frames, animations = struct.unpack_from('<H4I', anb)
assert (n, table, components, frames, animations) == (1, 57983, 58043, 58133, 58175)
first, selected = struct.unpack_from('<II', anb, animations)
assert (first, selected) == (0, 28)
parts = []
mask = bytearray(346 * 75)
for i in range(3):
    record, cx, cy, flags = struct.unpack_from('<IhhB', anb, components + i * 10)
    cx >>= 1
    cy >>= 1
    rel, width, height = struct.unpack_from('<IBB', anb, table + record)
    assert flags == 0
    pos = 18 + rel
    rows = []
    for y in range(height):
        row_start = pos
        x = 0
        while True:
            control = struct.unpack_from('<b', anb, pos)[0]
            pos += 1
            if control == 0:
                break
            if control < 0:
                x -= control
            else:
                assert x + control <= width
                for dx in range(control):
                    mask[(cy + y) * 346 + cx + x + dx] = 1
                x += control
                pos += control
            assert x <= width
        rows.append(row_start)
    parts.append({'sprite': record // 6, 'x': cx, 'y': cy, 'width': width, 'height': height, 'row_offsets': rows})

def direct_hit(part, x, y):
    x -= part['x']
    y -= part['y']
    if not (0 <= x < part['width'] and 0 <= y < part['height']):
        return False
    remaining = x + 1
    pos = part['row_offsets'][y]
    while remaining > 0:
        control = struct.unpack_from('<b', anb, pos)[0]
        pos += 1
        if control > 0:
            remaining -= control
            if remaining <= 0:
                return True
            pos += control
        elif control < 0:
            remaining += control
        else:
            return False
    return False

for y in range(75):
    for x in range(346):
        assert any(direct_hit(part, x, y) for part in reversed(parts)) == bool(mask[y * 346 + x])
assert all(not any(direct_hit(p, x, y) for p in parts) for x, y in [(-1, 0), (346, 0), (0, -1), (0, 75), (346, 75)])
out = G / 'menu-input'
out.mkdir(exist_ok=True)
(out / 'normal-hit-mask.bin').write_bytes(mask)

def step(captured, prior_inside, flags8, flags9, first_hit, captured_hit, prior_countdown=None):
    if prior_countdown is None:
        prior_countdown = int(bool(prior_inside)) if captured is not None else 0
    if captured is None:
        if flags8 & 1 and first_hit is not None:
            return {'event': first_hit, 'mask_write': 0, 'capture': first_hit, 'selected': True, 'countdown': 1}
        return {'event': -1, 'mask_write': None, 'capture': None, 'selected': False, 'countdown': prior_countdown}
    if flags8 & 2:
        return {'event': captured if captured_hit else -1, 'mask_write': 8, 'capture': None, 'selected': False, 'countdown': prior_countdown}
    result = -1
    mask_out = (2 if flags8 & 4 else 0) | (4 if flags9 & 1 else 0)
    if bool(captured_hit) != bool(prior_inside):
        result = captured
        if prior_inside:
            mask_out = 1
    return {'event': result, 'mask_write': mask_out, 'capture': captured, 'selected': bool(captured_hit), 'countdown': int(bool(captured_hit))}

vectors = []
for captured, inside, b8, b9, first_hit, hit in [
 (None, False, 0, 0, 0, False), (None, False, 1, 0, 0, False), (None, False, 3, 0, 0, False),
 (None, False, 0x40, 0, 0, False), (None, False, 1, 0, None, False),
 (0, True, 4, 0, None, True), (0, True, 4, 0, None, False), (0, False, 4, 0, None, True),
 (0, False, 4, 1, None, False), (0, True, 2, 0, None, True), (0, True, 2, 0, None, False),
 (0, False, 3, 0, 3, True), (0, False, 0x80, 1, None, True), (0, True, 1, 0, 3, True),
 (3, True, 2, 0, 0, False), (2, True, 2, 0, 0, True)]:
    args = {'captured': captured, 'prior_inside': inside, 'prior_countdown': int(inside) if captured is not None else 0, 'packet_byte8': b8, 'packet_byte9': b9, 'first_opaque_hit': first_hit, 'captured_opaque_hit': hit}
    vectors.append({'input': args, 'output': step(captured, inside, b8, b9, first_hit, hit)})
vectors.append({'input': {'captured': None, 'prior_inside': False, 'prior_countdown': 1, 'packet_byte8': 0, 'packet_byte9': 0, 'first_opaque_hit': None, 'captured_opaque_hit': False}, 'output': step(None, False, 0, 0, None, False, 1)})
checks = 0
for captured in [None, 0, 3, 2]:
    for inside in [False, True]:
        for b8 in range(256):
            for b9 in range(2):
                for hit in [False, True]:
                    r = step(captured, inside, b8, b9, 0 if hit else None, hit)
                    if captured is not None and b8 & 2:
                        assert r['capture'] is None and r['mask_write'] == 8
                    if captured is None and b8 & 1 and hit:
                        assert r['capture'] == 0 and r['mask_write'] == 0
                    if r['event'] != -1 and r['mask_write'] is not None and r['mask_write'] & 8:
                        assert captured is not None and b8 & 2 and hit
                    checks += 1
pixel_vectors = []
for x, y in [(0, 0), (345, 0), (0, 74), (345, 74), (173, 37), (116, 11), (236, 3), (-1, 0), (346, 0), (0, 75)]:
    pixel_vectors.append({'local_x': x, 'local_y': y, 'hit': any(direct_hit(p, x, y) for p in reversed(parts))})
receipt = {'candidate_sha256': candidate, 'instruction_contracts': listings, 'controls': controls,
 'scope': 'Flags0x1048, callback0, capture countdown0or1; three-button menu with normal-frame hit test and zero origin',
 'normal_hit': {'width': 346, 'height': 75, 'mask_sha256': sha(mask), 'opaque_count': sum(mask), 'private_mask': 'private/ghidra/menu-input/normal-hit-mask.bin', 'decoded_anb_sha256': sha(anb), 'components': [{k: p[k] for k in ['sprite', 'x', 'y', 'width', 'height']} for p in parts], 'pixel_comparisons': len(mask), 'vectors': pixel_vectors},
 'state_vectors': vectors, 'exhaustive_packet_cases': checks,
 'notes': ['Initial press returns ID with mask0, not -2; caller ignores it.', 'Release only activates on captured normal-frame opaque pixels; another control cannot substitute.', 'Null mask_write preserves the original stale global mask on uncaptured no-event calls.', 'Release leaves global countdown unchanged; its value is irrelevant after capture clear.', 'Positive literal zero is opaque; the hit routine does not inspect literal color bytes.', 'Selected artwork uses last frame while hit testing always uses first frame.']}
(P / 'research/ghidra/menu-input-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(f'PASS {len(listings)} instruction contracts; {len(mask)} direct normal-frame pixel hits; {checks} bounded packet cases; {len(vectors)} explicit vectors')

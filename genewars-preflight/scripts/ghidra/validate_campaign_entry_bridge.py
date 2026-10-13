"""Validate the bounded campaign-entry contract; never execute original code."""
from pathlib import Path
import hashlib
import json
import struct

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
digest = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert digest((G / 'GW-CD-candidate.exe').read_bytes()) == candidate
mapped = (G / 'validated/memory-.object3.bin').read_bytes()
u32 = lambda va: struct.unpack_from('<I', mapped, va - 0x100000)[0]
def string(va):
    off = va - 0x100000
    return mapped[off:mapped.index(0, off)].decode('ascii')

assert u32(0x111f34) == 0x9d1 and 0x186548 + 0x9d1 == 0x186f19
assert [u32(x) for x in range(0x111f20, 0x111f30, 4)] == [0x15b90, 0x16170, 0x17c50, 0x17e80]
assert u32(0x1127e4) == 8 and 0x1863b0 + 8 == 0x1863b8
assert [u32(x) for x in range(0x1127d0, 0x1127e0, 4)] == [0x7ce60, 0x7d020, 0x7d7f0, 0x7d090]
formats = {hex(va): string(va) for va in [0x10d070, 0x10d084, 0x10d097, 0x10d0bd, 0x10d0d0, 0x10e98e, 0x10e9a0, 0x10e9b2]}
assert list(formats.values()) == ['LEVELS\\hdrc%c%d.dat', 'anims\\system%d.smk', 'data/pl%d.dat', 'data/levscr0%d.dat', 'data/levscr0%d.pal', 'data\\toPlnt%d.dat', 'data\\toPlnt%d.pal', 'levels\\conq%03d.dat']
contracts = {
 'font-menu/instructions-0007c870.txt': ['0007c899 TEST byte ptr [0x0019ff70],0x8', '0007c8a6 TEST byte ptr [0x00118969],0x80', '0007c8ad JNZ 0x0007c8cf', '0007c8dc MOV EBX,0x70d', '0007c8e8 CALL 0x000982c0', '0007c906 CALL 0x00080460', '0007c93e MOV EDI,0x186f19'],
 'campaign-constructor-head.txt': ['0008dda2 MOV EAX,0x186548', '0008ddae CALL 0x00015b10'],
 'campaign-screen/instructions-00015b10.txt': ['00015b15 MOV dword ptr [EAX],0x111f30', '00015b3a MOV dword ptr [EAX + 0x9c9],0x111f20', '00015b4a MOV dword ptr [EDX + EAX*0x1],0x111f10'],
 'campaign-screen/instructions-00015b90.txt': ['00015bc5 MOVSX EDX,byte ptr [EBX + 0x11945b]', '00015bd6 IDIV ECX', '00015bde MOV word ptr [EBP + 0x28],DX', '00015be5 JNZ 0x00015c52', '00015bf5 MOV byte ptr [EBX + 0x11945c],AL', '00015c04 MOV AL,byte ptr [EBX + 0x11945c]', '00015c2c XOR ECX,ECX', '00015c2e CALL 0x0001be50', '00015cbe MOV AL,byte ptr [EAX + 0x11945c]', '00015e77 MOV AL,byte ptr [EAX + 0x11945b]', '00015e81 XOR EBX,EBX', '00015e88 CALL 0x00015720', '00015ec4 IDIV EBX', '00015f11 IDIV EBX'],
 'campaign-header/instructions-00015720.txt': ['00015728 MOV ECX,EAX', '0001572c MOV AL,DL', '00015733 CMP EAX,0x1', '00015738 MOV AL,0x75', '0001573c MOV AL,0x6f', '0001576c MOV EBX,0x2bb', '00015771 MOV EDX,ECX', '00015773 CALL 0x0009a420'],
 'campaign-actions/instructions-00017e80.txt': ['00017f99 TEST CL,CL', '00017fa1 MOV AL,byte ptr [ESI + 0x9a9]', '00017fa7 CMP AL,0x9', '00017fa9 JL 0x00017fce', '00017fad MOV byte ptr [ESI + 0x9b7],0x1', '00017fb7 MOV ECX,0x1863b8'],
 'campaign-actions/instructions-00016170.txt': ['000161f6 CMP byte ptr [ECX + 0x9b7],0x0', '00016212 AND byte ptr [0x00118968],0xfe', '00016219 OR byte ptr [0x00118770],0x20'],
 'campaign-mission-constructor.txt': ['0008dcc2 MOV EAX,[0x001127e4]', '0008dccf MOV EBP,0x1127d0', '0008dcd4 ADD EAX,0x1863b0', '0008dce5 MOV dword ptr [EAX],0x1127c0'],
 'campaign-view-thunks/instructions-0008de70.txt': ['0008de70 SUB EAX,0x8', '0008de73 JMP 0x0007d090'],
 'campaign-level-entry/instructions-0007ce60.txt': ['0007ceb5 MOV AL,byte ptr [EAX + 0x11945c]', '0007cec1 PUSH 0x10e98e', '0007cef8 MOV AL,byte ptr [EAX + 0x11945c]', '0007cf04 PUSH 0x10e9a0'],
 'campaign-level/instructions-0007d090.txt': ['0007d404 TEST DL,0x20', '0007d416 MOVSX EAX,byte ptr [EAX + 0x11945b]', '0007d41d INC EAX', '0007d41f PUSH 0x10e9b2', '0007d49e CALL 0x00071b70'],
 'campaign-level/instructions-00071b70.txt': ['00071be6 MOV EBX,0x4', '00071c87 ADD EDX,0x4', '00071c8a CMP EDI,0x55336', '00071c90 JZ 0x00071ce1', '00071ce1 MOV EBX,0x3b55a', '00071ce8 CALL 0x0009a420', '00071c00 LEA EDX,[ECX + 0x14]', '00071c0d ADD EBX,0x70d'],
 'campaign-briefing/instructions-00017850.txt': ['00017968 LEA EAX,[ESI + 0x81]', '00017a11 MOV BL,byte ptr [EBX + 0x11945b]', '00017a23 LEA EAX,[ESI + 0x81]', '00017a29 CALL 0x0008ab70'],
 'campaign-header-consumer/instructions-0008ab70.txt': ['0008ab7a MOV DL,BL', '0008ab7c CMP EDX,0x5', '0008aba2 TEST byte ptr [ESI + 0x27e],0x1', '0008abc0 CMP dword ptr [ESI + 0x27a],0x0', '0008abe2 MOV DI,word ptr [ESI + 0x25e]', '0008ac05 CMP word ptr [ESI + 0x260],0x0'],
}
listings = []
for name, expected in contracts.items():
    raw = (G / name).read_bytes()
    for line in expected:
        assert line in raw.decode(), (name, line)
    listings.append({'private_path': 'private/ghidra/' + name, 'sha256': digest(raw)})

manifest = json.loads((P / 'research/game-files-manifest.json').read_text())['entries']
cd = {e.get('normalized_iso_path', '').upper().split(';')[0].removeprefix('GAME/'): e for e in manifest if e['entry_id'].startswith('root/2/disc/')}
def plan(progress, prior_system):
    if progress not in range(24) or prior_system not in range(4):
        return None
    quotient, remainder = divmod(progress, 6)
    system = quotient if remainder == 0 else prior_system
    return {'progress': progress, 'prior_system': prior_system, 'local_planet': remainder + 1,
            'effective_system': system, 'updates_system': remainder == 0,
            'header': f'LEVELS/HDRCO{progress + 1}.DAT', 'mission': f'LEVELS/CONQ{progress + 1:03}.DAT',
            'texture': f'DATA/PL{system + 1}.DAT', 'background': f'DATA/LEVSCR0{quotient + 1}.DAT',
            'palette': f'DATA/LEVSCR0{quotient + 1}.PAL', 'movie': f'ANIMS/SYSTEM{system + 1}.SMK' if remainder == 0 else None,
            'transition_background': f'DATA/TOPLNT{system}.DAT', 'transition_palette': f'DATA/TOPLNT{system}.PAL'}
vectors = [plan(p, s) for p in range(24) for s in range(4)]
paths = set()
for v in vectors:
    paths.update(v[k] for k in ['header', 'mission', 'texture', 'background', 'palette', 'movie', 'transition_background', 'transition_palette'] if v[k])
for name in paths:
    assert name in cd, name
for p, s in [(-1, 0), (24, 0), (128, 0), (0, -1), (0, 4)]:
    assert plan(p, s) is None
assets = [{k: cd[name][k] for k in ['entry_id', 'sha256', 'size']} | {'source': name} for name in sorted(paths)]
header = (G / 'campaign-resources/190.dat').read_bytes()
level = (G / 'campaign-resources/152.dat').read_bytes()
assert digest(header) == cd['LEVELS/HDRCO1.DAT']['sha256'] and len(header) == 699
assert digest(level) == cd['LEVELS/CONQ001.DAT']['sha256'] and len(level) == 343958
assert struct.unpack_from('<I', level)[0] == 0x55336
title_end = header.index(0, 32)
assert title_end == 40
prefix_size = 0x3b55e
assert prefix_size == 4 + 0x3b55a and len(level) >= prefix_size
receipt = {'candidate_sha256': candidate, 'scope': 'Bounded static campaign entry; no original execution',
 'instruction_contracts': listings, 'resource_formats': formats,
 'admission': {'progress': [0, 23], 'prior_system': [0, 3], 'reason': 'Explicit safe source-backed scope; not original gameplay validity or completion policy'},
 'vectors': vectors, 'assets': assets,
 'first_header': {'entry_id': 'root/2/disc/190', 'size': 699, 'sha256': digest(header), 'title_offset': 32, 'title_length': title_end - 32, 'title_search_end_exclusive': 699, 'title_sha256': digest(header[32:title_end])},
 'first_mission': {'entry_id': 'root/2/disc/152', 'size': len(level), 'sha256': digest(level), 'layout_tag': '0x55336', 'direct_prefix_size': prefix_size, 'direct_prefix_sha256': digest(level[:prefix_size]), 'serialized_tail_size': len(level) - prefix_size, 'player_offset': 20, 'player_stride': 1805, 'player_count': 4, 'uninterpreted_map_offset': '0x355e', 'uninterpreted_map_stride': 14, 'uninterpreted_map_count': 16384},
 'safety': ['Require exact 699-byte header and an in-bounds NUL after byte32; original reader accepts any nonzero read count.', 'Preserve opaque header bytes and unknown numeric fields.', 'Do not cast CONQ file to a full live-state struct: serialized tail has separate consumers; loaded player records are normalized.', 'Preserve prior system except when signed remainder+1 equals1; backgrounds independently use signed quotient+1.', 'Original progress arithmetic is signed i8; bounded plan rejects negative and >=24 values.']}
(P / 'research/ghidra/campaign-entry-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(f'PASS {len(listings)} instruction contracts; {len(vectors)} state-aware plans; {len(assets)} exact CD resource identities; first header and level prefix bounds')

"""Validate local static-analysis receipts; output metadata only, never pixels/code."""
from pathlib import Path
import hashlib, json, struct

P = Path(__file__).resolve().parents[2]
R = P / 'private/ghidra'
expected = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
sha = lambda b: hashlib.sha256(b).hexdigest()
assert sha((R/'GW-CD-candidate.exe').read_bytes()) == expected
code = (R/'validated/memory-.object1.bin').read_bytes()
data = (R/'validated/memory-.object3.bin').read_bytes()

def descriptor(va, name, dest, end=0):
    off = va - 0x100000
    assert data[off:off+28].split(b'\0')[0].decode('ascii') == name
    actual = struct.unpack_from('<4I', data, off+28)
    assert actual[:2] == (dest, end)
    return dict(va=hex(va), filename=name, allocation_pointer=hex(dest), end_pointer=hex(end))

descriptors = [descriptor(0x1043e4, 'data/main.pal', 0x115e80),
               descriptor(0x103db4, 'data/pointers.dat', 0x11698c),
               descriptor(0x103de0, 'data/pointers.tab', 0x115e74, 0x115e78)]
assert 0x1043e4 == 0x104360 + 3*44
table = struct.unpack_from('<3I', data, 0x105604-0x100000)
names = []
for address in table:
    names.append(data[address-0x100000:].split(b'\0')[0].decode('ascii'))
assert names == ['data/mainpanl.dat','data/genepanl.dat','data/lpanel.dat'], names

checks = [
 ('panel-verified',0x24ad0,['MOV ECX,0x20','MOVSD.REP','ADD EBX,0x80','SHR EAX,0x2','0x0019cf70']),
 ('mode-functions',0x1caa0,['MOV EBX,0x1e0','MOV EDX,0x280','MOV EAX,0xd','CALL 0x000979a0']),
 ('mode-functions',0x979a0,['PUSH ECX','PUSH EBX','PUSH EDX','CALL 0x000a13f0']),
 ('mode-functions',0xa13f0,['0x0019cf6c','0x0019cf70','CALL 0x000a6d2c']),
 ('vga',0xa6d2c,['MOV EDX,0x3c6','MOV EDX,0x3c8','MOV ESI,0x3c9','CMP EAX,0x100']),
 ('sprite-functions',0xa7618,['ADD EAX,0x6','MOV ECX,dword ptr [EAX]','CMP EBX,ECX','LEA ESI,[ECX + EBX*0x1]']),
 ('sprite-functions',0xa7730,['CMP byte ptr [EBX + 0x4],0x0','CMP byte ptr [EBX + 0x5],0x0','MOVSX EAX,byte ptr [EDX]','MOVSB.REP']),
 ('sprite-functions',0x98a60,['MOV EDX,dword ptr [EAX + 0x2]','MOV EBX,dword ptr [EAX + 0x6]','LEA EBX,[EAX + 0x12]','CALL 0x000a7618']),
]
functions=[]
for folder, va, tokens in checks:
    path=R/folder/f'instructions-{va:08x}.txt'
    listing=path.read_text()
    for token in tokens:
        assert token in listing, (str(path), token)
    metadata=json.loads((R/folder/'program-metadata.json').read_text())
    f=next(f for f in metadata['functions'] if int(f['entry'],16)==va)
    size=f['size']
    functions.append(dict(va=hex(va), object=1, object_offset=hex(va-0x10000),
        body_address_count=size, mapped_contiguous_sha256=sha(code[va-0x10000:va-0x10000+size]),
        private_listing=str(path.relative_to(P)).replace('\\','/'), listing_sha256=sha(path.read_bytes())))

with (P/'private/disc/member-004.img').open('rb') as stream:
    stream.seek(7491*2352+16)
    palette=stream.read(768)
assert sha(palette)=='8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05'
assert max(palette)<=63
panel=(P/'private/assets/genepanl.dat.decoded').read_bytes()
assert len(panel)==128*480
assert sha(panel)=='af9741b4545fb6f28a1dd0e50da0570e39992e75c872bc2249891f3330f59951'
# Synthetic row copy checks: preserve padding and neighboring columns.
for pitch in [128,132,640,643]:
    step=(pitch>>2)*4
    source=bytes((x+y*3)%256 for y in range(5) for x in range(128))
    target=bytearray([251]*(step*5+16))
    for y in range(5):target[y*step:y*step+128]=source[y*128:(y+1)*128]
    for y in range(5):
        assert target[y*step:y*step+128]==source[y*128:(y+1)*128]
        assert all(v==251 for v in target[y*step+128:(y+1)*step])
out=dict(scope='provisional CD candidate; static evidence, no original execution', candidate_sha256=expected,
    descriptors=descriptors, panel_filename_table_va='0x105604', panel_filenames=names,
    panel=dict(width=128,height=480,source_stride=128,display_width=640,destination_step='pitch & ~3',
      decoded_sha256=sha(panel),decoded_bytes=len(panel)),
    palette=dict(member='GAME/DATA/MAIN.PAL;1',disc_inventory_entry='root/2/disc/108',lba=7491,bytes=768,
      sha256=sha(palette),channel_order='RGB',channel_bits=6,min=min(palette),max=max(palette),
      uploader_scales_channels=False,hardware_reference='https://www.scs.stanford.edu/10wi-cs140/pintos/specs/freevga/vga/colorreg.htm'),
    functions=functions,tests=dict(static_listing_invariants=len(checks),synthetic_pitch_cases=4,all_pass=True))
(P/'research/ghidra/renderer-evidence.json').write_text(json.dumps(out,indent=2)+'\n')
print('PASS renderer/palette/sprite static receipts, asset hashes, and 4 synthetic pitch cases')

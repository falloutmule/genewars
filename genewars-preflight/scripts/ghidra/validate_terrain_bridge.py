"""Hash-bound terrain field, height interpolation and B32A extraction evidence."""
from pathlib import Path
import hashlib
import json
import struct
import sys

P = Path(__file__).resolve().parents[2]
G = P / 'private/ghidra'
sha = lambda b: hashlib.sha256(b).hexdigest()
candidate = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G / 'GW-CD-candidate.exe').read_bytes()) == candidate
mem = (G / 'validated/memory-.object3.bin').read_bytes()
def text(va):
    o = va - 0x100000
    return mem[o:mem.index(0, o)].decode()
assert text(0x1038b8) == 'data/b32a.dat'
assert struct.unpack_from('<I', mem, 0x38b8 + 28)[0] == 0x115e6c
assert text(0x1043e4) == 'data/main.pal'
assert struct.unpack_from('<I', mem, 0x43e4 + 28)[0] == 0x115e80
contracts = {
 'terrain-map-refs/instructions-00074cc0.txt': ['00074cea MOVSX EDX,word ptr [ESI + 0x2]', '00074cee SAR EDX,0x8', '00074cf1 AND EDX,0x7f', '00074d00 MOV EDI,0x11c2be', '00074d08 SHL EAX,0x8', '00074d1b ADD EAX,EAX'],
 'terrain-map-refs/instructions-0007b140.txt': ['0007b14b MOVSX EBP,word ptr [EAX]', '0007b14e MOVSX ESI,word ptr [EAX + 0x2]', '0007b18a XOR AH,AH', '0007b1bd MOVSX ECX,word ptr [ECX + EDI*0x1 + 0x2]', '0007b1d7 XOR DH,DH', '0007b1e6 CMP EBX,EDX', '0007b1e8 JL 0x0007b209', '0007b1fc SUB EAX,ECX', '0007b1fe IMUL EAX,EBX', '0007b204 IMUL EDX,ESI', '0007b223 ADD EAX,EDX', '0007b225 SAR EAX,0x7', '0007b228 ADD EAX,EBP'],
 'terrain-map-refs/instructions-0008c860.txt': ['0x11c2be', 'SAR EAX,0x2'],
 'terrain-map-refs/instructions-0007b7f0.txt': ['0007b8f1 MOVSX EAX,word ptr [EBX + 0x2]', '0007b8fb SAR EAX,0x6', '0007b8fe IMUL EDX,EAX', '0007b93a MOV AL,byte ptr [EBX + 0x4]', '0007b940 MOV AL,byte ptr [EBX + 0x5]', '0007bab4 MOV EAX,dword ptr [EAX*0x4 + 0x115bec]', '0007bac3 AND AL,0x70', '0007bacd MOV EDX,0x1060ec', '0007bb8e CALL 0x000e1900'],
 'terrain-render-refs/instructions-0001d770.txt': ['0001d9aa MOV EDX,0x10388c', '0001d9bf CALL 0x000154b0', '0001dae7 MOV EAX,[0x00115e80]', '0001daec MOV [0x001863a4],EAX', '0001db7a MOV EBX,0x115bec', '0001db91 CMP EDX,0xa0', '0001ddd1 ADD ECX,EAX', '0001ddd4 ADD ECX,ESI', '0001ddd6 ADD EAX,0x20', '0001ddd9 MOV dword ptr [EBX + -0x4],ECX', '0001ddea MOV EAX,dword ptr [EBP + 0x72]', '0001ddf0 ADD EAX,0x2000'],
 'terrain-constructor/instructions-0008db90.txt': ['0008dbb9 MOV ECX,0x2', '0008dbbe CALL 0x000402e0', '0008dc1a MOV EDX,0x20', '0008dc24 MOV EBX,0x10', '0008dc29 MOV dword ptr [0x00184fe4],ECX', '0008dc32 MOV dword ptr [0x00184fd4],EDX', '0008dc76 MOV dword ptr [0x00184fd8],EBX'],
 'terrain-camera-init/instructions-000402e0.txt': ['000402e0 PUSH EBX', '00040310 RET'],
 'terrain-runtime/instructions-000a76d0.txt': ['000a76d0 PUSH ECX', '000a7712 POP ECX'],
 'terrain-runtime/instructions-000159f0.txt': ['000159f0 MOV byte ptr [EAX + 0x6],0x0', '000159f4 RET'],
 'terrain-runtime/instructions-0002f870.txt': ['0002f870 MOV dword ptr [EAX],0x105da0', '0002f87d RET'],
 'terrain-mode5.txt': ['000e406e XOR DH,DH', '000e4070 AND EBX,0xffff', '000e4082 MOV ESI,dword ptr [0x00100108]', '000e40d0 MOV AH,CH', '000e40d2 MOV BL,DL', '000e40d8 MOV AL,byte ptr [EBX + ESI*0x1]', '000e40df MOV AL,byte ptr [EAX + 0x188f48]', '000e40e9 MOV byte ptr [EDI],AL'],
 'campaign-refs/instructions-0008dfe0.txt': ['0008e67c MOV ECX,dword ptr [ESI + 0x2]', '0008e693 MOV EAX,ECX', '0008e695 CALL 0x000a3da0'],
}
listings = []
for name, lines in contracts.items():
    raw = (G / name).read_bytes()
    for line in lines:
        assert line in raw.decode(), (name, line)
    listings.append({'private_path': 'private/ghidra/' + name, 'sha256': sha(raw)})
assert 'ECX' not in (G / 'terrain-camera-init/instructions-000402e0.txt').read_text()
assert 'ECX' not in (G / 'terrain-runtime/instructions-000159f0.txt').read_text()
assert 'ECX' not in (G / 'terrain-runtime/instructions-0002f870.txt').read_text()

def interpolate(h, fx, fy):
    h00, h10, h01, h11 = h
    numerator = (h11 - h01)*fx + (h01 - h00)*fy if fx < fy else (h10 - h00)*fx + (h11 - h10)*fy
    return 2*h00 + (numerator >> 7)
def barycentric(h, fx, fy):
    h00, h10, h01, h11 = h
    weighted = h00*(256-fy) + h01*(fy-fx) + h11*fx if fx < fy else h00*(256-fx) + h10*(fx-fy) + h11*fy
    return weighted >> 7
sets = [(0,0,0,0), (7,7,7,7), (0,256,512,768), (-32768,32767,-32768,32767), (32767,-32768,32767,-32768), (-5,8,-13,21)]
comparisons = 0
for h in sets:
    for fx in range(256):
        for fy in range(256):
            value = interpolate(h,fx,fy)
            assert value == barycentric(h,fx,fy) and -65536 <= value <= 65534
            comparisons += 1
vectors = [{'heights_h00_h10_h01_h11': list(h), 'fraction_x': fx, 'fraction_y': fy, 'doubled_height': interpolate(h,fx,fy)}
           for h in sets for fx,fy in [(0,0),(0,255),(255,0),(255,255),(127,128),(128,127),(128,128)]]
coordinate_vectors = []
for x,y in [(0,0),(0x100,0),(0x7fff,0x7fff),(0x8000,0x8000),(0xffff,0xffff),(0xff00,0),(0,0xff00)]:
    cx,cy = (x>>8)&127,(y>>8)&127
    coordinate_vectors.append({'x_word': x,'y_word':y,'cell_x':cx,'cell_y':cy,'fraction_x':x&255,'fraction_y':y&255,
                               'record_index':cy*128+cx,'file_record_offset':0x355e+(cy*128+cx)*14,
                               'next_x':(cx+1)&127,'next_y':(cy+1)&127})

manifest = json.loads((P / 'research/game-files-manifest.json').read_text())['entries']
entry = lambda ident: next(e for e in manifest if e['entry_id']==ident)
results = json.loads((P / 'research/integration/rnc-corpus.json').read_text())['results']
atlas_result = next(r for r in results if r['entry_id']=='root/2/disc/40' and r['status']=='passed')
atlas = (P / atlas_result['private_output']).read_bytes()
assert len(atlas)==163840 and sha(atlas)==atlas_result['output_sha256']
tiles = []
coverage = bytearray(len(atlas))
for tile in range(160):
    base = (tile//8)*8192 + (tile%8)*32
    raster = bytearray()
    for y in range(32):
        for x in range(32):
            off = base+y*256+x
            coverage[off]+=1
            raster.append(atlas[off])
    tiles.append({'tile':tile,'source_base':base,'width':32,'height':32,'source_stride':256,'last_source_offset':base+31*256+31,'index_sha256':sha(raster)})
assert min(coverage)==max(coverage)==1
assert (160//8)*8192+(160%8)*32==len(atlas)
level = (G / 'campaign-resources/152.dat').read_bytes()
assert sha(level)==entry('root/2/disc/152')['sha256'] and struct.unpack_from('<I',level)[0]==0x55336
region = level[0x355e:0x3b55e]
assert len(region)==128*128*14
heights = [struct.unpack_from('<h',region,i*14+2)[0] for i in range(16384)]
selectors = [region[i*14+4] for i in range(16384)]
assert min(heights)==0 and max(heights)==4800 and min(selectors)==0 and max(selectors)==118
uv = [list(struct.unpack_from('<8I',mem,0x60ec+i*32)) for i in range(8)]
assert all(set(v)=={0,0x1fffff} for v in uv)
resources = []
for ident in ['root/2/disc/152','root/2/disc/40','root/2/disc/108','root/2/disc/61']:
    e=entry(ident)
    resources.append({'entry_id':ident,'source':e['normalized_iso_path'],'input_sha256':e['sha256'],'input_size':e['size']})
receipt = {'candidate_sha256':candidate,'instruction_contracts':listings,
 'map':{'source_entry':'root/2/disc/152','file_offset':0x355e,'width':128,'height':128,'record_stride':14,'row_stride':1792,'region_size':len(region),'region_sha256':sha(region),'height_i16_offset':2,'texture_u8_offset':4,'orientation_flag_u8_offset':5,'orientation_mask':112,'initial_height_min':min(heights),'initial_height_max':max(heights),'initial_texture_min':min(selectors),'initial_texture_max':max(selectors),'coordinate_vectors':coordinate_vectors},
 'height_interpolation':{'function_va':'0x7b140','input_coordinates':'two signed16 words; low bytes are unsigned fractions; high coordinates wrap with mask127','output':'signed32 doubled height; arithmetic shift7, diagonal fx<fy','vectors':vectors,'barycentric_comparisons':comparisons},
 'atlas':{'entry_id':'root/2/disc/40','decoded_sha256':sha(atlas),'decoded_size':len(atlas),'tile_count':160,'zero_index_transparent':False,'tile_vectors':tiles},
 'terrain_vertex':{'horizontal_step':32,'diagonal_vertical_step':16,'height_factor':2,'height_displacement':'2 * (signed_height >> 6)','note':'Do not replace with (2*height)>>6; original terrain-vertex rounding differs from arbitrary-point projection.'},
 'uv_orientation_table':{'va':'0x1060ec','index':'(record_byte5 & 0x70)>>4','corners_u16_16':uv},
 'palette':{'descriptor_va':'0x1043e4','filename':'data/main.pal','buffer_global':'0x115e80','active_view':'0x1863a2','active_view_palette_field':'0x1863a4'},
 'resources':resources,
 'limits':['Map record bytes beyond height/texture/orientation remain opaque; signed16 at0 and byte12 feed lighting but are not assigned gameplay names.','Mode5 samples stride256 and maps every source index through FADE.DAT; zero is not skipped.','Initial camera origin, live shade/visibility fields, full triangle rasterization/clipping and overlay composition are not implemented by this receipt.','Atlas pointer160 is one past source allocation; safe API rejects160+ rather than accepting the startup sentinel.','Height scale and steps are constructor facts, not proof of a selected camera position or a whole original terrain frame.']}
(P/'research/ghidra/terrain-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS {len(listings)} instruction contracts; {comparisons} height comparisons; 7 coordinate boundaries; 160 tiles/163840 source bytes; exact first-map field extents')

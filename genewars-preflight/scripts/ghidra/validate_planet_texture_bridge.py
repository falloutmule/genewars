"""Hash-bound PL1..4 texture slices and qualified PLANETS.DAT negative evidence."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
CANDIDATE='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==CANDIDATE
obj1=(G/'validated/memory-.object1.bin').read_bytes()
obj3=(G/'validated/memory-.object3.bin').read_bytes()
assert all(b'planets.dat' not in p.read_bytes().lower() for p in (G/'validated').glob('memory-*.object*.bin'))
assert obj3[0x2e58:].split(b'\0',1)[0]==b'data/planets.anb'
assert obj3[0xd097:].split(b'\0',1)[0]==b'data/pl%d.dat'
assert struct.unpack_from('<I',obj3,0x2e74)[0]==0x18654c
assert struct.unpack_from('<I',obj1,0xe3513+12-0x10000)[0]==0xe3a4f
assert struct.unpack_from('<I',obj1,0xe1a67+12-0x10000)[0]==0xe1d61
contracts={
 'planets-refs/instructions-00015b90.txt':[
  '00015cbe MOV AL,byte ptr [EAX + 0x11945c]', '00015cc9 INC EAX',
  '00015ccb PUSH 0x10d097','00015cd7 CALL 0x000981d3',
  '00015ce4 MOV EBX,dword ptr [0x00115e6c]', '00015cf0 CALL 0x000154f0',
  '00015cff MOV dword ptr [EBP + 0x18],EAX', '00015cfa MOV EDX,0x102e58',
  '00015d07 CALL 0x000154b0'],
 'planets-file/instructions-0009a09c.txt':[
  '0009a0ca CALL dword ptr [0x0010a6f0]', '0009a0d2 MOV ECX,EAX',
  '0009a121 MOV EAX,ECX','0009a123 CALL 0x0009954c'],
 'config-path/instructions-000150f0.txt':[
  '00015392 MOV EAX,0x150b0','00015399 CALL 0x0009a090'],
 'config-parser/instructions-000150b0.txt':[
  '000150b0 CMP byte ptr [EAX],0x2a','000150d4 PUSH 0x10cf0c','000150de CALL 0x000981d3'],
 'planets-consumers/instructions-000192d0.txt':[
  '000192db MOV AH,0x3','000192e3 MOV byte ptr [0x00102e25],AH',
  '00019697 AND DL,0x1','000196a3 SHL EDX,0x7',
  '000196ae MOV EAX,dword ptr [ESI + 0x18]', '000196b1 SAR EBX,0x1',
  '000196b7 SHL EBX,0xf','000196c8 MOV EBX,0x7f0000',
  '000196cd MOV dword ptr [0x00100108],EDX',
  '00019434 SUB EDX,0x8','00019441 SAR EAX,0x2',
  '00019444 SHL ECX,0x6','00019450 LEA EBX,[EAX + 0x200]',
  '00019456 AND ECX,0xff','00019469 SHL EAX,0x8',
  '00019474 MOV dword ptr [0x00100108],EDX','0001947a MOV EDX,0x3f0000',
  '00019497 CALL 0x000e1900','000194a5 CALL 0x000e1900'],
 'planets-texture/mode3-tail.txt':[
  '000e1f4f MOVZX EAX,byte ptr [0x00102e25]',
  '000e1f56 JMP dword ptr [EAX*0x4 + 0xe3513]'],
 'planets-texture/mode3-sampler.txt':[
  '000e3a62 XOR EBX,EBX','000e3adc ROL EDX,0x10',
  '000e3adf MOV BH,DL','000e3ae5 MOV BL,byte ptr [ESI + 0xa]',
  '000e3aec MOV ESI,dword ptr [0x00100108]',
  '000e3af2 MOV AL,byte ptr [EBX + ESI*0x1]',
  '000e3af5 OR AL,AL','000e3af7 JZ 0x000e3afb','000e3af9 MOV byte ptr [EDI],AL'],
}
listings=[]
for name,terms in contracts.items():
 raw=(G/name).read_bytes();text=raw.decode()
 for term in terms:assert term in text,(name,term)
 listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(raw)))
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries']
corpus=json.loads((P/'research/integration/rnc-corpus.json').read_text())['results']
assets=[];pixels=0
for entry_id in ('root/2/disc/117','root/2/disc/118','root/2/disc/119','root/2/disc/120'):
 source=next(e for e in manifest if e['entry_id']==entry_id)
 r=next(r for r in corpus if r['input_sha256']==source['sha256'])
 assert r['status']=='passed' and r['output_size']==163840
 data=(P/r['private_output']).read_bytes();assert len(data)==163840 and sha(data)==r['output_sha256']
 tiles=[]
 for kind in range(14):
  if kind<8:size=128;offset=(kind&1)*128+(kind>>1)*32768
  else:size=64;offset=131072+((kind-8)//4)*16384+((kind-8)%4)*64
  indices=bytes(data[offset+y*256+x] for y in range(size) for x in range(size))
  mask=bytes(int(x!=0) for x in indices);end=offset+(size-1)*256+size
  assert end<=len(data)
  # Two equivalent reads enforce the byte stride independently of the gather expression.
  rows=b''.join(data[offset+y*256:offset+y*256+size] for y in range(size))
  assert rows==indices
  pixels+=len(indices)
  tiles.append(dict(type=kind,width=size,height=size,source_offset=offset,source_stride=256,
                    end_exclusive=end,indices_sha256=sha(indices),mask_sha256=sha(mask),opaque=sum(mask)))
 assets.append(dict(entry_id=entry_id,source_path=source['path'],source_sha256=source['sha256'],
  decoded_sha256=r['output_sha256'],decoded_size=len(data),tiles=tiles))
assert pixels==622592
negative=next(e for e in manifest if e['entry_id']=='root/2/disc/122')
assert negative['sha256']=='1e3b70c2b6856c645af01db2a98f8a71f99b7c823324c24f6406bce734c5e23c'
receipt=dict(candidate_sha256=CANDIDATE,status='verified_bounded_texture_tiles',
 planet_dat=dict(entry_id=negative['entry_id'],source_sha256=negative['sha256'],status='consumer_unproven',
  finding='No literal planets.dat in four mapped objects; actual selection path loads numbered pl%d.dat and planets.anb. Not proof of dead data or of absence of every possible computed filename.'),
 load_va='0x15b90',load_sequence='format10d097 data/pl%d.dat with unsigned system byte11945c+1 ->15cd7 ->154f0 at15cf0, save buffer screen+18; independent planets.anb descriptor102e58 ->154b0 at15d07',
 consumer_va='0x192d0',texture_base_global='0x100108',mode_global='0x102e25',mode=3,
 sampler_va='0xe3a4f',sampler_rule='unsigned16 offset=(low8(v_integer)<<8)|low8(u_integer); read one index; zero preserves destination, nonzero writes unchanged',
 big_tile_rule='types0..7, 128x128, base=(type&1)*128+(type>>1)*32768, stride256, supplied UV0..127 in16.16',
 small_tile_rule='types8..13, 64x64, base=131072+floor((type-8)/4)*16384+((type-8)%4)*64, stride256, supplied UV0..63 in16.16',
 private_listings=listings,assets=assets,tests=dict(listing_contracts=len(contracts),tiles=56,indices_checked=pixels),
 limits=['No PLANETS.DAT parser or record count established',
  'Exact input hashes admitted; no filename-only codec selection',
  'Tile extraction preserves bytes and explicit opacity; complete triangle interpolation, projection, clipping and draw order excluded',
  'Palette upload association not yet traced; no substituted palette or RGB conversion',
  'Types14 and above excluded even though some offset expressions would remain in allocation',
  'Four decoded atlases have163840bytes; bounded tiles do not interpret remaining unused slots',
  'Raw proprietary indices remain private; receipt contains hashes/counts only'])
(P/'research/ghidra/planet-texture-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',listing_contracts=len(contracts),tiles=56,indices_checked=pixels)))

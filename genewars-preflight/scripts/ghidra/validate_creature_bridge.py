"""Original static contracts for caller-selected packed-nibble creature sprites."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2]
G=P/'private/ghidra'
sha=lambda x:hashlib.sha256(x).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
data=(G/'validated/memory-.object3.bin').read_bytes()
names=[]
for i in range(25):
    ptr=struct.unpack_from('<I',data,0x4494+4*i)[0]
    name=data[ptr-0x100000:].split(b'\0',1)[0].decode('ascii')
    names.append(name)
assert len(set(names))==25 and 'bones.anb' not in names
contracts={
 'animal-loader/instructions-0006ac50.txt':['MOV EDX,dword ptr [EAX*0x4 + 0x104494]','CALL 0x00098a60'],
 'creature-paths/instructions-0001d770.txt':['MOV EAX,0x180e20','CALL 0x0006ac50'],
 'creature-paths/instructions-00022fd0.txt':['CALL 0x000947e0','CALL 0x00094bc0'],
 'creature-consumer/instructions-000947e0.txt':['CALL 0x0008fbd0'],
 'creature-consumer/instructions-00094bc0.txt':['CALL 0x00093400'],
 'creature-consumer/instructions-00093400.txt':['CMP EBX,0x100','CALL 0x0008fbd0','SAR EDX,0x4','AND DL,0xf'],
 'creature-consumer/instructions-0008fbd0.txt':['0008fc0f SAR ECX,0x1','0008fc1b AND CL,0x1','0008fc1e INC EBP','0008fc22 ADD ECX,EBP','000900d1 SAR EDI,0x4','000900ed AND CH,0xf','0009011e SAR EBX,0x4','00090124 MOV BL,byte ptr [ESI + EBX*0x1]','0009019d ADD EAX,EBX','000901e8 JZ 0x0009044f','00090455 RET 0x4'],
}
listings=[]
for name,terms in contracts.items():
    raw=(G/name).read_bytes(); txt=raw.decode()
    for term in terms: assert term in txt,(name,term)
    listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(raw)))
rows=json.loads((P/'research/integration/sprite-corpus.json').read_text())['results']
admitted=[]
for name in names:
    matching=[r for r in rows if (r['entry_id'].startswith('root/1/') or r['entry_id'].startswith('root/2/disc/')) and r['source_path'].replace('\\','/').rsplit('/',1)[-1].split(';')[0].lower()==name]
    assert len(matching)==2,(name,len(matching))
    assert len({r['input_sha256'] for r in matching})==1
    for r in matching:
        raw=(P/r['private_database']).read_bytes()
        assert sha(raw)==r['decoded_sha256']
    admitted.append(dict(loader_index=names.index(name),filename=name,source_sha256=matching[0]['input_sha256'],decoded_sha256=matching[0]['decoded_sha256'],sources=[dict(entry_id=r['entry_id'],source_path=r['source_path']) for r in matching]))
def run(n,packed,base):
    assert len(packed)==(n+1)//2
    return [((packed[i//2]>>(4 if i%2==0 else 0))&15)+base &255 for i in range(n)]
vectors=[dict(count=3,packed_hex='abc0',base=0,expected=[10,11,12]),dict(count=4,packed_hex='0123',base=254,expected=[254,255,0,1]),dict(count=1,packed_hex='0f',base=0,expected=[0])]
for v in vectors: assert run(v['count'],bytes.fromhex(v['packed_hex']),v['base'])==v['expected']
receipt=dict(scope='Static provisional CD candidate; explicit caller-selected packed4 codec, no original execution',candidate_sha256=candidate,loader_name_table_va='0x104494',loader_va='0x6ac50',loader_destination_va='0x180e20',frame_drawers=['0x947e0','0x94bc0'],sprite_drawers=['0x8fbd0','0x93400'],packed4_source_sha256=[r['source_sha256'] for r in admitted],admitted_sources=admitted,private_listings=listings,synthetic_vectors=vectors,contract=dict(sprite_data_base=18,record_stride=6,width_offset=4,height_offset=5,positive_control='N opaque pixels from ceil(N/2) packed bytes; high nibble then low; odd low padding ignored',negative_control='signed -1 through -128 skips transparent pixels',zero_control='row end',base_color='u8 base + nibble, wrapping modulo256 when no translation table',translation='optional table indexes usize(base)+nibble without prewrap; maximum270',clipping='consume full packed run even if displayed writes clip to width',scope_limit='Safe complete-row parsing requires height zero terminators; original last visible row may return at width exhaustion before reading final zero'),excluded=['BONES.ANB not in verified25-name loader; separate caller unresolved','GPF wrapper/CRC unresolved','Scale, flip, lighting, palette bank association, animation timing not ported here'])
(P/'research/ghidra/creature-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',private_listing_contracts=len(listings),admitted_unique_inputs=len(admitted),source_aliases=sum(len(x['sources']) for x in admitted),synthetic_vectors=len(vectors))))

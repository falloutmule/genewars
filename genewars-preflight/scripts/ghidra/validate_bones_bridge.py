"""Hash-bound BONES packed4 caller admission and bounded raw-index diagnostics."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
H='2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f'
b=(P/f'private/integration/sprites/{H}.anb').read_bytes();assert sha(b)==H
n,t,c,f,a=struct.unpack_from('<H4I',b);assert (len(b),n,t,c,f,a)==(4077,5,3721,3787,3887,4027)
contracts={
 'creature-paths/instructions-0001d770.txt':['0001d9c4 MOV EAX,[0x00116800]','0001d9c9 CALL 0x00098a60','0001d9da MOV [0x00116804],EAX'],
 'bones-frame/instructions-0005a1c0.txt':['CMP byte ptr [EAX + 0x5e],0x1a','SETNC DL','MOV EDX,dword ptr [0x00116804]','MOV dword ptr [EAX + 0x1f],EDX'],
 'bones-consumer/entity-render.txt':['00059a45 MOV EDX,dword ptr [ESI + 0x1f]','00059a6c CALL 0x00094910','00059b8d MOV EBX,dword ptr [ESI + 0x1f]','00059b59 CALL 0x00094d70'],
 'bones-frame/instructions-00094910.txt':['MOV ESI,dword ptr [EBX]','MOV EBX,dword ptr [ESI]','CALL 0x00090460'],
 'bones-frame/instructions-00094d70.txt':['MOV ESI,dword ptr [EBX]','CALL 0x00094400'],
 'bones-blit/instructions-00090460.txt':['000904a5 SAR ECX,0x1','000904ad AND CL,0x1','000904bf INC ECX','000904c0 ADD ECX,EBP','000905e5 SAR ESI,0x4','00090596 AND CL,0xf'],
 'bones-blit/instructions-00094400.txt':['00094452 CALL 0x00090460','000944ba SAR EAX,0x1','000944bc AND BL,0x1','000944bf INC EAX','000944c3 ADD EAX,EBX'],
}
listings=[]
for name,terms in contracts.items():
 raw=(G/name).read_bytes();txt=raw.decode()
 for term in terms:assert term in txt,(name,term)
 listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(raw)))
obj=(G/'validated/memory-.object3.bin').read_bytes()
u32=lambda va:struct.unpack_from('<I',obj,va-0x100000)[0]
assert obj[0x401c:].split(b'\0',1)[0]==b'data/creature/bones.anb'
assert u32(0x104038)==0x116800
records=[];rows=0;opaque=0;short_rows=0
for i in range((c-t)//6):
 offset,w,h=struct.unpack_from('<IBB',b,t+i*6);start=offset+18;p=start
 if not w or not h:
  records.append(dict(record=i,width=w,height=h,active=False));continue
 indices=bytearray(w*h);mask=bytearray(w*h);run_widths=[]
 for y in range(h):
  x=0
  while True:
   assert p<t,(i,y,p,t)
   control=b[p];p+=1;control=control if control<128 else control-256
   if control==0:break
   if control<0:x-=control
   else:
    size=(control+1)//2;assert p+size<=t
    for j in range(control):
     assert x+j<w,(i,y,x,control,w)
     packed=b[p+j//2];nibble=packed>>4 if j%2==0 else packed&15
     indices[y*w+x+j]=nibble;mask[y*w+x+j]=1
    p+=size;x+=control
   assert x<=w,(i,y,x,w)
  run_widths.append(x);short_rows+=x<w;rows+=1
 count=sum(mask);opaque+=count
 records.append(dict(record=i,width=w,height=h,active=True,start=start,end=p,consumed=p-start,opaque=count,indices_sha256=sha(indices),mask_sha256=sha(mask),short_rows=sum(x<w for x in run_widths),maximum_row_width=max(run_widths)))
assert len(records)==11 and sum(r['active'] for r in records)==10
assert records[-1]['end']==t and max(r.get('end',0) for r in records)==t
active=[r for r in records if r['active']]
assert all(left['end']==right['start'] for left,right in zip(active,active[1:]))
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries']
aliases=[{k:e[k] for k in ['entry_id','path','sha256'] if k in e} for e in manifest if e.get('sha256')==H]
assert len(aliases)==2
receipt=dict(candidate_sha256='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',source_sha256=H,packed4_source_sha256=[H],source_aliases=aliases,status='verified_packed4',scope='Raw unscaled packed4 indices and opacity; caller color/lighting not ported',loader_descriptor_va='0x10401c',loader_pointer_va='0x116800',animation_base_va='0x116804',selector_va='0x5a1c0',selector_rule='unsigned byte entity+0x5e >=26 selects BONES, stores animation base at entity+0x1f',draw_regions=['0x59a45 -> 0x59a6c -> 0x94910 -> 0x90460','0x59b32 -> 0x59b59 -> 0x94d70 -> 0x94400'],rows=rows,short_rows=short_rows,opaque_pixels=opaque,data_end=t,records=records,private_listings=listings,limits=['No admission for other hashes or files named BONES','Short rows leave an implicit transparent remainder; no width-equality requirement','Lighting, palette translation, flip and scale behavior excluded','No original executable run by this analysis'])
(P/'research/ghidra/bones-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',active_records=10,rows=rows,short_rows=short_rows,opaque_pixels=opaque,final_stream_end=t,listing_contracts=len(listings))))

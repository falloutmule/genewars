"""Verify exact-CD config parser/setup evidence; never create config or run originals."""
from pathlib import Path
import hashlib,json,re,struct,collections
P=Path(__file__).resolve().parents[2]; G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
installer='570059e6b64f289e740491814fa775d4c87d53b4e2a61d53cea5e7cd7106feb7'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
B=(G/'config-setup/INST.EXE').read_bytes();assert sha(B)==installer
pad=(G/'config-setup/INST-analysis-padded.exe').read_bytes()
assert len(B)==1062561 and pad==B+bytes(863)
H=struct.unpack_from('<I',B,60)[0]
u32=lambda o:struct.unpack_from('<I',B,o)[0]
assert B[H:H+2]==b'LE' and u32(H+40)==4096
objects=[struct.unpack_from('<6I',B,H+u32(H+64)+i*24) for i in range(u32(H+68))]
images=[bytearray(o[0]) for o in objects];owners={};pages=u32(H+20)
for oi,(size,base,flags,index,count,_) in enumerate(objects):
 for local in range(count):
  page=index+local-1; owners[page]=(oi,local)
  entry=B[H+u32(H+72)+page*4:H+u32(H+72)+(page+1)*4]
  physical=int.from_bytes(entry[:3],'big');assert entry[3]==0 and 1<=physical<=pages
  disk=u32(H+128)+(physical-1)*4096;amount=min(4096,size-local*4096,len(B)-disk)
  assert amount>0
  images[oi][local*4096:local*4096+amount]=B[disk:disk+amount]
assert len(owners)==pages==232
ft=struct.unpack_from('<'+str(pages+1)+'I',B,H+u32(H+104));count=0
for page in range(pages):
 oi,local=owners[page];pos=H+u32(H+108)+ft[page];end=H+u32(H+108)+ft[page+1]
 while pos<end:
  source,flags=B[pos:pos+2];pos+=2;assert flags&15==0
  if source&32: n=B[pos];pos+=1;offsets=None
  else: offsets=[struct.unpack_from('<h',B,pos)[0]];pos+=2
  if flags&64: target_object=struct.unpack_from('<H',B,pos)[0];pos+=2
  else: target_object=B[pos];pos+=1
  kind=source&15
  if kind==2: target_offset=0
  elif flags&16: target_offset=u32(pos);pos+=4
  else: target_offset=struct.unpack_from('<H',B,pos)[0];pos+=2
  if offsets is None: offsets=struct.unpack_from('<'+str(n)+'h',B,pos);pos+=2*n
  target=objects[target_object-1][1]+target_offset
  assert 0<=target_offset<=objects[target_object-1][0]
  for relative in offsets:
   off=local*4096+relative;address=objects[oi][1]+off
   if kind==7: value,width=target,4
   elif kind==8: value,width=(target-address-4)&0xffffffff,4
   elif kind==2: value,width=target_object-1,2
   elif kind==5: value,width=target&65535,2
   else: raise AssertionError(kind)
   assert 0<=off and off+width<=len(images[oi])
   images[oi][off:off+width]=value.to_bytes(width,'little');count+=1
  assert pos<=end
 assert pos==end
assert count==7845
for i,image in enumerate(images): assert image==(G/f'config-installer/memory-.object{i+1}.bin').read_bytes()
contracts={
 'config-path/instructions-000150f0.txt':['00015110 JZ 0x000153d5','MOV EDX,0x50','CALL 0x00099e5e','00015370 INC ECX','00015384 CMP EBP,0x7','MOV byte ptr [ECX + 0x1c2],0x5','MOV dword ptr [ECX],0x7f','MOV dword ptr [ECX + 0x4],0x32'],
 'config-parser/instructions-000150b0.txt':['MOV EAX,0x1187bc','MOV EAX,0x11883c','CMP byte ptr [EAX],0x2a'],
 'config-parser/instructions-000154f0.txt':['CMP EAX,0x64','CALL 0x0009954c','CALL 0x0009a2b8'],
 'config-read/instructions-0009954c.txt':['CALL 0x000994a0','MOV EBX,EBP','CALL 0x0009a420'],
 'config-sound/instructions-00015050.txt':['MOV EAX,dword ptr [EAX]','MOV AL,byte ptr [EDX + 0x4]','CMP byte ptr [EDX + 0x8],0x0','MOV AL,byte ptr [EDX + 0x9]'],
 'config-installer-refs/instructions-0001ae48.txt':['MOV EDX,0x50893','MOV EAX,0x508b1','MOV EAX,0x508ba','MOV EAX,0xface0'],
 'config-installer-values/choice-range.txt':['MOV EDX,0x50212','MOV EDX,0x50222'],
 'config-installer-values/choice2-range.txt':['MOV EDX,0x502b4','MOV EDX,0x502c4'],
 'config-installer-values/choice3-range.txt':['MOV EDX,0x50360','MOV EDX,0x50370'],
}
listings=[]
for name,terms in contracts.items():
 raw=(G/name).read_bytes();txt=raw.decode()
 for term in terms:assert term in txt,(name,term)
 listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(raw)))
strings=lambda image,base,va: bytes(image[va-base:]).split(b'\0',1)[0].decode('ascii')
setup_strings={hex(va):strings(images[1],0x50000,va) for va in [0x508b1,0x508ba,0x508ce,0x508dd,0x508e9,0x508f5,0x50902,0x5090f]}
assert setup_strings['0x508ba']=='InstallDrive="%s\\"\n'
assert [strings(images[1],0x50000,v) for v in [0x50212,0x50216,0x5021a,0x5021e,0x50222]]==['All']*5
assert [strings(images[1],0x50000,v) for v in [0x502b4,0x502b8,0x502bc,0x502c0,0x502c4]]==['All','All','Min','Min','Min']
assert [strings(images[1],0x50000,v) for v in [0x50360,0x50364,0x50368,0x5036c,0x50370]]==['Min']*5
def tokens(line):return [x for x in re.split('[="]',line) if x]
vectors=[]
for line,recognized,installed in [('Data="All"\n',True,True),('data="All"\n',True,True),('Data ="All"\n',False,None),('Data="all"\n',True,False),('Data="Min"\n',True,False)]:
 t=tokens(line);r=t[0].lower()=='data';v=t[1][0]=='A' if r else None
 assert (r,v)==(recognized,installed)
 vectors.append(dict(line=line,recognized=r,installed_prefix=v))
assert tokens('CD="D:\\GAME\\"\n')[1]=='D:\\GAME\\'
assert sum(tokens('Data="All"\n')[1][0]=='A' for _ in range(7))==7
disc=json.loads((P/'research/disc-inventory.json').read_text())['entries']
assert not [e for e in disc if e.get('normalized_iso_path','').split('/')[-1].lower() in ['config.ini','sound.ini']]
assert any(e.get('sha256')==installer and e.get('normalized_iso_path')=='INSTALL/DOS/INST.EXE' for e in disc)
receipt=dict(candidate_sha256=candidate,installer_sha256=installer,scope='Exact CD static parser/setup writer; no generated config or execution',parser_va='0x150f0',setup_writer_va='0x1ae48',config_open=dict(path='config.ini',mode='r',relative_to='current working directory',failure_before_parsing=True),keys=['CD','InstallDrive','Language','Intro','Anims','Sounds','Levels','Data'],setup_format_strings=setup_strings,installer_mapping=dict(original_size=len(B),analysis_padding_zero_bytes=863,pages=pages,fixup_instances=count,all_three_mapped_objects_equal=True),category_choices=[['All']*5,['All','All','Min','Min','Min'],['Min']*5],sound_fallback_hex='7f000000320000000101',sound_read_path='InstallDrive prefix + sound.ini',sound_ini_absent_cd=True,config_ini_absent_cd=True,parser_vectors=vectors,private_listings=listings,limits=['No proof of successful original startup','Mount letters and chosen install location are preparation choices','No sound.ini input length guard proved','No UI install-selection default inferred','Missing keys are not validated but later consumers need valid paths','No new original session authorized by this receipt'])
(P/'research/ghidra/config-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',newline='\n')
print(json.dumps(dict(status='passed',installer_pages=pages,fixups=count,listing_contracts=len(listings),parser_vectors=len(vectors))))

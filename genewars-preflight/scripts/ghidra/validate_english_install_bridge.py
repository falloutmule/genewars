"""Validate CD English installer copy evidence without executing original programs."""
from pathlib import Path
import hashlib,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
installer='570059e6b64f289e740491814fa775d4c87d53b4e2a61d53cea5e7cd7106feb7'
game='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'config-setup/INST.EXE').read_bytes())==installer
assert sha((G/'GW-CD-candidate.exe').read_bytes())==game
ib=(G/'config-installer/memory-.object2.bin').read_bytes()
gb=(G/'validated/memory-.object3.bin').read_bytes()
def string(b,va,base):
 o=va-base;return b[o:b.index(0,o)].decode('ascii')
assert string(ib,0x501fa,0x50000)=='%sDATA\\TEXT\\%s\\TEXT.DAT'
assert string(ib,0x501de,0x50000)=='%sSPEECH\\%s\\*.*'
assert string(ib,0x503d6,0x50000)=='ENG'
assert string(ib,0x5050a,0x50000)=='%s\\%s%s'
assert string(gb,0x10443c,0x100000)=='data/text.dat'
assert struct.unpack_from('<I',gb,0x443c+0x1c)[0]==0x115e88
contracts={
 'font-installer-copy.txt':['000135d6 MOV EAX,0xface8','000135e2 MOV EAX,0x501fa','000135e8 MOV EAX,0x6f83c','000135ee CALL 0x00027b4c'],
 'font-installer-copy-loop.txt':['0001372e MOV EAX,dword ptr [EBP + 0xfffffe94]','00013734 CALL 0x000170c8','00013748 ADD dword ptr [EBP + 0xfffffe94],0x120'],
 'font-installer-refs/instructions-00015c38.txt':['00015ef3 MOV EDX,0x503ce','00015f02 MOV EDX,0x503d6','00015f07 MOV EAX,0xface8'],
 'font-installer-copy-worker/instructions-000170c8.txt':['000171af MOV EAX,dword ptr [EBP + -0x4]','000171b2 ADD EAX,0x90','000171b8 MOV EAX,0xfb1ee','000171be MOV EAX,0x5050a','000171ca CALL 0x00027b4c','00017333 MOV EDX,0x1472d4','0001733d CALL 0x0001c108'],
 'font-text/instructions-0001d770.txt':['0001d799 MOV AH,0x5','0001d79b MOV EDX,0x10443c','0001d7ab CALL 0x000154b0','0001da48 MOV EBX,0x25c','0001da60 DEC EBX','0001da64 JZ 0x0001da7d','0001da71 MOV dword ptr [EDX],EAX']}
listings=[]
for name,terms in contracts.items():
 b=(G/name).read_bytes();t=b.decode()
 for term in terms:assert term in t,(name,term)
 listings.append(dict(private_path='private/ghidra/'+name,sha256=sha(b)))
lists=[]
for base,count in [(0x6f71c,20),(0x70ebc,25),(0x72bfc,24)]:
 rows=[]
 for i in range(count):
  va=base+i*0x120; rows.append(dict(index=i,source_pattern=string(ib,va,0x50000),destination_directory=string(ib,va+0x90,0x50000)))
 assert rows[0]['destination_directory']=='SPEECH\\' and rows[1]['destination_directory']=='DATA\\'
 lists.append(dict(va=hex(base),records=rows))
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries']
selected=[]
for e in manifest:
 path=e.get('normalized_iso_path','')
 if e['entry_id'].startswith('root/2/disc/') and e['kind']=='file' and (path=='GAME/DATA/TEXT/ENG/TEXT.DAT' or path.startswith('GAME/SPEECH/ENG/')):
  dest='DATA/TEXT.DAT' if '/TEXT/' in path else 'SPEECH/'+path.rsplit('/',1)[1]
  selected.append(dict(entry_id=e['entry_id'],source=path,destination=dest,size=e['size'],sha256=e['sha256']))
assert len(selected)==19
text=next(e for e in selected if e['destination']=='DATA/TEXT.DAT')
assert text['entry_id']=='root/2/disc/592' and text['size']==36610 and text['sha256']=='23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786'
out=dict(status='verified_static_installer_copy_contract',installer_sha256=installer,game_candidate_sha256=game,language='ENG',copy_semantics='Byte copy, unchanged basename, install-prefix + destination_directory + basename',language_selection_va='0x15ef3..0x15f0c',text_source_format_va='0x135d6..0x135ee',copy_worker_va='0x170c8',destination_format_va='0x171a9..0x171ca',runtime_text_descriptor_va='0x10443c',runtime_loader_va='0x1d799..0x1d7ab',runtime_string_count=604,scope='Selected language rows only; does not install or execute originals; other table rows preserve relative source-directory layout.',copies=selected,installer_lists=lists,private_listings=listings)
(P/'research/ghidra/english-install-evidence.json').write_text(json.dumps(out,indent=2)+'\n')
print('PASS English copy contract: 5 instruction contracts, 3 installer lists, 19 exact CD source/destination records; runtime 604 strings.')

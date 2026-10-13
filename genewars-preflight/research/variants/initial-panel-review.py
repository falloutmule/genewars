"""Independent bounded startup/module/panel witness; no original execution."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,itertools,json,struct
ROOT=Path(__file__).resolve().parents[2]
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def main():
 proof_path=ROOT/'research/ghidra/initial-panel-evidence.json';proof=json.loads(proof_path.read_bytes());pins=[];text={}
 for c in proof['instruction_contracts']:
  p=ROOT/c['private_path'];r=pin(p);assert r['sha256']==c['sha256'];pins.append(r);text[p.name]=p.read_text()
 candidate=ROOT/'private/ghidra/GW-CD-candidate.exe';assert pin(candidate)['sha256']==proof['candidate_sha256']
 data=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes()
 def word(va):return struct.unpack_from('<I',data,va-0x100000)[0]
 def name(va):off=word(va)-0x100000;return data[off:data.index(0,off)].decode('ascii')
 # The outgoing base-to-module adjustment and incoming virtual thunk cancel.
 root=0x185dfc;delta=word(0x11259c+4);module=root+delta;back_slot=module-4
 assert delta==0x5a6 and module==0x1863a2 and word(0x11257c)==0x30120
 for line in ['0002ba5e MOV dword ptr [EBX + EAX*0x1 + 0xfffffacc],EAX','0002ba75 SUB EBX,0x530','0002ba7b ADD EAX,0x7','0002ba7e MOV dword ptr [EBX + EAX*0x1],0x11257c']:assert line in text['instructions-0002b7e0.txt']
 for line in ['00021944 MOV EBX,EAX','0002194b LEA EBX,[EAX + 0x8d]','00021951 SUB EBX,0x8d','00021a06 MOV EAX,EBX','00021a0c RET']:assert line in text['instructions-00021940.txt']
 assert ' CALL ' not in text['instructions-00021940.txt'] and ' CALL ' not in text['instructions-000252b0.txt']
 for line in ['000252bd ADD EAX,0x19','000252cd SUB EAX,0x19','00025320 RET']:assert line in text['instructions-000252b0.txt']
 # Both transitive constructors return their supplied pointer; the direct
 # ADD/LEA/SUB chain yields these root-relative intermediate offsets.
 offsets=[0x6b+0x51+0x10];offsets.append(offsets[-1]+0x16);offsets.append(offsets[-1]+0x97);offsets.append(offsets[-1]+0x1a);offsets.append(offsets[-1]+0x31);offsets.append(offsets[-1]+0x2c);offsets.append(offsets[-1]+0xaf);offsets.append(offsets[-1]+0x291)
 assert offsets==[0xcc,0xe2,0x179,0x193,0x1c4,0x1f0,0x29f,0x530]
 # EBX is root+0x530 here; the ctor writes delta at root+delta-4.
 assert root+0x530+delta-0x534==back_slot
 assert root+0x530-0x530+delta+7==module+7
 for line in ['00030120 SUB EAX,dword ptr [EAX + -0x4]','00030123 JMP 0x0002bbe0']:assert line in text['range-30110.txt']
 assert module-delta==root
 for line in ['0001bcc9 MOV EAX,[0x00116978]','0001bcce MOV EBX,dword ptr [EAX + 0x7]','0001bcd1 CALL dword ptr [EBX]']:assert line in text['instructions-0001bbc0.txt']
 for line in ['0002bdb9 CMP dword ptr [EBP + 0xc8],0x0','0002bddd MOV EBX,dword ptr [0x0011697c]','0002bde3 CMP EBX,0x185def','0002bdeb CMP EBX,0x1863b8','0002bdf3 CMP dword ptr [0x0011ac22],0x0','0002bdfc TEST byte ptr [0x00118770],0x20','0002be12 MOVSX EAX,byte ptr [EAX + 0x11945b]','0002be19 CMP EAX,0x4','0002be1c JL 0x0002be2d','0002be35 CALL 0x00025150']:assert line in text['instructions-0002bbe0.txt']
 child=root+0x6b+0x51+0x10;descriptor=0x111f04;panel=child+word(descriptor+4)
 assert child==0x185ec8 and panel==0x185ed4 and word(descriptor+4)==12
 for line in ['0002b80b MOV dword ptr [EAX],0x111f04','0002b84a MOV byte ptr [EDX + EAX*0x1 + 0x4],0x0']:assert line in text['instructions-0002b7e0.txt']
 names=[name(0x105604+i*4).lower() for i in range(3)];assert names==['data/mainpanl.dat','data/genepanl.dat','data/lpanel.dat']
 def branch(active,old,pending,flags,raw_progress):
  # Execute branch priorities, preserving source signed-byte interpretation.
  if active:return 'preserve-existing-child'
  if old!=0x185def:
   if old!=0x1863b8:return 'default-MAINPANL'
  if pending!=0:return 'default-MAINPANL'
  if flags&0x20:
   progress=raw_progress-256 if raw_progress>=128 else raw_progress
   if progress<4:return 'default-MAINPANL'
  return 'alternate-child'
 cases=0;default=alternate=preserved=0
 for active,old,pending,flags,raw in itertools.product([False,True],[0,0x185def,0x1863b8,0x1863a2,0x185dfc,0x7fffffff,0xffffffff],[0,1,0xffffffff],range(256),range(256)):
  result=branch(active,old,pending,flags,raw);progress=(raw+128)%256-128
  # Independent logical characterization of the only alternate route.
  alternate_condition=not active and old in [0x185def,0x1863b8] and pending==0 and(not(flags&32) or progress>=4)
  assert (result=='alternate-child')==alternate_condition
  if not active and flags&32 and progress<4:assert result=='default-MAINPANL'
  cases+=1;default+=result=='default-MAINPANL';alternate+=result=='alternate-child';preserved+=result=='preserve-existing-child'
 for v in proof['vectors']:assert branch(v['child_active'],0x185def if v['special_old'] else 0,1 if v['pending'] else 0,32 if v['campaign'] else 0,v['progress_i8']%256)==v['selection']
 manifest=json.loads((ROOT/'research/game-files-manifest.json').read_bytes())['entries'];entry=next(v for v in manifest if v['entry_id']=='root/2/disc/110');assert entry['sha256']==proof['resource']['input_sha256'] and entry['size']==17203
 compressed=ROOT/'private/reference-run/run-004/drive-c/GAME/DATA/MAINPANL.DAT';assert pin(compressed)['sha256']==entry['sha256']
 decoded=ROOT/f"private/integration/rnc-decoded/{entry['sha256']}.bin";decodedpin=pin(decoded);assert decodedpin['size']==61440 and decodedpin['sha256']==proof['resource']['decoded_sha256']
 result=dict(status='corroborated-conditional-default-MAINPANL',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,evidence=pin(proof_path),instruction_pins=pins,constructor_root_relative_offsets=offsets,branch_cases=cases,default_cases=default,alternate_cases=alternate,preserved_cases=preserved,module=module,root_object=root,default_child=child,active_panel=panel,selector=0,resource_input=pin(compressed),resource_decoded=decodedpin,required_runtime_gates=['Actual module transition requests1863A2 and enters2BBE0','R+C8 null at conditional initializer','Campaign bit20 set','Selected player progress signed byte below4','Successful panel resource load or valid selector0 cache; inherited initialized640destinationpitch'],unresolved=['Null R+C8 not established solely by raw CONQ001 bytes','Retained selected-player progress is live state, not inferred from raw zero','No universal initial active panel, later panel selection or dynamic controls claim'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n',encoding='utf8');tmp.replace(out);print(json.dumps({k:result[k] for k in ['status','branch_cases','selector']}))
if __name__=='__main__':main()

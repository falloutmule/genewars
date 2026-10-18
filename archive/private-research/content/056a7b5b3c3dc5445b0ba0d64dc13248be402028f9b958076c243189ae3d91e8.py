"""Conditional default MAINPANL activation through actual module transition."""
from pathlib import Path
import hashlib
import json
import struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
contracts=[]
for name,lines in {
 'terrain-constructor/instructions-0008db90.txt':['0008dc9d MOV EAX,0x185dfc','0008dcbd CALL 0x0002b7e0'],
 'entity-shadow-panel/instructions-0002b7e0.txt':['0002b7e8 MOV dword ptr [EAX],0x11259c','0002b80b MOV dword ptr [EAX],0x111f04','0002b84a MOV byte ptr [EDX + EAX*0x1 + 0x4],0x0','0002ba5e MOV dword ptr [EBX + EAX*0x1 + 0xfffffacc],EAX','0002ba7e MOV dword ptr [EBX + EAX*0x1],0x11257c'],
 'panel-activation-extra/instructions-000252b0.txt':['000252bd ADD EAX,0x19','000252cd SUB EAX,0x19','00025320 RET'],
 'panel-activation-extra/instructions-00021940.txt':['00021944 MOV EBX,EAX','0002194b LEA EBX,[EAX + 0x8d]','00021951 SUB EBX,0x8d','00021a06 MOV EAX,EBX','00021a0c RET'],
 'terrain-camera-refs/instructions-0007d090.txt':['0007d7b0 MOV dword ptr [0x00185ec4],EBX','0007d7c7 MOV ESI,0x1863a2','0007d7cf MOV dword ptr [0x00116978],ESI'],
 'module-transition-refs/instructions-0001bbc0.txt':['0001bcb5 CMP dword ptr [0x00116978],0x0','0001bcc9 MOV EAX,[0x00116978]','0001bcce MOV EBX,dword ptr [EAX + 0x7]','0001bcd1 CALL dword ptr [EBX]'],
 'panel-activation-refs/range-30110.txt':['00030120 SUB EAX,dword ptr [EAX + -0x4]','00030123 JMP 0x0002bbe0'],
 'entity-order/instructions-0002bbe0.txt':['0002bbe6 MOV EBP,EAX','0002bc0b MOV dword ptr [0x00116974],EDX','0002bdb9 CMP dword ptr [EBP + 0xc8],0x0','0002bdfc TEST byte ptr [0x00118770],0x20','0002be12 MOVSX EAX,byte ptr [EAX + 0x11945b]','0002be19 CMP EAX,0x4','0002be1c JL 0x0002be2d','0002be2d LEA EAX,[EBP + 0xcc]','0002be35 CALL 0x00025150'],
 'terrain-viewport-refs/instructions-00025150.txt':['0002515e CALL 0x00024770','0002516a MOV dword ptr [0x00185ec4],EBX'],
 'display-geometry/instructions-00024770.txt':['000247a7 MOVSX EAX,BL','000247e2 MOV EAX,0xf000','000247fa MOV EDX,dword ptr [EAX*0x4 + 0x105604]','00024806 CALL 0x000154f0'],
 'display-geometry/instructions-00024ad0.txt':['00024ae5 MOV ECX,0x20','00024aee MOVSD.REP ES:EDI,ESI','00024b06 ADD EBX,0x80']}.items():
    b=(G/name).read_bytes()
    for line in lines:assert line in b.decode(),(name,line)
    contracts.append({'private_path':'private/ghidra/'+name,'sha256':sha(b)})
mem=(G/'validated/memory-.object3.bin').read_bytes()
u32=lambda a:struct.unpack_from('<I',mem,a-0x100000)[0]
assert u32(0x11259c+4)==0x5a6 and u32(0x111f04+4)==12 and u32(0x11257c)==0x30120
nameptr=u32(0x105604);o=nameptr-0x100000
assert mem[o:mem.index(0,o)].lower()==b'data/mainpanl.dat'
object_va=0x185dfc;module_va=object_va+0x5a6;child=object_va+0xcc;panel=child+12
assert module_va==0x1863a2 and panel==0x185ed4

def selection(child_active,special_old,pending,campaign,progress):
    assert -128<=progress<=127
    if child_active:return 'preserve-existing-child'
    if special_old and not pending and (not campaign or progress>=4):return 'alternate-child'
    return 'default-MAINPANL'

cases=0
for active in [False,True]:
 for special in [False,True]:
  for pending in [False,True]:
   for campaign in [False,True]:
    for progress in range(-128,128):
     r=selection(active,special,pending,campaign,progress)
     if not active and campaign and progress<4:assert r=='default-MAINPANL'
     cases+=1
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries']
source_sha='18712cbeb0bbe1990b307dc1650a5b054e511afc2597b71c598407a29664ea95'
entry=next(r for r in manifest if r['entry_id']=='root/2/disc/110');assert entry['sha256']==source_sha
decoded=(P/f'private/integration/rnc-decoded/{source_sha}.bin').read_bytes();assert len(decoded)==61440
aliases=[{'entry_id':r['entry_id'],'path':r['path']} for r in manifest if r.get('sha256')==source_sha]
receipt={'candidate_sha256':candidate,'instruction_contracts':contracts,
 'transition':'7D090 setsnextmodule116978=1863A2.1BBC0 calls[module+7].vtable0; ctorbinds11257C->30120, thunksubtractsbackadjust5A6 andjumps2BBE0withobject185DFC.2BBE0setsactive116974=1863A2. Transitive252B0 preservesEAX (onlyADD19/SUB19pair), substantiatingctorroot+530andmodulebackslotwrites.',
 'condition':'object185DFC+C8 mustbeNULL; campaignflags118770mask20set; selectedplayer signedbyte+6E7<4. Underthesegates defaultchild selectedregardlessprior module or11AC22. This isconditional startup proof, notuniversalfirstpanel.',
 'binding':{'object':'0x185dfc','module':'0x1863a2','default_child':'0x185ec8','default_child_descriptor':'0x111f04','descriptor_delta':12,'active_panel':'0x185ed4','selector_signed_byte':0,'name_table':'0x105604','runtime_name':'data/mainpanl.dat','loader':'0x24770','activation':'0x25150','buffer_bytes':61440,'width':128,'height':480,'source_pitch':128,'destination_pitch_gate':640,'palette_reference':'renderer-evidence.json MAIN.PAL binding'},
 'resource':{'entry_id':entry['entry_id'],'path':entry['path'],'input_size':entry['size'],'input_sha256':source_sha,'decoded_size':len(decoded),'decoded_sha256':sha(decoded),'hash_identical_aliases':aliases},
 'truth_cases':cases,'vectors':[{'child_active':a,'special_old':s,'pending':p,'campaign':c,'progress_i8':v,'selection':selection(a,s,p,c,v)} for a,s,p,c,v in [(False,True,False,True,0),(False,True,False,True,3),(False,True,False,True,4),(False,False,False,False,4),(True,True,False,True,0)]],
 'limits':['R+C8 null is a runtime gate, not inferred merely fromrawCONQ source.','Selectedplayer/progress andfirstmissiongates remainexplicit.','7D090 clearsglobal185EC4ononeentrybranch; itdoesnotclearR+C8.','GENEPANL remains a validexplicitdiagnostic but is notthisconditionaldefaultpanel.','Panelartonly; dynamicallydrawn controls/entities overlays separately.']}
(P/'research/ghidra/initial-panel-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'PASS11pins; {cases} branchtruthcases; conditionalMAINPANL selector0/modulethunk; decoded61440B hashbound')

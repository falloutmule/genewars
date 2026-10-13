"""Hash-pinned opcode-3 producer, explicit resolver admission and setter plans."""
from pathlib import Path
import hashlib,itertools,json,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
pins=[]
def pin(path,*needles):
 b=(G/path).read_bytes()
 for line in needles:assert line in b.decode(),(path,line)
 pins.append({'private_path':'private/ghidra/'+path,'sha256':sha(b)})
pin('plant-command/instructions-0002c300.txt','0002c64c TEST byte ptr [0x0019ff71],0xa','0002c674 TEST byte ptr [0x0019ff70],0x8','0002c681 TEST byte ptr [0x00118968],0x2','0002c7dd CMP AL,CL','0002c807 TEST byte ptr [EBX + 0x70],0x1','0002c831 JGE 0x0002c72a','0002c84c MOV byte ptr [EAX + 0xa],0x3','0002c857 MOV word ptr [EAX + 0x4],DX')
pin('entity-order-refs/instructions-0001c120.txt','0001c52c CALL 0x00080fc0')
pin('plant-command/instructions-00080fc0.txt','00080fc8 MOVSX EDX,word ptr [EDX + 0x4]','00080fec TEST byte ptr [ESI + 0x71],0x10','00080ffb OR byte ptr [ESI + 0x57],0x2','0008101c MOV byte ptr [ESI + 0x5b],0x0','00081044 CALL 0x00074a50','00081049 MOV dword ptr [ESI + 0x5c],EAX','00081052 MOV dword ptr [ESI + 0xb],EAX','00081059 TEST byte ptr [ESI + 0x57],0x1','0008106f CALL 0x00051e80')
pin('plant-command/instructions-00074a50.txt','00074b5e SAR AX,0x1','00074b6b SAR DX,0x1','00074b76 CMP EAX,0x1000','00074b7f MOV dword ptr [ESP + 0x4],EAX','00074ba1 CALL 0x00074270','00074bfb CALL 0x0008b520','00074c0f TEST EAX,EDX','00074c49 CALL 0x0008b130','00074c4e CMP EAX,0xa','00074ca1 MOV EAX,dword ptr [ESP + 0x4]')
pin('command-resolver/instructions-000ae028.txt','000ae044 XOR AX,AX','000ae060 DIV EBX','000ae062 MOV AX,word ptr [EAX*0x2 + 0xae134]')
pin('command-resolver/instructions-00074270.txt','00074280 CALL 0x00074160','0007428f MOV EAX,EBX')
pin('command-resolver/instructions-0008b130.txt','0008b141 CALL dword ptr [EDX + 0x5c]','0008b25a CALL dword ptr [ECX + 0x10]','0008b275 ADD EBX,0xa')
code=(G/'validated/memory-.object1.bin').read_bytes()
table=code[0x1c094-0x10000:0x1c094-0x10000+35*4]
assert struct.unpack_from('<I',table,3*4)[0]==0x1c528
tablepin={'va':'0x1c094','length':len(table),'sha256':sha(table),'opcode3_target':'0x1c528'}
def eligible(packet,modifiers,pause,group_word,group_byte,present,kind,owner,player,state,ready,selection,count):
 assert all(0<=v<=255 for v in (packet,modifiers,pause,group_byte,kind,owner,player,ready))
 assert -128<=state<=127 and -128<=selection<=127 and 0<=count<=5
 return bool(packet&8) and not(modifiers&10 or pause&2 or group_word or group_byte) and present and kind==3 and owner==player and state not in (2,3,4,5) and bool(ready&1) and selection<count
def distance2(current,target):
 delta=lambda a,b:(((a-b+16384)&32767)-16384)
 return sum(delta(a,b)**2 for a,b in zip(current,target))
def accepted_target(current,target,terrain,avoid_mask,obstacle,blockers):
 assert len(current)==len(target)==2 and all(-32768<=v<=32767 for v in (*current,*target))
 assert 0<=terrain<=7 and 0<=avoid_mask<=255 and blockers>=0
 if distance2(current,target)<=4096:return {'resolved':list(current),'case':'near-current-snap'}
 if obstacle or (1<<terrain)&avoid_mask or blockers>=10:return {'unsupported':'backward-search-required'}
 return {'resolved':list(target),'case':'accepted-request'}
def setter(actor_id,flags71,flags57,byte82,requested,resolved):
 assert 0<=actor_id<20 and all(0<=v<=255 for v in (flags71,flags57,byte82))
 assert all(-32768<=v<=32767 for v in (*requested,*resolved))
 if flags71&16:return {'kind':'source-noop','writes':{}}
 if byte82 or flags57&1:return {'kind':'unsupported-reinitialize','writes':{}}
 return {'kind':'set-move-order','writes':{'flags57_u8':flags57|2,'byte5b_u8':0,'target5c_i16_pair':list(resolved),'destination0b_i16_pair':list(resolved)},'requested_before_resolver':list(requested)}
vectors=[]
for current,target,terrain,mask,obstacle,blockers in [((0,0),(64,0),0,255,True,10),((0,0),(65,0),0,0,False,0),((32767,-32768),(-32768,32767),7,255,True,255),((0,0),(65,0),2,4,False,0),((0,0),(65,0),2,0,False,10),((0,0),(65,0),2,0,True,0),((0,0),(64,1),2,0,False,9)]:
 vectors.append({'current':current,'requested':target,'terrain_class':terrain,'avoid_mask':mask,'obstacle_nonnull':obstacle,'blocker_weight':blockers,'distance_squared':distance2(current,target),'result':accepted_target(current,target,terrain,mask,obstacle,blockers)})
setter_vectors=[{'actor_id':0,'flags71':a,'flags57':b,'byte82':c,'result':setter(0,a,b,c,(100,200),(101,201))} for a,b,c in [(0,0,0),(0,254,0),(16,1,1),(0,1,0),(0,0,1)]]
gatecases=0
for state,ready,packet in itertools.product(range(-128,128),range(256),range(16)):
 result=eligible(packet,0,0,0,0,True,3,0,0,state,ready,0,2)
 assert result==(state not in range(2,6) and bool(ready%2) and bool(packet&8));gatecases+=1
setcases=0
for a,b,c in itertools.product(range(256),range(256),(0,1)):
 result=setter(0,a,b,c,(0,0),(100,-100))
 assert result['kind']==('source-noop' if a&16 else 'unsupported-reinitialize' if c or b&1 else 'set-move-order');setcases+=1
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,'opcode_table':tablepin,
 'actor_layout':{'base':'0x16360e','stride_decimal':200,'packet_actor_id':'signed i16 at packet+4; safe range0..19; global ID, not player-local','type6':'u8','owner18':'u8','state55':'i8','ready70':'u8 mask01','flags71':'u8 mask10 inhibits setter','flags57':'u8 mask02 pending move; mask01 invokes reinit','byte82':'u8 nonzero invokes reinit','byte5b':'u8','target5c':'two signed i16 words, four bytes','destination0b':'two signed i16 words, four bytes'},
 'producer_scope':'Ground world control route2C64C after current module/hit dispatch, with explicit874B0-projected world point. Require packet19FF70mask08 (left release), modifier19FF71mask0Aclear, pause118968mask02clear, rootgroup526word=0 and52Fbyte=0, selected entity present/type3/owned, signedstate55not2..5, ready70mask01set, signed current portrait index<signed player count. Safe input current index0..4/count0..5; original lacks lower bound. Emit packet+A=3,+0/+2targetwords,+4globalactorID. UIfeedback/audio37 are separate requests.',
 'dispatcher_scope':'Opcode3 table target1C528 calls80FC0. Outer player activity/tick gates remain explicit; this is not whole1C120.',
 'setter_scope':'flags71mask10set returns unchanged before any stores. Otherwise flags57|02, requestedpoint->5C,5B=0, resolver74A50, resolvedpoint->5C and+B. Admitted complete setter excludes byte82!=0 or OLDflags57mask01set, which invoke51E80. Safe errors are atomic; original can already have prefix writes.',
 'resolver_scope':'Only return-value subcontracts: signed15 torus squared distance<=4096 replaces candidate by current and eventually returns current; otherwise accepted candidate returns requested when obstacle74270=NULL, ((1<<terrainclass8B520)&actor53)==0 and blockerweight8B130<10. Type3 avoids type1 special class shortcut. Environmental helper results are explicit validated inputs; no map occupancy algorithm or virtual helper effects claimed. Other cases require original backward trig search and are rejected.',
 'vectors':vectors,'setter_vectors':setter_vectors,'eligibility_cases':gatecases,'setter_branch_cases':setcases,
 'limits':['No claim that raw or newly created actors are already ready.80830 produces flags700400;80AE0 is the ready-bit producer after landing/placement.','No harvest/cut meaning established; plant hover can fall through generic move producer.','No automatic world coordinate/hit conversion, occupancy resolution, physical mouse adapter or actor movement simulation.','Opcode3 consumer lacks owner/type/bounds rechecks; safe API must admit trusted producer result or validate these externally.']}
(P/'research/ghidra/move-order-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(f'PASS7pins;opcode3table;{gatecases}eligibilitycases;{setcases}settercases;12vectors')

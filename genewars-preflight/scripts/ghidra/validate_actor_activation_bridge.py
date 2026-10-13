"""Explicit-seed landing placement and actor activation plans; not a whole tick."""
from pathlib import Path
import hashlib,itertools,json,math,struct
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
pins=[]
def pin(path,*lines):
 b=(G/path).read_bytes()
 for line in lines:assert line in b.decode(),(path,line)
 pins.append({'private_path':'private/ghidra/'+path,'sha256':sha(b)})
pin('command-resolver/instructions-00080ae0.txt','00080b4e CALL 0x00073e00','00080b62 CALL 0x00074cc0','00080b69 JZ 0x00080b45','00080c35 MOV dword ptr [ESI + 0x7],EAX','00080c60 CALL 0x0008ac60','00080c6e MOV byte ptr [EAX + 0x1b],DL','00080c77 OR byte ptr [EAX + 0x20],0x2','00080c81 INC dword ptr [EAX + 0x1c]','00080cd2 TEST byte ptr [ESI + 0x71],0x4','00080cdc AND EDX,0xfbfe','00080cef MOV byte ptr [ESI + 0x16],BL','00080cf2 OR DH,0x1','00080d21 INC dword ptr [EDX + 0x49d]')
pin('actor-activation/instructions-00073e00.txt','00073e41 IMUL EAX,EDX,-0x44bf19d3','00073e5a MOV [0x0011878a],AX','00073e71 LEA EAX,[EDX*0x8 + 0x0]','00073ed1 IMUL EDX,EDI','00073ed4 SAR EDX,0x10','00073f28 SAR DX,CL','00073f2b SAR BX,CL','00073f40 MOV EAX,0x7fff')
pin('actor-activation/instructions-00074cc0.txt','00074d21 CALL 0x0008b520','00074d34 TEST EDX,EAX','00074d6d CALL 0x0008b130','00074d72 CMP EAX,0xa')
pin('actor-activation/instructions-0008ac60.txt','0008ac71 MOV byte ptr [EDX + 0x2],0x0','0008ac79 MOV word ptr [EBX],AX','0008aca7 MOV byte ptr [ESI + 0x2],0x0','0008acaf MOV word ptr [ECX + 0x6],AX')
pin('camera-player-input/instructions-00080830.txt','000808dc OR byte ptr [EDX + 0x71],0x4','0008092a CALL 0x00080ae0')
pin('plant-command/instructions-0004b630.txt','0004b66f CALL dword ptr [EDX + 0x94]','0004d0c6 CALL 0x0007ac00','0004d111 CMP EAX,dword ptr [EDX + 0x27]','0004d114 JL 0x0004d196','0004d16b CALL 0x00080ae0','0004d340 CALL 0x00048790')
pin('actor-activation/instructions-0006df30.txt','0006dfec MOVSX EAX,word ptr [EDI + 0x31]','0006dff3 SAR EAX,0x3','0006dff6 ADD ESI,EAX','0006e072 CMP ESI,EAX','0006e538 MOV dword ptr [EDI + 0x27],ESI')
pin('entity-shadow-panel/instructions-0007f3b0.txt','0007f428 LEA EBX,[EAX + 0x6400]','0007f431 MOV EAX,0x31','0007f436 CALL 0x00048790')
pin('campaign-level/instructions-00071b70.txt','00072340 MOV EBX,0x34','00072345 MOV EDX,0x118770')
memory=(G/'validated/memory-.object3.bin').read_bytes();code=(G/'validated/memory-.object1.bin').read_bytes()
sin=list(struct.unpack_from('<2048i',memory,0x7ee8));cos=list(struct.unpack_from('<2048i',memory,0x86e8))
assert sin==[int(65536*math.sin(i*2*math.pi/2048)) for i in range(2048)]
assert cos==[int(65536*math.cos(i*2*math.pi/2048)) for i in range(2048)]
tables=[{'va':hex(a),'length':8192,'sha256':sha(memory[a-0x100000:a-0x100000+8192]),'identity':name} for a,name in [(0x107ee8,'trunc65536sin'),(0x1086e8,'trunc65536cos')]]
assert memory[0x4eb4+0x31*6:0x4eb4+0x32*6]==bytes([5,0,1,255,0,1])
assert struct.unpack_from('<I',memory,0x12254+0x94)[0]==0x6df30
assert struct.unpack_from('<I',code,0x4b524-0x10000+(0x31-6)*4)[0]==0x4d089
word=lambda v:((v+32768)&65535)-32768
def placement(seed,center):
 assert 0<=seed<=65535 and len(center)==2 and all(0<=v<=32767 for v in center)
 next_seed=(seed*0xe62d)&65535;angle=(next_seed>>8)*8;index=(angle-512)&2047
 # Native words are wrapped before their arithmetic right shift and mask.
 dx=(1024*cos[index])>>16;dy=(1024*sin[index])>>16
 result=[(word((center[0]<<1)+dx)>>1)&32767,(word((center[1]<<1)+dy)>>1)&32767]
 return {'seed_after':next_seed,'angle':angle,'position_u15_pair':result}
def activation(active,flags70,flags16,category,resource18,resource1b,resource20,resource_count,actor72,player49d,accepted):
 assert all(0<=v<=255 for v in (active,flags16,category,resource18,resource1b,resource20,actor72))
 assert 0<=flags70<=65535 and 0<=resource_count<=0xffffffff and 0<=player49d<=0xffffffff
 if not active or not flags70&0x400:return {'eligible':False,'actor_writes':{},'resource_writes':{}}
 assert accepted is not None,'safe plan requires a successful finite placement; original retries indefinitely'
 count=(resource_count+1)&0xffffffff
 writes={'flags70_u16':(flags70&0xfbfe)|1,'flags16_u8':flags16&0xf7,'position7_i16_pair':list(accepted)}
 if count==1 and category==1:writes['byte72_u8']=40
 return {'eligible':True,'actor_writes':writes,'resource_writes':{'byte1b_u8':resource18,'flags20_u8':resource20|2,'count1c_u32':count},'requests':['insert-cell-head-preserving-raw-reference-indices'],'preserved':['height11_i32','shade17_u8','animation19_u8','frame1b_u8','state55_i8','pending-move-fields']}
vectors=[]
for seed,center in itertools.product([0,1,5428,32768,65535],[(20736,13568),(0,0),(32767,32767)]):vectors.append({'seed_before':seed,'center':center,'result':placement(seed,center)})
for seed in range(65536):
 r=placement(seed,(20736,13568));assert all(0<=v<=32767 for v in r['position_u15_pair'])
 assert r['seed_after']==((seed*58925)%65536)
flagcases=0
for flags70,active in itertools.product(range(65536),(0,3)):
 r=activation(active,flags70,255,0,7,9,0,0,0,0,(100,200))
 assert r['eligible']==bool(active and flags70&1024)
 if r['eligible']:assert r['actor_writes']['flags70_u16']==(flags70&0xfbfe)|1
 flagcases+=1
source=(G/'campaign-resources/152.dat').read_bytes();assert sha(source)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
seed_offset=0x52466+26;assert struct.unpack_from('<H',source,seed_offset)[0]==5428
private=G/'actor-activation/placement-fixture.json';private.write_text(json.dumps(vectors,indent=2)+'\n',encoding='utf-8')
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,'tables':tables,
 'rng':'73E00 updates shared11878A u16 seed=(seed*E62D)&FFFF, uses highbyte*8 as angle. One invocation consumes one draw and logs one RNG event. Explicit live seed required; source seed is not necessarily landing seed.',
 'placement':'Radius fixed512. Double center low16words; dx=floor(1024*cos[(angle-512)&2047]/65536),dy=floor(1024*sin[index]/65536); add into words, arithmetic shift signedword by1, mask7FFF. Result0..32767. Full2048mappedtables match independently generated trunc65536sin/cos, not rounded values.',
 'placement_acceptance':'Type3 actor avoids type1 special shortcut.74CC0 accepts iff ((1<<8B520class)&actor53)==0 and8B130blockerweight<10. It does not call74270 building obstacle lookup. The original repeatedly draws until accepted, without attempt cap. Safe bounded API rejects exhausted supplied candidates atomically; do not substitute center.',
 'activation_loop':'For the player+1 partition of five200-byte slots in ascending order, activebyte6!=0 and flags71mask04 qualify. Before each retry: word70=(word70&FBFE)|0001 and byte16&=F7. Only no-associated-unit branch admitted: player+496kind!=5, so placement centered on player+9. Associated type5 route invokes move resolver and is excluded. After accepted point copy to actor+7, insert8AC60; resourceC0+1B=resource+18,+20|02,+1CINC u32; ifnewcount1andcategory+16==1 setactor+72=40. Preserveheight11,shade17,animation/frame/state55. After allfive slots INCplayer49D u32 even ifnone eligible.',
 'link_byte_policy':'Actor.prev.kind=0,prev.index=cellID. Ifoldheadresolvesnonnull: oldhead.prev={actorID,actor.type}; actor.next={oldhead.ID,oldhead.type}. Else clearonlyactor.next.kind, preservingnext.index. Newhead={actorID,actor.type}. No canonical null rewrite.',
 'landing_route':'7F3B0 creates objectcode0x31 at target with physicalheight27=terrainheight+25600 and orientation0,1024,speed1088 -> verticalvelocity-1088.112254+94 resolves6DF30 physics before4B630 objectdispatch; code0x31 dispatches4D089. Phase0 tests terrainheight>=physicalheight27 afterphysics; then snapsheight27 toterrain, clearsvelocity2D/2F/31, incrementsphase46, timer4A=150, calls80AE0(ownerplayer), and emits effects/audio. Nonlandingphase0 consumes RNG and creates effect0x06 everyupdate. Thus applying80AE0 atstartup bypasses real descent and prior shared RNG consumption.',
 'source_seed':{'entry_id':'root/2/disc/152','source_sha256':sha(source),'file_offset':seed_offset,'initial_loaded_seed_u16':5428,'binding':'52-byte tail copied to118770; seed+26'},
 'placement_vectors':vectors,'activation_vectors':[activation(3,1024,128,0,7,9,0,0,0,0,(100,200)),activation(3,1024,136,1,7,9,0,0,0,0,(100,200)),activation(3,1024,0,1,7,9,2,0xffffffff,55,0,(100,200)),activation(3,0,0,0,0,0,0,0,0,0,None)],
 'validated_seed_cases':65536,'validated_flag_cases':flagcases,
 'limits':['No whole initial tick/RNG schedule, descent frame count or first controllable-frame claim.','Explicit accepted candidates/terrain and blocker results, live RNG seed, resource metadata and player counters are required. No invented native pointers.','No associated type5 unit route; no resource/audio effects executed by pure plans.','Activation alone does not perform portrait selection/controller gates or immediate actor motion.']}
(P/'research/ghidra/actor-activation-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print(f'PASS9pins;sin/cos4096entries;65536seeds;{flagcases}activationflags;19vectors')

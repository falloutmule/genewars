"""Private bounded child20 damage/reaction investigation; never executes game."""
from pathlib import Path
import importlib.util,struct,json,uuid,hashlib
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
 def get(self,o):
  if self.pc==0x2e1000:self.audio.append(dict(channel=self.r['EDX'],mode=self.r['EBX'],parameter=self.r['ECX'],stack=[self.load(self.r['ESP']+i*4,4)for i in range(2)]))
  return super().get(o)
model=Source();extra=[P/'private/ghidra'/x for x in ['construction-command/instructions-0005b9f0.txt','construction-command/instructions-0005c7e0.txt','construction-command/instructions-0001ee50.txt','activation-rng-refs/instructions-0005c920.txt','mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt','construction-command/instructions-0004f6a0.txt','activation-rng-refs/instructions-0005cdb0.txt','combat-impact/instructions-0004b070.txt']];model.files+=extra;model.code.update(m.m.Model(extra,b'').code)
obj=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes()
for pc,(op,args) in list(model.code.items()):
 if op=='MOV' and args==['[0x0011878a]','AX']:
  assert obj[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100');model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
model.code[0x2e1000]=('RET',['0x8']);keys=sorted(model.code);model.next=dict(zip(keys,keys[1:]))
mission=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes();packed=mission[0x3b55e:0x3b55e+116];base=bytearray(packed[:31]+bytes(8)+packed[31:]);struct.pack_into('<I',base,35,0x111fec)
base[94]=8;base[96]=base[97]=0;base[87]=base[93]=0;base[118]=30;base[122]=4;struct.pack_into('<hh',base,104,0,100)
effect=bytearray(81);effect[6]=5;effect[69]=20;effect[73]=4;struct.pack_into('<I',effect,35,0x112254)
state={0x1688ae+i:b for i,b in enumerate(effect)};state.update({0x111fec+0x44+i:b for i,b in enumerate(struct.pack('<I',0x2e1000))});state.update({0x118782+i:0 for i in range(4)});state[0x11872c]=0
model.audio=[]
resource=P/'private/integration/game-runtime/effects/EFF.ANB';db=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',db,10)
for i in range(struct.unpack_from('<H',db)[0]):
 for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x610000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
state.update({0x610000+i:b for i,b in enumerate(db)})
for a,data in [(0x116818,struct.pack('<I',0x610000+animations)),(0x220000,bytes(6)),(0x300004,struct.pack('<4I',0,0,0,0x1645ae)),(0x11c2b4,bytes(2)),(0x112254+0x44,struct.pack('<I',0x2e1000))]:state.update({a+i:b for i,b in enumerate(data)})
for slot in range(200):state.update({0x1688ae+slot*81+i:0 for i in range(81)})
state.update({0x1688ae+35+i:b for i,b in enumerate(struct.pack('<I',0x112254))})
for cell in range(16384):state.update({0x11c2be+cell*14+6+i:0 for i in range(3)})
at=struct.unpack_from('<hh',base,7);cell=((at[1]>>8)&127)*128+((at[0]>>8)&127);state[0x11c2be+cell*14+8]=1
base[:6]=bytes(6)
shooter=bytearray(67);shooter[6]=4;struct.pack_into('<I',shooter,35,0x112120);state.update({0x1645ae+i:b for i,b in enumerate(shooter)})
w=P/'private/review/turret-projectile'/uuid.uuid4().hex;w.mkdir();inputs=bytearray();expected=bytearray();cases=[]
for case in range(64):
 before=bytearray(base);struct.pack_into('<h',before,106,[1,2,100,32767][case%4]);before[118]=[0,30,90,100,255][case%5];before[21]=[0,32,255][case%3];before[22]=[0,255][case%2];struct.pack_into('<H',before,73,[0,65535][case%2]);before[84]=case;struct.pack_into('<h',before,63,case*31);before[96]=[0,0x18][case%2];before[97]=[0,0x90][case%2];struct.pack_into('<hh',before,51,case,-case);before[85:88]=bytes([7,0,5]);before[91:94]=bytes([8,0,5]);seed=[0,5428,65535,37][case//4%4];state[0x11872c]=case%32
 slot=case%8;owner=case//8%5
 for i in range(200):state[0x1688ae+i*81+6]=5 if i<slot else 0
 for address,data in [(0x1688ae+slot*81+35,struct.pack('<I',0x112254)),(0x11c2b4,struct.pack('<h',slot)),(0x300004,struct.pack('<4I',0,owner,0,0x1645ae))]:state.update({address+i:b for i,b in enumerate(data)})
 players=bytearray(4*1805)
 for player in range(4):struct.pack_into('<I',players,player*1805+0x6fd,0xffffffff-case)
 state.update({0x118d74+i:b for i,b in enumerate(players)})
 model.audio=[];r,_,_=model.run(entry=0x48790,seed=seed,native_before=before,state_before=state,registers=dict(EAX=20,EDX=int.from_bytes(before[7:11],'little'),EBX=struct.unpack_from('<i',before,39)[0],ECX=0x220000))
 after=bytearray(model.load(0x1542be+i,1)for i in range(124));after[31:35]=before[31:35]
 assert r['uninitialized_stack_reads']==0,r
 assert struct.unpack_from('<h',after,106)[0]==struct.unpack_from('<h',before,106)[0]-1
 assert after[94]==18 and struct.unpack_from('<h',after,108)[0]==64
 assert model.audio[-2:]==[dict(channel=57,mode=2,parameter=0,stack=[127,100])]*2
 kills=(0xffffffff-case+int(struct.unpack_from('<h',before,106)[0]==1))&0xffffffff
 if owner<4:struct.pack_into('<I',players,owner*1805+0x6fd,kills)
 assert bytes(model.load(0x118d74+i,1)for i in range(7220))==players
 inputs+=before+struct.pack('<HBIHB',seed,case%32,0xffffffff-case,slot,owner)
 expected+=after+struct.pack('<HBI',r['rng_seed_after'],r['rng_ring_after'],r['rng_draw_count_after'])+bytes([len(model.audio)])+bytes([a['channel']for a in model.audio])+bytes(4-len(model.audio))+players
 cases.append(dict(case=case,steps=r['steps'],draws=r['draws'],audio=[a['channel']for a in model.audio]))
(w/'damage-input.bin').write_bytes(inputs);(w/'damage-expected.bin').write_bytes(expected)
receipt=dict(status='literal-child20-factory-neutral-mule-damage',cases=cases,private_run=w.relative_to(P).as_posix(),pins={p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files+[Path(__file__),resource]},scope='Full factory20 with randomized spatial query, real damage5B9F0, lethal5CDB0, and reaction5C7E0. Audio is a typed port. Unlinked neutral idle Mule, health1/2/100/32767; effect slots0..7 and owners0..4; full124 recipient bytes and7220 player bytes plus RNG/audio. Retained animation address normalized. Source invokes reaction even after lethal damage: final health0/state18 is preserved, not corrected by assumption. Not full projectile/world integration or subsequent mortality-update proof.')
(w/'damage-source.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(private_run=receipt['private_run'],cases=len(cases),instructions=sum(x['steps']for x in cases))))

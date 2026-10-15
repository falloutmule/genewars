"""Bounded literal newborn callbacks on source-compared terminal fixture."""
from pathlib import Path
import importlib.util,struct,json,pickle,hashlib,uuid
P=Path(__file__).resolve().parents[2];fixture=P/'private/review/genepod-terminal-world-002/success-after';sp=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m);vm=m.Source();G=P/'private/ghidra'
files=[G/x for x in ['terrain-panel-refs/instructions-0005df50.txt','mission-array-refs/instructions-00081c60.txt','activation-rng-refs/instructions-0005c920.txt','construction-command/instructions-0005b9f0.txt','terrain-map-refs/instructions-00074a50.txt','terrain-map-refs/instructions-000305f0.txt','initial-creature/instructions-0006d230.txt','owned-mule-collision-59280.txt','terrain-map-refs/instructions-00084b40.txt','camera-player-input/instructions-0008b800.txt','terrain-map-refs/instructions-000847d0.txt','terrain-map-refs/instructions-0007af00.txt','entity-plant/instructions-0001f240.txt','terrain-map-refs/instructions-0007b240.txt','entity-plant/instructions-0001f280.txt','owned-mule-ai-6bfd0.txt','mission-array-refs/instructions-00043130.txt']];vm.code.update(m.m.Model(files,b'').code);vm.code[0x41910]=('RET',['0x10']);vm.code[0x5e068]=('JMP',['0x5e0a3']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));state={}
def put(a,n,v):state.update({a+i:b for i,b in enumerate((v%(1<<(8*n))).to_bytes(n,'little'))})
for name,address in {'backing.bin':0x118d60,'map.bin':0x11c2be,'players.bin':0x118d74,'buildings.bin':0x1645ae,'creatures.bin':0x1542be}.items():state.update({address+i:b for i,b in enumerate((fixture/name).read_bytes())})
# Reconstitute native pool headers from packed backing for linked shade visits.
backing=(fixture/'backing.bin').read_bytes()
layouts=[(277838,512,41,0x15d40e,49,0x1120ac),(298830,20,140,0x16360e,200,0x112194),(316734,200,73,0x1688ae,81,0x112254)]
for start,n,stride,address,width,vt in layouts:
 for slot in range(n):
  raw=backing[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:]);struct.pack_into('<I',native,35,vt);state.update({address+slot*width+i:b for i,b in enumerate(native)})
for i in range(256):put(0x1645ae+i*67+35,4,0x112120)
anbp=P/'private/integration/game-runtime/creatures/MULE.ANB';anb=bytearray(anbp.read_bytes());count,table,components,frames,animations=struct.unpack_from('<H4I',anb)
for i in range(count):
 for off in [0,4]:struct.pack_into('<I',anb,animations+i*10+off,0x400000+frames+struct.unpack_from('<I',anb,animations+i*10+off)[0])
state.update({0x400000+i:b for i,b in enumerate(anb)})
for i in range(300):put(0x1542be+i*124+31,4,0x400000+animations);put(0x1542be+i*124+35,4,0x111fec)
seed,ring,draws=struct.unpack('<HBI',(fixture/'rng.bin').read_bytes());put(0x11878a,2,seed);put(0x11872c,1,ring);put(0x118782,4,draws)
w=P/'private/review/owned-mule-terminal-callbacks'/uuid.uuid4().hex;w.mkdir(parents=True);rows=[];blocked=None;landing=None;command=None;firstmove=None;ptr=0x1542be+124
class CaptureCode(dict):
 def __getitem__(self,key):
  if key in [0x63ecc,0x649f4,0x5aa00,0x5aab3,0x649f9,0x64cd7,0x64cef,0x64def]:
   (w/f'{frame}-at-{key:x}.pickle').write_bytes(pickle.dumps({'memory':vm.mem.copy(),'registers':vm.r.copy()}))
  return super().__getitem__(key)
vm.code=CaptureCode(vm.code)
for frame in range(1,65):
 before=state.copy();rec=bytes(state.get(ptr+i,0)for i in range(124));seed=sum(state.get(0x11878a+i,0)<<(8*i)for i in range(2));(w/f'{frame}-before.pickle').write_bytes(pickle.dumps(before))
 try:r,_,trace=vm.run(entry=0x5ed20,counter=frame,state_before=state,seed=seed,registers={'EAX':ptr})
 except Exception as e:blocked={'frame':frame,'pc':hex(vm.pc),'reason':repr(e)};(w/f'{frame}-blocked.pickle').write_bytes(pickle.dumps(vm.mem));break
 state=vm.saved_state.copy();after=bytes(vm.load(ptr+i,1)for i in range(124));word=lambda o:int.from_bytes(after[o:o+2],'little',signed=True);(w/f'{frame}-after.pickle').write_bytes(pickle.dumps(state));(w/f'{frame}-trace.json').write_text(json.dumps([hex(x)for x in trace]));row={'frame':frame,'position':[word(7),word(9)],'ground':word(17),'height':word(39),'state':after[94],'flags15':after[21],'flags49':word(73),'flags60':word(96),'timer':word(108),'rng':[vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4)],'instructions':r['steps'],'calls':r['calls']};rows.append(row)
 if command is not None and rec[7:11]!=after[7:11]:firstmove=frame;break
 if landing is None and word(39)<=word(17):
  landing=frame;target=[word(7)+256,word(9)];put(0x440000,2,1);put(0x440002,2,target[0]);put(0x440004,2,target[1])
  try:cr,_,ct=vm.run(entry=0x81c60,counter=frame,state_before=state,seed=row['rng'][0],registers={'EAX':0x118d74,'EDX':0x440000})
  except Exception as e:blocked={'frame':frame,'phase':'command','reason':repr(e)};break
  state=vm.saved_state.copy();command={'after_frame':frame,'target':target,'instructions':cr['steps']};(w/'command-after.pickle').write_bytes(pickle.dumps(state))
result={'status':'passed'if firstmove else 'blocked','scope':'Source-compared synthetic completed Genepod terminal output on CONQ001 terrain; actual ejection, only newborn callback advanced; not authentic Krell initialization or native acceptance. Audio/selected-panel ports inherited.','stop_reason':'movement_after_command'if firstmove else ('new_source_boundary'if blocked else '64_callback_cap'),'private_run':w.relative_to(P).as_posix(),'landing':landing,'command':command,'first_move':firstmove,'blocked':blocked,'callbacks':rows,'pins':{p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in list(fixture.glob('*.bin'))+[anbp,Path(__file__)]+vm.files+files}}
(P/'research/campaign/krell-owned-mule-terminal-callbacks.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['pins','callbacks']}))

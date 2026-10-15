"""Full literal451F0 terminal success with real5A210 vs owned world capture."""
from pathlib import Path
import hashlib,importlib.util,json,struct,uuid,sys
P=Path(__file__).resolve().parents[2];root=P/(sys.argv[1]if len(sys.argv)>1 else 'private/review/genepod-terminal-world-002');mode=sys.argv[2]if len(sys.argv)>2 else 'success';before=root/(mode+'-before');after=root/(mode+'-after')
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
G=P/'private/ghidra';files=[G/'building-update/instructions-000451f0.txt',G/'activation-rng-refs/instructions-0005a210.txt',G/'genepod-mule-initializer.txt',G/'construction-command/instructions-0001ee50.txt',G/'bones-frame/instructions-0005a1c0.txt',G/'player-creation/instructions-0001f480.txt',G/'terrain-map-refs/instructions-0007b240.txt',G/'activation-rng-refs/instructions-00048790.txt',G/'player-creation/instructions-0001f070.txt',G/'mission-tail-init/instructions-00049e90.txt']
vm=m.Source();vm.code.update(m.m.Model(files,b'').code);vm.code[0x47f34]=('CALL',['0x2e0010']);vm.code[0x2e0010]=('RET',[]);vm.code[0x98210]=('RET',['0x8']);vm.code[0x1e810]=('RET',['0x8']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));state={}
def put(a,n,v):state.update({a+i:b for i,b in enumerate((v%(1<<(8*n))).to_bytes(n,'little'))})
layouts={'backing.bin':0x118d60,'map.bin':0x11c2be,'players.bin':0x118d74,'buildings.bin':0x1645ae,'creatures.bin':0x1542be}
if (before/'effects.bin').exists():layouts['effects.bin']=0x1688ae
for name,address in layouts.items():state.update({address+i:b for i,b in enumerate((before/name).read_bytes())})
anb_path=P/'private/integration/game-runtime/buildings/BGENE.ANB';anb=bytearray(anb_path.read_bytes());frames,animations=struct.unpack_from('<II',anb,10)
for i in range(5):
 for off in [0,4]:struct.pack_into('<I',anb,animations+i*10+off,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+off)[0])
state.update({0x620000+i:b for i,b in enumerate(anb)})
for i in range(256):put(0x1645ae+i*67+31,4,0x620000+animations);put(0x1645ae+i*67+35,4,0x112120)
for i in range(300):put(0x1542be+i*124+35,4,0x111fec)

if 'effects.bin'in layouts:
 for i in range(200):put(0x1688ae+i*81+35,4,0x112254)
put(0x180e84+16,4,0x400000);put(0x300004,4,0)
rng=(before/'rng.bin').read_bytes();seed,ring,draws=struct.unpack('<HBI',rng);put(0x11872c,1,ring);put(0x118782,4,draws)
r,_,trace=vm.run(counter=0,seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae},instruction_budget=30000)
w=P/'private/review/genepod-terminal-literal'/uuid.uuid4().hex;w.mkdir(parents=True);differences={};count=0
for name,address in layouts.items():
 if name=='backing.bin':continue
 expected=(after/name).read_bytes();actual=bytearray(vm.load(address+i,1)for i in range(len(expected)))
 stride=67 if name=='buildings.bin' else 124 if name=='creatures.bin' else 81 if name=='effects.bin' else 0
 if stride:
  for i in range(len(expected)//stride):actual[i*stride+31:i*stride+39]=expected[i*stride+31:i*stride+39]
 delta=[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b];differences[name]=delta[:20];count+=len(actual);(w/name).write_bytes(actual)
actual_rng=struct.pack('<HBI',vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4));differences['rng.bin']=[]if actual_rng==(after/'rng.bin').read_bytes()else[actual_rng.hex(),(after/'rng.bin').read_bytes().hex()];count+=7
receipt=dict(status='passed'if not any(differences.values())else'error',compared_bytes=count,instructions=r['steps'],draws=r['draws'],differences=differences,private_run=str(w.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files+[Path(__file__),P/'rust/crates/genewars-runtime/src/actor_world/genepod_production.rs']+list(before.glob('*.bin'))+list(after.glob('*.bin'))},scope='Full451F0 terminal '+mode+', real5A210/ejection or48790 effect20 factories, all buildings/creatures/effects/map/players/RNG; opaque31:39 normalization. Audio1E810/98210 and renderqueue explicit ports. Fixture synthetic completed Genepod, not ordinary mission acceptance.')
(P/('research/variants/genepod-terminal-world-'+mode+'-review.json')).write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt));assert receipt['status']=='passed'

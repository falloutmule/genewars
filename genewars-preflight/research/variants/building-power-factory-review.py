"""Literal free-slot effect33 and signed-angle smoke factories, no original execution."""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,uuid,subprocess,sys,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
W=P/'private/review/building-power-factory'/uuid.uuid4().hex;W.mkdir(parents=True)
files=[P/'private/ghidra'/x for x in ['mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt']];model=m.Source();model.code.update(m.m.Model(files,b'').code)
obj=P/'private/ghidra/validated/memory-.object1.bin'
for pc,(op,args)in list(model.code.items()):
 if op=='MOV' and args==['[0x0011878a]','AX']:
  assert obj.read_bytes()[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100');model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
keys=sorted(model.code);model.next=dict(zip(keys,keys[1:]));resource=P/'private/integration/game-runtime/effects/EFF.ANB';db=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',db,10)
for i in range(struct.unpack_from('<H',db)[0]):
 for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x610000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
flat=struct.pack('<HhBBBBBhhB',8192,3136,0,7,0,0,0,0,0,0);state0={0x610000+i:b for i,b in enumerate(db)};state0.update({0x11c2be+c*14+i:b for c in range(16384)for i,b in enumerate(flat)});state0.update({0x116818+i:b for i,b in enumerate(struct.pack('<I',0x610000+animations))})
r=random.Random(33);inputs=bytearray();expected=bytearray();stats=[]
for case in range(64):
 code=33 if case<32 else 6;slot=case%8;point=[20864,13696];angles=[case*61%2049,512,2048]if code==33 else [154,[-52,-1536,-1,0,511,512,2048,2559][case%8],1600];prior=bytearray(r.randbytes(81));prior[6]=0;native=bytearray(prior);struct.pack_into('<I',native,35,0x112254);seed=[65535,5428,0][case%3]
 state=state0.copy();state.update({0x1688ae+i*81+6:5 if i<slot else 0 for i in range(200)});state.update({0x1688ae+slot*81+i:b for i,b in enumerate(native)})
 for address,data in [(0x220000,struct.pack('<3h',*angles)),(0x11c2b4,struct.pack('<h',slot)),(0x300004,bytes(16)),(0x118782,struct.pack('<I',case))]:state.update({address+i:b for i,b in enumerate(data)})
 state[0x11872c]=case%32
 report,_,trace=model.run(entry=0x48790,seed=seed,state_before=state,registers=dict(EAX=code,EDX=point[0]|point[1]<<16,EBX=6784,ECX=0x220000));after=bytearray(model.load(0x1688ae+slot*81+i,1)for i in range(81));after[31:35]=bytes(4);after[35:39]=prior[35:39]
 if code==33:after[71:74]=bytes([7,0,4]);after[76]=case
 rng=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));inputs+=prior+struct.pack('<BB3hHBI',code,slot,*angles,seed,case%32,case);expected+=after+rng;stats.append(dict(case=case,code=code,angles=angles,instructions=report['steps'],draws=report['draws'],shade=after[23]))
(W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr);return r.stdout
rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');pins={}
for c in ['assets','core','runtime']:
 t=W/c;shutil.copytree(P/f'rust/crates/genewars-{c}/src',t)
 for f in t.rglob('*.rs'):pins[f'{c}/{f.relative_to(t)}']=sha(f)
 if c=='runtime' and 'pub mod building_power;'not in(t/'lib.rs').read_text():(t/'lib.rs').write_text((t/'lib.rs').read_text()+'\npub mod building_power;\n')
 a=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-type','rlib','--crate-name',f'genewars_{c}',str(t/'lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{c}.rlib')]
 if c=='runtime':
  for dep in ['assets','core']:a+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(a)
run([rustc,'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(W/'probe.exe')]);run([str(W/'probe.exe'),str(W)])
a=(W/'actual.bin').read_bytes();assert a==expected,[(i,x,y)for i,(x,y)in enumerate(zip(a,expected))if x!=y][:12]
receipt=dict(status='passed',cases=stats,private_run=W.relative_to(P).as_posix(),production_pins=pins,source_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[obj,P/'private/ghidra/validated/memory-.object3.bin',resource,Path(__file__),Path(__file__).with_suffix('.rs')]},scope='Free-slot factory numeric81B/RNG; destination7/payload assigned by producer after actual factory; no world lifecycle claim')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=64,private_run=receipt['private_run'])))

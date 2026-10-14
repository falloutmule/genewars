"""Complete grade0 source451F0 plus actual48790 ordered transfers, no original execution."""
from pathlib import Path
import hashlib,importlib.util,json,struct,uuid,subprocess,sys,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
W=P/'private/review/building-power-producer'/uuid.uuid4().hex;W.mkdir(parents=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr);return r.stdout
rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');pins={}
for c in ['assets','core','runtime']:
 t=W/c;shutil.copytree(P/f'rust/crates/genewars-{c}/src',t)
 for f in t.rglob('*.rs'):pins[f'{c}/{f.relative_to(t)}']=sha(f)
 if c=='runtime':
  q=t/'actor_world.rs';original=q.read_text();registration='' if 'mod power_buildings;' in original else '\nmod power_buildings;pub use power_buildings::PowerBuildingReceipt;';q.write_text(original+registration+'\nmod power_review{'+(P/'research/variants/building-power-producer-world.rs').read_text()+'}\npub use power_review::review_power_producers;\n')
 a=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-type','rlib','--crate-name',f'genewars_{c}',str(t/'lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{c}.rlib')]
 if c=='runtime':
  for dep in ['assets','core']:a+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(a)
probe=W/'world.rs';probe.write_text('fn main(){genewars_runtime::actor_world::review_power_producers(std::path::Path::new(r"'+str(P)+'"),std::path::Path::new(r"'+str(W)+'"));}')
run([rustc,'--edition=2021',str(probe),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(W/'world.exe')]);print(run([str(W/'world.exe')]))
files=[P/'private/ghidra'/x for x in ['building-update/instructions-000451f0.txt','mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt','landing-contact/instructions-0007ac00.txt']];model=m.Source();model.code.update(m.m.Model(files,b'').code)
obj=P/'private/ghidra/validated/memory-.object1.bin'
for pc,(op,args)in list(model.code.items()):
 if op=='MOV' and args==['[0x0011878a]','AX']:
  assert obj.read_bytes()[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100');model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
for pc in [0x45f40,0x462b8]:model.code[pc]=('CALL',['0x2e0010'])
model.code[0x4619b]=('CALL',['0x2e0020']);model.code[0x2e0010]=('RET',['8']);model.code[0x2e0020]=('RET',[]);model.code[0x83140]=('RET',[])
keys=sorted(model.code);model.next=dict(zip(keys,keys[1:]));resources=[]
def resource(path,base):
 resources.append(path);db=bytearray(path.read_bytes());frames,animations=struct.unpack_from('<II',db,10)
 for i in range(struct.unpack_from('<H',db)[0]):
  for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,base+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
 return {base+i:v for i,v in enumerate(db)},base+animations
state0,eff=resource(P/'private/integration/game-runtime/effects/EFF.ANB',0x610000)
root=P/'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING';animations={}
for selector,name,base in [(3,'BPOWE',0x620000),(6,'BSOLA',0x640000)]:
 db,animations[selector]=resource(root/(name+'.ANB'),base);state0.update(db)
state0.update({0x116818+i:v for i,v in enumerate(struct.pack('<I',eff))});stats=[]
for case in range(32):
 d=W/str(case);slot,selector,frame,solar,created=map(int,(d/'parameters.txt').read_text().split());state=state0.copy()
 for kind,base in [('map',0x11c2be),('buildings',0x1645ae),('effects',0x1688ae),('players',0x118d74)]:state.update({base+i:v for i,v in enumerate((d/f'before-{kind}.bin').read_bytes())})
 for i in range(200):state.update({0x1688ae+i*81+35+j:v for j,v in enumerate(struct.pack('<I',0x112254))})
 state.update({0x1645ae+slot*67+31+i:v for i,v in enumerate(struct.pack('<I',animations[selector]))})
 state.update({0x11a9bf+i:v for i,v in enumerate(struct.pack('<h',solar))});seed,ring,draws=struct.unpack('<HBI',(d/'before-rng.bin').read_bytes());state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))});state[0x11c2b4]=0;state[0x11c2b5]=0
 report,_,trace=model.run(counter=frame,seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae+slot*67})
 for kind,base in [('map',0x11c2be),('buildings',0x1645ae),('effects',0x1688ae),('players',0x118d74)]:
  expected=(d/f'after-{kind}.bin').read_bytes();actual=bytearray(model.load(base+i,1)for i in range(len(expected)))
  if kind=='buildings':actual[slot*67+31:slot*67+35]=expected[slot*67+31:slot*67+35]
  if kind=='effects':
   for i in range(200):actual[i*81+31:i*81+39]=expected[i*81+31:i*81+39]
  assert actual==expected,(case,kind,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12])
 assert struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))==(d/'after-rng.bin').read_bytes()
 assert struct.pack('<H',model.load(0x11c2b4,2))==(d/'after-count.bin').read_bytes()
 assert trace.count(0x48790)==created
 stats.append(dict(case=case,selector=selector,created=created,instructions=report['steps'],draws=report['draws']))
# Exact48846..48896 replacement scan, including all33 priority1 and marked slots.
model.code[0x48896]=('RET',[])
for case,expected in enumerate([0,0,0x1688ae+81,0x1688ae+162]):
 state={0x1688ae+i:v for i,v in enumerate((W/f'replacement{case}.bin').read_bytes())};state[0x2fff6e]=48
 model.run(state_before=state,entry=0x48846,registers={'EBP':0x2fff00});assert model.r['EDI']==expected
receipt=dict(replacement_scan_cases=4,status='passed' ,private_run=W.relative_to(P).as_posix(),cases=stats,late_allocation_rollback=True,production_pins=pins,source_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+resources+[obj,P/'private/ghidra/validated/memory-.object3.bin',Path(__file__),P/'research/variants/building-power-producer-world.rs',P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py']},scope='Complete grade0 451F0 with actual ordered48790 allocation/map links and7AC00 resource animation; audio and83140 queue are explicit services; all map/building/effect/player/RNG/count bytes compared except native pointers')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=32,private_run=receipt['private_run'])))

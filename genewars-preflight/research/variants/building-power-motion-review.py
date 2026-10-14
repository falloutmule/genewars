"""Full4B630 effect33 motion/animation/delivery, explicit final retirement request."""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,uuid,subprocess,sys,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
W=P/'private/review/building-power-motion'/uuid.uuid4().hex;W.mkdir(parents=True)
files=[P/'private/ghidra'/x for x in ['plant-command/instructions-0004b630.txt','actor-activation/instructions-0004b090.txt','actor-activation/instructions-0001e600.txt','landing-contact/instructions-0001f030.txt','combat-impact/instructions-0001e620.txt','combat-impact/instructions-0004a04c.txt','construction-command/instructions-0004af40.txt']];model=m.Source();model.code.update(m.m.Model(files,b'').code);model.code[0x49d30]=('RET',[]);keys=sorted(model.code);model.next=dict(zip(keys,keys[1:]))
resource=P/'private/integration/game-runtime/effects/EFF.ANB';db=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',db,10);assert struct.unpack_from('<IIBB',db,animations+330)==(2646,2688,4,0);assert all(db[frames+2646+14*i+10]==0 for i in range(4))
for i in range(struct.unpack_from('<H',db)[0]):
 for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x610000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
flat=struct.pack('<HhBBBBBhhB',8192,3136,0,7,0,0,0,0,0,0);state0={0x610000+i:b for i,b in enumerate(db)};state0.update({0x11c2be+c*14+i:b for c in range(16384)for i,b in enumerate(flat)})
origin=P/json.loads(Path(__file__).with_name('building-power-factory-review.json').read_text())['private_run'];factory=(origin/'expected.bin').read_bytes();data=bytearray();expected=bytearray();stats=[]
for case in range(64):
 before=bytearray(factory[(case%32)*88:(case%32)*88+81]);before[15:17]=bytes(2);before[76]=[1,2,10,255][case%4];before[27]=case%7;before[30]=[0,1,127,128,255][case%5];before[71:74]=bytes([7,0,4]);native=bytearray(before);struct.pack_into('<II',native,31,0x610000+animations+330,0x112254)
 target=bytearray(67);target[6]=4;target[15]=7;target[45]=[3,11][case%2];struct.pack_into('<h',target,61,1998 if case%2==0 else 199);struct.pack_into('<hh',target,7,20864+[0,128,256,512][case%4],13696)
 state=state0.copy();state.update({0x1688ae+i:v for i,v in enumerate(native)});state.update({0x1645ae+7*67+i:v for i,v in enumerate(target)});state.update({0x11c2be+(53*128+81)*14+6+i:v for i,v in enumerate(bytes([0,0,5]))})
 report,_,trace=model.run(counter=case,seed=5428,state_before=state,entry=0x4b630,registers={'EAX':0x1688ae});after=bytearray(model.load(0x1688ae+i,1)for i in range(81));after[31:39]=before[31:39];ta=bytes(model.load(0x1645ae+7*67+i,1)for i in range(67));retire=int(0x49d30 in trace)
 # Numeric pure motion returns link requests; compare source link afterimages separately in the world fixture.
 after[:6]=before[:6];data+=before+target;expected+=after+ta+bytes([retire]);stats.append(dict(case=case,instructions=report['steps'],retire=bool(retire),draws=report['draws'],uninitialized=report['uninitialized_stack_reads']));assert report['draws']==0
(W/'input.bin').write_bytes(data);(W/'expected.bin').write_bytes(expected)
for n in ['angles.bin','bounce.bin']:shutil.copyfile(P/'private/review/family8-state4'/n,W/n)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr);return r.stdout
rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');pins={}
for c in ['assets','core','runtime']:
 t=W/c;shutil.copytree(P/f'rust/crates/genewars-{c}/src',t)
 for f in t.rglob('*.rs'):pins[f'{c}/{f.relative_to(t)}']=sha(f)
 if c=='runtime':
  q=t/'actor_world.rs';original=q.read_text();registration='' if 'mod power_updates;' in original else '\nmod power_updates;pub use power_updates::PowerStepReceipt;';q.write_text(original+registration+'\nmod power_review{'+(P/'research/variants/building-power-world-review.rs').read_text()+'}\npub use power_review::review_power_world;\n')
 a=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-type','rlib','--crate-name',f'genewars_{c}',str(t/'lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{c}.rlib')]
 if c=='runtime':
  for dep in ['assets','core']:a+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(a)
run([rustc,'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(W/'probe.exe')]);run([str(W/'probe.exe'),str(W)])
world_probe=W/'world.rs';world_probe.write_text('fn main(){genewars_runtime::actor_world::review_power_world(std::path::Path::new(r"'+str(P)+'"),std::path::Path::new(r"'+str(W)+'"));}')
run([rustc,'--edition=2021',str(world_probe),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(W/'world.exe')]);print(run([str(W/'world.exe')]))
# Actual49D30/724C0, five native pools and shared map, empty original UI policy.
retirement_files=[P/'private/ghidra/terrain-map-refs/instructions-00049d30.txt',P/'private/ghidra/terrain-panel-refs/instructions-000724c0.txt'];files+=retirement_files;model.code.update(m.m.Model(retirement_files,b'').code)
model.code[0x409f0]=('RET',[]);model.code[0x2c200]=('RET',[]);keys=sorted(model.code);model.next=dict(zip(keys,keys[1:]))
state=state0.copy();state.update({i:0 for i in range(0x1542be,0x16e096)});state.update({0x11c2be+i:v for i,v in enumerate((W/'world-before-map.bin').read_bytes())});e=bytearray((W/'world-before-effect.bin').read_bytes());struct.pack_into('<II',e,31,0x610000+animations+330,0x112254);state.update({0x1688ae+i:v for i,v in enumerate(e)});state.update({0x1645ae+7*67+i:v for i,v in enumerate((W/'world-before-building.bin').read_bytes())});state.update({0x185ec4+i:0 for i in range(4)});state[0x11c2b4]=1;state[0x11c2b5]=0
world_steps=[]
for frame in range(12):
 report,_,trace=model.run(counter=frame,seed=5428,state_before=state,entry=0x4b630,registers={'EAX':0x1688ae});state.update(model.mem);world_steps.append(report['steps'])
 assert (0x49d30 in trace)==(frame==11)
source_effect=bytearray(model.load(0x1688ae+i,1)for i in range(81));source_effect[31:39]=(W/'world-effect.bin').read_bytes()[31:39]
assert bytes(source_effect)==(W/'world-effect.bin').read_bytes()
assert bytes(model.load(0x11c2be+i,1)for i in range(229376))==(W/'world-map.bin').read_bytes()
assert bytes(model.load(0x1645ae+7*67+i,1)for i in range(67))==(W/'world-building.bin').read_bytes();assert model.load(0x11c2b4,2)==0
(W/'world-source.json').write_text(json.dumps(dict(status='passed',callbacks=12,literal_instructions=sum(world_steps),map_bytes=229376,energy_received=39,ui_ports=['409F0','2C200']),indent=2))
a=(W/'actual.bin').read_bytes();assert a==expected,[(i,x,y)for i,(x,y)in enumerate(zip(a,expected))if x!=y][:16]
receipt=dict(status='passed',cases=stats,private_run=W.relative_to(P).as_posix(),production_pins=pins,world=dict(callbacks=12,literal_instructions=sum(world_steps),energy_received=39,map_bytes=229376,unsupported_rollback=True),source_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[resource,Path(__file__),Path(__file__).with_suffix('.rs'),P/'research/variants/building-power-world-review.rs',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py']},scope='64 numeric4B630 callbacks with retirement request port; separate12-callback world executes49D30/724C0 and compares229376 mapbytes/effect/target/count; empty native UI service ports explicit')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=64,private_run=receipt['private_run'])))

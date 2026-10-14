"""Frozen owned Extractor source comparison and rollback fixtures."""
from pathlib import Path
import hashlib,importlib.util,json,os,shutil,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
work=P/'private/review/bootstrap-visibility'/uuid.uuid4().hex;work.mkdir(parents=True);pins={}
for crate in ['assets','core','runtime']:
 origin=P/f'rust/crates/genewars-{crate}/src';target=work/f'source/{crate}';shutil.copytree(origin,target)
 for f in target.rglob('*.rs'):pins[(origin/f.relative_to(target)).relative_to(P).as_posix()]=hashlib.sha256(f.read_bytes()).hexdigest()
f=work/'source/runtime/actor_world/bootstrap_updates.rs';f.write_text(f.read_text()+(P/'scripts/ghidra/mission_contract_bootstrap_visibility_fixture.rs').read_text())
f=work/'source/runtime/actor_world.rs';s=f.read_text();
if 'mod bootstrap_updates;' not in s:s+='\nmod bootstrap_updates;\npub use bootstrap_updates::{BootstrapWorldError,BootstrapVisibilityReceipt};\n'
f.write_text(s)
f=work/'source/runtime/actor_world/terrain_refresh.rs';f.write_text(f.read_text().replace('    fn replace_terrain_entity(', '    pub(super) fn replace_terrain_entity('))
env=environment();env['CARGO_MANIFEST_DIR']=str(P/'rust/crates/genewars-runtime');env['GENEWARS_REVIEW_ROOT']=str(P);env['GENEWARS_REVIEW_OUT']=str(work);rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
def run(cmd):
 r=subprocess.run([str(v)for v in cmd],cwd=P,env=env,capture_output=True,timeout=240)
 assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'))
 return r.stdout.decode(errors='replace')
for crate in ['assets','core','runtime']:
 cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}']
 if crate=='runtime':
  cmd+=['--test','-o',work/'review.exe']
  for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
 else:cmd+=['--crate-type','rlib','-o',work/f'libgenewars_{crate}.rlib']
 run(cmd)
print(run([work/'review.exe','bootstrap_visibility_review','--nocapture']),flush=True)
print('PRIVATE_RUN',work,flush=True)
# Compare the complete bounded source72AD0 callback on each owned snapshot.
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Source(m.Source):
 def get(self,o):
  if o=='0x00084b40':self.vertices.append(self.r['EDX'])
  return super().get(o)
model=Source();files=[P/'private/ghidra'/n for n in ['camera-player-input/instructions-00072ad0.txt','camera-player-input/instructions-00072960.txt','plant-startup/instructions-0008aae0.txt','camera-player-input/instructions-00075b00.txt','terrain-map-refs/instructions-00084b40.txt','camera-player-input/instructions-0008b800.txt','terrain-map-refs/instructions-0007b240.txt','entity-plant/instructions-0001f240.txt','entity-plant/instructions-0001f280.txt']]
model.code.update(m.m.Model(files,b'').code);model.code[0x847d0]=('MOV',['EAX','1']);model.code[0x847d1]=('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]));total=steps=events=shades=0
BACK=343958;at=BACK;ranges=[]
for count,stride,address,kind in [(20,200,0x16360e,3),(300,124,0x1542be,1),(256,67,0x1645ae,4),(200,81,0x1688ae,5),(16,102,0x16da36,8),(4,1805,0x118d74,0)]:
 ranges.append((at,count,stride,address,kind));at+=count*stride
for case in range(4):
 before=(work/f'case{case}-before.bin').read_bytes();actual=(work/f'case{case}-after.bin').read_bytes();state={0x118d60+i:b for i,b in enumerate(before[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])})
 # Seven source-native layouts, with handle-only words supplied as explicit vtable bindings.
 for start,count,stride,address,kind in ranges:
  state.update({address+i:b for i,b in enumerate(before[start:start+count*stride])})
  if kind:
   for slot in range(count):
    ptr=0x2e1000+kind*256;state.update({address+slot*stride+35+i:b for i,b in enumerate(struct.pack('<I',ptr))});state.update({ptr+0x6c+i:b for i,b in enumerate(struct.pack('<I',0x1f280 if kind==4 else 0x1f240))})
 # Plants and dormant unowned families use the exact serialized records.
 for slot in range(512):
  packed=before[0x43d4e+slot*41:0x43d4e+(slot+1)*41];native=packed[:31]+bytes(8)+packed[31:];state.update({0x15d40e+slot*49+i:b for i,b in enumerate(native)});state.update({0x15d40e+slot*49+35+i:b for i,b in enumerate(struct.pack('<I',0x1120ac))})
 # Family7 starts at native backing+540B6, serialized5212E, stride89.
 for slot in range(32):
  packed=before[0x5212e+slot*89:0x5212e+(slot+1)*89];native=packed[:31]+bytes(8)+packed[31:];state.update({0x16ce16+slot*97+i:b for i,b in enumerate(native)})
 seed,ring,draws=struct.unpack_from('<HBI',before,at);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))});model.vertices=[]
 report,_,trace=model.run(seed=seed,counter=0,state_before=state,entry=0x72ad0,registers={'EAX':0x118d60,'EDX':0});expected=bytearray(before);expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
 for start,count,stride,address,kind in ranges:
  for slot in range(count):
   body=bytearray(model.load(address+slot*stride+i,1)for i in range(stride))
   if kind:body[31:39]=before[start+slot*stride+31:start+slot*stride+39]
   expected[start+slot*stride:start+(slot+1)*stride]=body
 for slot in range(512):expected[0x43d4e+slot*41+23]=model.load(0x15d40e+slot*49+23,1)
 struct.pack_into('<HBI',expected,at,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));ordered=b''.join(struct.pack('<I',v)for v in model.vertices)
 (work/f'case{case}-source.bin').write_bytes(expected);assert actual==expected,(case,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]);assert (work/f'case{case}-events.bin').read_bytes()==ordered
 total+=len(actual)+len(ordered);steps+=report['steps'];events+=len(model.vertices);shades+=trace.count(0x7b240);print('SOURCE CASE',case,'PASS',flush=True)
r={'status':'passed-bounded-startup-visibility-world','cases':4,'compared_bytes':total,'literal_instructions':steps,'viewport_requests':events,'shade_samples':shades,'private_run':work.relative_to(P).as_posix(),'source_pins':{f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in files},'implementation_pins':pins,'scope':'ActualCONQ, with/without preparedactors+landing andpreexisting fullreveal. Exact72AD0 mapreset/reveal/72960entityshades; no worldtick, originalprocess or completebootstrap claim.','ports':['847D0 viewport-cache lookup return1; actual84B40dirty/shade tail runs','typed vtable+6C familyidentity bindings'],'remaining':['nonzero terrainobject initialization','special8AAE0effect','activefamily7shadeadapter','nativeprior andstackprovenance']}
(P/'research/campaign/bootstrap-visibility-world.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items()if k not in ['source_pins','implementation_pins']}))

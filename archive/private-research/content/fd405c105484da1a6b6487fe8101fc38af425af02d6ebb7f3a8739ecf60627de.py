"""Literal linked-cell refresh comparison; synthetic owned-world fixtures only."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,shutil,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
ACTORS=343958;CREATURES=ACTORS+4000;BUILDINGS=CREATURES+37200;EFFECTS=BUILDINGS+17152;FAMILY8=EFFECTS+16200
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
BASES={1:CREATURES,3:ACTORS,4:BUILDINGS,5:EFFECTS,8:FAMILY8}
VT={1:0x111fec,2:0x1120ac,3:0x112194,4:0x112120,5:0x112254,8:0x11242c}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(c):
 r=subprocess.run(c,cwd=P,env=environment(),capture_output=True,timeout=240)
 assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'))
 return r.stdout.decode(errors='replace')
def main():
 work=P/'private/review/terrain-refresh'/uuid.uuid4().hex;work.mkdir(parents=True);pins={}
 for crate in ['assets','core','runtime']:
  origin=P/f'rust/crates/genewars-{crate}/src';target=work/f'source/{crate}';shutil.copytree(origin,target)
  for f in target.rglob('*.rs'):pins[(origin/f.relative_to(target)).relative_to(P).as_posix()]=sha(f)
 # Private fixture access only. Production has no raw-record setters or wrapper.
 runtime=work/'source/runtime'
 for file,typ in [('actor_pool','ActorPool'),('creature_world','CreaturePool'),('building','BuildingPool'),('effect_pool','EffectPool'),('family8_pool','Family8Pool')]:
  target=runtime/f'{file}.rs'
  if f'impl {typ}' not in target.read_text():
   target=next(f for f in runtime.rglob('*.rs')if f'impl {typ}' in f.read_text())
  extra='self.bindings[i]=Some(crate::building::BuildingAnimationBinding{selector:r[45]});' if typ=='BuildingPool' else ''
  with target.open('a')as f:f.write(f'\nimpl {typ} {{pub(crate) fn fixture_set(&mut self,i:usize,r:&[u8]){{self.records[i].copy_from_slice(r);{extra}}}}}\n')
 with (runtime/'actor_world.rs').open('a')as f:f.write('\npub mod terrain_refresh_fixture {\n'+(P/'scripts/desktop/terrain_refresh_harness.rs').read_text()+'\n}\n')
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}','-o',str(work/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
  run(cmd)
 (work/'main.rs').write_text('fn main(){let a:Vec<_>=std::env::args().collect();genewars_runtime::actor_world::terrain_refresh_fixture::run(std::path::Path::new(&a[1]),std::path::Path::new(&a[2]));}')
 run([rustc,'--edition=2021',str(work/'main.rs'),'-L',f'dependency={work}','--extern',f'genewars_runtime={work}/libgenewars_runtime.rlib','-o',str(work/'probe.exe')]);run([str(work/'probe.exe'),str(P),str(work)])
 spec=importlib.util.spec_from_file_location('initial_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);model=m.Source()
 files=[P/'private/ghidra'/f for f in ['construction-command/instructions-0008b840.txt','camera-player-input/instructions-0008b800.txt','terrain-map-refs/instructions-00084960.txt','terrain-map-refs/instructions-00084b40.txt','terrain-map-refs/instructions-0007b240.txt','entity-order/instructions-0001f440.txt','entity-plant/instructions-0001f240.txt','entity-plant/instructions-0001f280.txt']]
 model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));total=steps=0;cases=[]
 for line in (work/'cases.tsv').read_text().splitlines():
  name,x,y,slope=line.split('\t');x,y,slope=int(x),int(y),int(slope);folder=work/name;before=(folder/'before.bin').read_bytes();actual=(folder/'after.bin').read_bytes()
  state={0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])}
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    if kind in BASES:native=bytearray(before[BASES[kind]+slot*width:BASES[kind]+(slot+1)*width])
    else:
     raw=before[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
    if kind in VT:struct.pack_into('<I',native,35,VT[kind])
    state.update({address+slot*width+j:b for j,b in enumerate(native)})
  # Start the literal dirty/shade tail after geometry changes. Its locals hold
  # the original vertex position. Preserve original pushes/sub/add epilogue.
  entry=0x84d3d if slope else 0x84a6b
  if slope:
   assert model.code[entry]==('MOV',['EAX','0x8']),model.code.get(entry)
  size=0x68 if slope else 0x38;esp=0x300000-size-16
  state.update({esp+i:b for i,b in enumerate(struct.pack('<hh',x,y))})
  state.update({0x300000+i:b for i,b in enumerate(struct.pack('<I',0xdeadbeef))})
  report,_,trace=model.run(state_before=state,entry=entry,registers={'ESP':esp});steps+=report['steps'];all_trace=trace.copy();state=model.mem.copy()
  for dx,dy in [(0,0),(-1,0),(-1,-1),(0,-1)]:
   cell=(((y>>8)+dy)&127)*128+(((x>>8)+dx)&127)
   report,_,trace=model.run(state_before=state,entry=0x8b840,registers={'EAX':0x11c2be+cell*14});steps+=report['steps'];all_trace.extend(trace);state=model.mem.copy()
  assert all_trace.count(0x8b800)==4 and all_trace.count(0x8b840)==4 and all_trace.count(0x7b140)==24 and all_trace.count(0x7b240)==4,(name,'literal call coverage')
  expected=bytearray(before);expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    native=bytearray(model.load(address+slot*width+j,1)for j in range(width))
    if kind in BASES:
     base=BASES[kind]+slot*width;native[31:39]=before[base+31:base+39];expected[base:base+width]=native
    else:expected[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
  assert expected==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:20])
  total+=len(actual);cases.append(name);(folder/'source-after.bin').write_bytes(expected);print(name,'PASS',flush=True)
 receipt=dict(status='passed-terrain-refresh',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=total,literal_instructions=steps,implementation_pins=pins,listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']},harness_pins={str(f.relative_to(P)):sha(f)for f in [Path(__file__),P/'scripts/desktop/terrain_refresh_harness.rs']},private_run=work.relative_to(P).as_posix(),original_execution=False,scope='Synthetic linked lists kinds1/2/3/4/5/8, four corners and wrapped coordinates, shade flags2, varied physical heights and effect2E. Actual7B140/7B240. Literal84960/84B40 dirty/shade tails only: preceding terrain edits outside scope. Test-only private fixture setters. Helpers require staged caller; 48 late invalid-link/cycle fixture-wrapper rollbacks checked, not productioncaller rollback acceptance.')
 (P/'research/integration/terrain-refresh.json').write_text(json.dumps(receipt,indent=2)+'\n');print(total,steps)
if __name__=='__main__':main()


"""Literal completed Genepod idle callback on source-created whole-world fixtures.
No engine execution, production admission, or authentic gameplay acceptance.
"""
from pathlib import Path
import hashlib, importlib.util, json, struct, uuid, subprocess, sys
P=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
ACTORS=343958;CREATURES=ACTORS+4000;BUILDINGS=CREATURES+37200;PLAYERS=BUILDINGS+17152;RNG=PLAYERS+7220;BINDINGS=RNG+7
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]

spec=importlib.util.spec_from_file_location('initial_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
model=m.Source();listing=P/'private/ghidra/building-update/instructions-000451f0.txt'
model.code.update(m.m.Model([listing],b'').code)
model.code[0x47f34]=('CALL',['0x2e0010']);model.code[0x2e0010]=('RET',[])
order=sorted(model.code);model.next=dict(zip(order,order[1:]))
resource=P/'private/integration/game-runtime/buildings/BGENE.ANB'
assert sha(resource)=='a71f03d3cce3d36798cd7f8ad578d665e65883bfe9b541ba93d0ad9eff814cab'
anb=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',anb,10)
for i in range(5):
 for offset in [0,4]:struct.pack_into('<I',anb,animations+i*10+offset,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+offset)[0])
work=P/'private/review/genepod-idle-source'/uuid.uuid4().hex;work.mkdir(parents=True)
origin=P/'private/review/genepod-construction-world/5ca34ed40b0e438ebeee88697e97addb'
sys.path.insert(0,str(P/'scripts'));from cargo import environment
rust=P/'rust/crates/genewars-runtime/src/genepod_idle.rs';harness=work/'idle.rs';exe=work/'idle.exe'
harness.write_text(f'#[path="{rust.as_posix()}"]mod idle;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let q=idle::plan_completed_idle(b[..67].try_into().unwrap(),b[67..].try_into().unwrap()).unwrap();std::io::stdout().write_all(&q.building_after).unwrap();std::io::stdout().write_all(&q.player_after).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-o',str(exe)],env=environment(),check=True)
cases=[]
for owner in range(4):
 for fill in [0,255]:
  fixture=origin/f'selector0-owner{owner}-loadingfalse-step3/after.bin'
  before=fixture.read_bytes(); selected_slot=next(int(line.split("\t")[3]) for line in (origin/"cases.tsv").read_text().splitlines() if line.startswith(f"selector0-owner{owner}-loadingfalse-step3\t"))
  selected=BUILDINGS+selected_slot*67
  assert before[selected+45]==0 and before[selected+49]&1
  assert struct.unpack_from('<i',before,selected+54)[0]<=0
  state={0x118d60+i:b for i,b in enumerate(before[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])})
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
    if base is None:
     raw=before[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
    else:native=bytearray(before[base+slot*width:base+(slot+1)*width])
    if kind==1:struct.pack_into('<I',native,35,0x111fec)
    if kind==4:
     struct.pack_into('<I',native,35,0x112120)
     if native[6]!=0:struct.pack_into('<I',native,31,0x620000+animations)
    state.update({address+slot*width+j:b for j,b in enumerate(native)})
  state.update({0x118d74+j:b for j,b in enumerate(before[PLAYERS:RNG])});state.update({0x620000+j:b for j,b in enumerate(anb)})
  seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})

  report,_,trace=model.run(seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae+selected_slot*67},stack_fill=fill)
  expected=bytearray(before);expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    native=bytearray(model.load(address+slot*width+j,1)for j in range(width));base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
    if base is None:expected[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
    else:
     native[31:39]=before[base+slot*width+31:base+slot*width+39];expected[base+slot*width:base+(slot+1)*width]=native
  expected[PLAYERS:RNG]=bytes(model.load(0x118d74+i,1)for i in range(7220));struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))

  player_at=PLAYERS+owner*1805
  numeric=subprocess.run([str(exe)],input=before[selected:selected+67]+before[player_at:player_at+1805],capture_output=True,check=True).stdout
  actual=bytearray(before);actual[selected:selected+67]=numeric[:67];actual[player_at:player_at+1805]=numeric[67:]
  assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
  assert expected[RNG:BINDINGS]==before[RNG:BINDINGS]
  changed=[i for i,(a,b) in enumerate(zip(before,expected)) if a!=b]
  allowed=set(range(selected,selected+67))|set(range(PLAYERS+owner*1805,PLAYERS+(owner+1)*1805))
  assert set(changed)<=allowed,changed
  name=f'owner{owner}-fill{fill}';(work/f'{name}-after.bin').write_bytes(expected)
  if fill==0: prior=expected
  else: assert prior==expected
  cases.append(dict(name=name,fixture=str(fixture.relative_to(P)),fixture_sha256=sha(fixture),after_sha256=hashlib.sha256(expected).hexdigest(),whole_world_bytes=len(expected),literal_instructions=report['steps'],building_changes=[dict(offset=hex(i-selected),before=before[i],after=expected[i]) for i in changed if selected<=i<selected+67],player_changes=[hex(i-PLAYERS-owner*1805)for i in changed if i>=PLAYERS],rng_unchanged=True))
receipt=dict(status='passed-literal-source-idle-world-comparison',rust_sha256=sha(rust),cases=cases,private_run=str(work.relative_to(P)),listing_sha256=sha(listing),resource_sha256=sha(resource),script_sha256=sha(Path(__file__)),ports=['83140 render queue request only'],limitations=['Pure callback afterimages composed into wholeworld; scheduler checked separately','Construction fixtures, not actual Krell gameplay capture','No production request/factory branch executed'],power_base=model.load(0x104d10,1))
out=P/'research/genepod-production';out.mkdir(exist_ok=True);(out/'idle-source.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))

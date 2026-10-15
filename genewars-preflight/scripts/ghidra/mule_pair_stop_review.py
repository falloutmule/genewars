"""Literal69D9C pair-stop slice versus pure planner; no gameplay acceptance."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-pair-stop-boundaries/7e64eb0dfd124828ab3b5411f9a83b3e';before=fixture/'69d9c';after=fixture/'69dc8';current=(before/'creature-2.bin').read_bytes();peer=(before/'creature-0.bin').read_bytes();plants=(before/'plants.bin').read_bytes();rng=(before/'rng.bin').read_bytes()
backing=(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before/backing.bin').read_bytes();work=P/'private/review/mule-pair-stop'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/mule_pair_stop.rs'
h=work/'probe.rs';exe=work/'probe.exe'
h.write_text(f'#[path="{rust.as_posix()}"]mod pair;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut output=Vec::new();for c in b.chunks_exact(248){let q=pair::plan_stop(c[..124].try_into().unwrap(),c[124..248].try_into().unwrap());if std::env::args().any(|a|a=="reject"){assert!(q.is_err());output.push(1);continue;}let q=q.unwrap();output.extend(q.current_after);output.extend(q.peer_after);}std::io::stdout().write_all(&output).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[];plant_bytes=0
for flags in [None,0,0xffff,0x1234]:
 for speed in [None,-1,32767]:
  a=bytearray(current);b=bytearray(peer)
  if flags is not None:struct.pack_into('<H',a,73,flags);struct.pack_into('<H',b,73,flags^0xffff);a[22]=255;b[22]=128;a[84]=255;b[84]=1
  if speed is not None:struct.pack_into('<h',a,63,speed);struct.pack_into('<h',b,63,-speed if speed!= -32768 else 0)
  state={0x1542be+248+i:v for i,v in enumerate(a)};state.update({0x1542be+i:v for i,v in enumerate(b)});state.update({0x15d40e+i:v for i,v in enumerate(plants)})
  seed,ring,draws=struct.unpack('<HBI',rng);state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
  model=m.Source(backing);model.code[0x69dc8]=('RET',[])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x69d9c,registers={'ESI':0x1542be+248,'EDI':0x1542be},instruction_budget=10000)
  aa=bytes(model.load(0x1542be+248+i,1)for i in range(124));bb=bytes(model.load(0x1542be+i,1)for i in range(124));pp=bytes(model.load(0x15d40e+i,1)for i in range(len(plants)))
  assert pp==plants;assert r['draws']==0;assert struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))==rng
  assert trace.count(0x1ee50)==2
  if flags is None and speed is None:assert aa==(after/'creature-2.bin').read_bytes()and bb==(after/'creature-0.bin').read_bytes()
  inputs+=a+b;expected+=aa+bb;plant_bytes+=len(pp);cases.append(dict(flags=flags,speed=speed,instructions=r['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray()
for off,value in [(6,3),(24,0),(94,1),(97,0x40),(96,0x88),(122,5),(87,2),(85,1)]:
 a=bytearray(current);a[off]=value;negative+=a+peer
bad=bytearray(current);bad[15:17]=peer[15:17];negative+=bad+peer
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*9
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,record_bytes=len(actual),unchanged_plant_bytes=plant_bytes,negative_cases=9,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,Path(__file__),*before.glob('*.bin'),*after.glob('*.bin')]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='Stop69D9C..69DC8 bothrecords,actual721+flags/speeds;2literal1EE50calls,0RNG,plantsunchanged. Caller provesdistancebranch; timer32beforetail; no lifecycle/gameplay acceptance.')
(P/'research/variants/mule-pair-stop-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),record_bytes=len(actual),plant_bytes=plant_bytes)))

"""Literal5E271 retry relocation versus pure planner; excludes ordinary AI tail."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-newborn-choice/1e21f105b9914c7bb74f763dbd0a508e';before=fixture/'73c20-1/record.bin';base=before.read_bytes();backing=(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before/backing.bin').read_bytes();work=P/'private/review/mule-retry-relocation'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/mule_retry_relocation.rs';world=P/'rust/crates/genewars-runtime/src/creature_world.rs';text=world.read_text();funcs=[]
for signature in ['fn mule_skill_percent(', 'pub(crate) fn mule_requested_speed(']:
 start=text.index(signature);end=text.index('\n}',start)+2;funcs.append(text[start:end])
h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::{actor_activation,blueprint,construction_work};mod creature_world{type CreatureRecord=[u8;124];'+''.join(funcs)+'}'+f'#[path="{rust.as_posix()}"]mod retry;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let t=actor_activation::TrigTables::generate_pinned().unwrap();let mut out=Vec::new();for c in b.chunks_exact(131){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[124..126].try_into().unwrap()),ring:c[126],draw_count:u32::from_le_bytes(c[127..131].try_into().unwrap())};let q=retry::plan_relocation(c[..124].try_into().unwrap(),rng,&t);if std::env::args().any(|a|a=="reject"){assert!(q.is_err());out.push(1);continue;}let q=q.unwrap();out.extend(q.record_after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for seed in [30553,0,1,65535]:
 for flags in [None,0,65535]:
  a=bytearray(base)
  if flags is not None:struct.pack_into('<H',a,73,flags);a[22]=255;a[118]=100;a[123]=127
  state={0x1542be+496+i:v for i,v in enumerate(a)};state[0x11872c]=22;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',35193))})
  model=m.Source(backing);model.code[0x5e3b1]=('JMP',['0xdeadbeef'])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x5e271,registers={'ECX':0x1542be+496},instruction_budget=10000)
  aa=bytes(model.load(0x1542be+496+i,1)for i in range(124));rr=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));assert r['draws']==2
  # Wait planner stores delay before this branch; source keeps it in EDI. Both
  # are equivalent inputs because this branch overwrites6C without reading it.
  if flags is None and seed==30553:
   wait=bytearray(a);struct.pack_into('<H',wait,108,(seed>>8)%20+10);inputs+=wait+struct.pack('<HBI',seed,22,35193);expected+=aa+rr;cases.append(dict(actual_wait_adapter=True))
  inputs+=a+struct.pack('<HBI',seed,22,35193);expected+=aa+rr;cases.append(dict(seed=seed,route_flags=flags,instructions=r['steps'],draws=r['draws']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//131,i%131,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray()
for off,value in [(6,3),(24,0),(94,3),(122,5),(87,1),(93,1),(96,2),(126,32)]:
 a=bytearray(inputs[:131]);a[off]=value;negative+=a
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*8
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),negative_cases=8,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,world,Path(__file__),before,fixture/'events.json',P/'rust/crates/genewars-runtime/src/construction_work.rs']},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='Literal5E271..5E3B1 neutralMule true5B100 relocation, actual829 input plus routeflags/RNGseeds;2draws, own speed and13E60. Caller proves predicate and continues ordinaryAI; no callback/gameplay acceptance.')
(P/'research/variants/mule-retry-relocation-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

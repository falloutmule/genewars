"""Bounded literal null-food reset versus pure Rust, false cell predicate port."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-null-food-reset-boundary-001';base=(fixture/'66080-record.bin').read_bytes();original_rng=(fixture/'66080-rng.bin').read_bytes()
work=P/'private/review/mule-lost-food'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/mule_lost_food.rs'
h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::blueprint;'+f'#[path="{rust.as_posix()}"]mod reset;'+r"""use std::io::{Read,Write};fn main(){let mut input=Vec::new();std::io::stdin().read_to_end(&mut input).unwrap();let mut output=Vec::new();for c in input.chunks_exact(131){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[124..126].try_into().unwrap()),ring:c[126],draw_count:u32::from_le_bytes(c[127..131].try_into().unwrap())};let q=reset::plan_lost_food(c[..124].try_into().unwrap(),rng);if std::env::args().any(|a|a=="reject"){assert!(q.is_err());output.push(1);continue;}let q=q.unwrap();output.extend(q.record_after);output.extend(q.rng_after.seed.to_le_bytes());output.push(q.rng_after.ring);output.extend(q.rng_after.draw_count.to_le_bytes());}std::io::stdout().write_all(&output).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for seed in [48457,0,1,2,65535]:
 for flags in [0x40c8,0xfff8]:
  a=bytearray(base);struct.pack_into('<H',a,96,flags);native=bytearray(a);struct.pack_into('<I',native,35,0x111fec)
  state={0x15433a+i:v for i,v in enumerate(native)};state[0x11872c]=18;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',34965))})
  model=m.Source();extra=P/'private/ghidra/activation-rng-refs/instructions-0005c920.txt';model.code.update(m.m.Model([extra],b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.code[0x5b100]=('XOR',['EAX','EAX']);model.code[0x5b101]=('RET',[]);model.next[0x5b100]=0x5b101;model.code[0x65c5b]=('RET',[])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x66080,registers={'ESI':0x15433a,'EBP':0x2fff00},instruction_budget=10000)
  after=bytearray(model.load(0x15433a+i,1)for i in range(124));after[35:39]=a[35:39];rng=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  assert r['draws']==1;assert 0x5b100 in trace
  if seed==48457 and flags==0x40c8:
   assert after==(fixture/'65c5b-record.bin').read_bytes(),[(i,x,y)for i,(x,y)in enumerate(zip(after,(fixture/'65c5b-record.bin').read_bytes()))if x!=y];assert rng==(fixture/'65c5b-rng.bin').read_bytes()
  inputs+=a+struct.pack('<HBI',seed,18,34965);expected+=after+rng;cases.append(dict(seed=seed,flags=flags,instructions=r['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12]
negative=bytearray()
for off,value in [(6,3),(94,8),(87,2),(96,0x42),(96,0)]:
 a=bytearray(base);a[off]=value;negative+=a+original_rng
negative+=base+struct.pack('<HBI',1,32,0)
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*6
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),negative_cases=6,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,Path(__file__),*fixture.glob('*.bin')]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [*model.files,extra]},ports=['66112 current-cell predicate false; caller must prove','stop65C5B before normal timer decrement'],scope='Whole124byte record plus RNG; actual isolated588 boundary and synthetic flags/seeds. No full scheduler claim.')
(P/'research/variants/mule-lost-food-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

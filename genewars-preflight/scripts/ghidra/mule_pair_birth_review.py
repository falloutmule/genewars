"""Neutral same-Mule61C8A prefix; stop before factory62201, compare exact RNG."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-pair-birth-boundaries/13aed65056f14e3b9dc550e7337001de';before=fixture/'61c8a';after=fixture/'62201';current=(before/'creature-2.bin').read_bytes();peer=(before/'creature-0.bin').read_bytes();original_rng=(before/'rng.bin').read_bytes()
backing=(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before/backing.bin').read_bytes();work=P/'private/review/mule-pair-birth'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/mule_pair_birth.rs';h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::blueprint;'+f'#[path="{rust.as_posix()}"]mod birth;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(260){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[248..250].try_into().unwrap()),ring:c[250],draw_count:u32::from_le_bytes(c[251..255].try_into().unwrap())};let q=birth::plan_birth(c[..124].try_into().unwrap(),c[124..248].try_into().unwrap(),rng,c[255]!=0,u32::from_le_bytes(c[256..].try_into().unwrap()));if std::env::args().any(|a|a=="reject"){assert!(q.is_err());out.push(1);continue;}let q=q.unwrap();out.extend(q.current_after);out.extend(q.peer_after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());out.extend([q.request.owner,q.request.species,q.request.quality,q.request.inherit as u8]);out.extend(q.request.parent_id.to_le_bytes());for v in q.request.position{out.extend(v.to_le_bytes());}}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for seed in [61485,0,1,5428,65535]:
 for traits in [(50,50),(90,91),(91,90),(100,100),(0,255),(255,0)]:
  a=bytearray(current);b=bytearray(peer);a[118],b[118]=traits
  state={0x1542be+248+i:v for i,v in enumerate(a)};state.update({0x1542be+i:v for i,v in enumerate(b)});ebp=0x2ff000;state.update({ebp-0x7a+i:v for i,v in enumerate(struct.pack('<I',0x1542be+248))});state[0x11872c]=25;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',35132))})
  model=m.Source(backing);model.code[0x62201]=('JMP',['0xdeadbeef'])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x61c8a,registers={'EBP':ebp,'ECX':0},instruction_budget=100000)
  aa=bytes(model.load(0x1542be+248+i,1)for i in range(124));bb=bytes(model.load(0x1542be+i,1)for i in range(124));rng=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  request=bytes([model.r['EAX']&255,model.r['EDX']&255,model.r['ECX']&255,model.load(ebp+0x156,1)])+struct.pack('<H',2)+struct.pack('<I',model.r['EBX'])
  assert [model.load(0x104a6c+i,1)for i in range(2)]==[4,4];assert model.load(0x104a98+24,1)==4
  if seed==61485 and traits==(50,50):assert aa==(after/'creature-2.bin').read_bytes()and bb==(after/'creature-0.bin').read_bytes()and rng==(after/'rng.bin').read_bytes();assert r['draws']==7
  inputs+=a+b+struct.pack('<HBI',seed,25,35132)+struct.pack('<BI',0,0);expected+=aa+bb+rng+request;cases.append(dict(seed=seed,traits=traits,draws=r['draws'],instructions=r['steps'],inherit=request[3],quality=request[2]))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//265,i%265,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray()
for off,value in [(6,3),(24,0),(94,3),(122,5),(87,2),(108,1)]:
 a=bytearray(inputs[:260]);a[off]=value;negative+=a
for unavailable,distance in [(1,0),(0,16385)]:
 a=bytearray(inputs[:260]);a[255:]=struct.pack('<BI',unavailable,distance);negative+=a
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*8
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),negative_cases=8,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,Path(__file__),*before.glob('*.bin'),*after.glob('*.bin')]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='Neutral sameMule61C8A..62201 bothrecords,RNG,request; allgene4 but exactbranch/rejectiondraws retained. Caller virtual68/distance supplied. No factory/posttail/nativeacceptance.')
(P/'research/variants/mule-pair-birth-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

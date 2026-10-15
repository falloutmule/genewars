"""Literal62270 birth tail, controlled5B100 ports; no factory/gameplay claim."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-pair-birth-tail/7629bb6ae2d84eb396d8263a03167301/62270';end=P/'private/review/krell-pair-birth-tail/5c7b8bc54b514006abd208ae38721c86/5ef46';current=(fixture/'creature-2.bin').read_bytes();peer=(fixture/'creature-0.bin').read_bytes();newborn=(fixture/'creature-4.bin').read_bytes();rng=(fixture/'rng.bin').read_bytes()
backing=(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before/backing.bin').read_bytes();work=P/'private/review/mule-pair-birth-tail'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/mule_pair_birth_tail.rs';world=P/'rust/crates/genewars-runtime/src/creature_world.rs';text=world.read_text();funcs=[]
for signature in ['fn mule_skill_percent(', 'pub(crate) fn mule_requested_speed(']:
 start=text.index(signature);finish=text.index('\n}',start)+2;funcs.append(text[start:finish])
h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::{actor_activation,blueprint,construction_work};mod creature_world{type CreatureRecord=[u8;124];'+''.join(funcs)+'}'+f'#[path="{rust.as_posix()}"]mod tail;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let t=actor_activation::TrigTables::generate_pinned().unwrap();let mut out=Vec::new();for c in b.chunks_exact(259){let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[248..250].try_into().unwrap()),ring:c[250],draw_count:u32::from_le_bytes(c[251..255].try_into().unwrap())};let mut n=0;let q=tail::plan_tail(c[..124].try_into().unwrap(),c[124..248].try_into().unwrap(),u16::from_le_bytes(c[255..257].try_into().unwrap()),rng,&t,|r,cell|{assert_eq!(r[94],8);assert_eq!(cell,u16::from_le_bytes(c[255..257].try_into().unwrap()));assert_eq!(u16::from_le_bytes(r[15..17].try_into().unwrap()),if n==0{0}else{2});let v=c[257+n];n+=1;if v==2{Err(())}else{Ok(v!=0)}});if std::env::args().any(|a|a=="reject"){assert!(q.is_err());out.push(1);continue;}let q=q.unwrap();out.extend(q.current_after);out.extend(q.peer_after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[];seed0,ring,draws=struct.unpack('<HBI',rng);cell=struct.unpack_from('<h',current,9)[0]//256*128+struct.unpack_from('<h',current,7)[0]//256
for seed in [seed0,0,1,65535]:
 for flags in [(0,0),(0,1),(1,0),(1,1)]:
  a=bytearray(current);b=bytearray(peer);state={0x1542be+248+i:v for i,v in enumerate(a)};state.update({0x1542be+i:v for i,v in enumerate(b)});state.update({0x1542be+496+i:v for i,v in enumerate(newborn)});ebp=0x2ff000
  for address,value in [(ebp-10,0x1542be),(ebp-14,0x11c2be+cell*14),(0x118782,draws)]:state.update({address+i:v for i,v in enumerate(struct.pack('<I',value))})
  state[0x11872c]=ring;model=m.Source(backing);model.code[0x5ef46]=('JMP',['0xdeadbeef']);model.code[0x622fe]=('MOV',['AL',hex(flags[0])]);model.code[0x62427]=('MOV',['AL',hex(flags[1])])
  r,_,trace=model.run(seed=seed,state_before=state,entry=0x62270,registers={'EBP':ebp,'EDI':0x1542be+248},instruction_budget=10000)
  aa=bytes(model.load(0x1542be+248+i,1)for i in range(124));bb=bytes(model.load(0x1542be+i,1)for i in range(124));rr=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));assert bytes(model.load(0x1542be+496+i,1)for i in range(124))==newborn
  if seed==seed0 and flags==(0,1):assert aa==(end/'creature-2.bin').read_bytes()and bb==(end/'creature-0.bin').read_bytes()and rr==(end/'rng.bin').read_bytes()
  inputs+=a+b+struct.pack('<HBIHBB',seed,ring,draws,cell,*flags);expected+=aa+bb+rr;cases.append(dict(seed=seed,predicates=flags,instructions=r['steps'],draws=r['draws']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//255,i%255,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
negative=bytearray()
for off,value in [(6,3),(24,0),(94,3),(122,5),(87,2),(250,32),(258,2)]:
 a=bytearray(inputs[:259]);a[off]=value;negative+=a
bad=bytearray(inputs[:259]);bad[255:257]=struct.pack('<H',16384);negative+=bad
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*8
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
report=dict(status='passed',cases=cases,compared_bytes=len(actual),unchanged_newborn_bytes=124*len(cases),negative_cases=8,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,world,Path(__file__),P/'rust/crates/genewars-runtime/src/construction_work.rs',*fixture.glob('*.bin'),*end.glob('*.bin')]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='62270..5EF46 pre-common-tail. Explicit two5B100 boolean ports; allfour predicate combinations, literal73C20 andMulevirtual80/category. Actual785 tail matches captured predicates false/true. No5CC60/no74D90/no reference clearing beyond4F/52; no factory or gameplay acceptance.')
(P/'research/variants/mule-pair-birth-tail-rust-review.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

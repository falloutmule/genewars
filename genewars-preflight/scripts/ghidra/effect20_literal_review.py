"""Literal48790 effect20 null-associated factory versus native numeric planner."""
from pathlib import Path
import importlib.util,struct,json,subprocess,sys,uuid,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
spec=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s)
files=[P/'private/ghidra/activation-rng-refs/instructions-00048790.txt',P/'private/ghidra/player-creation/instructions-0001f070.txt',P/'private/ghidra/mission-tail-init/instructions-00049e90.txt',P/'private/ghidra/terrain-map-refs/instructions-0007b240.txt']
w=P/'private/review/effect20-literal'/uuid.uuid4().hex;w.mkdir(parents=True);inputs=bytearray();expected=bytearray();cases=[]
for fill in [0,165]:
 for angles in [(0,0,8192),(2040,298,16320),(256,100,10048),(1024,0,8192)]:
  v=s.Source();v.code.update(s.m.Model(files,b'').code);order=sorted(v.code);v.next=dict(zip(order,order[1:]));state={}
  def put(a,n,x):state.update({a+i:b for i,b in enumerate((x%2**(8*n)).to_bytes(n,'little'))})
  for i in range(200*81):put(0x1688ae+i,1,fill)
  for i in range(200):put(0x1688ae+i*81+6,1,0);put(0x1688ae+i*81+35,4,0x112254)
  for i in range(16384):
   for j in range(14):put(0x11c2be+i*14+j,1,0)
   put(0x11c2be+i*14+12,1,128)
  for a,n,x in [(0x11c2b4,2,0),(0x300004,4,0),(0x300008,4,0),(0x30000c,4,0),(0x300010,4,0),(0x280010,2,angles[0]),(0x280012,2,angles[1]),(0x280014,2,angles[2])]:put(a,n,x)
  before=bytes(state[0x1688ae+i]for i in range(81));r,_,trace=v.run(seed=5428,state_before=state,entry=0x48790,registers={'EAX':32,'EDX':(1024<<16)|1024,'EBX':8000,'ECX':0x280010},instruction_budget=10000)
  after=bytearray(v.load(0x1688ae+i,1)for i in range(81));after[31:39]=before[31:39]
  assert r['draws']==0 and v.load(0x11c2b4,2)==1 and v.load(0x11c2be+(4*128+4)*14+8,1)==5
  inputs+=before+struct.pack('<3HiB',*angles,struct.unpack_from('<i',after,17)[0],after[23]);expected+=after;cases.append(dict(fill=fill,angles=angles,instructions=r['steps'],draws=0))
module=P/'rust/crates/genewars-runtime/src/effect20_factory.rs';h=w/'probe.rs';h.write_text('use genewars_runtime::{landing,water_effect};\n#[path="'+module.as_posix()+'"]mod effect20_factory;\n'+r'use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let t=genewars_runtime::actor_activation::TrigTables::generate_pinned().unwrap();let mut out=Vec::new();for c in b.chunks_exact(92){let mut pool=[[0;81];200];pool[0].copy_from_slice(&c[..81]);let angles=core::array::from_fn(|i|u16::from_le_bytes([c[81+i*2],c[82+i*2]]));let ground=i32::from_le_bytes(c[87..91].try_into().unwrap());let p=effect20_factory::plan_factory(&pool,0,0,[1024,1024],ground,c[91],8000,angles,None,t.sine().try_into().unwrap(),t.cosine().try_into().unwrap()).unwrap();let mut a=p.after_nonpointer;a[31..39].copy_from_slice(&c[31..39]);out.extend(a);}std::io::stdout().write_all(&out).unwrap();}')
exe=w/'probe.exe';target=P/'private/build/rust-target/debug';subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True);actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
(w/'inputs.bin').write_bytes(inputs);(w/'expected.bin').write_bytes(expected)
paths=files+[module,Path(__file__)];receipt=dict(status='passed',cases=cases,compared_bytes=len(actual),private_run=str(w.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in paths},scope='Full literal48790 including1F070,49E90,terrain sampling and8AC60; native pure factory81bytes with pointer31:39 normalized. Explicit sampledground/shade inputs; empty map link/count asserted on source; no native world commit/lifecycle/fullpool proof. No RNG or audio ports.')
(P/'research/variants/effect20-literal-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

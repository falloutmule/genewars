"""Literal4B630 after explicit physics port, EFF32 lifetime and retirement request."""
from pathlib import Path
import importlib.util,struct,json,subprocess,sys,uuid,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
files=[P/'private/ghidra/plant-command/instructions-0004b630.txt',P/'private/ghidra/mission-array-refs/instructions-0004b3b0.txt'];v=m.Source();v.code.update(m.m.Model(files,b'').code);v.code[0x6df30]=('RET',[]);v.code[0x49d30]=('RET',[]);order=sorted(v.code);v.next=dict(zip(order,order[1:]))
row=next(x for x in json.loads((P/'research/integration/sprite-corpus.json').read_text())['results']if x['entry_id']=='root/2/disc/59');asset=P/row['private_database'];db=bytearray(asset.read_bytes());frames,animations=struct.unpack_from('<II',db,10)
for i in range(80):
 if animations+i*10+10>len(db):break
 for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x620000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
w=P/'private/review/effect20-animation'/uuid.uuid4().hex;w.mkdir(parents=True);inputs=bytearray();expected=bytearray();cases=[]
for frame in [0,8,9,255]:
 for counter in [0,1,127,128,255]:
  for timer in [-1,0,1,2]:
   a=bytearray(81);a[6]=5;a[69]=32;a[27]=frame;a[28]=1;a[30]=counter;struct.pack_into('<h',a,74,timer);source=bytearray(a);struct.pack_into('<II',source,31,0x620000+animations+320,0x112254);state={0x620000+i:b for i,b in enumerate(db)};state.update({0x1688ae+i:b for i,b in enumerate(source)})
   r,_,trace=v.run(seed=5428,state_before=state,entry=0x4b630,registers={'EAX':0x1688ae},instruction_budget=10000);b=bytearray(v.load(0x1688ae+i,1)for i in range(81));b[31:39]=a[31:39];inputs+=a;expected+=bytes([0x49d30 in v.calls])+b;cases.append(dict(frame=frame,counter=counter,timer=timer,instructions=r['steps']))
module=P/'rust/crates/genewars-runtime/src/effect20_factory.rs';h=w/'probe.rs';h.write_text('use genewars_runtime::{landing,water_effect};\n#[path="'+module.as_posix()+'"]mod effect20_factory;\n'+r'use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(81){let p=effect20_factory::plan_animation_after_physics(c.try_into().unwrap()).unwrap();out.push(p.retirement_required as u8);out.extend(p.after);}std::io::stdout().write_all(&out).unwrap();}')
exe=w/'probe.exe';target=P/'private/build/rust-target/debug';subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True);actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
(w/'inputs.bin').write_bytes(inputs);(w/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),private_run=str(w.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files+[module,Path(__file__),asset]},scope='Full4B630 control/7AC00 EFF32 animation after explicit6DF30 no-op physics port;49D30 retirement requested but not executed. Byte31:39 normalized. No worldphysics/lifecycle admission or gameplay acceptance.')
(P/'research/variants/effect20-animation-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

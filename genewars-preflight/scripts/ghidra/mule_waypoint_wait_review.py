"""Literal6FC5C prefix versus native planner, including numeric73970 heading."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
base=(P/'private/review/krell-mule-callbacks-013/frame-557-slot-1-before/creatures.bin').read_bytes()[124:248]
work=P/'private/review/mule-waypoint-wait'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/mule_waypoint_wait.rs';motion=P/'rust/crates/genewars-runtime/src/actor_motion.rs'
text=motion.read_text();start=text.index('pub(crate) fn angle(');end=text.index('\n}\n',start)+2;angle=text[start:end]
tables=P/'private/review/zone-marker-motion/c59459093bcf4e978d5e0e5dc0b22706'
h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::physics_tables;mod actor_motion{'+angle+'}'+f'#[path="{rust.as_posix()}"]mod wait;'+r"""use std::io::{Read,Write};fn main(){let args:Vec<_>=std::env::args().collect();let physics=physics_tables::PhysicsTables::from_mapped_bytes(&std::fs::read(&args[1]).unwrap(),&std::fs::read(&args[2]).unwrap()).unwrap();let mut input=Vec::new();std::io::stdin().read_to_end(&mut input).unwrap();let mut output=Vec::new();for c in input.chunks_exact(129){let mut calls=0;let q=wait::plan_wait(c[..124].try_into().unwrap(),u32::from_le_bytes(c[124..128].try_into().unwrap()),&physics,|_|{calls+=1;Ok::<_,()>(c[128]!=0)});if args.len()>3{assert!(q.is_err());assert_eq!(calls,0);output.push(1);continue;}let q=q.unwrap();output.extend(q.record_after);output.extend(q.saved_target_speed.to_le_bytes());output.push(calls);}std::io::stdout().write_all(&output).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for timer in [0,1,20,80,255]:
 for phase in range(4):
  for predicate in [False,True]:
   a=bytearray(base);a[84]=timer;native=bytearray(a);struct.pack_into('<I',native,35,0x111fec);state={0x15433a+i:v for i,v in enumerate(native)}
   model=m.Source();model.code[0x5b100]=('MOV',['AL',str(int(predicate))]);model.code[0x5b101]=('RET',[]);model.next[0x5b100]=0x5b101;model.code[0x6fd67]=('RET',[])
   frame=(phase-1)&0xffffffff;r,_,trace=model.run(counter=frame,seed=63161,state_before=state,entry=0x6fc5c,registers={'ESI':0x15433a},instruction_budget=10000)
   after=bytearray(model.load(0x15433a+i,1)for i in range(124));after[35:39]=a[35:39];calls=int(0x5b100 in trace)
   assert calls==int(phase==3);assert r['draws']==0
   inputs+=a+struct.pack('<IB',frame,predicate);expected+=after+a[63:65]+bytes([calls]);cases.append(dict(timer=timer,phase=phase,predicate=predicate,instructions=r['steps']))
cmd=[str(exe),str(tables/'angles.bin'),str(tables/'bounce.bin')]
actual=subprocess.run(cmd,input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12]
negative=bytearray()
for offset,value in [(6,3),(73,0),(73,1),(73,2),(82,1),(82,4)]:
 a=bytearray(base);a[offset]=value;negative+=a+struct.pack('<IB',2,1)
assert subprocess.run(cmd+['reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*6
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
receipt=dict(status='passed',cases=cases,compared_bytes=len(actual),negative_cases=6,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,motion,Path(__file__),tables/'angles.bin',tables/'bounce.bin']},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},scope='Full124byte prefix plus savedtargetspeed and querycadence; actual557 record variedtimer/phase/predicate. Heading73970 numeric compared. Predicate explicitport, stopsbeforesteering/physics; nofullworldclaim.')
(P/'research/variants/mule-waypoint-wait-rust-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

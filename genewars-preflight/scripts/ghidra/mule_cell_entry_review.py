"""Literal5B260/73F70 versus pure planner. Environment predicates are explicit ports."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
 def get(self,operand):
  if self.pc==0x5b38a:
   point=tuple(struct.unpack('<hh',bytes(self.load(self.r['EAX']+i,1)for i in range(4))));self.points.append(point)
   self.code[0x74d90]=('MOV',['EAX',str(int(len(self.points)==self.accept_at))])
  return super().get(operand)
x=Source();extra=P/'private/ghidra/krell-cell-entry/instructions-00073f70.txt';x.code.update(m.m.Model([extra],b'').code)
x.code[0x5b271]=('CALL',['0x2e0010']);x.code[0x2e0010]=('XOR',['EAX','EAX']);x.code[0x2e0011]=('RET',[])
x.code[0x5b285]=('CALL',['0x2e0020']);x.code[0x2e0020]=('MOV',['EAX','1']);x.code[0x2e0021]=('RET',[])
x.code[0x74d90]=('MOV',['EAX','0']);x.code[0x74d91]=('RET',[])
assert (P/'private/ghidra/validated/memory-.object1.bin').read_bytes()[0x4b2e5:0x4b2eb]==bytes.fromhex('66a38a871100')
x.code[0x5b2e5]=('MOV',['word ptr [0x0011878a]','AX'])
order=sorted(x.code);x.next=dict(zip(order,order[1:]))
work=P/'private/review/mule-cell-entry'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/mule_cell_entry.rs';h=work/'probe.rs'
h.write_text('extern crate genewars_runtime; pub use genewars_runtime::{actor_activation,blueprint};'+f'#[path="{rust.as_posix()}"]mod entry;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let t=actor_activation::TrigTables::generate_pinned().unwrap();let mut out=Vec::new();for c in b.chunks_exact(134){let r=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[126..128].try_into().unwrap()),ring:c[128],draw_count:u32::from_le_bytes(c[129..133].try_into().unwrap())};let mut n=0;let q=entry::plan_waypoint(c[..124].try_into().unwrap(),u16::from_le_bytes(c[124..126].try_into().unwrap()),r,&t,|_|{n+=1;Ok::<_,()>(n==c[133])}).unwrap();out.extend(q.record_after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());out.push(q.queried_points.len()as u8);for p in q.queried_points{for v in p{out.extend(v.to_le_bytes());}}}std::io::stdout().write_all(&out).unwrap();}""")
target=P/'private/build/rust-target/debug';exe=work/'probe.exe'
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
base=(P/'private/review/krell-mule-callbacks-004/frame-8-slot-1-before/creatures.bin').read_bytes()[124:248]
inputs=bytearray();expected=bytearray();cases=[]
for seed in [1,2,29645]:
 for accept_at in range(9):
  for goal_equal in [False,True]:
   b=bytearray(base);struct.pack_into('<hh',b,7,31988,22528);b[0x5e]=8
   goal=struct.unpack_from('<hh',b,11);goalcell=((goal[1]>>8)&127)*128+((goal[0]>>8)&127);cell=goalcell if goal_equal else(goalcell+1)%16384
   state={0x1542be+i:v for i,v in enumerate(b)};state[0x11872c]=1
   state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',33668))});x.points=[];x.accept_at=accept_at
   report,_,_=x.run(seed=seed,state_before=state,entry=0x5b260,registers={'EAX':0x1542be,'EDX':0x11c2be+cell*14})
   a=bytes(x.load(0x1542be+i,1)for i in range(124));rng=struct.pack('<HBI',x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4))
   inputs+=b+struct.pack('<HHBI',cell,seed,1,33668)+bytes([accept_at]);expected+=a+rng+bytes([len(x.points)])+b''.join(struct.pack('<hh',*pt)for pt in x.points)
   cases.append(dict(seed=seed,accepted_at=accept_at,goal_equal=goal_equal,queries=len(x.points),instructions=report['steps']))
# Exact intermediate record captured at5B271 during full frame8 literal callback.
actual_entry=P/'private/review/mule-cell-entry-actual';b=(actual_entry/'record.bin').read_bytes();context=(actual_entry/'context.bin').read_bytes();cell,seed,ring,draws=struct.unpack('<HHBI',context)
state={0x1542be+i:v for i,v in enumerate(b)};state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))});x.points=[];x.accept_at=1
report,_,_=x.run(seed=seed,state_before=state,entry=0x5b260,registers={'EAX':0x1542be,'EDX':0x11c2be+cell*14})
a=bytes(x.load(0x1542be+i,1)for i in range(124));rng=struct.pack('<HBI',x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4))
assert x.points==[(31782,22375)]
inputs+=b+context+bytes([1]);expected+=a+rng+bytes([1])+struct.pack('<hh',*x.points[0])
cases.append(dict(actual_frame8_intermediate=True,record_sha256=hashlib.sha256(b).hexdigest(),context_sha256=hashlib.sha256(context).hexdigest(),queries=1,instructions=report['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
receipt=dict(status='passed',cases=cases,compared_bytes=len(actual),rust_sha256=hashlib.sha256(rust.read_bytes()).hexdigest(),script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),private_run=str(work.relative_to(P)),source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [extra,P/'private/ghidra/initial-creature/instructions-0005b100.txt',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']},oracle_annotation='Raw66A3 at5B2E5 proves word seed store; unsized listing normalized.',ports=['virtual10 false','virtual8C true','74D90 deterministic accept ordinal; original querypoints preserved'],scope='54 synthetic callback fixtures and exact frame8 intermediate record, literal5B260/73F70. No world adapter/ordinary gameplay claim.')
(P/'research/variants/mule-cell-entry-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status=receipt['status'],cases=len(cases),compared_bytes=len(actual))))

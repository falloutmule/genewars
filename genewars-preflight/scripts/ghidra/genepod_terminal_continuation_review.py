"""Terminal continuations: literal factory-return and failure requests with explicit factory ports."""
from pathlib import Path
import hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
listing=P/'private/ghidra/building-update/instructions-000451f0.txt';B=0x1645ae;F=0x500000;P0=0x118d74;N=0x1542be
class Source(m.Source):
 def load(self,a,n):
  if getattr(self,'pc',0)==0x48790 and not getattr(self,'inside_effect',False):
   self.inside_effect=True;self.effects.append(struct.pack('<3H',*[super(Source,self).load(self.r['ECX']+2*i,2)for i in range(3)]))
  if getattr(self,'pc',0)!=0x48790:self.inside_effect=False
  return super().load(a,n)
work=P/'private/review/genepod-terminal-continuation'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/genepod_terminal.rs';payment=P/'rust/crates/genewars-runtime/src/genepod_positive_work.rs';target=P/'private/build/rust-target/debug'
h=work/'probe.rs';exe=work/'probe.exe'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::blueprint;'+f'#[path="{payment.as_posix()}"]mod genepod_positive_work;#[path="{rust.as_posix()}"]mod t;'+r"""use std::io::{Read,Write};fn main(){let mut bytes=Vec::new();std::io::stdin().read_to_end(&mut bytes).unwrap();let mut out=Vec::new();for c in bytes.chunks_exact(7296){let mut b:[u8;67]=c[..67].try_into().unwrap();let players=std::array::from_fn(|i|c[67+1805*i..67+1805*(i+1)].try_into().unwrap());let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[7287..7289].try_into().unwrap()),ring:c[7289],draw_count:u32::from_le_bytes(c[7290..7294].try_into().unwrap())};let mode=c[7294];let request=if mode==2{t::TerminalRequest::FailedQuality}else{t::TerminalRequest::Factory{owner:0,species:4,quality:35,position:[1234,5678],sound:26}};let q=t::TerminalContinuation{building_staged:b,players_staged:players,rng_after:rng,payer:0,quality:35,roll:0,request};let mut angles=Vec::new();let q=if mode<2{let q=t::plan_factory_return(&q,if mode==0{Some(0)}else{None},&q.players_staged,rng).unwrap();if let Some(e)=q.ejection{angles.extend(e.yaw.to_le_bytes());angles.extend(e.pitch.to_le_bytes());angles.extend(e.speed.to_le_bytes());}q}else{let mut f=t::begin_failure(&q).unwrap();assert!(t::finish_failure(&f,f.rng_after).is_err());while f.remaining>0{let(next,e)=t::next_failure_effect(&f,f.rng_after).unwrap();assert_eq!(e.code,32);assert_eq!(e.position,[1234,5678]);assert_eq!(e.physical_height,5840);angles.extend(e.yaw.to_le_bytes());angles.extend(e.pitch.to_le_bytes());angles.extend(e.speed.to_le_bytes());f=next;}t::finish_failure(&f,f.rng_after).unwrap()};out.extend(q.building_staged);for p in q.players_staged{out.extend(p);}out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());out.push((angles.len()/6)as u8);out.extend(angles);}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for mode in [0,1,2]:
 for grade in [0,1,2]:
  for seed in [1,5428,65535]:
   b=bytearray(67);b[6]=4;b[46]=grade;b[49]=5;b[51]=4;b[27]=1;struct.pack_into("<I",b,35,0x600000);struct.pack_into('<hh',b,7,1234,5678);struct.pack_into('<i',b,17,2000);players=bytearray(7220);struct.pack_into('<i',players,0x62a,71680);state={B+i:v for i,v in enumerate(b)};state.update({P0+i:v for i,v in enumerate(players)});state.update({N+i:0 for i in range(124)})
   for a,v in [(F-0x26,B),(F-0x22,P0),(B+35,0x600000),(0x600044,0x900000),(0x118782,100)]:state.update({a+i:v for i,v in enumerate(struct.pack('<I',v))})
   state[0x11872c]=31;vm=Source();vm.effects=[];vm.code.update(m.m.Model([listing],b'').code);vm.code[0x45d8e]=('JMP',['0xdeadbeef']);vm.code[0x900000]=('RET',['0x8']);vm.code[0x48790]=('RET',['0x10']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
   r,_,trace=vm.run(seed=seed,state_before=state,entry=0x456c8 if mode<2 else 0x459e8,registers={'EBP':F,'EAX':N if mode==0 else 0},instruction_budget=20000)
   angles=bytes(vm.load(N+57+i,1)for i in range(6))if mode==0 else b''.join(vm.effects)if mode==2 else b''
   inputs+=b+players+struct.pack('<HBIBB',seed,31,100,mode,0);expected+=bytes(vm.load(B+i,1)for i in range(67))+bytes(vm.load(P0+i,1)for i in range(7220))+struct.pack('<HBIB',vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4),len(angles)//6)+angles;cases.append(dict(mode=mode,grade=grade,seed=seed,effects=len(vm.effects),draws=r['draws']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:15]
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,payment,listing,Path(__file__)]},scope='456C8 success/null factory continuation +459E8 failure burst,67building4playersRNGandangle requests. Factory return supplied;48790 explicitzero-draw request port; realfactoryRNG must be passed between requests. Ejectionvelocity/world patch and audio consumption caller obligations; no gameplay acceptance.')
(P/'research/variants/genepod-terminal-continuation-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

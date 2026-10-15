"""Final paid work and exact quality decision, explicit continuation boundaries."""
from pathlib import Path
import hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
listing=P/'private/ghidra/building-update/instructions-000451f0.txt';vm=m.Source();vm.code.update(m.m.Model([listing],b'').code)
for a in [0x45d8e,0x4568f,0x459e8]:vm.code[a]=('JMP',['0xdeadbeef'])
order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));B=0x1645ae;F=0x500000;P0=0x118d74
work=P/'private/review/genepod-terminal'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/genepod_terminal.rs';payment=P/'rust/crates/genewars-runtime/src/genepod_positive_work.rs';h=work/'probe.rs';exe=work/'probe.exe';target=P/'private/build/rust-target/debug'
h.write_text('extern crate genewars_runtime;pub use genewars_runtime::blueprint;'+f'#[path="{payment.as_posix()}"]mod genepod_positive_work;#[path="{rust.as_posix()}"]mod terminal;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(7298){let players=std::array::from_fn(|i|c[67+i*1805..67+(i+1)*1805].try_into().unwrap());let rng=blueprint::BlueprintRng{seed:u16::from_le_bytes(c[7291..7293].try_into().unwrap()),ring:c[7293],draw_count:u32::from_le_bytes(c[7294..].try_into().unwrap())};let q=terminal::plan_terminal(c[..67].try_into().unwrap(),&players,u32::from_le_bytes(c[7287..7291].try_into().unwrap()),rng);if std::env::args().any(|a|a=="reject"){assert!(q.is_err());out.push(1);continue;}let (a,p,r,status)=match q.unwrap(){terminal::TerminalPlan::Waiting(w)=>(w.building_after,w.players_after,rng,0),terminal::TerminalPlan::Continue(q)=>{let status=match q.request{terminal::TerminalRequest::Factory{owner,species,quality,position,sound}=>{assert_eq!(owner,q.payer);assert_eq!(species,4);assert_eq!(quality,q.quality as u8);assert_eq!(sound,26);assert_eq!(position,[1234,5678]);1},terminal::TerminalRequest::FailedQuality=>2};(q.building_staged,q.players_staged,q.rng_after,status)}};out.extend(a);for p in p{out.extend(p);}out.extend(r.seed.to_le_bytes());out.push(r.ring);out.extend(r.draw_count.to_le_bytes());out.push(status);}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
params=[(5,0,grade,10,0,quality,seed)for grade in [0,2]for quality in [-1,0,35,100,150]for seed in [0,1,5428,65535]]+[(1,1,0,10,0,35,5428),(5,0,0,0,0,35,5428),(5,0,0,10,1,35,5428)]
for flags,frame,grade,funds,payer,quality,seed in params:
 b=bytearray(67);b[6]=4;b[46]=grade;b[49]=flags;b[51]=4;b[58]=payer;struct.pack_into('<hh',b,7,1234,5678);struct.pack_into('<i',b,54,1);players=bytearray(7220);struct.pack_into('<i',players,0x4a3,funds);struct.pack_into('<i',players,payer*1805+0x62a,quality<<11)
 if payer:struct.pack_into('<i',players,payer*1805+0x4a3,9);players[payer*1805+0x4a1]=payer
 state={B+i:v for i,v in enumerate(b)};state.update({P0+i:v for i,v in enumerate(players)});state[0x11872c]=31
 for address,value in [(F-0x26,B),(F-0x16,P0+0x4a1),(0x118782,0xffffffff)]:state.update({address+i:v for i,v in enumerate(struct.pack('<I',value))})
 r,_,trace=vm.run(counter=frame,seed=seed,state_before=state,entry=0x45470,registers={'EBP':F});status=1 if 0x4568f in trace else 2 if 0x459e8 in trace else 0
 inputs+=b+players+struct.pack('<IHBI',frame,seed,31,0xffffffff);expected+=bytes(vm.load(B+i,1)for i in range(67))+bytes(vm.load(P0+i,1)for i in range(7220))+struct.pack('<HBIB',vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4),status);cases.append(dict(grade=grade,quality=quality,seed=seed,status=status,payer=payer,draws=r['draws']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//7295,i%7295,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12]
negative=bytearray()
for off,value in [(6,3),(45,1),(46,3),(51,5),(54,2),(7293,32)]:
 a=bytearray(inputs[:7298]);a[off]=value;negative+=a
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*6
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),negative_cases=6,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,payment,listing,Path(__file__)]},scope='45470 terminalpayment plus455C2 quality decision to4568F/459E8; waiting and bothqualityoutcomes. Continuation is not commitready; factory/ejection/failureburst/research/common tail pending. No gameplay acceptance.')
(P/'research/variants/genepod-terminal-prefix-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

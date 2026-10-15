"""Nonterminal positive work: full building/players versus literal45470."""
from pathlib import Path
import hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
listing=P/'private/ghidra/building-update/instructions-000451f0.txt';vm=m.Source();vm.code.update(m.m.Model([listing],b'').code);vm.code[0x45d8e]=('JMP',['0xdeadbeef']);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));B=0x1645ae;F=0x500000;P0=0x118d74
work=P/'private/review/genepod-positive-work'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/genepod_positive_work.rs';h=work/'probe.rs';exe=work/'probe.exe'
h.write_text(f'#[path="{rust.as_posix()}"]mod work;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(7291){let players=std::array::from_fn(|i|c[67+i*1805..67+(i+1)*1805].try_into().unwrap());let q=work::plan_work(c[..67].try_into().unwrap(),&players,u32::from_le_bytes(c[7287..].try_into().unwrap()));if std::env::args().any(|a|a=="terminal"){assert_eq!(q,Err(work::WorkError::TerminalBoundary));out.push(1);continue;}if std::env::args().any(|a|a=="reject"){assert!(q.is_err());out.push(1);continue;}let q=q.unwrap();out.extend(q.building_after);for p in q.players_after{out.extend(p);}}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True)
def put(a,i,v):struct.pack_into('<i',a,i,v)
inputs=bytearray();expected=bytearray();cases=[]
params=[(flags,frame,grade,primary,secondary,0,0)for flags in [1,5]for frame in [0,1,6]for grade in [0,1,2]for primary,secondary in [(0,0),(0,20),(3,20),(-1,20),(2147483647,1)]]
params +=[(5,1,grade,10,20,payer,ai)for grade in [0,1,2]for payer,ai in [(1,0),(1,1),(0,1)]]
for flags,frame,grade,primary,secondary,payer,ai in params:
 b=bytearray(67);b[6]=4;b[46]=grade;b[49]=flags;b[51]=4;b[58]=payer;put(b,54,2);players=bytearray(7220);put(players,0x4a3,primary);put(players,0x4a7,secondary);players[2]=ai
 if payer:put(players,1805+0x4a3,7);players[1805+0x4a1]=1;players[1805+2]=ai
 state={B+i:v for i,v in enumerate(b)};state.update({P0+i:v for i,v in enumerate(players)})
 for address,value in [(F-0x26,B),(F-0x16,P0+0x4a1)]:state.update({address+i:v for i,v in enumerate(struct.pack('<I',value))})
 r,_,_=vm.run(counter=frame,state_before=state,entry=0x45470,registers={'EBP':F});assert r['draws']==0
 inputs+=b+players+struct.pack('<I',frame);expected+=bytes(vm.load(B+i,1)for i in range(67))+bytes(vm.load(P0+i,1)for i in range(7220));cases.append(dict(flags=flags,frame=frame,grade=grade,primary=primary,secondary=secondary,payer=payer,owner_ai=ai))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//7287,i%7287,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12]
terminal=bytearray(inputs[-7291:]);struct.pack_into('<i',terminal,54,1);assert subprocess.run([str(exe),'terminal'],input=terminal,capture_output=True,check=True).stdout==b'\1'
negative=bytearray()
for off,value in [(6,3),(45,1),(46,3),(49,0),(49,0x41),(50,1),(51,5),(58,4)]:
 a=bytearray(terminal);a[2]=0;a[67+2]=0;a[off]=value;struct.pack_into('<i',a,54,2);negative+=a
assert subprocess.run([str(exe),'reject'],input=negative,capture_output=True,check=True).stdout==bytes([1])*8
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),terminal_boundary_cases=1,unsupported_cases=8,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,listing,Path(__file__)]},source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in vm.files},scope='Nonterminal45470 workpayment full67building+4players. No sharedpowerprefix/tail or terminalquality/factory; terminal returnsnoafterimage. No gameplay acceptance.')
(P/'research/variants/genepod-positive-work-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

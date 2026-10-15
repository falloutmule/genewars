"""Full451F0 grade0 nonterminal work, exact BGENE animation and shared prefix/tail."""
from pathlib import Path
import hashlib,importlib.util,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
listing=P/'private/ghidra/building-update/instructions-000451f0.txt';vm=m.Source();vm.code.update(m.m.Model([listing],b'').code);vm.code[0x47f34]=('CALL',['0x2e0010']);vm.code[0x2e0010]=('RET',[]);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
anb_path=P/'private/integration/game-runtime/buildings/BGENE.ANB';anb=bytearray(anb_path.read_bytes());frames,animations=struct.unpack_from('<II',anb,10)
for i in range(5):
 for off in [0,4]:struct.pack_into('<I',anb,animations+i*10+off,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+off)[0])
base={0x620000+i:b for i,b in enumerate(anb)};B=0x1645ae;P0=0x118d74
work=P/'private/review/genepod-work-callback'/uuid.uuid4().hex;work.mkdir(parents=True);src=P/'rust/crates/genewars-runtime/src';rust=src/'genepod_work_callback.rs';h=work/'probe.rs';exe=work/'probe.exe'
h.write_text(''.join(f'#[path="{(src/(name+".rs")).as_posix()}"]mod {name};'for name in ['genepod_idle','genepod_positive_work','genepod_work_callback'])+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(7291){let p=std::array::from_fn(|i|c[67+1805*i..67+1805*(i+1)].try_into().unwrap());let q=genepod_work_callback::plan_completed_work(c[..67].try_into().unwrap(),&p,u32::from_le_bytes(c[7287..].try_into().unwrap()));if std::env::args().any(|a|a=="terminal"){assert_eq!(q,Err(genepod_work_callback::GenepodWorkError::Work(genepod_positive_work::WorkError::TerminalBoundary)));out.push(1);continue;}let q=q.unwrap();assert!(q.queue_requested);out.extend(q.building_after);for p in q.players_after{out.extend(p);}}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(h),'-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for power in [0,1,200,32767,-32768]:
 for frame in [0,1,6]:
  for funds,payer,ai in [(0,0,0),(10,0,0),(10,1,0),(10,0,1)]:
   b=bytearray(67);b[6]=4;b[49]=5;b[51]=4;b[58]=payer;b[63]=2;b[25]=1;b[27]=2;b[26]=3;b[29]=1;struct.pack_into('<h',b,61,power);struct.pack_into('<i',b,54,3);p=bytearray(7220);p[2]=ai
   for i in range(4):p[1805*i+1]=i;p[1805*i+0x4a1]=i
   struct.pack_into('<i',p,0x4a3,funds);struct.pack_into('<i',p,1805+0x4a3,8)
   native=bytearray(b);struct.pack_into('<II',native,31,0x620000+animations,0x112120);state=base.copy();state.update({B+i:v for i,v in enumerate(native)});state.update({P0+i:v for i,v in enumerate(p)})
   r,_,trace=vm.run(counter=frame,state_before=state,entry=0x451f0,registers={'EAX':B});aa=bytearray(vm.load(B+i,1)for i in range(67));aa[31:39]=b[31:39];assert r['draws']==0
   inputs+=b+p+struct.pack('<I',frame);expected+=aa+bytes(vm.load(P0+i,1)for i in range(7220));cases.append(dict(power=power,frame=frame,funds=funds,payer=payer,ai=ai,instructions=r['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i//7287,i%7287,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:15]
terminal=bytearray(inputs[-7291:]);struct.pack_into('<i',terminal,54,1);struct.pack_into('<h',terminal,61,200);assert subprocess.run([str(exe),'terminal'],input=terminal,capture_output=True,check=True).stdout==b'\1'
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),terminal_boundary_cases=1,private_run=str(work.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [rust,src/'genepod_idle.rs',src/'genepod_positive_work.rs',listing,anb_path,Path(__file__)]},scope='Full451F0 grade0positivework>1 callback, powerexpiry/wrapping/cap,framecadence,funds,distinctpayer,AI fundsfloor,BGENEanimation,all4players,0RNG. Renderqueue explicitport. Terminal noafterimage; no missionacceptance.')
(P/'research/variants/genepod-work-callback-rust-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=len(actual))))

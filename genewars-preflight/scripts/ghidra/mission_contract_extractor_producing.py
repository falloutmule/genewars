"""Whole451F0 admitted producing Extractor callbacks versus Rust typed requests."""
from pathlib import Path
import hashlib, importlib.util, json, random, struct, subprocess, uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
x=m.Source();listing=P/'private/ghidra/building-update/instructions-000451f0.txt';x.code.update(m.m.Model([listing],b'').code)
# Explicit audio-service query returns false; queue/audio callbacks are ports.
x.code[0x97830]=('XOR',['EAX','EAX']);x.code[0x97831]=('RET',[])
for pc in [0x46bac,0x47e34,0x47f34,0x47098]:x.code[pc]=('CALL',['0x2e0010'])
for pc in [0x46bc3,0x46c9d,0x470af,0x470d9]:x.code[pc]=('CALL',['0x2e0020'])
for pc in [0x8b000,0x8b080,0x7b140]:x.code[pc]=('XOR',['EAX','EAX']);x.code[pc+1]=('RET',[])
for pc in [0x8cba0,0x84e10]:x.code[pc]=('RET',[])
x.code[0x48790]=('RET',['0x10'])
for pc in [0x46d55,0x46dc3,0x46e33]:x.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
x.code[0x2e0010]=('RET',[]);x.code[0x2e0020]=('RET',['0x8']);order=sorted(x.code);x.next=dict(zip(order,order[1:]))
anb_path=P/'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BMINE.ANB';anb=bytearray(anb_path.read_bytes());frames,animations=struct.unpack_from('<II',anb,10)
assert struct.unpack_from('<IIBB',anb,animations+10)==(42,308,20,0)
assert [anb[frames+42+i*14+10]for i in range(20)]==[int(i==12)for i in range(20)]
for i in range(5):
 for off in [0,4]:struct.pack_into('<I',anb,animations+i*10+off,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+off)[0])
base={0x620000+i:b for i,b in enumerate(anb)};rng=random.Random(0x451f011);inputs=bytearray();expected=bytearray();steps=0
for case in range(64):
 b=bytearray(rng.randbytes(67));p=bytearray(rng.randbytes(1805));owner=case%4;b[6]=4;b[24]=owner;b[45]=11;b[46]=0;b[49:51]=struct.pack('<H',1+(case%2)*4);b[27]=[7,8,9,10][case%4]
 b[51]=case%239;b[63]=case%3
 struct.pack_into('<h',b,61,[0,1,2,200,32767,-32768][case%6]);p[1]=p[0x4a1]=owner
 struct.pack_into('<I',p,0x4a7,[0,599,600,0xffffffff][case%4]);struct.pack_into('<I',p,0x5e6,600)
 native=bytearray(b);struct.pack_into('<II',native,31,0x620000+animations,0x112120)
 resources=bytes([1+case%8])*16384
 state=base.copy();state.update({0x11c2be+i*14+13:v for i,v in enumerate(resources)});state.update({0x1645ae+i:v for i,v in enumerate(native)});state.update({0x118d74+owner*1805+i:v for i,v in enumerate(p)})
 report,_,_=x.run(entry=0x451f0,state_before=state,registers={'EAX':0x1645ae})
 after=bytearray(x.load(0x1645ae+i,1)for i in range(67));after[31:39]=b[31:39]
 expected+=after+bytes(x.load(0x118d74+owner*1805+i,1)for i in range(1805))+bytes(x.load(0x11c2be+i*14+13,1)for i in range(16384));inputs+=b+p+resources;steps+=report['steps']
work=P/'private/review/extractor-producing'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/extractor_production.rs';h=work/'review.rs'
h.write_text(f'#[path="{rust.as_posix()}"]mod extractor;'+r'''use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(18256){let mut map=vec![0;229376];for (i,v) in c[1872..].iter().enumerate(){map[i*14+13]=*v;}let q=extractor::plan_completed_with_requests(c[..67].try_into().unwrap(),c[67..1872].try_into().unwrap(),false,&map,1,5428,0,0).unwrap();out.extend(q.state.building_after);out.extend(q.state.player_after);if let Some(h)=q.harvest{if let Some(r)=h.resource{map[r.cell as usize*14+13]=r.after;}}for i in 0..16384{out.push(map[i*14+13]);}}std::io::stdout().write_all(&out).unwrap();}''')
s=importlib.util.spec_from_file_location('cargo_local',P/'scripts/cargo.py');cargo=importlib.util.module_from_spec(s);s.loader.exec_module(cargo);exe=work/'review.exe';compiler=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'
subprocess.run([str(compiler),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],env=cargo.environment(),check=True)
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12]
receipt={'status':'passed-producing-extractor-with-ports','cases':64,'compared_bytes':len(actual),'literal_instructions':steps,'rust_sha256':hashlib.sha256(rust.read_bytes()).hexdigest(),'listing_sha256':hashlib.sha256(listing.read_bytes()).hexdigest(),'resource_sha256':hashlib.sha256(anb_path.read_bytes()).hexdigest(),'ports':['audio service queryfalse','audio requests','render queue','height sample','effect6 factory','empty enemy and own kind1 queries','terrain depletion8CBA0','map dirty84E10'],'exclusions':['empty-resource shutdown','occupied contact cell','terrain depletion mutations','effect6 allocation mutations','disabled/destruction/repair states','whole-world scheduler']}
(P/'research/campaign/extractor-producing-callback.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))

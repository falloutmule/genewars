"""Literal CD46CBA..46F0C contact/RNG requests versus production Rust."""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,subprocess,uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
x=m.Source();listing=P/'private/ghidra/building-update/instructions-000451f0.txt';x.code.update(m.m.Model([listing],b'').code)
x.code[0x46f0c]=('RET',[])
for pc in [0x8b000,0x8b080,0x7b140]:x.code[pc]=('XOR',['EAX','EAX']);x.code[pc+1]=('RET',[])
x.code[0x48790]=('RET',['0x10'])
# Ghidra unsized absolute stores have literal66 prefix and are word-sized.
for pc in [0x46d55,0x46dc3,0x46e33]:x.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
order=sorted(x.code);x.next=dict(zip(order,order[1:]));rng=random.Random(0x46cba);inputs=bytearray();expected=bytearray();steps=0
for case in range(128):
 b=bytearray(rng.randbytes(67));b[6]=4;b[45]=11;b[46]=case%3;b[24]=case%4
 seed=rng.randrange(65536);ring=case%32;counter=case;draws=(0xfffffffd+case)&0xffffffff;base=0x1645ae;frame=0x2fff00
 state={base+i:v for i,v in enumerate(b)}
 def put(at,v,n=4):state.update({at+i:q for i,q in enumerate(v.to_bytes(n,'little'))})
 put(frame-0x26,base);put(0x11872c,ring,1);put(0x118782,draws)
 report,_,_=x.run(counter=counter,seed=seed,entry=0x46cba,state_before=state,registers={'EDX':base,'EBP':frame})
 pos=bytes(x.load(frame+0x1e+i,1)for i in range(4));px,py=struct.unpack('<hh',pos);cell=((py>>8)&127)*128+((px>>8)&127)
 smoke=(counter+b[15])&1;angles=bytes(x.load(frame-0x5a+i,1)for i in range(6))if smoke else bytes(6)
 expected+=pos+struct.pack('<HBHB',cell,b[24],b[46]*8,smoke)+angles+struct.pack('<HBI',report['rng_seed_after'],report['rng_ring_after'],report['rng_draw_count_after'])
 inputs+=b+struct.pack('<IHB I',counter,seed,ring,draws);steps+=report['steps']
work=P/'private/review/extractor-contact'/uuid.uuid4().hex;work.mkdir(parents=True);rust=P/'rust/crates/genewars-runtime/src/extractor_production.rs';h=work/'review.rs'
h.write_text(f'#[path="{rust.as_posix()}"]mod extractor;'+r'''use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(78){let q=extractor::plan_contact_pass(c[..67].try_into().unwrap(),u32::from_le_bytes(c[67..71].try_into().unwrap()),u16::from_le_bytes(c[71..73].try_into().unwrap()),c[73],u32::from_le_bytes(c[74..78].try_into().unwrap())).unwrap();for v in q.position{out.extend(v.to_le_bytes());}out.extend(q.cell.to_le_bytes());out.push(q.owner);out.extend(q.damage.to_le_bytes());out.push(q.smoke_angles.is_some()as u8);for v in q.smoke_angles.unwrap_or([0;3]){out.extend(v.to_le_bytes());}out.extend(q.seed_after.to_le_bytes());out.push(q.ring_after);out.extend(q.draw_count_after.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}''')
s=importlib.util.spec_from_file_location('cargo_local',P/'scripts/cargo.py');cargo=importlib.util.module_from_spec(s);s.loader.exec_module(cargo);exe=work/'review.exe';compiler=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'
subprocess.run([str(compiler),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],env=cargo.environment(),check=True)
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12]
r={'status':'passed-contact-request-planner','cases':128,'compared_bytes':len(actual),'literal_instructions':steps,'rust_sha256':hashlib.sha256(rust.read_bytes()).hexdigest(),'listing_sha256':hashlib.sha256(listing.read_bytes()).hexdigest(),'ports':['height sample','effect6 allocation','ordered enemy8B000 then own8B080 queries returning empty'],'unverified':['occupied contact cell predicates and damage','factory allocation side effects','whole-world callback composition']}
(P/'research/campaign/extractor-contact-pass.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))

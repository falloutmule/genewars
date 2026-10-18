"""Bounded literal6DF30 for captured stationary code31 phase1 effect."""
from pathlib import Path
import importlib.util,hashlib,json,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
fixture=P/'private/review/krell-pre-effects-294-001';backing=(fixture/'backing.bin').read_bytes();base=(fixture/'effects.bin').read_bytes()[:81]
x=m.Source(backing);work=P/'private/review/landing-phase1-stationary'/uuid.uuid4().hex;work.mkdir(parents=True)
rust=P/'rust/crates/genewars-runtime/src/landing.rs';h=work/'probe.rs';exe=work/'probe.exe'
h.write_text(f'#[path="{rust.as_posix()}"]mod landing;'+r"""use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(90){let q=landing::plan_landing_phase1_rest_physics(c[..81].try_into().unwrap(),i32::from_le_bytes(c[81..85].try_into().unwrap()),c[85],i32::from_le_bytes(c[86..90].try_into().unwrap()));if std::env::args().any(|a|a=="--reject"){assert!(q.is_err());out.push(1);}else{out.extend(q.unwrap().after);}}std::io::stdout().write_all(&out).unwrap();}""")
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],env=environment(),check=True)
inputs=bytearray();expected=bytearray();cases=[]
for gap in [0,1,14,256,4096]:
 for yaw in [0,462,2047,-1]:
  for fill in [0,255]:
   b=bytearray(base);struct.pack_into('<i',b,39,5362+gap);struct.pack_into('<h',b,57,yaw);struct.pack_into('<h',b,59,123);struct.pack_into('<h',b,61,456)
   native=bytearray(b);struct.pack_into('<I',native,35,0x112254)
   state={0x11c2be+i:v for i,v in enumerate(backing[0x355e:0x3b55e])};state.update({0x1688ae+i:v for i,v in enumerate(native)})
   seed,ring,draws=struct.unpack('<HBI',(fixture/'rng.bin').read_bytes());state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
   report,_,trace=x.run(seed=seed,state_before=state,entry=0x6df30,registers={'EAX':0x1688ae},counter=294,stack_fill=fill)
   after=bytearray(x.load(0x1688ae+i,1)for i in range(81));after[31:39]=b[31:39]
   assert report['draws']==0
   inputs+=b+struct.pack('<iBi',5362,base[23],1632);expected+=after;cases.append(dict(gap=gap,yaw=yaw,stack_fill=fill,instructions=report['steps']))
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
rejected=bytearray();rejection_names=[]
for name,offset,value in [('velocity',45,1),('acceleration',51,1),('fraction',67,1),('freeflight',21,0),('collision-height',63,0),('prior-ground',17,0),('phase',70,2),('animation',26,255),('owner',24,4)]:
 b=bytearray(base);b[offset]=value;rejected+=b+struct.pack('<iBi',5362,base[23],1632);rejection_names.append(name)
b=bytearray(base);struct.pack_into('<i',b,39,5361);rejected+=b+struct.pack('<iBi',5362,base[23],1632);rejection_names.append('below-ground')
rejected+=base+struct.pack('<iBi',5362,base[23],5042);rejection_names.append('water-boundary')
rejection_output=subprocess.run([str(exe),'--reject'],input=rejected,capture_output=True,check=True).stdout
assert rejection_output==bytes([1])*len(rejection_names)
(work/'inputs.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,rejected_boundaries=rejection_names,compared_bytes=len(actual),rust_sha256=hashlib.sha256(rust.read_bytes()).hexdigest(),script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),fixture_pins={f.name:hashlib.sha256(f.read_bytes()).hexdigest()for f in fixture.glob('*.bin')},source_pins={str(f.relative_to(P)):hashlib.sha256(f.read_bytes()).hexdigest()for f in x.files},private_run=str(work.relative_to(P)),scope='Numeric6DF30 whole81byte record, capturedKrellmap, gap0/1/14/256/4096 and4yaws, stackfillindependence. No fullphase1timer/factory proof.')
(P/'research/variants/landing-phase1-stationary-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status=r['status'],cases=len(cases),compared_bytes=len(actual))))

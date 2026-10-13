from pathlib import Path
import importlib.util,struct,hashlib,json,subprocess,uuid
p=Path(__file__).resolve().parents[2];s=importlib.util.spec_from_file_location('m',p/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Source(m.Source):
 def get(self,o):
  if self.pc==0x58652 and o=='BL':self.gates.append({'BL':self.r['EBX']&255,'EAX_before':self.r['EAX']})
  if self.pc==0x58751 and o=='0x00048790':self.dust+=1;return 0x2e0010
  if self.pc==0x5876c:self.audio+=1;return 0x2e0008
  return super().get(o)
expected_pins={'private/ghidra/actor-motion/instructions-00057a10.txt': '01c1e7947bedc42beae3087e94c62840c1d31efd3319f76a7891fecc89a5c50a', 'private/ghidra/actor-motion/instructions-000526c0.txt': 'a6b39ca6b310a62ecb36fdfd08f205ae48e83bde37f949df96023277761c57c2', 'private/ghidra/actor-motion/instructions-0001ee30.txt': '3590a299b0f529da8f941168d806f9a5a8edbcc2365e36192d863a006d34d539', 'private/ghidra/activation-rng-refs/instructions-00052780.txt': '048be1420c3c58c6e56a11954c565c2f3ea254710600c635d77605020d103a20', 'private/ghidra/display-geometry/instructions-00083140.txt': '9d18112e4f02934ede585142dda83c1ea16af0b4f4bbd51acd09226c08f28c52'}
for name,digest in expected_pins.items():assert hashlib.sha256((p/name).read_bytes()).hexdigest()==digest,name
x=Source();names=['actor-motion/instructions-00057a10.txt','actor-motion/instructions-000526c0.txt','actor-motion/instructions-0001ee30.txt','activation-rng-refs/instructions-00052780.txt','display-geometry/instructions-00083140.txt'];files=[p/'private/ghidra'/n for n in names];x.code.update(m.m.Model(files,b'').code);x.code[0x57a3e]=('CALL',['0x2e0000']);x.code[0x58274]=('CALL',['0x2e0000']);x.code[0x2e0000]=('RET',[]);x.code[0x2e0010]=('RET',['0x10']);x.code[0x2e0008]=('RET',['0x8']);order=sorted(x.code);x.next=dict(zip(order,order[1:]));rows=[]
inputs=bytearray();expected=bytearray()
for case in range(144):
 state_id=[1,8,9][case//48];flags=[1,0x11,0x41,0x81,0x91,0x101][case//8%6];moving=state_id==8;a=bytearray(200);a[6]=3;a[0x55]=state_id;a[0x58]=2 if moving else 1 if state_id==9 else 0;struct.pack_into("<H",a,0x70,flags);a[0x49]=16;a[0x1c]=1;a[0x19]=0 if moving else 8;struct.pack_into('<hh',a,7,16384,15872);struct.pack_into('<hh',a,11,17000,15872);struct.pack_into('<i',a,17,6272);struct.pack_into('<i',a,39,6272);struct.pack_into('<h',a,45,100);struct.pack_into('<h',a,47,20);struct.pack_into('<h',a,57,(case%8)*256);struct.pack_into('<h',a,65,80);struct.pack_into('<h',a,63,960);a[35:39]=(0x112194).to_bytes(4,'little');a[192:196]=(0x2d0000).to_bytes(4,'little');a[31:35]=(0x2c0000).to_bytes(4,'little');bp=bytes(36);state={0x16360e+i:b for i,b in enumerate(a)};state.update({0x2d0000+i:b for i,b in enumerate(bp)});state.update({0x11c2bc:0x70,0x11c2bd:0x17,0x118d76:0,0x118968:1,0x180d80:0})
 for animation in range(16):
  descriptor=struct.pack('<IIBB',0x2c1000+animation*28,0x2c1000+animation*28+14,2,0);state.update({0x2c0000+animation*10+i:b for i,b in enumerate(descriptor)});state[0x2c1000+animation*28+10]=0;state[0x2c1000+animation*28+24]=0
 x.gates=[];x.dust=x.audio=0;report,_,_=x.run(counter=3,entry=0x57a10,state_before=state,registers={'EAX':0x16360e});rows.append({'case':case,'state':a[0x55],'flag70':a[0x70],'yaw':(case%8)*256,'gate':x.gates,'dust':x.dust,'audio':x.audio,'steps':report['steps']})
 inputs.extend(a);expected.extend(bytes([x.gates[0]['BL'] if x.gates else 255,x.dust]));assert x.audio==x.dust
w=p/'private/ghidra/actor-footstep'/uuid.uuid4().hex;w.mkdir(parents=True)
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix()
h=w/'proof.rs';h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for raw in b.chunks_exact(200){let a=raw.try_into().unwrap();out.push(actor_motion::actor_selected_animation_state(&a).unwrap());out.push(actor_motion::plan_actor_footstep(&a,3,7).unwrap().is_some() as u8);}std::io::stdout().write_all(&out).unwrap();}')
rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';exe=w/'proof.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True)
out=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert out==expected,[(i,list(out[2*i:2*i+2]),list(expected[2*i:2*i+2]))for i in range(144)if out[2*i:2*i+2]!=expected[2*i:2*i+2]]
r={'scope':'Literal whole57A10 with explicit post6FB60 port and construction-stage port; actual animation selector,7AC00,virtual4; factory/audio request ports. Production selected BL and dust requests compared.','cases':len(rows),'compared_bytes':len(out),'literal_instructions':sum(q['steps']for q in rows),'dust_requests':sum(q['dust']for q in rows),'source_pins':{f.relative_to(p).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in files},'production_sha256':hashlib.sha256(Path(module).read_bytes()).hexdigest(),'output_sha256':hashlib.sha256(out).hexdigest()}
(p/'research/ghidra/actor-footstep-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

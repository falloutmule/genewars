"""Bounded source-transcribed48790 null-associated effect20 factory comparison."""
from pathlib import Path
import ast,struct,random,subprocess,hashlib,json,uuid,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
w=P/'private/review/effect20-factory'/uuid.uuid4().hex;w.mkdir(parents=True)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
mem=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes();assert mem[0x4eb4+32*6:0x4eb4+33*6]==bytes([0,1,42,255,0,0])
tree=ast.parse((P/'scripts/ghidra/validate_landing_bridge.py').read_text());scope={'struct':struct};exec(compile(ast.Module(body=[x for x in tree.body if isinstance(x,ast.FunctionDef)and x.name=='factory'],type_ignores=[]),'factory','exec'),scope)
# The existing constructor transcription is independent of the Rust common factory.
cos=struct.unpack_from('<2048i',mem,0x86e8);sin=struct.unpack_from('<2048i',mem,0x7ee8)
r=random.Random(32);inputs=bytearray();expected=bytearray();cases=[]
for i in range(128):
 a=bytearray(r.randbytes(81));a[6]=0;angles=(8*(i%256),2*(i%150),8192+64*(i%128));inputs+=a+struct.pack('<3H',*angles)
 b=bytearray(scope['factory'](a,0,0,[20736,13568],6272,40));struct.pack_into('<i',b,39,10112);struct.pack_into('<H',b,21,0);b[69]=32;b[73]=0;b[76]=a[76];struct.pack_into('<3H',b,57,*angles)
 yaw,pitch,speed=angles;yi=(yaw-512)&2047;pi=(pitch-512)&2047;horizontal=cos[pi]*speed>>16
 for off,val in [(45,cos[yi]*horizontal>>16),(47,sin[yi]*horizontal>>16),(49,-sin[pi]*speed>>16)]:struct.pack_into('<H',b,off,val&65535)
 expected+=b
module=P/'rust/crates/genewars-runtime/src/effect20_factory.rs';h=w/'probe.rs';h.write_text('use genewars_runtime::{landing,water_effect};\n#[path="'+module.as_posix()+'"]mod effect20_factory;\n'+r'use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let t=genewars_runtime::actor_activation::TrigTables::generate_pinned().unwrap();let mut out=Vec::new();for c in b.chunks_exact(87){let mut pool=[[0;81];200];pool[0].copy_from_slice(&c[..81]);let angles=core::array::from_fn(|i|u16::from_le_bytes([c[81+i*2],c[82+i*2]]));let p=effect20_factory::plan_factory(&pool,0,0,[20736,13568],6272,40,10112,angles,None,t.sine().try_into().unwrap(),t.cosine().try_into().unwrap()).unwrap();assert_eq!(p.animation_index,32);out.extend(p.after_nonpointer);}std::io::stdout().write_all(&out).unwrap();}')
exe=w/'probe.exe';target=P/'private/build/rust-target/debug';subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True)
actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:10]
(w/'inputs.bin').write_bytes(inputs);(w/'expected.bin').write_bytes(expected)
paths=[module,Path(__file__),P/'private/ghidra/activation-rng-refs/instructions-00048790.txt',P/'scripts/ghidra/validate_landing_bridge.py']
receipt=dict(status='passed',cases=128,compared_bytes=len(actual),private_run=str(w.relative_to(P)),pins={str(p.relative_to(P)):sha(p)for p in paths},scope='Source-transcribed common constructor/48790 velocity and selector20 null-associated branch,128 prior-byte/angle variations. Not a literal execution comparison or full-world factory/lifecycle proof. Full pool replacement unsupported; no gameplay acceptance.')
(P/'research/variants/effect20-factory-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','cases':128,'compared_bytes':len(actual)}))

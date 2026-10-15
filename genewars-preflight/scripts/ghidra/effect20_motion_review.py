"""Actual emitted failed-quality particles: literal6DF30 vs bounded native motion."""
from pathlib import Path
import sys,struct,json,subprocess,uuid,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sys.argv=[__file__,'private/review/genepod-terminal-world-005','failure']
# Reuse exact audited packed-world preparation, not its callback execution.
scope={'__file__':str(P/'scripts/ghidra/genepod_terminal_world_review.py')};source=(P/'scripts/ghidra/genepod_terminal_world_review.py').read_text().split('r,_,trace=vm.run')[0];exec(compile(source,'terminal-world-preparation','exec'),scope)
m=scope['m'];base=scope['state'];before=scope['after'];layouts=scope['layouts']
for name,address in layouts.items():base.update({address+i:b for i,b in enumerate((before/name).read_bytes())})
for i in range(256):base.update({0x1645ae+i*67+35+j:b for j,b in enumerate(struct.pack('<I',0x112120))})
for i in range(300):base.update({0x1542be+i*124+35+j:b for j,b in enumerate(struct.pack('<I',0x111fec))})
for i in range(200):base.update({0x1688ae+i*81+35+j:b for j,b in enumerate(struct.pack('<I',0x112254))})
files=[P/'private/ghidra/actor-activation/instructions-0004b090.txt',P/'private/ghidra/landing-contact/instructions-0001f030.txt']
v=m.Source();v.code.update(m.m.Model(files,b'').code);order=sorted(v.code);v.next=dict(zip(order,order[1:]));w=P/'private/review/effect20-motion'/uuid.uuid4().hex;w.mkdir(parents=True)
inputs=bytearray();expected=bytearray();cases=[];pool=(before/'effects.bin').read_bytes()
for slot in range(200):
 a=pool[slot*81:(slot+1)*81]
 if a[6]!=5 or a[69]!=32:continue
 r,_,trace=v.run(seed=5428,state_before=base,entry=0x6df30,registers={'EAX':0x1688ae+slot*81},instruction_budget=20000)
 b=bytearray(v.load(0x1688ae+slot*81+i,1)for i in range(81));b[31:39]=a[31:39];b[:6]=a[:6]
 # Source.run configures gravity1024/water1632; recorded exact queried terrain.
 inputs+=a+struct.pack('<iB',struct.unpack_from('<i',b,17)[0],b[23]);expected+=b;cases.append(dict(slot=slot,instructions=r['steps'],draws=r['draws']))
module=P/'rust/crates/genewars-runtime/src/effect20_motion.rs';h=w/'probe.rs';h.write_text('use genewars_runtime::landing;\n#[path="'+module.as_posix()+'"]mod effect20_motion;\n'+r'use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let p=genewars_runtime::physics_tables::PhysicsTables::from_mapped_bytes(&std::fs::read("C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/private/integration/game-runtime/physics/angle.bin").unwrap(),&std::fs::read("C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight/private/integration/game-runtime/physics/bounce.bin").unwrap()).unwrap();let mut out=Vec::new();for c in b.chunks_exact(86){let a=c[..81].try_into().unwrap();let ground=i32::from_le_bytes(c[81..85].try_into().unwrap());let q=effect20_motion::plan_airborne(&a,ground,c[85],1632,1024,true,p.angles()).unwrap();out.extend(q.after);}std::io::stdout().write_all(&out).unwrap();}')
exe=w/'probe.exe';target=P/'private/build/rust-target/debug';subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(h),'--extern',f'genewars_runtime={target}/libgenewars_runtime.rlib','-L',f'dependency={target}/deps','-o',str(exe)],env=environment(),check=True);actual=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:20]
(w/'inputs.bin').write_bytes(inputs);(w/'expected.bin').write_bytes(expected)
r=dict(status='passed',cases=cases,compared_bytes=len(actual),private_run=str(w.relative_to(P)),pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files+v.files+[module,Path(__file__),before/'effects.bin',before/'map.bin',P/'private/integration/game-runtime/physics/angle.bin',P/'private/integration/game-runtime/physics/bounce.bin']},scope='Actual seven Genepod failed-quality emitted records, literalfull6DF30 including collision/map query; pure motion compares81bytes, pointer31:39 and registry header0:6 normalized. Sampledground/shade explicit; bothsource/native contextgravity1024/water1632. World registry side effects and lifecycle separatelypending.')
(P/'research/variants/effect20-motion-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))

from pathlib import Path
import importlib.util,struct,json,sys,subprocess,uuid,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
s=importlib.util.spec_from_file_location('s',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
W=P/'private/review/krell-zone-target'/uuid.uuid4().hex;W.mkdir(parents=True)
fixture=P/'private/review/krell-mule-callbacks-061/mission-frame-1456-blocked-before';back=(fixture/'backing.bin').read_bytes();mapdata=back[0x355e:0x3b55e];(W/'map.bin').write_bytes(mapdata)
a=bytearray((fixture/'family8.bin').read_bytes()[204:306]);a[86]=2;a[87]=0
model=m.Model();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['114b0','13e60']]+[P/'private/ghidra/actor-activation/instructions-00073e00.txt'];model.files+=files;model.code.update(m.m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831;model.code[0x1170d]=("MOVSX",["AX","AL"]) # exact CBW semantics
inputs=bytearray();expected=bytearray();steps=0
for prior in [0,1,2]:
 for seed in [0,1,5428,65535]:
  z=bytearray(back[0x2ff8:0x2ff8+19])*3;struct.pack_into('<i',z,14,0);struct.pack_into('<i',z,33,1024);struct.pack_into('<i',z,52,-1);z[18]=0;z[37]=255;a[87]=prior
  rng=(seed,31,0xffffffff);state={0x11bd58+i:v for i,v in enumerate(z)};state[0x11c218]=3
  out,rr,n,_,_=model.step(a,mapdata,1456,rng,entry=0x114b0,extra_state=state);steps+=n
  zout=bytes(model.load(0x11bd58+i,1)for i in range(len(z)))
  inputs+=a+struct.pack('<HBI',*rng)+z;expected+=out+struct.pack('<HBI',*rr)+zout
(W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected)
rust=P/'rust/crates/genewars-runtime/src/family8_zone_target.rs';h=W/'probe.rs';h.write_text('extern crate genewars_runtime;extern crate genewars_core;pub use genewars_runtime::{actor_activation,blueprint,family8_creation,family8_motion};'+f'#[path="{rust.as_posix()}"] mod target;'+r'''use std::io::{Read,Write};fn main(){let p=std::env::args().nth(1).unwrap();let map=std::fs::read(format!("{p}/map.bin")).unwrap();let input=std::fs::read(format!("{p}/input.bin")).unwrap();let trig=actor_activation::TrigTables::generate_pinned().unwrap();let mut out=Vec::new();for c in input.chunks_exact(166){let q=target::plan_zone_target(c[..102].try_into().unwrap(),&map,&c[109..],&trig,blueprint::BlueprintRng{seed:u16::from_le_bytes(c[102..104].try_into().unwrap()),ring:c[104],draw_count:u32::from_le_bytes(c[105..109].try_into().unwrap())}).unwrap();out.extend(q.after);out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());out.extend(q.zones_after);}std::io::stdout().write_all(&out).unwrap();}''')
t=P/'private/build/rust-target/debug';exe=W/'probe.exe';subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'--extern',f'genewars_runtime={t}/libgenewars_runtime.rlib','--extern',f'genewars_core={t}/libgenewars_core.rlib','-L',f'dependency={t}/deps','-o',str(exe)],env=environment(),check=True)
got=subprocess.run([str(exe),str(W)],capture_output=True,check=True).stdout
assert got==expected,[(i//166,i%166,x,y)for i,(x,y)in enumerate(zip(got,expected))if x!=y][:20]
r={'status':'passed','cases':12,'compared_bytes':len(got),'literal_instructions':steps,'scope':'Synthetic114B0state2 cases on actualKrell terrain, prior occupancy and positive/zero/negative quota; no full callback claim','production_sha256':hashlib.sha256(rust.read_bytes()).hexdigest(),'fixture':str(W)};p=Path(__file__).with_suffix('.json');p.write_text(json.dumps(r,indent=2));print(json.dumps(r))

"""Independent source-pinned shipping landing validation; no original execution."""
from pathlib import Path
import ast,struct,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];work=p/'private/ghidra/landing-phase1';work.mkdir(exist_ok=True);source=p/'scripts/ghidra/validate_landing_bridge.py';tree=ast.parse(source.read_text());node=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='factory');scope={'struct':struct};exec(compile(ast.Module(body=[node],type_ignores=[]),str(source),'exec'),scope);factory=scope['factory'];before=bytearray(factory(bytes(81),0,2,[20736,13568],6272,40));before[70]=1;struct.pack_into('<i',before,39,6272);before[45:51]=bytes(6);struct.pack_into('<hh',before,59,512,0)
expected=bytearray();data=bytearray(struct.pack('<I',65536*5))
for seed in range(65536):
 for timer in [150,50,49,2,1]:
  a=bytearray(before);struct.pack_into('<h',a,74,timer);data.extend(a+struct.pack('<H',seed));out=bytearray(a);t=timer-1;struct.pack_into('<h',out,74,t);s=seed;draws=[];message=65535;angles=[0,0,0]
  def draw():
   global s
   s=s*0xe62d&65535;return s>>8
  if t==0:struct.pack_into('<h',out,74,30);struct.pack_into('<hhh',out,57,0,0,512);struct.pack_into('<hhh',out,45,0,0,512);out[70]=2
  elif t<50:
   if t==49:message=79+draw()%8;draws.append(0x802)
   speed=(40+draw()%10)*64;pitch=1024-draw()%128;yaw=draw()*8;draws.extend([0x806]*3);angles=[yaw,pitch,speed]
  expected.extend(out+struct.pack('<HBIHB3H',s,(31+len(draws))&31,(0xfffffffe+len(draws))&0xffffffff,message,len(draws),*angles));expected.extend(struct.pack('<'+'H'*len(draws),*draws))
module=(p/'rust/crates/genewars-runtime/src/landing.rs').as_posix();h=work/'phase1-oracle.rs';h.write_text(f'#[path="{module}"]mod landing;\n'+r'''use landing::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let q=*p;*p+=n;&b[q..*p]}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let mut out=Vec::new();for _ in 0..n{let a=take(&b,&mut p,81).try_into().unwrap();let seed=u16::from_le_bytes(take(&b,&mut p,2).try_into().unwrap());let r=plan_landing_phase1_after_physics(&a,LandingRng{seed,ring:31,draw_count:0xfffffffe}).unwrap();let mut message=65535u16;let mut angles=[0;3];for e in &r.events{match e{LandingEvent::LandingMessage{message_index,..}=>message=*message_index,LandingEvent::CreateEffect{angles:a,..}=>angles=*a,_=>{}}}out.extend(r.after);out.extend(r.rng_after.seed.to_le_bytes());out.push(r.rng_after.ring);out.extend(r.rng_after.draw_count.to_le_bytes());out.extend(message.to_le_bytes());out.push(r.draws.len() as u8);for a in angles{out.extend(a.to_le_bytes());}for d in r.draws{out.extend(d.journal_code.to_le_bytes());}}assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}''');exe=work/'phase1-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
r={'cases':65536*5,'timers':[150,50,49,2,1],'seed_cases':65536,'compared_bytes':len(out),'output_sha256':hashlib.sha256(out).hexdigest(),'source_sha256':hashlib.sha256((p/'rust/crates/genewars-runtime/src/landing.rs').read_bytes()).hexdigest()};(p/'research/ghidra/landing-phase1-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

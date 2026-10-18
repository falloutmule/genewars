"""Compare source7AC00/58435 register model against actor animation APIs."""
from pathlib import Path
import struct,random,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];w=p/'private/ghidra/actor-motion';rng=random.Random(0x7ac00);sha=lambda b:hashlib.sha256(b).hexdigest();data=bytearray(struct.pack('<I',4096));expected=bytearray()
for i in range(4096):
 a=bytearray(rng.randbytes(200));a[6]=3;a[85]=8;a[112]&=~64;struct.pack_into('<h',a,57,i%2048);a[28]=1 if i%2 else 255;holds=rng.randbytes(2+i%23);flags=i%2;data.extend(a+bytes([flags,len(holds)])+holds)
 out=bytearray(a);anim=(((i%2048>>7)+1>>1)+1)&7
 if anim!=a[25]:out[25]=anim;out[28]=1
 speed=struct.unpack_from('<h',a,65)[0];out[29]=int((1024-speed)/256)&255
 # Follow7AC00 instruction groups including signed counter and unsigned hold.
 frame=min(out[27],len(holds)-1);out[27]=frame
 signed=lambda b:b-256 if b>=128 else b
 if signed(out[30])>signed(out[29]):out[30]=out[29]
 old=signed(out[30]);out[30]=(out[30]-1)&255
 if old<=0:
  out[30]=out[29];out[26]=min(out[26],holds[frame]);old=out[26];out[26]=(old-1)&255
  if old==0:
   nxt=frame+signed(out[28])
   if nxt>=len(holds):
    if flags:out[28]=255;nxt=len(holds)-2
    else:nxt=0
   elif nxt<0:
    if flags:out[28]=1;nxt=1
    else:nxt=len(holds)-1
   out[27]=nxt;out[26]=holds[nxt]
 expected.extend(out)
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix();h=w/'animation-oracle.rs';h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''use actor_motion::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],q:&mut usize,n:usize)->&'a[u8]{let s=*q;*q+=n;&b[s..*q]}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut q=0;let n=u32::from_le_bytes(take(&b,&mut q,4).try_into().unwrap());let mut out=Vec::new();for _ in 0..n{let a=take(&b,&mut q,200).try_into().unwrap();let f=take(&b,&mut q,1)[0];let n=take(&b,&mut q,1)[0] as usize;let holds=take(&b,&mut q,n);out.extend(plan_actor_moving_animation(&a,holds,f).unwrap());}assert_eq!(q,b.len());std::io::stdout().write_all(&out).unwrap();}''');exe=w/'animation-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
r={'cases':4096,'compared_bytes':len(out),'output_sha256':sha(out),'source_sha256':sha(Path(module).read_bytes()),'instruction_contracts':[{'path':str(f.relative_to(p)).replace('\\','/'),'sha256':sha(f.read_bytes())} for f in [w/'instructions-00057a10.txt',p/'private/ghidra/initial-creature/instructions-0001f060.txt',p/'private/ghidra/landing-contact/instructions-0007ac00.txt']],'scope':'State8 directional animation58435 plus7AC00: signed delay/counter, unsigned holds, bothdirections, flags0/1,2..24frames; metadata of selected animation explicit.'};(p/'research/ghidra/actor-animation-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

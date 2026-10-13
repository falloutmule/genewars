"""Independent source-register model for type3 pending move consumption."""
from pathlib import Path
import struct, subprocess, random, hashlib, json
p=Path(__file__).resolve().parents[2]; g=p/'private/ghidra'; work=g/'actor-motion'; work.mkdir(exist_ok=True)
sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha((g/'GW-CD-candidate.exe').read_bytes())=='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
mem=(g/'validated/memory-.object3.bin').read_bytes(); code=(g/'validated/memory-.object1.bin').read_bytes()
speeds=[struct.unpack_from('<h',mem,0x4ae8+16*c)[0] for c in range(4)]
atan=struct.unpack_from('<257H',code,0x9e134); rng=random.Random(0x57e0f)
short=lambda x:(x+32768)%65536-32768
def angle(x,y):
 if x==y==0:return 0
 v=atan[min(abs(x),abs(y))*256//max(abs(x),abs(y))]
 if x>=0:return (v if abs(x)<abs(y) else 512-v) if y<0 else (1024-v if abs(x)<abs(y) else 512+v)
 return (2048-v if abs(x)<abs(y) else 1536+v) if y<0 else (1024+v if abs(x)<abs(y) else 1536-v)
data=bytearray(struct.pack('<4h257HI',*speeds,*atan,2048)); expected=bytearray()
for i in range(2048):
 a=bytearray(rng.randbytes(200));bp=bytearray(rng.randbytes(36));a[6]=3;a[0x71]&=~16;a[0x55]=1 if i%2 else 8;a[0x70]|=1;a[0x57]|=2;a[0x58]&=~1;a[0x5b]=0;bp[21]=i%4;bp[22]=i//4%4
 if i%7==0:a[0x71]|=16
 if i%11==0:a[0x57]&=~2
 data.extend(a+bp);out=bytearray(a);consume=not(a[0x71]&16)and bool(a[0x57]&2)
 if consume:
  out[122]=0;out[112]&=~16;out[85]=8;out[90]=0;out[104:108]=bytes(4);struct.pack_into('<h',out,63,speeds[bp[21]]);out[88]|=2;out[73]|=4;out[11:15]=out[92:96]
  if not out[73]&16:
   x,y=struct.unpack_from('<2h',out,7);tx,ty=struct.unpack_from('<2h',out,11);v=angle(short(2*(tx-x)),short(2*(ty-y)));struct.pack_into('<H',out,75,v);struct.pack_into('<H',out,57,v)
  out[84]=20;out[73]|=20;out[22]&=63;struct.pack_into('<H',out,73,struct.unpack_from('<H',out,73)[0]&0xeffc);out[79]=out[82]=0;out[87]&=~2
 expected.extend(out+bytes([consume]))
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix();h=work/'motion-oracle.rs'
h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''use actor_motion::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],q:&mut usize,n:usize)->&'a[u8]{let s=*q;*q+=n;&b[s..*q]}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut q=0;let mut speeds=[0;4];for v in &mut speeds{*v=i16::from_le_bytes(take(&b,&mut q,2).try_into().unwrap());}let mut atan=[0;257];for v in &mut atan{*v=u16::from_le_bytes(take(&b,&mut q,2).try_into().unwrap());}let n=u32::from_le_bytes(take(&b,&mut q,4).try_into().unwrap());let mut out=Vec::new();for _ in 0..n{let a=take(&b,&mut q,200).try_into().unwrap();let bp=take(&b,&mut q,36).try_into().unwrap();let r=plan_pending_move_after_motion(&a,&bp,&speeds,&atan).unwrap();assert_eq!(r.acknowledgement.is_some(),r.consumed);out.extend(r.after);out.push(r.consumed as u8);}assert_eq!(q,b.len());std::io::stdout().write_all(&out).unwrap();}''')
exe=work/'motion-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True)
out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
pins=[]
for f in ['actor-motion/instructions-00057a10.txt','actor-motion/instructions-00051930.txt','mission-array-refs/instructions-00052340.txt','initial-creature/instructions-00073970.txt']:
 pins.append({'path':'private/ghidra/'+f,'sha256':sha((g/f).read_bytes())})
r={'candidate_sha256':sha((g/'GW-CD-candidate.exe').read_bytes()),'source_sha256':sha(Path(module).read_bytes()),'instruction_contracts':pins,'cases':2048,'compared_bytes':len(out),'output_sha256':sha(out),'speed_words':speeds,'scope':'Post6FB60/postblueprint input at57E0F only. Inhibited/no-pending branches and opcode3 consume; acknowledgements typed. Earlier movement, state dispatch and animation not claimed.'}
(p/'research/ghidra/actor-motion-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

"""Independent literal743C0 numeric walk compared to shipping actor helper."""
from pathlib import Path
import struct,random,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];g=p/'private/ghidra';w=g/'actor-motion';sha=lambda b:hashlib.sha256(b).hexdigest();rng=random.Random(0x743c0);mem=(g/'validated/memory-.object3.bin').read_bytes();code=(g/'validated/memory-.object1.bin').read_bytes();sine=struct.unpack_from('<2048i',mem,0x7ee8);cosine=struct.unpack_from('<2048i',mem,0x86e8);atan=struct.unpack_from('<257H',code,0x9e134);short=lambda x:(x+32768)%65536-32768;long=lambda x:(x+2147483648)%4294967296-2147483648
def angle(x,y):
 if x==y==0:return 0
 v=atan[min(abs(x),abs(y))*256//max(abs(x),abs(y))]
 return {(True,False,True):v,(True,False,False):512-v,(True,True,True):1024-v,(True,True,False):512+v,(False,False,True):2048-v,(False,False,False):1536+v,(False,True,True):1024+v,(False,True,False):1536-v}[x>=0,y>=0,abs(x)<abs(y)]
data=bytearray(struct.pack('<2048i2048i257HI',*sine,*cosine,*atan,2048));expected=bytearray()
for i in range(2048):
 start=[rng.randrange(-32768,32768) for _ in range(2)];target=[short(start[a]+rng.randrange(-12000,12001)) for a in range(2)];fail=[0,1,2,7,33,10000][i%6];data.extend(struct.pack('<4hI',*start,*target,fail));samples=[start];out=None;clear=True;fixed=[v<<16 for v in start];step=[0,0]
 if fail!=0:
  for n in range(1024):
   point=[short(v>>16) for v in fixed]
   if n%64==0:
    yaw=angle(short(2*(target[0]-point[0])),short(2*(target[1]-point[1])));j=yaw-512 if yaw>=512 else yaw+1536;step=[long(cosine[j]<<6),long(sine[j]<<6)]
   dx,dy=[short(2*(target[a]-point[a]))>>1 for a in range(2)];done=dx*dx+dy*dy<=4096;sample=target if done else point;samples.append(sample)
   if len(samples)-1==fail:clear=False;out=[short(long(fixed[a]-step[a])>>16) for a in range(2)];break
   fixed=[long(fixed[a]+step[a]) for a in range(2)]
   if done:break
  else:raise AssertionError('source walk exceeded bounded domain')
 expected.extend(bytes([clear,out is not None]));
 if out is not None:expected.extend(struct.pack('<2h',*out))
 expected.extend(struct.pack('<I',len(samples)))
 for sample in samples:expected.extend(struct.pack('<2h',*sample))
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix();h=w/'ray-oracle.rs';h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''use actor_motion::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],q:&mut usize,n:usize)->&'a[u8]{let s=*q;*q+=n;&b[s..*q]}fn u(b:&[u8],q:&mut usize)->u32{u32::from_le_bytes(take(b,q,4).try_into().unwrap())}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut q=0;let mut s=[0;2048];let mut c=[0;2048];for t in [&mut s,&mut c]{for v in t{*v=u(&b,&mut q) as i32;}}let mut atan=[0;257];for v in &mut atan{*v=u16::from_le_bytes(take(&b,&mut q,2).try_into().unwrap());}let mut out=Vec::new();for _ in 0..u(&b,&mut q){let mut points=[[0;2];2];for p in &mut points{for v in p{*v=i16::from_le_bytes(take(&b,&mut q,2).try_into().unwrap());}}let fail=u(&b,&mut q);let mut call=0;let r=actor_straight_ray::<ActorMotionError,_>(points[0],points[1],&atan,&s,&c,|_|{let ok=call!=fail;call+=1;Ok(ok)}).unwrap();out.push(r.clear as u8);out.push(r.failure_point.is_some()as u8);if let Some(p)=r.failure_point{for v in p{out.extend(v.to_le_bytes());}}out.extend((r.sampled_points.len()as u32).to_le_bytes());for p in r.sampled_points{for v in p{out.extend(v.to_le_bytes());}}}assert_eq!(q,b.len());std::io::stdout().write_all(&out).unwrap();}''');exe=w/'ray-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
f=g/'terrain-map-refs/instructions-000743c0.txt';r={'candidate_sha256':sha((g/'GW-CD-candidate.exe').read_bytes()),'instruction_contracts':[{'path':'private/ghidra/terrain-map-refs/instructions-000743c0.txt','sha256':sha(f.read_bytes())}],'cases':2048,'compared_bytes':len(out),'output_sha256':sha(out),'source_sha256':sha(Path(module).read_bytes()),'scope':'Literalnumeric743C0 walk; actualpointpredicatecaller-owned. Initialrejectiontrue,duplicatefirstprobe,recompute64,modularnearesttarget,snap4096,failurepreviousfixedstep. Explicit1024bound.'};(p/'research/ghidra/actor-ray-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

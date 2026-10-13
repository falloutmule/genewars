"""Independent ordinary actor postmovement-prefix, distance and footstep models."""
from pathlib import Path
import struct,random,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];g=p/'private/ghidra';w=g/'actor-motion';rng=random.Random(0x57a10);sha=lambda b:hashlib.sha256(b).hexdigest();i16=lambda x:(x+32768)%65536-32768;i32=lambda x:(x+2147483648)%4294967296-2147483648
mem=(g/'validated/memory-.object3.bin').read_bytes();code=(g/'validated/memory-.object1.bin').read_bytes();speeds=[struct.unpack_from('<h',mem,0x4ae8+c*16)[0] for c in range(4)];atan=struct.unpack_from('<257H',code,0x9e134)
word=lambda a,n:struct.unpack_from('<h',a,n)[0]
def angle(x,y):
 if x==y==0:return 0
 v=atan[min(abs(x),abs(y))*256//max(abs(x),abs(y))]
 return {(True,False,True):v,(True,False,False):512-v,(True,True,True):1024-v,(True,True,False):512+v,(False,False,True):2048-v,(False,False,False):1536+v,(False,True,True):1024+v,(False,True,False):1536-v}[x>=0,y>=0,abs(x)<abs(y)]
def path(a):
 a[11:15]=a[92:96]
 if not a[73]&16:
  yaw=angle(i16(2*(word(a,11)-word(a,7))),i16(2*(word(a,13)-word(a,9))));struct.pack_into('<H',a,75,yaw);struct.pack_into('<H',a,57,yaw)
 a[84]=20;a[73]|=20;a[22]&=63;struct.pack_into('<H',a,73,struct.unpack_from('<H',a,73)[0]&0xeffc);a[79]=a[82]=0
data=bytearray(struct.pack('<4h257HI',*speeds,*atan,2048));expected=bytearray()
for i in range(2048):
 a=bytearray(rng.randbytes(200));b=bytearray(rng.randbytes(36));a[6]=3;a[85]=1 if i%3==0 else 8;a[88]=2;a[87]=2 if a[85]==1 or i%2 else 0;a[112]=1|([0,16,32][i%3]);a[113]&=~16;a[73]=[0,16,19][i%3];a[74]&=~0;a[90]=a[91]=0;b[21]=i%4;b[22]=i%4;frame=[0,1,127,128,255,256,0xffffffff][i%7];data.extend(a+b+struct.pack('<I',frame));out=bytearray(a);bp=bytearray(b)
 if bp[22] in (2,3):
  v=struct.unpack_from('<i',out,100)[0]
  if v>0:struct.pack_into('<i',out,100,v-1)
 mask=127 if bp[21]==2 else 255
 signed=lambda v:v-256 if v>=128 else v
 if ((frame+word(out,15))&mask)==0 and signed(bp[26])<signed(bp[25]):bp[26]=(bp[26]+1)&255
 if out[85]==8 and not out[112]&32 and not out[73]&19:path(out)
 ack=bool(out[87]&2)
 if ack:
  out[122]=0;out[112]&=~16;out[85]=8;out[90]=0;struct.pack_into('<h',out,63,speeds[bp[21]]);out[104:108]=bytes(4);out[88]|=2;out[73]|=4;path(out);out[87]&=~2
 if bp[21]==3:out[115]=0
 if signed(bp[27])<signed(bp[24]) and frame&127==0:bp[27]=(bp[27]+1)&255
 expected.extend(out+bp+bytes([ack]))
data.extend(struct.pack('<I',2048))
for i in range(2048):
 a=bytearray(rng.randbytes(200));a[6]=3;frame=i;struct.pack_into('<h',a,15,i%20);ground=i*13-6000;physical=ground+(i%5==0);struct.pack_into('<i',a,17,ground);struct.pack_into('<i',a,39,physical);struct.pack_into('<3h',a,57,i%2048,i*17%2048,i%2000);struct.pack_into('<3h',a,45,0 if i%7==0 else i%100,0,0);terrain=i%4;data.extend(a+struct.pack('<IB',frame,terrain));yes=(frame+word(a,15))&3==3 and physical==ground and word(a,45)!=0 and terrain>1;expected.append(yes)
 if yes:expected.extend(a[7:11]+a[39:43]+struct.pack('<3HB',word(a,57),max(0,word(a,59)-256),word(a,61)>>1,0x4b))
data.extend(struct.pack('<I',2048))
for i in range(2048):
 a=bytearray(rng.randbytes(200));a[6]=3;a[82]=0;a[74]&=~4;a[73]=(a[73]&~3)|(i%4);ground=rng.randrange(-2147483648,2147483648);target=rng.randrange(-2147483648,2147483648);data.extend(a+struct.pack('<2i',ground,target));o=69 if a[73]&3 else 11;dx=i16(2*(word(a,o)-word(a,7)))>>1;dy=i16(2*(word(a,o+2)-word(a,9)))>>1;dz=i32(target-ground)>>3;expected.extend(a[o:o+4]+struct.pack('<i',i32(dx*dx+dy*dy+dz*dz)))
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix();h=w/'callback-oracle.rs';h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''use actor_motion::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],q:&mut usize,n:usize)->&'a[u8]{let s=*q;*q+=n;&b[s..*q]}fn u(b:&[u8],q:&mut usize)->u32{u32::from_le_bytes(take(b,q,4).try_into().unwrap())}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut q=0;let mut speeds=[0;4];for v in &mut speeds{*v=i16::from_le_bytes(take(&b,&mut q,2).try_into().unwrap());}let mut atan=[0;257];for v in &mut atan{*v=u16::from_le_bytes(take(&b,&mut q,2).try_into().unwrap());}let mut out=Vec::new();for _ in 0..u(&b,&mut q){let a=take(&b,&mut q,200).try_into().unwrap();let bp=take(&b,&mut q,36).try_into().unwrap();let f=u(&b,&mut q);let r=plan_actor_post_motion_prefix_with_class3(&a,&bp,f,&speeds,&atan,true).unwrap();out.extend(r.after);out.extend(r.blueprint_after);out.push(r.acknowledgement.is_some() as u8);}for _ in 0..u(&b,&mut q){let a=take(&b,&mut q,200).try_into().unwrap();let f=u(&b,&mut q);let c=take(&b,&mut q,1)[0];let r=plan_actor_footstep(&a,f,c).unwrap();out.push(r.is_some() as u8);if let Some(r)=r{for v in r.position{out.extend(v.to_le_bytes());}out.extend(r.physical_height.to_le_bytes());for v in r.angles{out.extend(v.to_le_bytes());}out.push(r.audio_channel);}}for _ in 0..u(&b,&mut q){let a=take(&b,&mut q,200).try_into().unwrap();let g=u(&b,&mut q) as i32;let t=u(&b,&mut q) as i32;for v in actor_contact_distance_target(&a).unwrap(){out.extend(v.to_le_bytes());}out.extend(actor_contact_distance_squared(&a,g,t).unwrap().to_le_bytes());}assert_eq!(q,b.len());std::io::stdout().write_all(&out).unwrap();}''');exe=w/'callback-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
r={'candidate_sha256':sha((g/'GW-CD-candidate.exe').read_bytes()),'instruction_contracts':[{'path':'private/ghidra/'+f,'sha256':sha((g/f).read_bytes())} for f in ['actor-motion/instructions-00057a10.txt','actor-motion/instructions-000526c0.txt','actor-motion/instructions-0006f760.txt','initial-creature/instructions-000736f0.txt']],'prefix_cases':2048,'footstep_cases':2048,'distance_cases':2048,'compared_bytes':len(out),'output_sha256':sha(out),'source_sha256':sha(Path(module).read_bytes()),'scope':'Post6FB60 activatedstate1/8,class0..3(unpowered3) withoutarrival/error/reinitialization; sourceprefix,typedcode6/audiofootstep,actualsampledheightdistance. Movementitselfnotclaimed.'};(p/'research/ghidra/actor-callback-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

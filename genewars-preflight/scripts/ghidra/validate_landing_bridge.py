"""Bounded landing factory/phase0 contracts; no original execution."""
from pathlib import Path
import hashlib,itertools,json,struct,subprocess,sys
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';sha=lambda b:hashlib.sha256(b).hexdigest()
candidate='4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
assert sha((G/'GW-CD-candidate.exe').read_bytes())==candidate
mem=(G/'validated/memory-.object3.bin').read_bytes();code=(G/'validated/memory-.object1.bin').read_bytes()
pins=[]
def pin(path,*lines):
 b=(G/path).read_bytes()
 for line in lines:assert line in b.decode(),(path,line)
 pins.append({'private_path':'private/ghidra/'+path,'sha256':sha(b)})
pin('entity-shadow-panel/instructions-0007f3b0.txt','0007f428 LEA EBX,[EAX + 0x6400]','0007f431 MOV EAX,0x31','0007f436 CALL 0x00048790','0007f449 MOV word ptr [EBX],CX','0007f44f MOV byte ptr [EBX + 0x2],CL','0007f465 CALL 0x00075b00')
pin('mission-array-refs/instructions-00048790.txt','000487b5 CMP EAX,0xc8','000488e0 CALL dword ptr [ESI + 0x78]','000489ad MOV byte ptr [EDX + 0x46],0x0','00048acd CALL 0x0008ac60','00048ad8 MOV word ptr [EAX + 0x3f],0xffff','00048aec MOV word ptr [0x0011c2b4],SI','00048c96 MOV byte ptr [EAX + 0x2],0x0','00048cde MOV byte ptr [EAX + 0x4c],0x0')
pin('player-creation/instructions-0001f070.txt','0001f0a8 MOV dword ptr [EBP + 0x11],EAX','0001f0b2 MOV byte ptr [EBP + 0x17],AL','0001f1b5 MOV dword ptr [EBP + 0x27],EAX')
pin('mission-tail-init/instructions-00049e90.txt','00049fb6 MOV dword ptr [EDX + 0x1f],EBX')
pin('plant-command/instructions-0004b630.txt','0004b66f CALL dword ptr [EDX + 0x94]','0004d0c6 CALL 0x0007ac00','0004d111 CMP EAX,dword ptr [EDX + 0x27]','0004d114 JL 0x0004d196','0004d138 INC byte ptr [EDX + 0x46]','0004d14d MOV word ptr [EAX + 0x4a],0x96','0004d16b CALL 0x00080ae0','0004d178 CALL 0x000a7120','0004d1cd ADD EAX,0x3','0004d340 CALL 0x00048790','0004d349 OR byte ptr [EAX + 0x15],0x1','0004dac7 CALL 0x00075240')
pin('actor-activation/instructions-0006df30.txt','0006dfca TEST BL,0x20','0006dff3 SAR EAX,0x3','0006e053 CALL 0x0007b380','0006e072 CMP ESI,EAX','0006e0b5 CALL 0x000733a0','0006e7f2 CALL 0x00074270')
pin('landing-contact/instructions-0007ac00.txt','0007ac0b JC 0x0007ac16','0007ac4c MOV byte ptr [EDX + 0x4],CL','0007ac9f JBE 0x0007acc4','0007acbe MOV byte ptr [EDX + 0x1],0x0')
pin('command-resolver/instructions-00080ae0.txt','00080b76 MOV dword ptr [ESI + 0x7],EAX','00080c00 CALL 0x00074a50','00080c60 CALL 0x0008ac60','00080cfc CMP byte ptr [EAX + 0x2],0x5','00080d09 IMUL EAX,EAX,0x51','00080d21 INC dword ptr [EDX + 0x49d]')
pin('plant-command/instructions-00074a50.txt','00074ba1 CALL 0x00074270','00074bfb CALL 0x0008b520','00074c49 CALL 0x0008b130')
pin('actor-activation/instructions-00073e00.txt','00073e41 IMUL EAX,EDX,-0x44bf19d3')
pin('actor-activation/instructions-0008ac60.txt','0008ac71 MOV byte ptr [EDX + 0x2],0x0','0008acaf MOV word ptr [ECX + 0x6],AX')
pin('landing-contact/instructions-000a7120.txt','000a7162 CMP ESI,dword ptr [EBX + 0xe]','000a7180 CALL 0x000b1a10')
pin('player-creation/instructions-0001e810.txt','0001e831 CALL 0x000417e0')
pin('terrain-camera-refs/instructions-0007d090.txt','0007d4d4 TEST CH,0x80','0007d4d9 TEST CH,0x1','0007d515 MOV [0x0011878a],AX','0007d531 CALL 0x000a5efc','0007d544 MOV [0x0011878a],AX','0007d5d5 CALL 0x0007f580')
pin('landing-contact/instructions-000a71c4.txt','000a71cc CALL 0x000c55d5')
pin('landing-contact/instructions-000c55d5.txt','000c55da MOV AH,0x2c','000c55dc INT 0x21')
pin('landing-contact/instructions-000733a0.txt','00073454 CMP EBP,EBX','00073548 CALL 0x00097910','00073621 CALL 0x00097910','0007363d IDIV EBX')
pin('command-resolver/instructions-000ae028.txt','000ae028 PUSH EBP')
pin('terrain-map-refs/instructions-00049d30.txt','00049e48 CALL 0x0008acc0')
pin('mission-array-refs/instructions-0004b3b0.txt','0004b3bd CMP EAX,0xff')
assert mem[0x4eb4+0x31*6:0x4eb4+0x32*6]==bytes([5,0,1,255,0,1])
assert struct.unpack_from('<I',mem,0x12254+0x94)[0]==0x6df30
assert struct.unpack_from('<I',code,0x4b524-0x10000+(0x31-6)*4)[0]==0x4d089
manifest=json.loads((P/'research/game-files-manifest.json').read_text())['entries'];corpus=json.loads((P/'research/integration/sprite-corpus.json').read_text())['results']
entry=next(e for e in manifest if e['entry_id']=='root/2/disc/59');asset=next(c for c in corpus if c['entry_id']==entry['entry_id']);db=(P/asset['private_database']).read_bytes();assert sha(db)==asset['decoded_sha256']
count,tab,components,frames,animations=struct.unpack_from('<H4I',db);first,last,nframes,flags=struct.unpack_from('<IIBB',db,animations+49*10)
assert count==68 and (first,last,nframes,flags)==(3486,3486,1,0) and db[frames+first+10]==0
def factory(prior,index,owner,position,ground,shade,head=None):
 assert len(prior)==81 and prior[6]==0 and 0<=index<200 and 0<=owner<4 and -65536<=ground<=65534
 b=bytearray(prior);b[2]=b[5]=0;b[6]=5;struct.pack_into('<hh',b,7,*position);struct.pack_into('<H',b,15,index);struct.pack_into('<i',b,17,ground);struct.pack_into('<H',b,21,0x20);b[23]=shade;b[24]=owner
 b[11:15]=bytes(4);b[25:28]=bytes(3);b[28]=1;b[29]=0;struct.pack_into('<iHhhh',b,39,ground+25600,16,0,0,-1088);b[51:57]=bytes(6);struct.pack_into('<hhh',b,57,0,1024,1088);struct.pack_into('<h',b,63,-1)
 b[69]=0x31;b[70]=0;struct.pack_into('<h',b,74,-1);b[73]=0;b[76]=0
 # The native animation pointer is a declared typed-handle placeholder.
 b[31:35]=bytes(4)
 cell=(((position[1]>>8)&127)*128+((position[0]>>8)&127));struct.pack_into('<hB',b,0,cell,0)
 if head is not None:struct.pack_into('<hB',b,3,*head)
 return bytes(b)
def phase0(postphysics,ground):
 assert len(postphysics)==81 and postphysics[6]==5 and postphysics[69]==0x31 and postphysics[70]==0 and 0<=postphysics[24]<4
 assert postphysics[73]==0,'bounded fresh route excludes attached kind4 child'
 after=bytearray(postphysics);physical=struct.unpack_from('<i',after,39)[0];contact=ground>=physical
 if contact:struct.pack_into('<i',after,39,ground);after[45:51]=bytes(6);after[70]=1;struct.pack_into('<h',after,74,150)
 return bytes(after),contact
def noncontact_rng(seed,ring,count):
 assert 0<=seed<=65535 and 0<=ring<32 and 0<=count<=0xffffffff
 states=[]
 for _ in range(3):seed=seed*0xe62d&65535;states.append(seed);ring=(ring+1)&31
 high=[s>>8 for s in states]
 return {'seed_after':seed,'ring_after':ring,'draw_count_after':(count+3)&0xffffffff,'seeds':states,'effect_code':6,'angles_u16':[high[2]*8,1024-high[1]%128,2560+64*(high[0]%10)],'effect_owner':0}
folder=G/'landing-contact';factory_vectors=[];phase_vectors=[]
for i in range(32):
 prior=bytearray((i*17+j*7)&255 for j in range(81));prior[6]=0;position=[-32768+i*1024,32767-i*1000];ground=[-65536,-1,0,6272,65534][i%5]
 after=factory(prior,i,i%4,position,ground,i*7,None if i%2 else (i,2));assert after[30]==prior[30] and after[35:39]==prior[35:39] and after[77:81]==prior[77:81]
 factory_vectors.append({'prior_hex':prior.hex(),'index':i,'owner':i%4,'position':position,'ground':ground,'shade':i*7,'resolved_oldhead':None if i%2 else [i,2],'after_hex':after.hex()})
 for physical in [ground-1,ground,ground+1]:
  b=bytearray(after);struct.pack_into('<i',b,39,physical);out,contact=phase0(b,ground);phase_vectors.append({'postphysics_hex':b.hex(),'ground':ground,'contact':contact,'after_hex':out.hex()})
# Independent executable oracle, freshly authored Rust; no game code is run.
harness=folder/'landing-oracle.rs';harness.write_text(r'''use std::io::{Read,Write};fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let a=*p;*p+=n;&b[a..*p]}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let mut out=Vec::new();for _ in 0..n{let mut a=take(&b,&mut p,81).to_vec();let g=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let h=i32::from_le_bytes(a[39..43].try_into().unwrap());let hit=!(g<h);if hit{a[39..43].copy_from_slice(&g.to_le_bytes());a[45..51].fill(0);a[70]=a[70].wrapping_add(1);a[74..76].copy_from_slice(&150i16.to_le_bytes());}out.push(hit as u8);out.extend(a);}for seed in 0..=65535u16{let mut s=seed;let mut hs=[0u16;3];for v in &mut hs{s=s.wrapping_mul(58925);*v=s>>8;}out.extend(s.to_le_bytes());for x in [hs[2]<<3,1024-hs[1]%128,2560+((hs[0]%10)<<6)]{out.extend(x.to_le_bytes());}}assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}''',encoding='utf-8')
native_input=bytearray();expected=bytearray();phase_cases=[]
for i in range(8192):
 b=bytearray(factory_vectors[i%32]['after_hex'] and bytes.fromhex(factory_vectors[i%32]['after_hex']));g=[-65536,-1,0,6272,65534][i%5];h=[-2147483648,g-1,g,g+1,2147483647][i%5];struct.pack_into('<i',b,39,h)
 out,hit=phase0(b,g);phase_cases.append(bytes(b)+struct.pack('<i',g));expected.extend(bytes([hit])+out)
for seed in range(65536):
 r=noncontact_rng(seed,31,0xfffffffe);expected.extend(struct.pack('<HHHH',r['seed_after'],*r['angles_u16']));assert r['ring_after']==2 and r['draw_count_after']==1
sys.path.insert(0,str(P/'scripts'));from cargo import environment
rustc=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';native_exe=folder/'landing-oracle.exe'
subprocess.run([str(rustc),'--edition=2021',str(harness),'-o',str(native_exe)],env=environment(),check=True,capture_output=True)
native=subprocess.run([str(native_exe)],input=struct.pack('<I',len(phase_cases))+b''.join(phase_cases),capture_output=True,check=True).stdout;assert native==expected
# Compare the same independently generated bytes with the shipping module.
production=P/'rust/crates/genewars-runtime/src/landing.rs'
production_harness=folder/'landing-production.rs'
production_harness.write_text('#[path='+json.dumps(str(production).replace('\\','/'))+'] mod landing;\n'+r"""use landing::*;use std::io::{Read,Write};
fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let a=*p;*p+=n;&b[a..*p]}
fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let mut out=Vec::new();let rng=LandingRng{seed:0,ring:31,draw_count:0xfffffffe};
for _ in 0..n{let a:LandingRecord=take(&b,&mut p,81).try_into().unwrap();let g=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let plan=plan_landing_phase0_after_physics(&a,g,rng).unwrap();out.push(plan.contact as u8);out.extend(plan.after);}
let mut a=[0;81];a[6]=5;a[69]=0x31;a[39..43].copy_from_slice(&1i32.to_le_bytes());
for seed in 0..=65535u16{let plan=plan_landing_phase0_after_physics(&a,0,LandingRng{seed,..rng}).unwrap();assert_eq!(plan.rng_after.ring,2);assert_eq!(plan.rng_after.draw_count,1);assert_eq!(plan.draws.len(),3);out.extend(plan.rng_after.seed.to_le_bytes());match plan.events[0]{LandingEvent::CreateEffect{angles,..}=>for x in angles{out.extend(x.to_le_bytes());},_=>panic!("missing effect")}}
let nf=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());for _ in 0..nf{let prior:LandingRecord=take(&b,&mut p,81).try_into().unwrap();let slot=take(&b,&mut p,1)[0] as usize;let owner=take(&b,&mut p,1)[0];let at=[i16::from_le_bytes(take(&b,&mut p,2).try_into().unwrap()),i16::from_le_bytes(take(&b,&mut p,2).try_into().unwrap())];let ground=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let shade=take(&b,&mut p,1)[0];let head=take(&b,&mut p,3);let old=if head[2]==0{None}else{Some(ResolvedCellHead{identity:LandingReference{index:i16::from_le_bytes(head[..2].try_into().unwrap()),kind:head[2]}})};let mut pool=[[0;81];200];for r in &mut pool{r[6]=5;}pool[slot]=prior;let plan=plan_landing_factory(&pool,slot as i16,owner,owner,at,ground,shade,old).unwrap();assert_eq!(plan.slot,slot as u16);assert_eq!(plan.pool_count_after,slot as i16+1);out.extend(plan.after_nonpointer);}
assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
""",encoding='utf-8')
production_input=bytearray(struct.pack('<I',len(phase_cases))+b''.join(phase_cases));production_input.extend(struct.pack('<I',len(factory_vectors)));production_expected=bytearray(expected)
for v in factory_vectors:
 head=v['resolved_oldhead'] or [0,0]
 production_input.extend(bytes.fromhex(v['prior_hex'])+struct.pack('<BBhhiBhB',v['index'],v['owner'],*v['position'],v['ground'],v['shade'],*head));production_expected.extend(bytes.fromhex(v['after_hex']))
production_exe=folder/'landing-production.exe'
subprocess.run([str(rustc),'--edition=2021',str(production_harness),'-o',str(production_exe)],env=environment(),check=True,capture_output=True)
production_output=subprocess.run([str(production_exe)],input=production_input,capture_output=True,check=True).stdout
assert production_output==production_expected,'shipping landing API differs from independent fixtures'
subprocess.run([sys.executable,'-B',str(P/'scripts/cargo.py'),'test','-p','genewars-runtime','landing::tests','--offline','--locked'],cwd=P,check=True)
# Independently model fresh first-contact normal/bounce/angle and code6 factory.
import math,random
sine=struct.unpack_from('<2048i',mem,0x7ee8);cosine=struct.unpack_from('<2048i',mem,0x86e8);atan=struct.unpack_from('<257H',code,0xae134-0x10000)
def norm(v):
 n=max(1,math.isqrt(sum(x*x for x in v)));return [int(x*256/n) for x in v]
def normal(at,h):
 h=[x>>3 for x in h]
 a,b=([256,256,h[3]-h[0]],[-256,0,h[2]-h[3]]) if at[0]&255<at[1]&255 else ([256,0,h[1]-h[0]],[0,256,h[3]-h[1]])
 a,b=norm(a),norm(b);return norm([(a[1]*b[2]-b[1]*a[2])>>8,(a[2]*b[0]-b[2]*a[0])>>8,(a[0]*b[1]-b[0]*a[1])>>8])
def angle(x,y):
 if x==y==0:return 0
 ax,ay=abs(x),abs(y);v=atan[min(ax,ay)*256//max(ax,ay)]
 return {(True,False,True):v,(True,False,False):512-v,(True,True,True):1024-v,(True,True,False):512+v,(False,False,True):2048-v,(False,False,False):1536+v,(False,True,True):1024+v,(False,True,False):1536-v}[x>=0,y>=0,ax<ay]
i16=lambda x:(x+32768)%65536-32768
physics_input=bytearray(struct.pack('<2048i2048i257H',*sine,*cosine,*atan));physics_expected=bytearray();randomizer=random.Random(0x6df30)
physics_input.extend(struct.pack('<I',1024))
for i in range(1024):
 at=[20736+(i%16),13568+((i*7)%16)];corners=[randomizer.randrange(-32000,32000) for _ in range(4)];bounce=[0,64,128,256][i%4]
 a=bytearray(factory(bytes(81),0,0,at,6272,40));struct.pack_into('<i',a,39,6273+i%136);out=bytearray(a);struct.pack_into('<i',out,39,6272);n=normal(at,corners);vz=i16(1088*bounce>>8);vx=i16(int(vz*n[0]/256));vy=i16(int(vz*n[1]/256));struct.pack_into('<hhh',out,45,vx,vy,vz);xy=math.isqrt(vx*vx+vy*vy);speed=math.isqrt(xy*xy+vz*vz)
 if xy:struct.pack_into('<h',out,57,angle(vx,vy))
 struct.pack_into('<hh',out,59,angle(xy,-vz) if speed else 512,speed)
 physics_input.extend(a+struct.pack('<4hh',*corners,bounce));physics_expected.extend(out)
physics_input.extend(struct.pack('<I',1024))
for i in range(1024):
 prior=bytearray(randomizer.randbytes(81));prior[6]=0;at=[20736,13568];angles=noncontact_rng(i,0,0)['angles_u16'];rng=i
 a=bytearray(factory(prior,0,0,at,6272,40));a[73]=prior[73];a[76]=prior[76];a[69]=6;struct.pack_into('<i',a,39,30000);struct.pack_into('<3H',a,57,*angles)
 sh=lambda v:v-512 if v>=512 else v+1536
 horizontal=cosine[sh(angles[1])]*angles[2]>>16
 velocity=[i16(cosine[sh(angles[0])]*horizontal>>16),i16(sine[sh(angles[0])]*horizontal>>16),i16(-sine[sh(angles[1])]*angles[2]>>16)]
 struct.pack_into('<3h',a,45,*velocity);struct.pack_into('<h',a,55,-(90>>6));seed=i*0xe62d&65535;a[29]=1+(seed>>8)%3
 physics_input.extend(prior+struct.pack('<3HH',*angles,i));physics_expected.extend(a+struct.pack('<H',seed))
physics_harness=folder/'landing-physics.rs'
physics_harness.write_text('#[path='+json.dumps(str(production).replace('\\','/'))+'] mod landing;\n'+r"""use landing::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let q=*p;*p+=n;&b[q..*p]}fn w(b:&[u8],p:&mut usize)->u16{u16::from_le_bytes(take(b,p,2).try_into().unwrap())}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let mut sine=[0;2048];let mut cosine=[0;2048];let mut atan=[0;257];for t in [&mut sine,&mut cosine]{for v in t{*v=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());}}for v in &mut atan{*v=w(&b,&mut p);}let mut out=Vec::new();let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());for _ in 0..n{let a=take(&b,&mut p,81).try_into().unwrap();let mut h=[0;4];for x in &mut h{*x=w(&b,&mut p) as i16;}let bounce=w(&b,&mut p) as i16;out.extend(plan_landing_vertical_contact(&a,6272,40,1632,true,h,bounce,&atan).unwrap().after);}let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());for _ in 0..n{let prior=take(&b,&mut p,81).try_into().unwrap();let angles=[w(&b,&mut p),w(&b,&mut p),w(&b,&mut p)];let seed=w(&b,&mut p);let mut pool=[[0;81];200];pool[0]=prior;let plan=plan_landing_effect6_factory(&pool,0,[20736,13568],6272,40,30000,angles,None,90,LandingRng{seed,ring:31,draw_count:u32::MAX},&sine,&cosine).unwrap();assert_eq!(plan.rng_after.ring,0);assert_eq!(plan.rng_after.draw_count,0);assert_eq!(plan.draw.journal_code,0x21d);assert_eq!(plan.factory.animation_index,6);out.extend(plan.factory.after_nonpointer);out.extend(plan.rng_after.seed.to_le_bytes());}assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
""",encoding='utf-8')
physics_exe=folder/'landing-physics.exe';subprocess.run([str(rustc),'--edition=2021',str(physics_harness),'-o',str(physics_exe)],env=environment(),check=True,capture_output=True)
physics_output=subprocess.run([str(physics_exe)],input=physics_input,capture_output=True,check=True).stdout
assert physics_output==physics_expected,'production first-contact/effect6 differs from independent model'
# General code6 flight and lifetime, independently generated stream.
flight_input=bytearray(struct.pack('<257H2048i2048i',*atan,*sine,*cosine)+struct.pack('<I',2048));flight_expected=bytearray()
for i in range(2048):
 a=bytearray(factory(bytes(81),0,0,[20736+i%200 if i%2 else -300+i%200,13568+i%170 if i%3 else -400+i%170],6272,40));a[69]=6;a[21]|=1;a[67]=i%256;a[68]=(i*13)%256;struct.pack_into('<hh',a,45,i%601-300,(i*7)%601-300);struct.pack_into('<h',a,49,-100-i%1800);struct.pack_into('<h',a,55,-1);struct.pack_into('<i',a,39,6273+i%500)
 h=[i%32000,(i*3)%32000,(i*7)%32000,(i*11)%32000];bounce=[0,64,128,256][i%4];out=bytearray(a);vel=list(struct.unpack_from('<hhh',a,45));vel[2]=i16(vel[2]-1);at=[]
 for j in range(2):
  fixed=(struct.unpack_from('<h',a,7+j*2)[0]<<16)+(a[67+j]<<8)+(vel[j]<<10);v=i16(fixed>>16);at.append(v);struct.pack_into('<h',out,7+j*2,v);out[67+j]=(fixed>>8)&255
 ch=struct.unpack_from('<i',a,39)[0]+(vel[2]>>3);struct.pack_into('<i',out,39,max(6272,ch,0))
 if ch<=6272:
  n=normal(at,h);vel[2]=i16(abs(vel[2])*bounce>>8);vel[0]=i16(vel[0]+int(vel[2]*n[0]/256));vel[1]=i16(vel[1]+int(vel[2]*n[1]/256))
 struct.pack_into('<hhh',out,45,*vel);xy=math.isqrt(vel[0]**2+vel[1]**2);speed=math.isqrt(xy*xy+vel[2]**2)
 if xy:struct.pack_into('<h',out,57,angle(vel[0],vel[1]))
 struct.pack_into('<hh',out,59,angle(xy,-vel[2]) if speed else 512,speed)
 for offset in [7,9]:struct.pack_into('<h',out,offset,struct.unpack_from('<h',out,offset)[0]&32767)
 flight_input.extend(a+struct.pack('<4hh',*h,bounce));flight_expected.extend(out)
first6,last6,count6,flags6=struct.unpack_from('<IIBB',db,animations+6*10)
assert (count6,flags6)==(13,0) and last6-first6==12*14 and all(db[frames+first6+i*14+10]==0 for i in range(13))
flight_input.extend(struct.pack('<I',13*3*4))
for frame,delay,counter in itertools.product(range(13),range(1,4),range(4)):
 a=bytearray(factory(bytes(81),0,0,[20736,13568],6272,40));a[69]=6;a[27]=frame;a[29]=delay;a[30]=counter;struct.pack_into('<hhh',a,45,-1001,999,-1);out=bytearray(a);c=min(counter,delay);out[30]=(c-1)&255
 if c==0:out[30]=delay;out[27]=(frame+1)%13
 retired=out[27]==12 and out[30]==0
 if not retired:
  for offset in [45,47,49]:struct.pack_into('<h',out,offset,struct.unpack_from('<h',out,offset)[0]*15>>4)
 flight_input.extend(a);flight_expected.extend(bytes([retired])+out)
flight_input.extend(struct.pack('<I',1024))
for i in range(1024):
 a=bytearray(factory(bytes(81),0,0,[20736,13568],6272,40));a[69]=6;struct.pack_into('<i',a,39,6272);a[67]=i%256;a[68]=(i*7)%256;struct.pack_into('<hhh',a,45,i%601-300,(i*7)%601-300,-1);struct.pack_into('<h',a,55,-16);g=6272+i%17;out=bytearray(a);vx,vy=struct.unpack_from('<hh',a,45);vz=(g-6272)*8;xy=math.isqrt(vx*vx+vy*vy);speed=math.isqrt(xy*xy+vz*vz);yaw=angle(vx,vy) if xy else 0;pitch=angle(xy,-vz) if speed else 512;struct.pack_into('<hhh',out,57,yaw,pitch,speed);horizontal=cosine[sh(pitch)]*speed>>16;v=[i16(cosine[sh(yaw)]*horizontal>>16),i16(sine[sh(yaw)]*horizontal>>16),i16(-sine[sh(pitch)]*speed>>16)];struct.pack_into('<hhh',out,45,*v)
 for j in range(2):
  fixed=(struct.unpack_from('<h',a,7+j*2)[0]<<16)+(a[67+j]<<8)+(v[j]<<10);struct.pack_into('<h',out,7+j*2,i16(fixed>>16));out[67+j]=(fixed>>8)&255
 struct.pack_into('<i',out,17,g+1);struct.pack_into('<i',out,39,g+1);out[23]=44
 flight_input.extend(a+struct.pack('<ii',g,g+1));flight_expected.extend(out)
flight_harness=folder/'effect6-flight.rs'
flight_harness.write_text('#[path='+json.dumps(str(production).replace('\\','/'))+'] mod landing;\n'+r"""use landing::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],p:&mut usize,n:usize)->&'a[u8]{let q=*p;*p+=n;&b[q..*p]}fn w(b:&[u8],p:&mut usize)->u16{u16::from_le_bytes(take(b,p,2).try_into().unwrap())}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut p=0;let mut atan=[0;257];for v in &mut atan{*v=w(&b,&mut p);}let mut sine=[0;2048];let mut cosine=[0;2048];for t in [&mut sine,&mut cosine]{for v in t{*v=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());}}let mut out=Vec::new();let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());for _ in 0..n{let a=take(&b,&mut p,81).try_into().unwrap();let mut h=[0;4];for x in &mut h{*x=w(&b,&mut p) as i16;}let bounce=w(&b,&mut p) as i16;out.extend(plan_effect6_flight(&a,6272,40,1632,true,h,bounce,&atan).unwrap().after);}let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());for _ in 0..n{let a=take(&b,&mut p,81).try_into().unwrap();let r=plan_effect6_animation_after_physics(&a).unwrap();out.push(r.retirement_required as u8);out.extend(r.after);}let n=u32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());for _ in 0..n{let a=take(&b,&mut p,81).try_into().unwrap();let g=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let g2=i32::from_le_bytes(take(&b,&mut p,4).try_into().unwrap());let candidate=effect6_grounded_candidate(&a,g,&atan,&sine,&cosine).unwrap();out.extend(plan_effect6_grounded_after_second_sample(&a,&candidate,g2,44,1632,true).unwrap().after);}assert_eq!(p,b.len());std::io::stdout().write_all(&out).unwrap();}
""",encoding='utf-8')
flight_exe=folder/'effect6-flight.exe';subprocess.run([str(rustc),'--edition=2021',str(flight_harness),'-o',str(flight_exe)],env=environment(),check=True,capture_output=True)
flight_output=subprocess.run([str(flight_exe)],input=flight_input,capture_output=True,check=True).stdout
assert flight_output==flight_expected,'production effect6 flight/lifetime differs from independent model'
lifecycle_source=P/'rust/crates/genewars-runtime/src/landing_effect_lifecycle.rs'
lifecycle_test=folder/'lifecycle-test.rs'
lifecycle_test.write_text('#[path='+json.dumps(str(production).replace('\\','/'))+'] mod landing;\n#[path='+json.dumps(str(lifecycle_source).replace('\\','/'))+'] mod landing_effect_lifecycle;\n',encoding='utf-8')
lifecycle_exe=folder/'lifecycle-test.exe'
subprocess.run([str(rustc),'--edition=2021','--test',str(lifecycle_test),'-o',str(lifecycle_exe)],env=environment(),check=True,capture_output=True)
subprocess.run([str(lifecycle_exe)],env=environment(),check=True,capture_output=True)
mission=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes()
assert sha(mission)=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
assert struct.unpack_from('<h',mission,0x1c5d)[0]==1024 and struct.unpack_from('<h',mission,0x1c63)[0]*2==1632 and struct.unpack_from('<i',mission,0x1c67)[0]==2000
pin('terrain-camera-refs/instructions-00071b70.txt','0007245d MOVSX EAX,word ptr [EAX + 0x1c63]','0007246b ADD EAX,EAX','0007246d MOV dword ptr [EDX + 0x1c67],EAX')
pin('landing-contact/instructions-0001f030.txt','0001f030 RET')
pin('actor-activation/instructions-0001f260.txt','0001f263 MOV EAX,dword ptr [EAX + 0x27]')
pin('entity-order-refs/instructions-0004e950.txt','0004e982 CALL dword ptr [ECX + 0x64]','0004e9bd MOV byte ptr [EDX + 0x15],CL','0004e9c5 DEC EBX','0004e9c6 CALL dword ptr [ECX + 0x64]')
pin('panel-activation-refs/instructions-000724c0.txt','000724c0 PUSH EBX')
payload=(json.dumps({'factory':factory_vectors,'phase0':phase_vectors,'rng':[dict(seed_before=s,**noncontact_rng(s,31,0xfffffffe)) for s in [0,1,5428,32768,65535]]},indent=2)+'\n').encode();(folder/'landing-fixtures.json').write_bytes(payload)
receipt={'candidate_sha256':candidate,'instruction_contracts':pins,
 'factory':{'entry':'7F3B0->48790','code_hex':'31','family':4,'native_type':5,'base':'1688AE','stride':81,'capacity':200,'admission':'Explicit freefirstslot/poolcount0..199; nofullpoolreplacement. Prior81bytesrequired. Owner0..3,positioni16pair,explicitterrainheight/shade.','writes':'See executablefactorymodel. Linkkindsnull/indexwordspreserved;position7,IDF,groundheight11,shade17,owner18;flags15word0020;animation1A..1D=[0,0,1,0],1Epreserved;physicalheight27=ground+6400hex;width2B16;velocity(0,0,-1088);acceleration0;angles(0,1024,1088);3F=-1;selector4531hex,phase460,timer4A=-1,childref47kind0/indexpreserved,4C0.1Fanimationtypedhandle49,vptr23preserved.','requests':['insertcellhead8AC60','INCfamily4countword11C2B4','saveplayer496={factoryID,kind5}','audiochannel50mode2volume127duration180','ifowner==selectedplayer,reveal75B00radius4096'],'oldhead':'NextreferenceusesresolvedoldheadID/type, requireidentityconsistency or explicitresolvedrecord. Nullnextclearskindonly.','player_reference':'Savedplayer496type5persistsduringphase0/contact/80AE0; it is not cleared.'},
 'effect_resource':{'entry_id':entry['entry_id'],'input_sha256':entry['sha256'],'decoded_sha256':sha(db),'decoded_size':len(db),'descriptor_va':'1038E4','raw_slot':'116814','animation_slot':'116818','animation':49,'frame_count':1,'frame_flags':0,'frame_offset':3486,'frame_hold':0,'fresh_animation_identity':'7AC00 with1A..1E=[0,0,1,0,0] returns same fivebytes everycall; prior1E0required, actualinactiveCDsource1E0.'},
 'phase0_contact':'Requiresactive5/code31/phase460, post6DF30physics+7AC00state andground7B140sample. Signedground>=physical27:27=ground,velocity2D/2F/31=0,phase461,timer4A150; ifchildref47kind4resolved, child3C=1/330; call80AE0ownerplayer; stopaudio50A7120,play51virtual44; tail75240followrequestifchildnonnull. Freshkind0branchadmittedbyexecutablemodel. ContactprefixconsumesnoRNGbeforeactivation. Unmentionedrecordfieldsincludingpostphysicsanglespreserved.',
 'phase0_noncontact':'Signedground<physical27: threeE62Dsharedseedupdates, ring+3mod32,count+3u32wrap. Requesteffectfactorycode06 atsameposition/physicalheight,owner0,angles(high3*8,1024-high2%128,2560+64*(high1%10)),otherargs0. Successmarknewentityflags15mask01andreturncreatedcount1;failure0. Audio50volumeparameter=70+trunc0((wrapped(physical27-ground11)>>6)*110/400), plus98210volumeadjustrequest. Factory/audio/childfolloweffectsremainexplicit, notsilentlydiscarded; thismodeldoesnotclaimtheirtransitiveRNGfootprint.',
 'physics_boundary_historical':'4B630callsfamily4virtual94=6DF30beforephase0. Freshverticalflight hasgravitydisabledbyflags15mask20 andvelocity-1088; airborneheightcandidate=physical27+(velocityZ>>3)=physical27-136.7B380refreshesgroundheight11/shade17. Fullcontact/bounce terrainnormal733A0/angleupdates andparity74270collisioncallbacksremainexplicitpostphysicsinputs; nofixed189frameorfullphysicsclaim. Explicitterrain>water+320/noobstacle gates canexcludeadditionalwater/collisioneffects, butmustnotinferlivegates.',
 'associated_activation':'80AE0scansfive200Bslotsascendinginplayerpartition. Qualifyactive!=0&&71mask04; safeonlytype3. Clearword70mask0401thenOR01,byte16&F7. Ifplayer496kind5,resolve81Bfamily4IDandcenter73E00there; repeatedlydrawradius512until74CC0terrain/plantpredicateaccepts. SetactorpositionLANDINGCENTER, theninternalopcode3-like74A50movetargetacceptedcandidateunless71mask10; thatinhibitdoesnotpreventposition/link/blueprintupdates. Admitted82zero/57mask01clearavoid51E80. Acceptedcandidateisfarther64;NULL74270buildinglookupplusalreadyaccepted74CC0predicatesmakesresolverreturncandidate. InsertactoratCENTERcell,blueprint1B=18/20|2/1C++,optionalactor72=40forfirstcategory1use. AfterallfiveINCplayer49Dexactlyonce,includingzeroqualifying. Sharedblueprints/cellheadswritesmustbesequential. Landingreference496unchanged. No-associatedrouteinsteadcopiescandidatepositionanddoesnotsetmoveorder.',
 'seed_provenance':'Missiontailcopiesseed5428to11878A, but7D090 subsequentlybranchesonRUNTIMEmode118968:mask80setsseed=(saved1169A0-1)mod65536;elsemask01useshosttimeorreceivedsynchronized2Bseed;otherwiseleavespost884D0seed. HosttimeA71C4->C55D5INT21AH2C; seed=(second+1+minute)*(centisecond+1)low16. Initial1D770alsoreadsruntimeclockandsets1169A0. ThusCDbytesalonecannotchoosecurrentseed/mode. Campaignskips8A9F0targetRNG;fresh7F580consumes64known draws oncecurrentseedknown. Subsequentordinaryfamily0/plant/effectcallbacksandphase0effectsconsume sharedstream; nofullschedulerclosure.',
 'scratch_provenance':'7FAD0doesnotwrite18/1Bofreused36Bstackrecord;7F580copiesbothinto20blueprints. Noowningpriorwriter/stackhistoryhasbeenproved. Requireexplicitcallerpriorbytes, notzeroorCDresource-derivedvalues.',
 'fixtures':{'private_path':'private/ghidra/landing-contact/landing-fixtures.json','sha256':sha(payload),'factory_vectors':32,'phase_vectors':96,'rng_vectors':5},
 'independent_executable':{'private_harness':'private/ghidra/landing-contact/landing-oracle.rs','harness_sha256':sha(harness.read_bytes()),'phase_cases':8192,'rng_seed_cases':65536,'compared_bytes':len(expected),'output_sha256':sha(native),'scope':'Freshly authored Rust scalar/byte oracle compared to Python source-basedmodel; nooriginalmachinecodeexecution. Shippinglanding.rs APIs comparedagainstsameindependentfixtures,including32factoryvectors;6shippingmoduletestsrun.'},
 'effect6_flight_comparison':{'flight_cases':2048,'grounded_cases':1024,'animation_cases':156,'compared_bytes':len(flight_expected),'output_sha256':sha(flight_output),'animation_frames':count6,'animation_flags':flags6,'animation_all_holds':0},
 'contact_effect_comparison':{'contact_cases':1024,'effect6_factory_cases':1024,'compared_bytes':len(physics_expected),'output_sha256':sha(physics_output),'tables_sha256':sha(physics_input[:16898])},
 'production_comparison':{'source_sha256':sha(production.read_bytes()),'harness_sha256':sha(production_harness.read_bytes()),'compared_bytes':len(production_expected),'output_sha256':sha(production_output),'factory_cases':32,'phase_cases':8192,'rng_seed_cases':65536},
 'lifecycle':{'production_source_sha256':sha(lifecycle_source.read_bytes()),'tests':3,'retirement':'Code6rowchildFF:49D30mapunlink,724C0clearsmatchingreferenceKINDBYTESonlyin nativekind1refs4D/50/55/58/5B(kind1flags60&~2if5Bmatch),kind3refs4D/50/7F/7B,kind5ref47,kind7refs4D/50/57/5A,kind8ref59. UI25150requestifselected;409F0nine22Bpointerrecordsclearsdeadpointer;2C200activepanel193+19For1C4+1CCpointerclear. Thenkind6=0,count11C2B4--. NoRNG. NativeemptyUIisexplicitpolicy.','scheduler':'4E950primaryascending200activeflag1clear,sumu8callbackcreationreturns. Newbornascendingactiveflag1set,clearbitbeforecallback,remaining--,ignorecallbackreturn,stopremaining0. Unaccountedcreationcountrejectsinboundedport. Queueascendingactiveexceptselector46.'},
 'mission_physics':{'entry_id':'root/2/disc/152','source_sha256':sha(mission),'gravity_i16_1C5D':1024,'serialized_water_i32_1C67':2000,'height_i16_1C63':816,'loaded_water_i32_1C67':1632,'writer':'71B70tail7245D..7246D doublesSIGNED1C63. Gravitypreserved;futureUIgravityeditsnotinferred.'},
 'phase1_quiet':'plan_landing_phase1_quiet_grounded admitsfreshphase1/4C0/nochild,zeroV/A/fractions,physical=ground,ground>water320;sourcegroundphysicssets3B512/3D0,preservesyaw;timer150through50takes100passeswithoutRNG. Beforetimer50rejectsnext49journal802UI+806particle2Fbranch,not silently skipped.',
 'first_contact_physics':'plan_landing_vertical_contact computesactual733A0triangle-normal/bouncecoefficient/AE028angles fromactual4nativeheightwords,actual8B520-indexed104530coefficientand257wordAE134table. Noobstacle/watergatesremainexplicit. Source6DF30firstimpactonly, freshverticalcode31.',
 'effect6_factory':'Nullparent/freepoolfactory usesEFFanimation6, consumesoneadditionalE62Ddrawjournal21Dafterallocation; byte1D=1+high%3;acceleration37=-(gravity11A9BD>>6). Priorchild73/76preserved;flags20. Successparentmarkmask1external. Fullpoolreplacementunadmitted.',
 'effect6_update':'plan_effect6_flight computesfractionalXY,airborneorfirstimpactnormalbounceandAE028orientation;actualcandidatepositionground/shade/corners/8B520coefficientrequired. Repeatedgroundcontact/water/nonnullobstacleunadmitted. EFF6 thirteenhold0framesflags0;delay1..3source7AC00;lastframe12counter0andt<0requests49D30retirement. Rowbyte3FFmeans4B3B0returns0andconsumesnochildRNG. Survivingselector4C3DDdamps15*v>>4withoutRNG. Hostmustexecutechangedcellunlink/relink/virtual88andretirement724C0cleanup/countdecrementtransactionally.',
 'limits':['No original launch or full live scheduler/initialseed claim.','No fullpoolreplacement/nativepointer/resource/audioexecution.','Bounded contact model begins AFTERexplicitphysics; postcontactorientationnotinvented.','Acceptedplacement and resolver environment outcomesmustbevalidated, notassumed.']}
(P/'research/ghidra/landing-evidence.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
print('PASS landing production:32factory/8192phase/65536rng;1024contact/1024effect6factory/2048effect6flight/1024grounded/156animation;9unit tests')

"""Independent arrival6E9E0/57A10 and52780 idle RNG/record comparison."""
from pathlib import Path
import struct,subprocess,hashlib,json
p=Path(__file__).resolve().parents[2];g=p/'private/ghidra';w=g/'actor-motion';sha=lambda b:hashlib.sha256(b).hexdigest();data=bytearray(struct.pack('<I',65536*4+2048));expected=bytearray();base=bytearray((i*37+13)&255 for i in range(200));base[6]=3;base[85]=8;base[73]=16;base[74]=32;base[88]=2;base[87]=0;base[112]=1;base[113]=0;base[22]=2;bp=bytearray(36);bp[21]=2;bp[22]=2;bp[24]=20;bp[25]=20;bp[26]=1;bp[27]=0;struct.pack_into('<i',base,100,7)
for case in range(65536*4+2048):
 arrival=case<65536*4;seed=case//4 if arrival else case%65536;mode,override,count=[(0,0,0),(0,1,6000),(1,1,6000),(0,0,6000)][case%4];frame=239 if arrival else 240;a=bytearray(base);b=bytearray(bp);struct.pack_into('<h',a,15,0);a[85]=8 if arrival else 1;a[88]=2 if arrival else 0
 if not arrival:struct.pack_into('<i',a,104,case%3)
 empty=case>=65536*4+1024
 if empty:b[23]=127
 if not arrival and case%2:b[21]=3
 data.extend(bytes([2 if empty else arrival])+a+b+struct.pack('<IHBH',frame,seed,mode,override)+struct.pack('<H',count));out=bytearray(a);s=seed;draws=[];ring=31;counter=0xfffffffe
 def draw(journal):
  global s,ring,counter
  s=s*0xe62d&65535;draws.append(journal);ring=(ring+1)&31;counter=(counter+1)&0xffffffff;return s>>8
 if arrival:
  struct.pack_into('<2h',out,7,20736,13568);struct.pack_into('<i',out,17,6272);out[23]=15;out[67]=out[68]=0;struct.pack_into('<h',out,65,0);out[45:51]=bytes(6);struct.pack_into('<h',out,61,0);struct.pack_into('<h',out,63,0);out[84]=0;struct.pack_into('<H',out,73,(struct.unpack_from('<H',out,73)[0]&0xfdcf)|0x220);out[22]&=63
 v=struct.unpack_from('<i',out,100)[0]
 if b[22] in (2,3) and v>0:struct.pack_into('<i',out,100,v-1)
 if frame&127==0 and b[26]<b[25]:b[26]+=1
 if arrival:
  out[22]&=63;out[84]=0;out[73]&=~16;out[88]&=~2;out[85]=1;out[104:108]=bytes(4);out[25]=8;out[28]=1;out[27]=0;out[29]=16
  if draw(0x21b)%4<2:out[112]|=4
  else:out[112]&=~4
 if frame&15==0:
  draw(0x666)
  if empty:draw(0x6bd);draw(0x6be)
 ov=override if mode&1==0 else 0
 if count!=0 and ov==0:out[25]=8
 else:
  timer=struct.unpack_from('<i',out,104)[0]-1;struct.pack_into('<i',out,104,timer)
  if timer<=0:
   out[25]=11 if ov else 8 if draw(0x6a0)%10<6 else 11;struct.pack_into('<i',out,104,20+draw(0x6a3)%30)
   if draw(0x6a5)%4<2:out[112]|=4
   else:out[112]&=~4
 if b[21]==3:out[115]=0
 if b[27]<b[24] and frame&31==0:b[27]+=1
 expected.extend(out+b+struct.pack('<HBI',s,ring,counter)+bytes([len(draws)])+struct.pack('<'+'H'*len(draws),*draws))
module=(p/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix();h=w/'idle-oracle.rs';h.write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''use actor_motion::*;use std::io::{Read,Write};fn take<'a>(b:&'a[u8],q:&mut usize,n:usize)->&'a[u8]{let s=*q;*q+=n;&b[s..*q]}fn u(b:&[u8],q:&mut usize)->u32{u32::from_le_bytes(take(b,q,4).try_into().unwrap())}fn s(b:&[u8],q:&mut usize)->u16{u16::from_le_bytes(take(b,q,2).try_into().unwrap())}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut q=0;let n=u(&b,&mut q);let mut out=Vec::new();for _ in 0..n{let tag=take(&b,&mut q,1)[0];let arrival=tag==1;let mut a=take(&b,&mut q,200).try_into().unwrap();let bp=take(&b,&mut q,36).try_into().unwrap();let frame=u(&b,&mut q);let seed=s(&b,&mut q);let mode=take(&b,&mut q,1)[0];let ov=s(&b,&mut q) as u8;let count=s(&b,&mut q);let context=ActorIdleContext{runtime_mode:mode as u16,override_180d80:ov,runtime_count_11c2bc:count,player_mode:0};let rng=ActorCallbackRng{seed,ring:31,draw_count:0xfffffffe};let r=if arrival{a=plan_actor_arrival_snap(&a,[20736,13568],6272,15).unwrap();plan_actor_arrival_idle_with_empty_scorer(&a,&bp,frame,rng,context,false,true)}else{plan_actor_idle_with_empty_scorer(&a,&bp,frame,rng,context,tag==2,true)}.unwrap();out.extend(r.after);out.extend(r.blueprint_after);out.extend(r.rng_after.seed.to_le_bytes());out.push(r.rng_after.ring);out.extend(r.rng_after.draw_count.to_le_bytes());out.push(r.draws.len()as u8);for d in r.draws{out.extend(d.journal_code.to_le_bytes());}}assert_eq!(q,b.len());std::io::stdout().write_all(&out).unwrap();}''');exe=w/'idle-oracle.exe';rustc=p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe';subprocess.run([str(rustc),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected
r={'candidate_sha256':sha((g/'GW-CD-candidate.exe').read_bytes()),'instruction_contracts':[{'path':'private/ghidra/'+f,'sha256':sha((g/f).read_bytes())}for f in ['actor-motion/instructions-0006e9e0.txt','actor-motion/instructions-00057a10.txt','activation-rng-refs/instructions-00052780.txt','activation-rng-refs/instructions-0008d7b0.txt','actor-motion/instructions-00058a70.txt']],'arrival_cases':65536*4,'idle_cases':2048,'empty_scorer_cases':1024,'seed_cases':65536,'compared_bytes':len(out),'output_sha256':sha(out),'source_sha256':sha(Path(module).read_bytes()),'scope':'Arrival snap BL1 preservesphysicalheight; arrivalidle21B and52780 globals: count0/6000,override0/1,mode0/1; normalidle0/1/2timer,periodic666chance0/126,empty-source-scorer6BD/6BE; unpoweredclass3clears73; eligible targets explicitlyexcluded.'};(p/'research/ghidra/actor-idle-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

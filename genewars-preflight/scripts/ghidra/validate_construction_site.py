"""74E80 source site classification and ordered entity-query trace."""
from pathlib import Path
import importlib.util,random,struct,subprocess,hashlib,json,uuid
p=Path(__file__).resolve().parents[2];w=p/'private/ghidra/construction-command'/uuid.uuid4().hex;w.mkdir(parents=True);sha=lambda b:hashlib.sha256(b).hexdigest();spec=importlib.util.spec_from_file_location('site_source',p/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
 def get(self,o):
  if self.pc==0x74fb4 and o=='0x0008b340':
   index=(self.r['EAX']-0x11c2be)//14;self.queries.append(index);self.r['EAX']=int((index+self.salt)%97==0);return 0x2e0000
  return super().get(o)
model=Source();f=p/'private/ghidra/terrain-map-refs/instructions-00074e80.txt';assert sha(f.read_bytes())=='3b841e53446aedf505ac35be4ac8f78a8d677e2845963a40960354bf14cd0a09','74E80 source listing changed';ext=m.m.Model([f],b'');model.code.update(ext.code);model.code[0x2e0000]=('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.mem={};offset=struct.pack('<I',model.load(0x1124f0,4));assert struct.unpack('<hh',offset)==(-128,640),'1124F0 source offset changed';rng=random.Random(0x74e80);data=bytearray(offset);expected=bytearray();steps=0;classes={}
for case in range(224):
 x=[0,127,81,40][case%4]*256+rng.randrange(256);y=[127,0,53,60][case%4]*256+rng.randrange(256);selector=case%14;salt=case%97;mapbytes=bytearray(229376)
 for i in range(16384):mapbytes[i*14+5]=2;mapbytes[i*14+13]=2
 for dy in range(-3,4):
  for dx in range(-3,4):
   cell=(((y//256+dy)&127)*128+((x//256+dx)&127))*14
   if case%4==0:mapbytes[cell+5]=rng.randrange(8)
   if case%4==1:mapbytes[cell+11]=int(rng.randrange(16)==0);mapbytes[cell+4]=32 if rng.randrange(16)==0 else 0
   if case%4==2 or case==11:mapbytes[cell+13]=0
 state={0x11c2be+i:b for i,b in enumerate(mapbytes)};state.update({0x220000+i:b for i,b in enumerate(struct.pack('<HH',x,y))});discriminator=selector if case<112 else 100;footprint=[3,1,2,3,2,2,1,2,1,2,3,3,2,1][selector];model.queries=[];model.salt=salt;report,_,_=model.run(entry=0x74e80,state_before=state,registers={'EAX':0x220000,'EDX':footprint,'EBX':discriminator});value=model.r['EAX']&255;classes[value]=classes.get(value,0)+1;expected+=bytes([value])+struct.pack('<H',len(model.queries))+b''.join(struct.pack('<H',i)for i in model.queries);data+=struct.pack('<HHBBH',x,y,discriminator,footprint,salt)+mapbytes;steps+=report['steps']
module=(p/'rust/crates/genewars-runtime/src/construction_site.rs').as_posix();h=w/'site.rs';h.write_text(f'#[path="{module}"]mod site;\n'+r'''use site::*;use std::io::{Read,Write};fn w(b:&[u8],i:usize)->u16{u16::from_le_bytes(b[i..i+2].try_into().unwrap())}fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let offset=[w(&b,0)as i16,w(&b,2)as i16];let mut out=Vec::new();for c in b[4..].chunks_exact(229384){let salt=w(c,6);let mut queries=Vec::new();let r=classify_construction_site_with_footprint(&c[8..],[w(c,0)as i16,w(c,2)as i16],c[5],c[4],offset,|i|{queries.push(i);Ok((i+salt)%97==0)}).unwrap();out.push(r as u8);out.extend_from_slice(&(queries.len()as u16).to_le_bytes());for i in queries{out.extend_from_slice(&i.to_le_bytes());}}std::io::stdout().write_all(&out).unwrap();}''');exe=w/'site.exe';subprocess.run([str(p/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'),'--edition=2021','-Awarnings',str(h),'-o',str(exe)],check=True);out=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout;assert out==expected,next((i for i,(a,b)in enumerate(zip(out,expected))if a!=b),None)
r={'cases':224,'classes':classes,'compared_bytes':len(out),'literal_instruction_steps':steps,'output_sha256':sha(out),'source_sha256':sha(Path(module).read_bytes()),'listing_sha256':sha(f.read_bytes()),'selector5_offset':list(struct.unpack('<hh',offset)),'scope':'Full74E80 source return class and ordered8B340 query port, including selector5 recursive second footprint; no original execution'};(p/'research/ghidra/construction-site-evidence.json').write_text(json.dumps(r,indent=2)+'\n');print(r)

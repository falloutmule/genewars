"""Standalone alpha geometry versus literal98C40; synthetic resource fixtures."""
from pathlib import Path
import struct,json,hashlib,importlib.util,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
vm=m.Source();paths=['private/ghidra/ui-frame/instructions-00098c40.txt','private/ghidra/sprite-scaled/instructions-00098b60.txt'];vm.code.update(m.m.Model([P/s for s in paths],b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
w=P/'private/review/building-pick'/uuid.uuid4().hex;w.mkdir(parents=True);module=P/'rust/crates/genewars-desktop/src/bin/support/building_pick.rs'
h=w/'probe.rs';exe=w/'probe.exe'
h.write_text(r'''extern crate self as genewars_runtime;
pub mod building_resources {
pub struct Metadata{pub component_count:u8}pub struct Record{pub flags:u8,pub x:i16,pub y:i16}impl Record{pub fn local_position(&self)->(i16,i16){(self.x,self.y)}}
pub struct Sprite{pub width:u8,pub height:u8,pub opaque:Vec<bool>}pub struct Component{pub record:Record,pub sprite:Sprite}pub struct BuildingAnimation0Frame{pub metadata:Metadata,pub components:Vec<Component>}}
'''+f'#[path="{module.as_posix()}"]mod pick;'+r'''
use building_resources::*;use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(6){let flags=c[0];let frame=c[1];let x=i16::from_le_bytes([c[2],c[3]]);let y=i16::from_le_bytes([c[4],c[5]]);let frames:Vec<_>=(0..2).map(|f|BuildingAnimation0Frame{metadata:Metadata{component_count:2},components:(0..2).map(|i|Component{record:Record{flags,x:i-1,y:f},sprite:Sprite{width:3,height:2,opaque:vec![true,false,true,false,true,false]}}).collect()}).collect();out.push(pick::hit_animation(&frames,frame,[100,200],[x as i32,y as i32]).unwrap().map(|i|i as u8).unwrap_or(255));}std::io::stdout().write_all(&out).unwrap();}
''')
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(h),'-o',str(exe)],env=environment(),check=True,capture_output=True)
def put(s,a,b):s.update({a+i:v for i,v in enumerate(b)})
data=bytearray();expected=bytearray()
for flags in range(4):
 for frame in [0,1,255]:
  f=min(frame,1);s={};put(s,0x400000,struct.pack('<I7xB2x',0x410000,2))
  for i in range(2):put(s,0x410000+i*10,struct.pack('<IhhBB',0x420000,i-1,f,flags,0))
  put(s,0x420000,struct.pack('<IBB',0x430000,3,2));put(s,0x430000,bytes([1,0,255,1,9,0,255,1,7,255,0]))
  for y in range(198,205):
   for x in range(97,105):
    put(s,0x300004,struct.pack('<i',y));vm.run(entry=0x98c40,state_before=s,registers={'EAX':0x400000,'EDX':100,'EBX':200,'ECX':x});expected.append(vm.r['EAX']&255);data.extend(struct.pack('<BBhh',flags,frame,x,y))
actual=subprocess.run([str(exe)],input=data,capture_output=True,check=True).stdout
assert actual==expected,next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None)
r={'status':'passed','cases':len(expected),'scope':'Literal98C40/98B60 versus exact Rust module using fixture stand-in resource types; 2 overlapping components,4 flips,frame0/1/255 clamp,transparent pixels and opaque palette0. Queue/publication/native not tested.','module_sha256':hashlib.sha256(module.read_bytes()).hexdigest(),'source_pins':{s:hashlib.sha256((P/s).read_bytes()).hexdigest()for s in paths},'output_sha256':hashlib.sha256(actual).hexdigest(),'private_run':w.relative_to(P).as_posix()}
(P/'research/campaign/building-pick-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r))

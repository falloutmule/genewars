"""Literal7F3B0 with explicit read-only height/factory/reveal ports."""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
work=P/'private/review/landing-request'/uuid.uuid4().hex;work.mkdir(parents=True)
spec=importlib.util.spec_from_file_location('oracle',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
 def load(self,a,n):
  if a==0x11873d and n==1:return self.selected
  return super().load(a,n)
 def get(self,operand):
  if self.pc==0x7f423 and operand=='0x2e0000':
   self.height_point=struct.pack('<I',self.load(self.r['EAX'],4))
  if self.pc==0x7f436 and operand=='0x2e0010':
   self.factory=struct.pack('<BII3H4I',self.r['EAX']&255,self.r['EDX'],self.r['EBX'],*[self.load(self.r['ECX']+i*2,2)for i in range(3)],*[self.load(self.r['ESP']+i*4,4)for i in range(4)])
  if self.pc==0x7f465 and operand=='0x2e0020':
   self.reveal=struct.pack('<IH',self.load(self.r['EAX'],4),self.r['EDX'])
  return super().get(operand)
model=Source();listing=P/'private/ghidra/entity-shadow-panel/instructions-0007f3b0.txt';model.code.update(m.m.Model([listing],b'').code)
for pc,target in [(0x7f423,0x2e0000),(0x7f436,0x2e0010),(0x7f465,0x2e0020)]:model.code[pc]=('CALL',[hex(target)])
model.code[0x2e0001]=('RET',[]);model.code[0x2e0011]=('RET',['0x10']);model.code[0x2e0020]=('RET',[])
randomizer=random.Random(0x7f3b0);data=bytearray();expected=bytearray();steps=0
for case in range(256):
 player=bytearray(randomizer.randbytes(1805));player[1]=case%4;player[2]=[0,1,255][(case//16)%3]
 counter=[0,1,0x80000000,0xffffffff][(case//4)%4];special=struct.pack('<hh',randomizer.randrange(-32768,32768),randomizer.randrange(-32768,32768));selected=(case//8)%4
 height=[-65536,-1,0,6272,65534][case%5];success=(case//2)%2
 effect=bytearray(randomizer.randbytes(81));effect[6]=5;struct.pack_into('<H',effect,15,case%200)
 model.selected=selected;model.factory=None;model.reveal=None;model.height_point=None
 model.code[0x2e0000]=('MOV',['EAX',hex(height&0xffffffff)])
 model.code[0x2e0010]=('MOV',['EAX',hex(0x1688ae if success else 0)])
 order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 state={0x118d74+i:v for i,v in enumerate(player)};state.update({0x1688ae+i:v for i,v in enumerate(effect)})
 state.update({0x11ac22+i:v for i,v in enumerate(struct.pack('<I',counter))});state.update({0x11a9fa+i:v for i,v in enumerate(special)})
 report,_,_=model.run(state_before=state,entry=0x7f3b0,registers={'EAX':0x118d74+0x15,'EDX':0x118d74})
 assert report['map_changed_bytes']==0 and report['draws']==0
 output=bytes(model.load(0x118d74+i,1)for i in range(1805));created=model.r['EAX']&255
 expected+=model.height_point+model.factory+output+bytes([created,model.reveal is not None])+(model.reveal or bytes(6))
 data+=player+struct.pack('<I',counter)+special+bytes([selected])+struct.pack('<iB',height,success)+effect;steps+=report['steps']
module=P/'private/integration/landing-request-draft.rs'
harness=work/'harness.rs';harness.write_text('#[path="'+module.as_posix()+'"]mod draft;'+r'''
use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(1900){let player=c[..1805].try_into().unwrap();let counter=u32::from_le_bytes(c[1805..1809].try_into().unwrap());let special=[i16::from_le_bytes(c[1809..1811].try_into().unwrap()),i16::from_le_bytes(c[1811..1813].try_into().unwrap())];let selected=c[1813];let height=i32::from_le_bytes(c[1814..1818].try_into().unwrap());let effect=c[1819..].try_into().unwrap();let q=draft::request(player,counter,special,|pos|{for x in pos{out.extend(x.to_le_bytes());}Ok::<_,()>(height)}).unwrap();out.push(q.object_code);for x in q.position{out.extend(x.to_le_bytes());}out.extend(q.height.to_le_bytes());for x in q.motion{out.extend(x.to_le_bytes());}for x in q.options{out.extend(x.to_le_bytes());}let done=draft::complete(player,selected,if c[1818]!=0{Some(effect)}else{None});out.extend(done.player);out.push(done.created as u8);out.push(done.reveal.is_some()as u8);let(pos,radius)=done.reveal.unwrap_or(([0,0],0));for x in pos{out.extend(x.to_le_bytes());}out.extend(radius.to_le_bytes());}std::io::stdout().write_all(&out).unwrap();}
''',encoding='utf-8')
binary=work/'harness.exe';r=subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-Awarnings','-C','overflow-checks=yes',str(harness),'-o',str(binary)],cwd=P,env=environment(),capture_output=True);assert r.returncode==0,r.stderr.decode()
result=subprocess.run([str(binary)],input=data,capture_output=True,check=True).stdout
(work/'expected.bin').write_bytes(expected);(work/'actual.bin').write_bytes(result)
assert result==expected,next((i for i,(a,b)in enumerate(zip(result,expected))if a!=b),('length',len(result),len(expected)))
receipt={'status':'passed-private-landing-request-draft','cases':256,'compared_bytes':len(result),'literal_instructions':steps,'listing_sha256':hashlib.sha256(listing.read_bytes()).hexdigest(),'draft_sha256':hashlib.sha256(module.read_bytes()).hexdigest(),'ports':['7B140 read-only height','48790 factory supplied null/success record','75B00 reveal request only'],'scope':'Source7F3B0 position/optional-argument selection, factory calling convention and player reference/reveal completion. Not factory execution or whole-world integration.'}
(work/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt));print(work.relative_to(P))

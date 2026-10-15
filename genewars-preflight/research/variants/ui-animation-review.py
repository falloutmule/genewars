"""Literal7AC00 five-byte state comparison; synthetic edges plus originalPLGBL."""
from pathlib import Path
import importlib.util,struct,json,hashlib,random,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
sp=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
LIST=P/'private/ghidra/landing-contact/instructions-0007ac00.txt'
class Model(m.Source):
 def __init__(self):m.m.Model.__init__(self,[LIST],b'')
 def load(self,a,n):return m.m.Model.load(self,a,n)
 def step(self,state,holds,flags):
  self.r=dict(EAX=0x400000,EDX=0x500000,EBX=0,ECX=0,ESI=0,EDI=0,EBP=0,ESP=0x300000);self.mem={};self.pc=0x7ac00;self.z=self.s=self.cf=self.of=False
  self.store(0x300000,4,0xdeadbeef);self.store(0x400000,4,0x600000);self.store(0x400004,4,0x600000+14*(len(holds)-1));self.store(0x400008,1,len(holds));self.store(0x400009,1,flags)
  for i,v in enumerate(state):self.store(0x500000+i,1,v)
  for i,v in enumerate(holds):self.store(0x600000+14*i+10,1,v)
  for n in range(150):
   if self.pc==0xdeadbeef:break
   op,a=self.code[self.pc];nxt=self.next.get(self.pc,0xdeadbeef)
   if op=='MOV':self.put(a[0],self.get(a[1]))
   elif op in ('MOVSX','MOVZX'):self.put(a[0],m.signed(self.get(a[1]),self.width(a[1]))if op=='MOVSX'else self.get(a[1]))
   elif op=='LEA':self.put(a[0],self.addr(a[1]))
   elif op=='PUSH':self.push(self.get(a[0]))
   elif op=='POP':self.put(a[0],self.pop())
   elif op in ('ADD','SUB','XOR'):self.put(a[0],self.arithmetic(op,self.get(a[0]),self.get(a[1]),self.width(a[0])))
   elif op=='DEC':self.put(a[0],self.arithmetic(op,self.get(a[0]),1,self.width(a[0])))
   elif op in ('CMP','TEST'):self.arithmetic(op,self.get(a[0]),self.get(a[1]),self.width(a[0]))
   elif op.startswith('J'):
    if op=='JMP'or self.cond(op[1:]):nxt=self.get(a[0])
   elif op=='RET':nxt=self.pop()
   else:raise AssertionError(op)
   self.pc=nxt
  else:raise AssertionError('budget')
  index=(self.r['EAX']-0x600000)//14;assert 0<=index<len(holds)
  return bytes(self.load(0x500000+i,1)for i in range(5))+bytes([index]),n

def main():
 W=P/'private/review/ui-animation'/uuid.uuid4().hex;W.mkdir(parents=True);model=Model();cases=[];rng=random.Random(0x7ac00)
 for count in [1,2,4,8,255]:
  for flags in ([0,2,254]if count==1 else[0,1,2,255]):
   holds=[rng.randrange(256)for _ in range(count)]
   for frame in [0,count-1,255]:
    for direction in [0,1,255,127,128]:
     for delay in [0,1,127,128,255]:cases.append(([255,frame,direction,delay,rng.choice([0,1,127,128,255])],holds,flags))
   for _ in range(150):cases.append(([rng.randrange(256)for _ in range(5)],holds,flags))
 asset=P/'private/review/PLGBL-decoded.bin';b=asset.read_bytes();assert hashlib.sha256(b).hexdigest()=='a39cc8ac8c987de5c4865ec77a4459221f276286f254653e4a3f3231424f6444';_,_,_,f,a=struct.unpack_from('<H4I',b)
 descriptors=[]
 for offset in [0x15e]+[0x118+i*10 for i in range(7)]:
  start,end,count,flags=struct.unpack_from('<IIBB',b,a+offset);holds=[b[f+start+14*i+10]for i in range(count)];state=[0,0,1,1,0];descriptors.append({'offset':offset,'holds':holds,'flags':flags})
  for _ in range(128):cases.append((state,holds,flags));state=list(model.step(state,holds,flags)[0][:5])
 inputs=bytearray();expected=bytearray();steps=0
 for state,holds,flags in cases:
  out,n=model.step(state,holds,flags);inputs+=bytes([len(holds),flags])+bytes(state)+bytes(holds)+bytes(255-len(holds));expected+=out;steps+=n
 (W/'input.bin').write_bytes(inputs);(W/'expected.bin').write_bytes(expected);(W/'plgbl.anb').write_bytes(b)
 cmd=[P/'private/tools/rust/cargo/bin/rustc.exe','--edition=2021','-O',Path(__file__).with_suffix('.rs'),'-o',W/'review.exe'];r=subprocess.run([str(x)for x in cmd],env=environment(),capture_output=True,timeout=60);assert r.returncode==0,r.stderr.decode(errors='replace')
 r=subprocess.run([str(W/'review.exe'),str(W)],capture_output=True,timeout=30);assert r.returncode==0,r.stderr.decode(errors='replace');actual=(W/'actual.bin').read_bytes();assert actual==expected,[(i,x,y)for i,(x,y)in enumerate(zip(actual,expected))if x!=y][:10]
 obj=P/'private/ghidra/validated/memory-.object3.bin';raw=obj.read_bytes();mapped={hex(at):list(raw[at-0x100000:at-0x100000+5])for at in [0x184d04]+[0x184d0c+5*i for i in range(7)]};assert all(v==[0]*5 for v in mapped.values())
 pins=[LIST,obj,P/'private/ghidra/mission-array-refs/instructions-00010010.txt',asset,P/'rust/crates/genewars-runtime/src/ui_animation.rs',Path(__file__),Path(__file__).with_suffix('.rs')]
 result={'status':'passed','cases':len(cases),'compared_bytes':len(actual),'literal_instructions':steps,'private_run':str(W),'mapped_priors':mapped,'initialized_state':[0,0,1,1,0],'initialization':'10010 writes2/3=1 only; preserveotherpriors onlatercalls','descriptors':descriptors,'sha256':{str(x.relative_to(P)):hashlib.sha256(x.read_bytes()).hexdigest()for x in pins},'limits':['nonemptycontiguousframearray; singleframepingpong explicitlyunsupported','numericstateonly; hostresourceidentity/renderingseparate','10010 startup initial state; no claimitresetseverymission']}
 Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in ['status','cases','compared_bytes','literal_instructions','private_run']}))
if __name__=='__main__':
 try:main()
 except Exception as e:Path(__file__).with_suffix('.json').write_text(json.dumps({'status':'error','error':repr(e)})+'\n');raise

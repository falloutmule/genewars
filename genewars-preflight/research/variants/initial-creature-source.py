"""Closed listing interpreter for bounded initial kind1 investigation.

Reads private pinned disassembly as data, never runs the original program.
The queue callback is deliberately a separate explicit request boundary.
"""
from pathlib import Path
import hashlib,importlib.util,json,math,re,struct
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('picker_model',Path(__file__).with_name('actor-environment-terrain-target.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
signed=m.signed;trunc=m.trunc
class Source(m.Model):
 def __init__(self):
  files=[P/'private/ghidra/player-creation'/f'instructions-000{x}.txt'for x in ['5ed20','6fb60','5eb30','5e640','1ec50']]
  files +=[P/'private/ghidra/initial-creature'/f'instructions-000{x}.txt'for x in ['6f980','6b150','23800','23820','1ec10','1ec20','1f060','5b400','6f8f0','8b340','5ad70','6db00','5a110']]
  files +=[P/'private/ghidra/actor-activation'/f'instructions-000{x}.txt'for x in ['6df30','7b380','1f260','1ee20','1f220']]
  files +=[P/'private/ghidra/landing-contact'/f'instructions-000{x}.txt'for x in ['733a0','7ac00']]
  files +=[P/'private/ghidra/primitives/instructions-0007a6c0.txt',P/'private/ghidra/activation-rng-refs/instructions-0005e0b0.txt',P/'private/ghidra/terrain-map-refs/instructions-0008b520.txt',P/'private/ghidra/mission-array-refs/instructions-0006d390.txt',P/'private/ghidra/initial-creature/instructions-000736f0.txt',P/'private/ghidra/terrain-map-refs/instructions-0007b140.txt',P/'private/ghidra/actor-activation/instructions-0001ee10.txt']
  files +=[P/'private/ghidra/command-resolver'/f'instructions-000{x}.txt'for x in ['74270','8b130']]+[P/'private/ghidra/terrain-map-refs'/f'instructions-000{x}.txt'for x in ['74d90','74160']]
  files +=[P/'private/ghidra/initial-creature'/f'instructions-000{x}.txt'for x in ['401a0','8d930','6b700','73970','8b7d0','5b100','8b780']]+[P/'private/ghidra/activation-rng-refs/instructions-0008d7b0.txt',P/'private/ghidra/primitives/instructions-00073690.txt',P/'private/ghidra/command-resolver/instructions-000ae028.txt',P/'private/ghidra/plant-startup/instructions-0001ebd0.txt',P/'private/ghidra/mission-array-refs/instructions-0005d700.txt',P/'private/ghidra/activation-rng-refs/instructions-00073c20.txt',P/'private/ghidra/actor-activation/instructions-00074cc0.txt']
  files +=[P/'private/ghidra/initial-creature'/f'instructions-000{x}.txt'for x in ['13e50','13e70','5a990','3fbe0']]+[P/'private/ghidra/player-creation/instructions-00013eb0.txt']
  files +=[P/'private/ghidra/actor-activation/instructions-0001f410.txt']
  files +=[P/'private/ghidra/terrain-map-refs/instructions-000743c0.txt']
  files +=[P/'private/ghidra/terrain-map-refs/instructions-0008acc0.txt',P/'private/ghidra/terrain-map-refs/instructions-0008ac60.txt']
  files +=[P/'private/ghidra/initial-creature/instructions-0006f0c0.txt',P/'private/ghidra/initial-creature/instructions-0006ede0.txt']
  files +=[P/'private/ghidra/initial-creature'/f'instructions-000{x}.txt'for x in ['1f010','73770','1f270','1f450']]
  files +=[P/'private/ghidra/actor-motion/instructions-0006e9e0.txt']
  files +=[P/'private/ghidra/mission-array-refs/instructions-0005cc60.txt']
  super().__init__(files,b'');self.files=files;self.objects=[(0x10000,(P/'private/ghidra/validated/memory-.object1.bin').read_bytes()),(0x100000,(P/'private/ghidra/validated/memory-.object3.bin').read_bytes())]
 def load(self,a,n):
  value=0
  for i in range(n):
   at=a+i
   if at in self.mem:b=self.mem[at]
   else:
    found=next((data[at-base]for base,data in self.objects if base<=at<base+len(data)),None)
    assert found is not None or 0x2f0000<=at<=0x300100,(hex(self.pc),'unbound read',hex(at))
    if found is None:self.uninitialized_stack_reads.add((self.pc,at))
    b=found if found is not None else self.stack_fill
   value|=b<<(i*8)
  return value
 def store(self,a,n,v):
  for i in range(n):self.mem[a+i]=(v>>(8*i))&255
 def arithmetic(self,op,a,b,bits):
  mask=(1<<bits)-1;oldcf=self.cf
  if op in ('ADD','INC'):v=a+b;cf=v>mask;of=bool((~(a^b)&(a^v))>>(bits-1)&1)
  elif op in ('SUB','CMP','DEC','SBB'):b+=int(oldcf)if op=='SBB'else 0;v=a-b;cf=a<b;of=bool(((a^b)&(a^v))>>(bits-1)&1)
  else:v={'XOR':a^b,'OR':a|b,'AND':a&b,'TEST':a&b}[op];cf=of=False
  self.flags(v,bits);self.cf=oldcf if op in ('INC','DEC')else cf;self.of=of;return v&mask
 def cond(self,k):return {'Z':self.z,'NZ':not self.z,'S':self.s,'NS':not self.s,'C':self.cf,'NC':not self.cf,'A':not self.cf and not self.z,'BE':self.cf or self.z,'G':not self.z and self.s==self.of,'GE':self.s==self.of,'L':self.s!=self.of,'LE':self.z or self.s!=self.of,'B':self.cf,'AE':not self.cf}[k]
 def run(self,counter=1,seed=5428,native_before=None,state_before=None,stack_fill=0,entry=0x5ed20,registers=None,capture_physics=False,instruction_budget=1000000,gravity=1024):
  mission=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes();packed=mission[0x3b55e:0x3b55e+116];native=native_before if native_before is not None else packed[:31]+bytes(8)+packed[31:];self.mem={}
  self.r=dict(EAX=0x1542be,EBX=0,ECX=0,EDX=0,ESI=0,EDI=0,EBP=0,ESP=0x300000);self.pc=0x5ed20;self.z=self.s=self.cf=self.of=False;self.queue=[];self.calls=[];self.uninitialized_stack_reads=set();self.stack_fill=stack_fill
  for a,data in [(0x1542be,native),(0x11c2be,mission[0x355e:0x355e+229376])]:
   for i,b in enumerate(data):self.mem[a+i]=b
  for i in range(512):
   raw=mission[0x43d4e+i*41:0x43d4e+(i+1)*41];plant=raw[:31]+bytes(8)+raw[31:]
   for j,b in enumerate(plant):self.mem[0x15d40e+i*49+j]=b
   self.store(0x15d40e+i*49+0x23,4,0x1120ac)
  self.store(0x1542be+0x23,4,0x111fec)
  anb=bytearray((P/'private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB').read_bytes());_,table,components,frames,animations=struct.unpack_from('<H4I',anb)
  for i in range(struct.unpack_from('<H',anb)[0]):
   for offset in [0,4]:struct.pack_into('<I',anb,animations+i*10+offset,0x400000+frames+struct.unpack_from('<I',anb,animations+i*10+offset)[0])
  for i,b in enumerate(anb):self.mem[0x400000+i]=b
  self.store(0x1542be+0x1f,4,0x400000+animations)
  if state_before is not None:self.mem.update(state_before)
  else:self.store(0x11872c,1,31);self.store(0x118782,4,0xffffffff)
  rng_count_before=self.load(0x118782,4)
  for a,n,v in [(0x11877e,4,counter),(0x11873d,1,0),(0x11a9bd,2,gravity),(0x11a9c1,2,struct.unpack_from('<H',mission,0x1c61)[0]),(0x11a9c7,4,1632),(0x11878a,2,seed),(0x300000,4,0xdeadbeef)]:self.store(a,n,v)
  self.pc=entry
  if registers:self.r.update(registers)
  before=self.mem.copy();trace=[];self.physics_snapshots=[];self.contact_snapshots=[];self.waypoints=[];physics_return=None;contact_return=None;waypoint_return=None
  for steps in range(instruction_budget):
   if self.pc==0xdeadbeef:break
   if self.pc not in self.code:raise AssertionError(('missing listing',hex(self.pc),[hex(x)for x in self.calls[-12:]],'recent',[hex(x)for x in trace[-30:]],'state',self.load(0x1542be+0x5e,1),'owner',self.load(0x1542be+0x18,1),'6c',self.load(0x1542be+0x6c,2)))
   op,args=self.code[self.pc];nxt=self.next.get(self.pc,0xdeadbeef);trace.append(self.pc)
   if capture_physics and self.pc==0x6df30:
    self.physics_snapshots.append(dict(before=bytes(self.load(0x1542be+i,1)for i in range(124)),map=bytes(self.load(0x11c2be+i,1)for i in range(229376))));physics_return=self.load(self.r['ESP'],4)
   if self.pc==physics_return:
    self.physics_snapshots[-1]['after']=bytes(self.load(0x1542be+i,1)for i in range(124));physics_return=None
   if capture_physics and self.pc==0x5b400:
    self.contact_snapshots.append(dict(before=bytes(self.load(0x1542be+i,1)for i in range(124)),normal=[signed(self.load(self.r['EDX']+i*4,4))for i in range(3)],previous_low=signed(self.r['EBX']&65535,16)));contact_return=self.load(self.r['ESP'],4)
   if self.pc==contact_return:
    self.contact_snapshots[-1]['after']=bytes(self.load(0x1542be+i,1)for i in range(124));contact_return=None
   if self.pc==0x6f0c0:
    self.waypoints.append(dict(start=self.r['EDX'],side=self.r['EBX']&255,output_ptr=self.r['ECX'],limit=self.load(self.r['ESP']+4,1)));waypoint_return=self.load(self.r['ESP'],4)
   if self.pc==waypoint_return:
    self.waypoints[-1].update(cost=self.r['EAX']&255,output=self.load(self.waypoints[-1]['output_ptr'],4));waypoint_return=None
   if op=='MOV':self.put(args[0],self.get(args[1]))
   elif op in ('MOVSX','MOVZX'):self.put(args[0],signed(self.get(args[1]),self.width(args[1]))if op=='MOVSX'else self.get(args[1]))
   elif op=='LEA':self.put(args[0],self.addr(args[1]))
   elif op=='PUSH':self.push(self.get(args[0]))
   elif op=='POP':self.put(args[0],self.pop())
   elif op=='LEAVE':self.r['ESP']=self.r['EBP'];self.r['EBP']=self.pop()
   elif op=='XCHG':a=self.get(args[0]);b=self.get(args[1]);self.put(args[0],b);self.put(args[1],a)
   elif op=='NOP':pass
   elif op in ('ADD','SUB','XOR','OR','AND','SBB'):self.put(args[0],self.arithmetic(op,self.get(args[0]),self.get(args[1]),self.width(args[0])))
   elif op in ('CMP','TEST'):self.arithmetic(op,self.get(args[0]),self.get(args[1]),self.width(args[0]))
   elif op in ('INC','DEC'):self.put(args[0],self.arithmetic(op,self.get(args[0]),1,self.width(args[0])))
   elif op=='NEG':a=self.get(args[0]);bits=self.width(args[0]);v=-a;self.flags(v,bits);self.cf=a!=0;self.of=a==1<<(bits-1);self.put(args[0],v)
   elif op in ('SHL','SAR','SHR'):
    bits=self.width(args[0]);a=self.get(args[0]);b=self.get(args[1])&31
    if b:self.cf=bool(a>>(bits-b)&1)if op=='SHL'else bool(a>>(b-1)&1)
    v=a<<b if op=='SHL'else signed(a,bits)>>b if op=='SAR'else a>>b;self.put(args[0],v);self.flags(v,bits);self.of=False
   elif op=='SHRD':a=self.get(args[0]);b=self.get(args[1]);c=self.get(args[2])&31;self.put(args[0],((b<<32)|a)>>c)
   elif op=='IMUL':
    if len(args)==1:v=signed(self.r['EAX'])*signed(self.get(args[0]));self.r['EAX']=v&0xffffffff;self.r['EDX']=(v>>32)&0xffffffff
    else:self.put(args[0],signed(self.get(args[1]))*signed(self.get(args[2]))if len(args)==3 else signed(self.get(args[0]))*signed(self.get(args[1])))
   elif op in ('IDIV','DIV'):
    v=(self.r['EDX']<<32)|self.r['EAX'];d=self.get(args[0]);v=signed(v,64)if op=='IDIV'else v;d=signed(d)if op=='IDIV'else d;q=trunc(v,d)if op=='IDIV'else v//d;self.r['EAX']=q&0xffffffff;self.r['EDX']=(v-q*d)&0xffffffff
   elif op=='CDQ':self.r['EDX']=0xffffffff if self.r['EAX']&0x80000000 else 0
   elif op=='CWDE':self.r['EAX']=signed(self.r['EAX']&65535,16)&0xffffffff
   elif op.startswith('SET'):self.put(args[0],int(self.cond(op[3:])))
   elif op=='CALL':
    target=self.get(args[0]);self.calls.append(target)
    if target==0x97910:self.r['EAX']=math.isqrt(self.r['EAX'])
    elif target==0x982c0:
     for i in range(self.r['EBX']):self.store(self.r['EAX']+i,1,self.r['EDX']&255)
    elif target==0x83140:self.queue.append(bytes(self.mem.get(0x1542be+i,0)for i in range(124)));self.r['EAX']=0
    else:self.push(nxt);nxt=target
   elif op=='RET':nxt=self.pop();self.r['ESP']+=self.get(args[0])if args else 0
   elif op.startswith('J'):
    if op=='JMP'or self.cond(op[1:]):nxt=self.get(args[0])
   elif op in ('MOVSD','MOVSW'):
    n=4 if op=='MOVSD'else 2;self.store(self.r['EDI'],n,self.load(self.r['ESI'],n));self.r['EDI']+=n;self.r['ESI']+=n
   else:raise AssertionError((hex(self.pc),op,args))
   self.pc=nxt
  else:raise AssertionError(('model instruction budget exceeded',hex(self.pc),[hex(x)for x in trace[-15:]]))
  after=bytes(self.mem.get(0x1542be+i,0)for i in range(124));deltas=[dict(offset=i,before=before[0x1542be+i],after=after[i])for i in range(124)if before[0x1542be+i]!=after[i]]
  map_changed=[a for a in range(0x11c2be,0x11c2be+229376)if self.mem[a]!=before[a]]
  self.saved_state=self.mem.copy()
  return dict(counter=counter,seed_before=seed,steps=steps,delta=deltas,after_sha256=hashlib.sha256(after).hexdigest(),map_changed_bytes=len(map_changed),rng_seed_after=self.load(0x11878a,2),rng_ring_after=self.load(0x11872c,1),rng_draw_count_after=self.load(0x118782,4),draws=(self.load(0x118782,4)-rng_count_before)&0xffffffff,queue_requests=len(self.queue),uninitialized_stack_reads=len(self.uninitialized_stack_reads),calls=sorted(set(hex(x)for x in self.calls))),after,trace
if __name__=='__main__':
 model=Source();report,after,trace=model.run();work=P/'private/review/initial-creature';work.mkdir(parents=True,exist_ok=True);(work/'native-after.bin').write_bytes(after);(work/'source-trace.json').write_text(json.dumps(dict(report=report,trace=[hex(x)for x in trace]),indent=2)+'\n');print(json.dumps(report,indent=2))

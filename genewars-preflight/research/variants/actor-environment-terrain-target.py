"""Independent literal-register model of the two pinned pure picker routines.

No DOS emulator, original process, UI or native game instruction execution.
The interpreter recognizes only the closed arithmetic/listing subset below.
"""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,random,re,struct,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
WORK=P/'private/review/actor-environment-terrain-target'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def signed(x,bits=32):return (x+(1<<(bits-1)))%(1<<bits)-(1<<(bits-1))
def trunc(a,b):return (abs(a)//abs(b))*(-1 if (a<0)!=(b<0)else 1)
class Model:
 def __init__(self,listings,mapdata):
  self.code={}
  for path in listings:
   for line in path.read_text().splitlines():
    m=re.fullmatch(r'([0-9a-f]{8}) ([A-Z]+)(?: (.*))?',line);assert m,line
    self.code[int(m[1],16)]=(m[2],[]if not m[3]else m[3].split(','))
  order=sorted(self.code);self.next=dict(zip(order,order[1:]));self.map=mapdata
  self.alias={}
  for full,low,high,short in [('EAX','AL','AH','AX'),('EBX','BL','BH','BX'),('ECX','CL','CH','CX'),('EDX','DL','DH','DX')]:self.alias[low]=(full,8,0);self.alias[high]=(full,8,8);self.alias[short]=(full,16,0)
  for short,full in [('SI','ESI'),('DI','EDI'),('BP','EBP'),('SP','ESP')]:self.alias[short]=(full,16,0)
 def width(self,op):
  if op.startswith('byte ptr '):return 8
  if op.startswith('word ptr '):return 16
  if op in self.alias:return self.alias[op][1]
  return 32
 def addr(self,op):
  expr=op[op.index('[')+1:op.index(']')].replace(' ','');total=0
  for term in re.findall(r'[+-]?[^+-]+',expr):
   sign=-1 if term.startswith('-')else 1;term=term.lstrip('+-');v=1
   for factor in term.split('*'):v*=self.r[factor]if factor in self.r else int(factor,0)
   total+=sign*v
  return total&0xffffffff
 def load(self,a,n):
  return sum((self.map[a+i-0x11c2be]if 0x11c2be<=a+i<0x11c2be+len(self.map)else self.mem.get(a+i,0))<<(8*i)for i in range(n))
 def store(self,a,n,v):
  for i in range(n):self.mem[a+i]=(v>>(8*i))&255
  if 0x7ae20<=self.pc<=0x7aef4 and a in [0x300000,0x300002]:self.writes+=1
 def get(self,op):
  if '['in op:return self.load(self.addr(op),self.width(op)//8)
  if op in self.r:return self.r[op]
  if op in self.alias:
   reg,bits,shift=self.alias[op];return (self.r[reg]>>shift)&((1<<bits)-1)
  return int(op,0)&0xffffffff
 def put(self,op,v):
  bits=self.width(op);v&=(1<<bits)-1
  if '['in op:self.store(self.addr(op),bits//8,v)
  elif op in self.alias:
   reg,bits,shift=self.alias[op];mask=((1<<bits)-1)<<shift;self.r[reg]=(self.r[reg]&~mask)|(v<<shift)
  else:self.r[op]=v
 def flags(self,v,bits=32):self.z=(v&((1<<bits)-1))==0;self.s=signed(v,bits)<0
 def push(self,v):self.r['ESP']=(self.r['ESP']-4)&0xffffffff;self.store(self.r['ESP'],4,v)
 def pop(self):v=self.load(self.r['ESP'],4);self.r['ESP']+=4;return v
 def run(self,camera,screen):
  self.r=dict(EAX=0x300000,EDX=0x300002,EBX=screen[0],ECX=screen[1],ESI=0,EDI=0,EBP=0,ESP=0x200000);self.mem={};self.pc=0x7b550;self.z=False;self.s=False;self.writes=0;self.triangle=0
  for a,v in [(0x184fd4,32),(0x184fd8,16),(0x184fdc,camera[0]),(0x184fe0,camera[1]),(0x184fe4,2)]:self.store(a,4,v)
  self.store(0x200000,4,0xdeadbeef);self.store(0x300000,4,0xa5a5a5a5)
  for count in range(20000):
   if self.pc==0xdeadbeef:break
   op,args=self.code[self.pc];nxt=self.next.get(self.pc,0xdeadbeef)
   if op=='MOV':self.put(args[0],self.get(args[1]))
   elif op in ('MOVSX','MOVZX'):
    v=self.get(args[1]);self.put(args[0],signed(v,self.width(args[1]))if op=='MOVSX'else v)
   elif op=='LEA':self.put(args[0],self.addr(args[1]))
   elif op=='PUSH':self.push(self.get(args[0]))
   elif op=='POP':self.put(args[0],self.pop())
   elif op in ('ADD','SUB','XOR','AND','IMUL'):
    a=self.get(args[0]);b=self.get(args[1]);v={'ADD':lambda:a+b,'SUB':lambda:a-b,'XOR':lambda:a^b,'AND':lambda:a&b,'IMUL':lambda:b*self.get(args[2])if len(args)==3 else a*b}[op]();self.put(args[0],v);self.flags(v,self.width(args[0]))
   elif op in ('INC','DEC','NEG'):
    a=self.get(args[0]);v=a+1 if op=='INC'else a-1 if op=='DEC'else -a;self.put(args[0],v);self.flags(v,self.width(args[0]))
   elif op in ('SHL','SAR'):
    bits=self.width(args[0]);a=self.get(args[0]);b=self.get(args[1])&31;v=a<<b if op=='SHL'else signed(a,bits)>>b;self.put(args[0],v);self.flags(v,bits)
   elif op=='CMP':self.flags(self.get(args[0])-self.get(args[1]),self.width(args[0]))
   elif op=='TEST':self.flags(self.get(args[0])&self.get(args[1]),self.width(args[0]))
   elif op=='CWDE':self.r['EAX']=signed(self.r['EAX']&65535,16)&0xffffffff
   elif op=='IDIV':
    dividend=signed((self.r['EDX']<<32)|self.r['EAX'],64);divisor=signed(self.get(args[0]));q=trunc(dividend,divisor);assert -2**31<=q<2**31;self.r['EAX']=q&0xffffffff;self.r['EDX']=(dividend-q*divisor)&0xffffffff
   elif op=='CALL':
    assert self.get(args[0])==0x7ae20;self.triangle=0 if self.pc==0x7b6a1 else 1;self.push(nxt);nxt=0x7ae20
   elif op=='RET':nxt=self.pop();self.r['ESP']+=self.get(args[0])if args else 0
   elif op in ('JMP','JZ','JNZ','JL','JGE','JG'):
    take={'JMP':True,'JZ':self.z,'JNZ':not self.z,'JL':self.s,'JGE':not self.s,'JG':not self.s and not self.z}[op]
    if take:nxt=self.get(args[0])
   else:raise AssertionError((hex(self.pc),op,args))
   self.pc=nxt
  else:raise AssertionError('bounded model exceeded instruction cap')
  if self.writes!=2:return bytes(9)
  point=[self.load(0x300000,2),self.load(0x300002,2)];cell=((point[1]>>8)&127)*128+((point[0]>>8)&127)
  pair=self.load(0x200000-12-0x38+0x34,1)
  return b'\1'+struct.pack('<3H2B',*point,cell,pair,self.triangle)
def cases():
 for camera in [[0,0],[66,48],[127,127],[255,255]]:
  for x in [0,1,31,32,127,128,319,320,638,639]:
   for y in [0,4,5,16,239,240,478,479]:yield camera,[x,y]
 r=random.Random(0x7b5507ae20)
 for _ in range(1024):yield [r.randrange(256),r.randrange(256)],[r.randrange(640),r.randrange(480)]
def main():
 WORK.mkdir(parents=True,exist_ok=True);source=P/'rust/crates/genewars-runtime/src/terrain_target.rs';sourcepin=pin(source);listings=[P/'private/ghidra/terrain-render-refs'/s for s in ['instructions-0007b550.txt','instructions-0007ae20.txt']]
 mapdata=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes()[0x355e:0x355e+229376];(WORK/'map.bin').write_bytes(mapdata);(WORK/'target-input.bin').write_bytes(b''.join(struct.pack('<4H',*c,*s)for c,s in cases()))
 rustc=P/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe'
 for cmd in [[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)]]:
  r=subprocess.run(cmd,env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 randommap=bytearray(229376);rng=random.Random(0x7ae207b550)
 for at in range(16384):struct.pack_into('<h',randommap,at*14+2,rng.randrange(-32768,32768))
 counts=[0,0];digest=hashlib.sha256();maps=[]
 for name,data in [('exactCONQ001',mapdata),('flatZero',bytes(229376)),('syntheticFullI16',bytes(randommap))]:
  (WORK/'map.bin').write_bytes(data);r=subprocess.run([str(exe),str(WORK)],env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
  model=Model(listings,data);actual=(WORK/'target-output.bin').read_bytes();local=[0,0]
  for i,(c,s)in enumerate(cases()):
   expected=model.run(c,s);native=actual[i*9:i*9+9];assert native==expected,(name,i,c,s,native.hex(),expected.hex());counts[expected[0]]+=1;local[expected[0]]+=1;digest.update(native)
  assert len(actual)==sum(local)*9;maps.append(dict(name=name,map_sha256=hashlib.sha256(data).hexdigest(),cases=sum(local),resolved=local[1],unresolved=local[0]))
 assert sourcepin==pin(source)
 report=dict(status='passed-independent-native-terrain-target-review',utc=datetime.now(timezone.utc).isoformat(),cases=sum(counts),resolved=counts[1],unresolved=counts[0],maps=maps,source=sourcepin,instruction_pins=[pin(p)for p in listings],output_sha256=digest.hexdigest(),harness=pin(exe),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],original_execution=False,gui_execution=False,limits=['Exact admitted32/16/factor2 and640x480 domain','Unwritten-helper and40-step fallback return explicit error instead of caller prior bytes','No UI/controller command dispatch or complete original startup'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k]for k in ['status','cases','resolved','unresolved']}))
if __name__=='__main__':main()

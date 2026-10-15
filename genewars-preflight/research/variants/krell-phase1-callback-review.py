"""Literal code49 phase1 callback on captured Krell world; no native execution."""
from pathlib import Path
import importlib.util,struct,sys,json,io,contextlib,hashlib
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('k',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
Old=m.Source
requests=[]
class Source(Old):
 def __init__(self,b):
  super().__init__(b)
  extra=[P/'private/ghidra'/name for name in ['plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt','actor-activation/instructions-0004b090.txt','landing-contact/instructions-0001e600.txt','terrain-map-refs/instructions-00049d30.txt','terrain-map-refs/instructions-0007b240.txt','mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt']]
  self.files+=extra;self.code.update(m.m.m.Model(extra,b'').code)
  self.code[0x98210]=('RET',[])
  self.code[0x5ed20]=('JMP',['0x0004b630']);order=sorted(self.code);self.next=dict(zip(order,order[1:]))
 def load(self,a,n):
  if getattr(self,'pc',None)==0x98210 and not requests:requests.append({'service':'98210','entity':self.r['EAX'],'sound':self.r['EDX'],'parameter':self.r['EBX']})
  return super().load(a,n)
 def run(self,*args,**kw):
  state=kw['state_before'];eff=bytearray((P/'private/integration/game-runtime/effects/EFF.ANB').read_bytes());frames,anims=struct.unpack_from('<II',eff,10)
  for i in range(struct.unpack_from('<H',eff)[0]):
   for off in [0,4]:struct.pack_into('<I',eff,anims+10*i+off,0x610000+frames+struct.unpack_from('<I',eff,anims+10*i+off)[0])
  state.update({0x610000+i:v for i,v in enumerate(eff)})
  def put(a,v):state.update({a+i:x for i,x in enumerate(struct.pack('<I',v))})
  put(0x116818,0x610000+anims)
  for i in range(200):put(0x1688ae+i*81+31,0x610000+anims+state[0x1688ae+i*81+69]*10)
  kw['registers']={'EAX':0x1688ae};return super().run(*args,**kw)
m.Source=Source
if __name__=='__main__':
 capture=io.StringIO()
 with contextlib.redirect_stdout(capture):m.main()
 report=json.loads(capture.getvalue());path=Path(report['output'])/'result.json';full=json.loads(path.read_text())
 full['scope']='Full code49 phase1 callback4B630 at sourceframe294, slot0 only; other effects not scheduled'
 full['source_phase_addresses']=['4B630','6DF30','4D5BB'];full['ports']=['98210 audio parameter request deferred; numeric world unaffected']
 full['audio_requests']=requests
 full['driver_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest();path.write_text(json.dumps(full,indent=2)+'\n')
 print(json.dumps({k:v for k,v in full.items()if k not in ['source_pins','input_pins']},indent=2))



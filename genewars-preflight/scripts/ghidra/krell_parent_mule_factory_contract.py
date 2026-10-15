"""Actual785 caller replay to complete parent-factory return; no native run."""
from pathlib import Path
import importlib.util,json,struct,sys,uuid,pickle,hashlib
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra'
spec=importlib.util.spec_from_file_location('review',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
files=[G/'activation-rng-refs/instructions-0005a210.txt',G/'genepod-mule-initializer.txt',G/'construction-command/instructions-0001ee50.txt',G/'bones-frame/instructions-0005a1c0.txt',G/'player-creation/instructions-0001f480.txt',G/'terrain-map-refs/instructions-0007b240.txt']
out=P/'private/review/krell-parent-mule-factory'/uuid.uuid4().hex;out.mkdir(parents=True)
class Complete(BaseException):pass
class Hook(dict):
 def __getitem__(self,k):
  x=self.owner
  if k==0x62201:
   # Bind the already-loaded exact Mule animation resource as startup does.
   # This is a typed resource address, not a fabricated factory result.
   x.store(0x180e84+16,4,x.load(0x1542be+31,4))
   self.request={'owner':x.r['EAX'],'species':x.r['EDX'],'position':x.r['EBX'],'quality':x.r['ECX'],'parent':x.load(x.r['ESP'],4)}
   assert self.request=={'owner':4,'species':4,'position':1418492787,'quality':50,'parent':1393590}
   self.before_rng=[x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)]
   self.before_count=x.load(0x11c2b0,2)
   (out/'before-memory.pickle').write_bytes(pickle.dumps(x.mem))
  if k==0x5a7d8:
   ptr=x.r['EBP'];b=bytes(x.load(ptr+i,1)for i in range(124));rng=[x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)]
   assert b[24]==4 and b[0x7a]==4 and b[0x76]==50 and b[0x5e]==8 and b[0x5f]==0
   assert rng[2]-self.before_rng[2]==3 and x.load(0x11c2b0,2)==self.before_count+1
   (out/'after-memory.pickle').write_bytes(pickle.dumps(x.mem));(out/'newborn.bin').write_bytes(b)
   (out/'creatures.bin').write_bytes(bytes(x.load(0x1542be+i,1)for i in range(37200)))
   (out/'terrain.bin').write_bytes(bytes(x.load(0x11c2be+i,1)for i in range(229376)))
   (out/'rng.bin').write_bytes(struct.pack('<HBI',*rng))
   report={'status':'source_prepared','production_comparison':'not_run','scope':'Actual frame785 slot2 pre-factory caller and literal5A210 parent2 return boundary; post-factory caller not executed.','request':self.request,'pointer':ptr,'slot':struct.unpack_from('<h',b,15)[0],'rng_before':self.before_rng,'rng_after':rng,'count_before':self.before_count,'count_after':x.load(0x11c2b0,2),'record_sha256':hashlib.sha256(b).hexdigest(),'fields':{hex(o):struct.unpack_from('<h',b,o)[0]for o in [0x62,0x64,0x66,0x68,0x6a,0x6c]},'bytes':{hex(o):b[o]for o in [0x76,0x77,0x78,0x79,0x7b]},'output':out.relative_to(P).as_posix(),'source_pins':{p.relative_to(P).as_posix():m.sha(p)for p in x.files},'resource_port':'180E94 is exact loaded Mule animation pointer from active pool binding; no factory callbacks stubbed','original_execution':False}
   (P/'research/campaign/krell-parent-mule-factory.json').write_text(json.dumps(report,indent=2)+'\n');(out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k!='source_pins'}));raise Complete()
  return super().__getitem__(k)
Original=m.Source
class Source(Original):
 def __init__(self,b):
  super().__init__(b);self.files+=files;self.code.update(m.m.m.Model(files,b'').code);order=sorted(self.code);self.next=dict(zip(order,order[1:]));self.code=Hook(self.code);self.code.owner=self
m.Source=Source
sys.argv=['review','--before',str(P/'private/review/krell-mule-callbacks-039/frame-785-slot-2-before'),'--frame','785','--slot','2']
try:m.main()
except Complete:pass

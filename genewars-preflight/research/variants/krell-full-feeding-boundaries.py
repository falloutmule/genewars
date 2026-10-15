from pathlib import Path
import importlib.util,sys,json,struct,uuid
P=Path(__file__).resolve().parents[2];s=importlib.util.spec_from_file_location('b',P/'research/variants/krell-pair-birth-callback-review.py');b=importlib.util.module_from_spec(s);s.loader.exec_module(b);m=b.m
out=P/'private/review/krell-newborn-choice'/uuid.uuid4().hex;out.mkdir(parents=True);events=[]
class H(dict):
 def __getitem__(self,k):
  if k in [0x5b100,0x73c20,0x624b8]:
   x=self.owner;key=f'{k:x}-{len(events)}';q=out/key;q.mkdir();(q/'record.bin').write_bytes(bytes(x.load(0x1542be+248+i,1)for i in range(124)));(q/'registers.json').write_text(json.dumps(x.r,indent=2));events.append({'pc':hex(k),'return':hex(x.load(x.r['ESP'],4)),'rng':[x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)],'boundary':key});(out/'events.json').write_text(json.dumps(events,indent=2))
  return super().__getitem__(k)
class C(b.WithFactory):
 def __init__(self,x):
  super().__init__(x);self.code=H(self.code);self.code.owner=self
m.Source=C;sys.argv=['x','--before',str(P/'private/review/krell-mule-callbacks-052/frame-1005-slot-2-blocked-before'),'--frame','1005','--slot','2'];m.main();print(out);print(events)


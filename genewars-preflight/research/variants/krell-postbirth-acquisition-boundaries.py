from pathlib import Path
import importlib.util,sys,json,struct,uuid
P=Path(__file__).resolve().parents[2];s=importlib.util.spec_from_file_location('b',P/'research/variants/krell-pair-birth-callback-review.py');b=importlib.util.module_from_spec(s);s.loader.exec_module(b);m=b.m
out=P/'private/review/krell-postbirth-acquisition'/uuid.uuid4().hex;out.mkdir(parents=True)
class H(dict):
 def __getitem__(self,k):
  if k in [0x624b8,0x5b580,0x5cc60,0x5b6f9]:
   x=self.owner;q=out/f'{k:x}';q.mkdir(exist_ok=True);(q/'record.bin').write_bytes(bytes(x.load(0x1542be+248+i,1)for i in range(124)));(q/'rng.bin').write_bytes(struct.pack('<HBI',x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)));(q/'registers.json').write_text(json.dumps(x.r,indent=2))
  return super().__getitem__(k)
class C(b.WithFactory):
 def __init__(self,x):
  super().__init__(x);self.code=H(self.code);self.code.owner=self
m.Source=C;sys.argv=['x','--before',str(P/'private/review/krell-mule-callbacks-042/frame-787-slot-2-blocked-before'),'--frame','787','--slot','2'];m.main();print(out)

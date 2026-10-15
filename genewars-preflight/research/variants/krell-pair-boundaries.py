"""Source-only actual651 pair boundaries; no production acceptance."""
from pathlib import Path
import importlib.util,sys,json,struct,uuid
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('review',Path(__file__).with_name('krell-mule-callback-review.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
Original=m.Source
out=P/'private/review/krell-pair-boundaries'/uuid.uuid4().hex;out.mkdir(parents=True)
class Hook(dict):
 def __getitem__(self,k):
  if k in (0x693ae,0x694d3,0x6bc40):
   x=self.owner
   if k==0x6bc40:
    self.events.append(dict(pc=hex(k),candidate=x.load(x.r['ECX']+15,2),distance_weight=x.r['EDI'],score=x.r['EAX'],prior=x.r['EDX']))
   else:
    q=out/f'{k:x}';q.mkdir(exist_ok=True)
    for slot in [0,2]:(q/f'creature-{slot}.bin').write_bytes(bytes(x.load(0x1542be+124*slot+i,1)for i in range(124)))
    (q/'plants.bin').write_bytes(bytes(x.load(0x15d40e+i,1)for i in range(512*49)))
    (q/'rng.bin').write_bytes(struct.pack('<HBI',x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)))
    (q/'registers.json').write_text(json.dumps(x.r,indent=2))
    (out/'candidates.json').write_text(json.dumps(self.events,indent=2))
  return super().__getitem__(k)
class Capture(Original):
 def __init__(self,b):
  super().__init__(b);self.code=Hook(self.code);self.code.owner=self;self.code.events=[]
m.Source=Capture
sys.argv=['review','--before',str(P/'private/review/krell-mule-callbacks-018/frame-651-slot-2-before'),'--frame','651','--slot','2'];m.main();print('BOUNDARIES',out)

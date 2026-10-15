"""Source-only actual785 post-factory boundary records; no native claim."""
from pathlib import Path
import importlib.util,sys,json,struct,uuid
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('birth',Path(__file__).with_name('krell-pair-birth-callback-review.py'));b=importlib.util.module_from_spec(s);s.loader.exec_module(b);m=b.m
out=P/'private/review/krell-pair-birth-tail'/uuid.uuid4().hex;out.mkdir(parents=True)
class Hook(dict):
 def __getitem__(self,k):
  if k in (0x62206,0x62270,0x62398,0x624ac,0x62304,0x6242d,0x5ef46):
   x=self.owner;q=out/f'{k:x}';q.mkdir(exist_ok=True)
   for slot in [0,2,4]:(q/f'creature-{slot}.bin').write_bytes(bytes(x.load(0x1542be+124*slot+i,1)for i in range(124)))
   (q/'rng.bin').write_bytes(struct.pack('<HBI',x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)))
   (q/'registers.json').write_text(json.dumps(x.r,indent=2))
  return super().__getitem__(k)
class Capture(b.WithFactory):
 def __init__(self,x):
  super().__init__(x);self.code=Hook(self.code);self.code.owner=self
m.Source=Capture
sys.argv=['review','--before',str(P/'private/review/krell-mule-callbacks-039/frame-785-slot-2-before'),'--frame','785','--slot','2'];m.main();print('BOUNDARIES',out)


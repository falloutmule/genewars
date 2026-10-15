"""Literal actual785 pre-birth boundary; factory deliberately not executed."""
from pathlib import Path
import importlib.util,sys,json,struct,uuid
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('review',Path(__file__).with_name('krell-mule-callback-review.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
out=P/'private/review/krell-pair-birth-boundaries'/uuid.uuid4().hex;out.mkdir(parents=True)
Original=m.Source
class Hook(dict):
 def __getitem__(self,k):
  if k in (0x61c8a,0x61df4,0x62125,0x62201):
   x=self.owner;q=out/f'{k:x}';q.mkdir(exist_ok=True)
   for slot in [0,2]:(q/f'creature-{slot}.bin').write_bytes(bytes(x.load(0x1542be+124*slot+i,1)for i in range(124)))
   (q/'rng.bin').write_bytes(struct.pack('<HBI',x.load(0x11878a,2),x.load(0x11872c,1),x.load(0x118782,4)))
   (q/'registers.json').write_text(json.dumps(x.r,indent=2))
   if k==0x62201:
    report={'status':'blocked','boundary':'5A210 factory not executed','frame':785,'slot':2,'production_comparison':'not_run','owner':x.r['EAX'],'species':x.r['EDX'],'position':x.r['EBX'],'quality':x.r['ECX'],'parent':x.load(x.r['ESP'],4),'inherit_flag':x.load(x.r['EBP']+0x156,1),'source_pins':{str(p.relative_to(P)):m.sha(p)for p in x.files},'output':str(out)}
    (out/'result.json').write_text(json.dumps(report,indent=2));print(json.dumps(report,indent=2));raise RuntimeError('Explicit factory boundary: complete pre-factory packet saved')
  return super().__getitem__(k)
class Capture(Original):
 def __init__(self,b):
  super().__init__(b);self.code=Hook(self.code);self.code.owner=self
m.Source=Capture
sys.argv=['review','--before',str(P/'private/review/krell-mule-callbacks-039/frame-785-slot-2-before'),'--frame','785','--slot','2'];m.main();print('BOUNDARIES',out)

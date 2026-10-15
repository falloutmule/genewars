"""Read-only interpreter capture of actual frame8 post-physics cell entry."""
from pathlib import Path
import importlib.util,sys,struct,json,hashlib
P=Path(__file__).resolve().parents[2];sp=importlib.util.spec_from_file_location('review',P/'research/variants/krell-mule-callback-review.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
frame=int(sys.argv[1]) if len(sys.argv)>1 else 8
fixture='krell-mule-callbacks-010' if frame==498 else 'krell-mule-callbacks-004'
base=m.Source
class Captured(base):
 def get(self,operand):
  if self.pc==0x5b271:
   out=P/('private/review/mule-cell-entry-actual' if frame==8 else 'private/review/mule-cell-entry-actual-498');out.mkdir(exist_ok=True)
   (out/'record.bin').write_bytes(bytes(self.load(self.r['EBP']+i,1)for i in range(124)))
   (out/'context.bin').write_bytes(struct.pack('<HHBI',(self.load(self.r['ESP']+4,4)-0x11c2be)//14,self.load(0x11878a,2),self.load(0x11872c,1),self.load(0x118782,4)))
  return super().get(operand)
m.Source=Captured;sys.argv=['review','--before',str(P/f'private/review/{fixture}/frame-{frame}-slot-1-before'),'--slot','1','--frame',str(frame)];m.main()

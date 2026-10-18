import importlib.util,sys
from pathlib import Path
p=Path('research/variants/krell-mule-callback-review.py').resolve();s=importlib.util.spec_from_file_location('k',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);Old=m.Source
class Source(Old):
 def __init__(self,b):
  super().__init__(b);self.code[0x5ed20]=('JMP',['0x0006df30'])
 def run(self,*args,**kw):
  kw['registers']={'EAX':0x1688ae+81*int(slot)};
  try:return super().run(*args,**kw)
  finally:
   base=0x1688ae+81*int(slot);print('NUMERIC',slot,[(hex(o),self.load(base+o,n))for o,n in [(7,2),(9,2),(17,4),(39,4),(45,2),(47,2),(49,2),(57,2),(59,2),(61,2)]])
m.Source=Source
slot=sys.argv[1];sys.argv=['probe','--before','private/review/krell-pre-effects-294-001','--frame','294','--slot',slot]
m.main()


"""Literal seed65535 callback5859, with48790 particle creation a typed port.

This records factory arguments; it does not allocate or update an effect.
"""
from pathlib import Path
import importlib.util,sys,json,hashlib
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('mule',Path(__file__).with_name('creature-hungry-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
START=P/'private/review/family8-trajectory/07e0a0d3f4944407b813480131408781/frame5859/boundary'
Original=m.CapturedSource
instances=[]
class WithFactoryPort(Original):
 def __init__(self):
  super().__init__();self.code[0x48790]=('RET',['0x10']);order=sorted(self.code);self.next=dict(zip(order,order[1:]));self.requests=[];instances.append(self)
 def load(self,a,n):
  if getattr(self,'pc',None)==0x48790 and not self.requests:
   self.requests.append(dict(code=self.r['EAX'],position=[m.m.signed(self.r['EDX']&65535,16),m.m.signed(self.r['EDX']>>16,16)],physical_height=m.m.signed(self.r['EBX']),angles=[super(WithFactoryPort,self).load(self.r['ECX']+2*i,2)for i in range(3)],stack_arguments=[super(WithFactoryPort,self).load(self.r['ESP']+4+4*i,4)for i in range(4)]))
  return super().load(a,n)

def main():
 m.CapturedSource=WithFactoryPort
 sys.argv=['source','5859','--snapshot='+str(START.relative_to(P))];m.main()
 requests=instances[0].requests;assert len(requests)==1 and all(x.requests==requests for x in instances)
 trace=json.loads((START/'source-trace5859.json').read_text())
 after=(START/'source-after5859.bin').read_bytes()
 report=dict(status='literal-deep-water-request',frame=5859,seed=65535,requests=requests,literal_instructions=trace['report']['steps'],
  after_sha256=hashlib.sha256(after).hexdigest(),after_physical_height=int.from_bytes(after[0x27:0x2b],'little',signed=True),after_ground_height=int.from_bytes(after[0x11:0x15],'little',signed=True),
  inputs=trace['snapshot_inputs'],listing_pins=trace['listing_pins'],script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
  original_execution=False,scope='Full Mule record/map numeric callback with effect48790 allocation deferred. Factory/lifetime and owned-world commits require separate proof.')
 out=P/'research/variants/mule-deep-water-request.json';out.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','requests','after_physical_height','after_ground_height','literal_instructions']}))
if __name__=='__main__':main()

"""Literal793C0 with real marker/particle factories over all eight world buffers.
The selected synthetic fixture must contain exactly one eligible Mule in radius.
No activation, exhaustion or campaign acceptance is established here.
"""
from pathlib import Path
import argparse,importlib.util,json,struct,uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('markers',Path(__file__).with_name('zone-marker-captured-review.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
g=m.g
class Source(m.Source):
 def __init__(self,backing):
  super().__init__(backing)
  extra=[P/'private/ghidra'/n for n in ['mission-array-refs/instructions-000793c0.txt','mission-zone-792c0.txt','mission-zone-score.txt','mission-array-refs/instructions-0007a280.txt']]
  self.files+=extra;self.code.update(g.k.m.m.Model(extra,b'').code)
  # Ghidra omits memory width for absolute MOV stores: infer it from the
  # narrow source register, otherwise the inherited interpreter stores4 bytes
  # for MOV[11878A],AX and incorrectly erases participating-player count11878C.
  self.width_fixes=[]
  for pc,(op,operands)in list(self.code.items()):
   if op=='MOV'and '['in operands[0]and not operands[0].startswith(('byte ptr ','word ptr ','dword ptr '))and self.width(operands[1])in (8,16):
    prefix='byte ptr 'if self.width(operands[1])==8 else 'word ptr '
    self.code[pc]=(op,[prefix+operands[0],*operands[1:]]);self.width_fixes.append(hex(pc))
  order=sorted(self.code);self.next=dict(zip(order,order[1:]))
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--kind',choices=('credit','schedule'),default='credit');ap.add_argument('--before',type=Path,required=True);ap.add_argument('--expected',type=Path);ap.add_argument('--frame',type=int,required=True);args=ap.parse_args()
 before=g.resolve(args.before);expected=g.resolve(args.expected) if args.expected else None
 out=P/'private/review/zone-credit-captured'/uuid.uuid4().hex;out.mkdir(parents=True,exist_ok=False)
 report={'status':'error','scope':'single eligible Mule literal793C0 with real ordered factories; other families frozen; no campaign acceptance','output':str(out),'frame':args.frame,'kind':args.kind}
 try:
  if args.kind=='schedule':report['scope']='bounded literal7A280 full zone pass; real marker/particle factories; no exhaustion or campaign acceptance'
  buffers=g.read_buffers(before);back=buffers['backing.bin'];zone=back[0x2ff8:0x300b]
  if zone[4]!=12 or zone[18]<=0:raise ValueError('requires active typed zone fixture')
  radius=struct.unpack_from('<i',zone,6)[0];center=struct.unpack_from('<hh',zone);eligible=[]
  for slot in range(300):
   raw=buffers['creatures.bin'][slot*124:(slot+1)*124]
   if not raw[6] or raw[94]>=24 or raw[94]==15:continue
   xy=struct.unpack_from('<hh',raw,7)
   delta=[((2*(xy[i]-center[i])+32768)%65536-32768)>>1 for i in range(2)]
   if sum(d*d for d in delta)<=radius*radius:eligible.append(slot)
  if args.kind=='credit'and len(eligible)!=1:raise ValueError(('single-Mule adapter comparison requires one eligible slot',eligible))
  state,seed,resources=g.prepare(buffers);g.k.put(state,0x11878c,1,1);g.k.put(state,0x11873d,1,0)
  vm=Source(back);executed=[];credit_trace=[]
  class Trace(dict):
   def __getitem__(self,pc):
    executed.append(pc)
    if pc in [0x807d2,0x6d500,0x807ff]:credit_trace.append({'pc':hex(pc),'EAX':vm.r['EAX'],'EDX':vm.r['EDX'],'EBP':hex(vm.r['EBP']),'stack_credit':vm.load(vm.r['EBP']-4,4),'score':vm.load(0x118d81,4),'stack_args':[vm.load(vm.r['ESP']+4+i*4,4)for i in range(3)]})
    return super().__getitem__(pc)
  vm.code=Trace(vm.code)
  run,_,_=vm.run(entry=0x793c0 if args.kind=='credit'else 0x7a280,counter=args.frame,seed=seed,state_before=state,registers={'EAX':0x11bd58,'EDX':4},instruction_budget=250000)
  result=g.export(vm,buffers)
  for name,data in result.items():(out/name).write_bytes(data)
  compared=sum(map(len,result.values()));assert compared==427369
  differences=[]
  if expected:
   reference=g.read_buffers(expected)
   for name in g.NAMES:
    offsets=[i for i,(a,b)in enumerate(zip(result[name],reference[name]))if a!=b]
    if len(result[name])!=len(reference[name]) or offsets:differences.append({'buffer':name,'count':len(offsets),'first_offsets':offsets[:12]})
  journals=[x['journal'] for x in vm.journals]
  report.update(status=('passed' if not differences else 'error') if expected else 'source_prepared',eligible=eligible,compared_bytes=compared if expected else None,differences=differences,draws=run['draws'],journals=journals,credit_trace=credit_trace,instruction_width_fixes=vm.width_fixes,calls=run['calls'],instructions=run['steps'],input_pins={n:g.pin(before/n)for n in g.NAMES},source_pins={str(p.relative_to(P)):g.pin(p)for p in vm.files},resource_pins=resources,normalization='g.export: opaque31:39 retained from INPUT only, stale serialized player mirror retained')
 except Exception as error:report.update(reason=repr(error))
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
 return 0 if report['status']in ('passed','source_prepared')else 1
if __name__=='__main__':raise SystemExit(main())

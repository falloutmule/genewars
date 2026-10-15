"""Bounded literal owned-Mule5ED20 full-world comparison with narrow MOV widths.
Uses captured environmental globals, actual single-player prior1, and input-only
native-pointer normalization. Does not execute the original game or establish
native/campaign acceptance. Historical interpreters and receipts are unchanged.
"""
from pathlib import Path
import argparse,importlib.util,json,struct,uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('owned_width_source',Path(__file__).with_name('zone-credit-captured-review.py'))
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c);g=c.g
class Source(c.Source):
 def __init__(self,backing):
  super().__init__(backing)
  # Exact trivial virtual10 getters reached when a zone marker shares the
  # target-query cell. Keep real predicate returns, never skip the candidate.
  extra=[P/'private/ghidra/countdown-expiry/instructions-0001eac0.txt']
  self.files+=extra;self.code.update(g.k.m.m.Model(extra,b'').code)
  order=sorted(self.code);self.next=dict(zip(order,order[1:]))
PORTS=['83140 render enqueue returns0; numeric world retained','41910 audio engine deferred via RET10; source1E810 wrapper remains interpreted','2C200 UI cleanup deferred','5E068 portrait/module refresh deferred to5E0A3']
def main():
 ap=argparse.ArgumentParser(description=__doc__)
 ap.add_argument('--before',type=Path,required=True);ap.add_argument('--expected',type=Path)
 ap.add_argument('--frame',type=int,required=True);ap.add_argument('--slot',type=int,required=True)
 args=ap.parse_args();before=g.resolve(args.before);expected=g.resolve(args.expected)if args.expected else None
 out=P/'private/review/owned-mule-captured-width'/uuid.uuid4().hex;out.mkdir(parents=True,exist_ok=False)
 report={'status':'error','output':str(out.relative_to(P)),'before':str(before),'expected':str(expected)if expected else None,'frame':args.frame,'slot':args.slot,'scope':'one ownedMule callback5ED20; other families frozen; no native or campaign acceptance','ports':PORTS,'normalization':'g.export retains opaque31:39 from INPUT only and stale serialized player mirror; no expected-derived normalization'}
 try:
  buffers=g.read_buffers(before)
  if not 0<=args.slot<300:raise ValueError('invalid creature slot')
  if not 0<=args.frame<=0xffffffff:raise ValueError('invalid source frame')
  record=buffers['creatures.bin'][args.slot*124:(args.slot+1)*124]
  if record[6]!=1 or record[24]!=0 or record[122]!=4 or record[94]not in(0,1,8)or struct.unpack_from('<h',record,15)[0]!=args.slot:
   raise ValueError('requires actual singleplayer owner0 live typed Mule state0/8 or exact hostile state1 with matching identity')
  if record[94]==1:
   target=struct.unpack_from('<h',record,0x55)[0]
   if not 0<=target<300 or target==args.slot:raise ValueError('invalid owned hostile target identity')
   enemy=buffers['creatures.bin'][target*124:(target+1)*124]
   if record[0x60] not in(0x48,0xc8)or record[0x61]!=0 or record[0x78]!=7 or record[0x4f]!=0 or record[0x5d]!=0 or record[0x52]!=1 or record[0x57]!=1 or record[0x5a]!=1 or record[0x50:0x53]!=record[0x55:0x58]or record[0x58:0x5b]!=record[0x55:0x58]:
    raise ValueError('unsupported owned hostile pursuit caller')
   if enemy[6]!=1 or enemy[24]!=4 or enemy[122]!=4 or enemy[94]!=1 or struct.unpack_from('<h',enemy,15)[0]!=target or struct.unpack_from('<H',enemy,0x49)[0]&0x400:
    raise ValueError('requires exact live neutralMule state1 hostile target')
  state,seed,resources=g.prepare(buffers);g.k.put(state,0x11878c,1,1)
  vm=Source(buffers['backing.bin'])
  run,_,trace=vm.run(entry=0x5ed20,counter=args.frame,seed=seed,state_before=state,registers={'EAX':0x1542be+args.slot*124},instruction_budget=500000)
  result=g.export(vm,buffers)
  for name,data in result.items():(out/name).write_bytes(data)
  (out/'trace.txt').write_text('\n'.join(hex(pc)for pc in trace)+'\n',encoding='utf-8')
  differences=[];counts={name:0 for name in g.NAMES}
  if expected:
   reference=g.read_buffers(expected)
   for name in g.NAMES:
    offsets=[i for i,(a,b)in enumerate(zip(result[name],reference[name]))if a!=b]
    counts[name]=len(offsets)
    if offsets:differences.append({'buffer':name,'count':len(offsets),'first_offsets':offsets[:12]})
  participating_after=vm.load(0x11878c,1)
  if participating_after!=1:raise ValueError(('participating context changed',participating_after))
  executed=set(trace)
  report.update(status=('passed'if not differences else'mismatch')if expected else'source_prepared',compared_bytes=sum(map(len,result.values()))if expected else None,mismatch_counts=counts if expected else None,differences=differences,draws=run['draws'],instructions=run['steps'],participating_before=1,participating_after=participating_after,executed_width_fixes=[pc for pc in vm.width_fixes if int(pc,16)in executed],calls=run['calls'],input_pins={n:g.pin(before/n)for n in g.NAMES},expected_pins={n:g.pin(expected/n)for n in g.NAMES}if expected else{},resource_pins=resources,source_pins={str(p.relative_to(P)):g.pin(p)for p in vm.files},harness_pins={str(p.relative_to(P)):g.pin(p)for p in [Path(__file__),Path(c.__file__),Path(g.__file__),Path(g.k.__file__),P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']})
 except Exception as error:report['reason']=repr(error)
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
 summary={k:v for k,v in report.items()if not k.endswith('_pins')}
 text=json.dumps(summary,indent=2)+'\n';(out/'summary.log').write_text(text,encoding='utf-8');print(text,end='')
 return 0 if report['status']in('passed','source_prepared')else 1
if __name__=='__main__':raise SystemExit(main())

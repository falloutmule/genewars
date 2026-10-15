"""Bounded literal118E0 selector1 captured callback; no original executable.
All eight buffers compared; opaque pointers retained from INPUT only.
Audio query97830 is explicitly false; moving audio1E810 is deferred.
No child allocation is supported. Real48790 particle factory is interpreted.
"""
from pathlib import Path
import argparse,importlib.util,json,struct,uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('credit_source',Path(__file__).with_name('zone-credit-captured-review.py'))
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c);g=c.g
ANB=P/'private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin'
class Source(c.Source):
 def __init__(self,backing):
  super().__init__(backing)
  extra=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['118e0','11943','1eab0','1eac0','1ea90','11270','11380','11aeb','12e11','13108','11400','73a50','11720','114b0','13e60','1ea80']]
  extra += [P/'private/ghidra/construction-command/instructions-0001ee40.txt',P/'private/ghidra/landing-contact/instructions-0001f030.txt',P/'private/ghidra/actor-activation/instructions-00073e00.txt',P/'private/ghidra/initial-creature/instructions-0006d230.txt',P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt']
  self.files+=extra;self.code.update(g.k.m.m.Model(extra,b'').code)
  self.code={a:v for a,v in self.code.items()if not 0x11720<=a<=0x1188d}
  self.code.update(g.k.m.m.Model([P/'private/ghidra/countdown-expiry/instructions-00011720.txt'],b'').code)
  for pc,(op,args)in list(self.code.items()):
   if op=='MOV'and '['in args[0]and not args[0].startswith(('byte ptr ','word ptr ','dword ptr '))and self.width(args[1])in(8,16):
    self.code[pc]=(op,[('byte ptr 'if self.width(args[1])==8 else'word ptr ')+args[0],*args[1:]])
  self.code[0x98210]=('RET',[]);self.code[0x1e810]=('RET',['0x8']);self.code[0x97830]=('XOR',['AL','AL']);self.code[0x97831]=('RET',[])
  self.code[0x101d0]=('UNIMPLEMENTED_CHILD_FACTORY',[])
  self.code[0x11a18]=('UNIMPLEMENTED_UI_CHILD_PATH',[])
  order=sorted(self.code);self.next=dict(zip(order,order[1:]));self.next[0x97830]=0x97831

def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--before',type=Path,required=True);ap.add_argument('--expected',type=Path);ap.add_argument('--frame',type=int,required=True);ap.add_argument('--slot',type=int,required=True);a=ap.parse_args()
 before=g.resolve(a.before);expected=g.resolve(a.expected)if a.expected else None
 out=P/'private/review/family8-captured'/uuid.uuid4().hex;out.mkdir(parents=True,exist_ok=False)
 report={'status':'error','output':str(out),'frame':a.frame,'slot':a.slot,'scope':'single selector1 original listing callback118E0, other families frozen; no campaign/native acceptance','ports':['97830 audio-active query returnsfalse','1E810 moving audio/98210 audio parameter update deferred','83140 render enqueue deferred; no numeric mutation','101D0 child factory/11A18 child UI reject']}
 try:
  b=g.read_buffers(before);state,seed,resources=g.prepare(b)
  if not 0<=a.slot<16:raise ValueError('family8 slot')
  raw=b['family8.bin'][a.slot*102:(a.slot+1)*102]
  if raw[6]!=8 or raw[0x55]!=1 or raw[0x18]>=4:raise ValueError('requires selector1 owner<4')
  anb=bytearray(ANB.read_bytes());assert g.pin(ANB)=='318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4'
  count,_,_,frames,anims=struct.unpack_from('<H4I',anb)
  for i in range(count):
   for off in(0,4):struct.pack_into('<I',anb,anims+10*i+off,0x500000+frames+struct.unpack_from('<I',anb,anims+10*i+off)[0])
  state.update({0x500000+i:x for i,x in enumerate(anb)})
  for slot in range(16):
   r=b['family8.bin'][slot*102:(slot+1)*102];g.k.put(state,0x16da36+slot*102+31,4,0x500000+anims+(0 if r[0x55]==1 else 80))
  g.k.put(state,0x116820,4,0x500000);g.k.put(state,0x11878c,1,1)
  vm=Source(b['backing.bin']);run,_,trace=vm.run(entry=0x118e0,counter=a.frame,seed=seed,state_before=state,registers={'EAX':0x16da36+a.slot*102},instruction_budget=500000)
  result=g.export(vm,b)
  for name,data in result.items():(out/name).write_bytes(data)
  (out/'trace.txt').write_text('\n'.join(hex(pc)for pc in trace)+'\n')
  differences=[]
  if expected:
   wanted=g.read_buffers(expected)
   for name in g.NAMES:
    offsets=[i for i,(x,y)in enumerate(zip(result[name],wanted[name]))if x!=y]
    if offsets:differences.append({'buffer':name,'count':len(offsets),'first_offsets':offsets[:12]})
  report.update(status=('passed'if not differences else'mismatch')if expected else'source_prepared',differences=differences,compared_bytes=sum(map(len,result.values()))if expected else None,draws=run['draws'],instructions=run['steps'],participating_after=vm.load(0x11878c,1),calls=run['calls'],particle_factory=0x48790 in trace,target_selection=0x114b0 in trace,audio_query=0x97830 in trace,resource_pins={**resources,str(ANB.relative_to(P)):g.pin(ANB)},input_pins={n:g.pin(before/n)for n in g.NAMES},expected_pins={n:g.pin(expected/n)for n in g.NAMES}if expected else{},source_pins={str(p.relative_to(P)):g.pin(p)for p in vm.files})
 except Exception as e:report['reason']=repr(e)
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if not k.endswith('_pins')},indent=2))
 return 0 if report['status']in('passed','source_prepared')else 1
if __name__=='__main__':raise SystemExit(main())

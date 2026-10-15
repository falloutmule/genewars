"""Bounded literal Krell Mule callback, captured inputs; no game execution/build.

--before captures one callback boundary, --slot identifies its unchanged pool ID.
--expected optionally supplies the corresponding production callback afterimage.
Without --expected, result is source_prepared, never a production pass.
"""
from pathlib import Path
import argparse, importlib.util, json, struct, hashlib, uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('literal',Path(__file__).with_name('initial-creature-source.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
LAYOUTS=[(243038,300,116,0x1542be,124,'creatures.bin',0x111fec),(277838,512,41,0x15d40e,49,None,0x1120ac),(298830,20,140,0x16360e,200,'actors.bin',0x112194),(301630,256,59,0x1645ae,67,'buildings.bin',None),(316734,200,73,0x1688ae,81,'effects.bin',0x112254),(331334,32,41,0x16c7f6,49,None,None),(332646,32,89,0x16ce16,97,None,None),(335494,16,94,0x16da36,102,'family8.bin',0x11242c)]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def put(state,a,n,v):state.update({a+i:(v>>(8*i))&255 for i in range(n)})
class Source(m.Source):
 def __init__(self,backing):
  super().__init__();self.context={0x11a9bd:(2,int.from_bytes(backing[0x1c5d:0x1c5f],'little')),0x11a9c1:(2,int.from_bytes(backing[0x1c61:0x1c63],'little')),0x11a9c7:(4,int.from_bytes(backing[0x1c67:0x1c6b],'little'))}
  extra=[P/'private/ghidra'/directory/('instructions-000'+name+'.txt') for directory,names in [('initial-creature',['5d360','6c900','5c730','6d230','5df20']),('terrain-map-refs',['5b580','74a50','6c780','305f0']),('activation-rng-refs',['5c920']),('player-creation',['1e810']),('landing-contact',['417e0']),('entity-plant',['7af00']),('display-geometry',['979e8']),('plant-startup',['409f0'])] for name in names]
  extra += [P/'private/ghidra/krell-cell-entry/instructions-00073f70.txt']
  self.files+=extra
  parse_files=[P/'private/review/creature-motion/listing-6c900-closed.txt' if p.name=='instructions-0006c900.txt' else p for p in extra]
  self.code.update(m.m.Model(parse_files,b'').code)
  self.code[0x2c200]=('RET',[]);self.code[0x5e068]=('JMP',['0x0005e0a3']);order=sorted(self.code);self.next=dict(zip(order,order[1:]))
 def store(self,a,n,v):
  # Base harness initializes CONQ001 globals before executing; substitute the
  # captured Krell environmental globals at that initialization boundary only.
  if getattr(self,'pc',None)==0x5ed20 and a in self.context:n,v=self.context[a]
  super().store(a,n,v)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--before',type=Path,default=P/'private/review/krell-first-tick-001');ap.add_argument('--expected',type=Path);ap.add_argument('--frame',type=int,default=0);ap.add_argument('--slot',type=int,default=0);a=ap.parse_args()
 out=P/'private/review/krell-mule-callback'/uuid.uuid4().hex;out.mkdir(parents=True)
 report={'status':'error','frame':a.frame,'slot':a.slot,'before':str(a.before),'output':str(out),'production_comparison':'not_run','ports':['83140 render enqueue returns0; numeric state retained','2C200 UI cleanup deferred','5E068 portrait/module refresh deferred'],'source_phase_addresses':['6FB9C..6FBAA','6FD3D..6FD47','70348..70352','5EE28..5EE32']}
 try:
  buffers={p.name:p.read_bytes() for p in a.before.glob('*.bin')};b=buffers['backing.bin'];state={0x118d60+i:x for i,x in enumerate(b[:0x355e])};state.update({0x11c2be+i:x for i,x in enumerate(b[0x355e:0x3b55e])})
  anb=(P/'private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB').read_bytes();animations=struct.unpack_from('<I',anb,14)[0]
  for start,count,stride,base,width,name,vt in LAYOUTS:
   for slot in range(count):
    raw=buffers[name][slot*width:(slot+1)*width] if name else b[start+slot*stride:start+(slot+1)*stride];raw=bytearray(raw if name else raw[:31]+bytes(8)+raw[31:])
    if vt:struct.pack_into('<I',raw,35,vt)
    if name=='creatures.bin':struct.pack_into('<I',raw,31,0x400000+animations)
    state.update({base+slot*width+j:v for j,v in enumerate(raw)})
  state.update({0x118d74+i:v for i,v in enumerate(buffers['players.bin'])});seed,ring,draw=struct.unpack('<HBI',buffers['rng.bin']);put(state,0x11872c,1,ring);put(state,0x118782,4,draw);put(state,0x118770,1,b[0x52466]);put(state,0x185ec4,4,0)
  model=Source(b);r,_,trace=model.run(counter=a.frame,seed=seed,state_before=state,registers={'EAX':0x1542be+a.slot*124},instruction_budget=500000)
  result={name:bytearray(data) for name,data in buffers.items()}
  # Native players overlay serialized backing. Preserve its stale serialized
  # mirror unless the literal callback actually changes a prefix byte.
  for i in range(0x355e):
   value=model.load(0x118d60+i,1)
   if value!=state[0x118d60+i]:result['backing.bin'][i]=value
  result['backing.bin'][0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
  for start,count,stride,base,width,name,vt in LAYOUTS:
   for slot in range(count):
    raw=bytearray(model.load(base+slot*width+j,1)for j in range(width))
    if name:
     raw[31:39]=buffers[name][slot*width+31:slot*width+39];result[name][slot*width:(slot+1)*width]=raw
    else:result['backing.bin'][start+slot*stride:start+(slot+1)*stride]=raw[:31]+raw[39:]
  result['players.bin']=bytes(model.load(0x118d74+i,1)for i in range(7220));result['rng.bin']=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  for name,data in result.items():(out/name).write_bytes(data)
  (out/'trace.json').write_text(json.dumps([hex(x)for x in trace]))
  report.update(status='source_prepared',literal_instructions=r['steps'],draws=r['draws'],queue_requests=r['queue_requests'],calls=r['calls'],source_pins={str(p.relative_to(P)):sha(p)for p in model.files},input_pins={str(p):sha(p)for p in a.before.glob('*.bin')})
  if a.expected:
   for name,data in result.items():
    actual=(a.expected/name).read_bytes();assert len(data)==len(actual),(name,'length');diff=next((i for i,(x,y)in enumerate(zip(data,actual))if x!=y),None);assert diff is None,(name,diff,data[diff],actual[diff])
   report.update(status='passed',production_comparison='passed',compared_bytes=sum(map(len,result.values())))
 except Exception as e:report.update(status='error',reason=repr(e))
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k not in ['source_pins','input_pins']},indent=2))
if __name__=='__main__':main()

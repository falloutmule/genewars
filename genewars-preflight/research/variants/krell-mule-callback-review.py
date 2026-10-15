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
  extra += [P/'private/ghidra/krell-cell-entry/instructions-00073f70.txt', P/'private/ghidra/krell-pair-target', P/'private/ghidra/countdown-expiry/instructions-00013e60.txt', P/'private/ghidra/initial-creature/instructions-0003fbe0.txt', P/'private/ghidra/construction-command/instructions-0001ee50.txt']
  extra += [P/'private/ghidra/activation-rng-refs/instructions-0005cdb0.txt']
  extra += [P/'private/ghidra'/name for name in ['countdown-expiry/instructions-0006dd50.txt','countdown-expiry/instructions-00073a50.txt','mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt']]
  extra += [P/'private/ghidra'/name for name in ['terrain-panel-refs/instructions-0005df50.txt', 'mission-array-refs/instructions-00081c60.txt', 'construction-command/instructions-0005b9f0.txt', 'owned-mule-collision-59280.txt', 'terrain-map-refs/instructions-00084b40.txt', 'camera-player-input/instructions-0008b800.txt', 'terrain-map-refs/instructions-000847d0.txt', 'terrain-map-refs/instructions-0007af00.txt', 'entity-plant/instructions-0001f240.txt', 'entity-plant/instructions-0001f280.txt', 'owned-mule-ai-6bfd0.txt', 'mission-array-refs/instructions-00043130.txt']]
  extra += [P/'private/ghidra/panel-activation-refs/instructions-000724c0.txt', P/'private/ghidra/construction-command/instructions-0004080b.txt']
  self.files+=extra
  parse_files=[P/'private/review/creature-motion/listing-6c900-closed.txt' if p.name=='instructions-0006c900.txt' else p for p in extra]
  self.code.update(m.m.Model(parse_files,b'').code)
  self.code[0x41910]=('RET',['0x10']);self.code[0x2c200]=('RET',[]);self.code[0x5e068]=('JMP',['0x0005e0a3']);order=sorted(self.code);self.next=dict(zip(order,order[1:]))
 def store(self,a,n,v):
  # Base harness initializes CONQ001 globals before executing; substitute the
  # captured Krell environmental globals at that initialization boundary only.
  if getattr(self,'pc',None)==0x5ed20 and a in self.context:n,v=self.context[a]
  super().store(a,n,v)
def main():
 ap=argparse.ArgumentParser();ap.add_argument('--before',type=Path,default=P/'private/review/krell-first-tick-001');ap.add_argument('--expected',type=Path);ap.add_argument('--frame',type=int,default=0);ap.add_argument('--slot',type=int,default=0);ap.add_argument('--command-target',nargs=2,type=int);a=ap.parse_args()
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
  # Airborne obstacle queries read building animation component heights.
  # Rebind only opaque native pointers to the actual source ANB tables.
  building_resources={0:'BGENE.ANB',3:'BPOWE.ANB',11:'BMINE.ANB'};resource_pins={};building_tables={}
  for i,(selector,filename)in enumerate(building_resources.items()):
   path=P/'private/integration/game-runtime/buildings'/filename;data=bytearray(path.read_bytes());resource_pins[str(path.relative_to(P))]=sha(path)
   count,table,components,frames,anims=struct.unpack_from('<H4I',data);address=0x600000+i*0x100000
   for animation in range(count):
    for offset in [0,4]:struct.pack_into('<I',data,anims+animation*10+offset,address+frames+struct.unpack_from('<I',data,anims+animation*10+offset)[0])
   state.update({address+j:v for j,v in enumerate(data)});building_tables[selector]=address+anims
  effect_path=P/'private/integration/game-runtime/effects/EFF.ANB';effect_data=bytearray(effect_path.read_bytes());resource_pins[str(effect_path.relative_to(P))]=sha(effect_path)
  count,table,components,frames,anims=struct.unpack_from('<H4I',effect_data);effect_address=0xa00000
  for animation in range(count):
   for offset in [0,4]:struct.pack_into('<I',effect_data,anims+animation*10+offset,effect_address+frames+struct.unpack_from('<I',effect_data,anims+animation*10+offset)[0])
  state.update({effect_address+j:v for j,v in enumerate(effect_data)});put(state,0x116818,4,effect_address+anims)
  for slot in range(256):
   record=buffers['buildings.bin'][slot*67:(slot+1)*67]
   if record[6]!=0:
    assert record[45]in building_tables,('unbound building selector',record[45])
    put(state,0x1645ae+slot*67+31,4,building_tables[record[45]]);put(state,0x1645ae+slot*67+35,4,0x112120)
  state.update({0x118d74+i:v for i,v in enumerate(buffers['players.bin'])});seed,ring,draw=struct.unpack('<HBI',buffers['rng.bin']);put(state,0x11872c,1,ring);put(state,0x118782,4,draw);put(state,0x118770,1,b[0x52466]);put(state,0x185ec4,4,0)
  model=Source(b)
  class CaptureCode(dict):
   def __getitem__(self,key):
    if key==0x6dd50 and not(out/'collision-entry.json').exists():
     for name,register,width in [('entry-mule.bin','EAX',124),('entry-obstacle.bin','EDX',67)]:
      pointer=model.r[register];(out/name).write_bytes(bytes(model.load(pointer+i,1)for i in range(width)))
     (out/'collision-entry.json').write_text(json.dumps({'source_function':'6DD50','frame':a.frame,'slot':a.slot,'mule':'entry-mule.bin','obstacle':'entry-obstacle.bin'})+'\n')
    return super().__getitem__(key)
  model.code=CaptureCode(model.code)
  entry=0x5ed20;registers={'EAX':0x1542be+a.slot*124}
  if a.command_target is not None:
   assert all(-32768<=v<=32767 for v in a.command_target)
   for offset,value in [(0,a.slot),(2,a.command_target[0]),(4,a.command_target[1]),(6,0)]:put(state,0x440000+offset,2,value)
   put(state,0x118740,4,0);entry=0x81c60;registers={'EAX':0x118d74,'EDX':0x440000}
   report.update(source_phase_addresses=['81C60..81D27','5DF50','5CC60','5C920'],command_target=a.command_target)
   report['ports'].append('41910 feedback audio deferred; numeric state retained')
  r,_,trace=model.run(entry=entry,counter=a.frame,seed=seed,state_before=state,registers=registers,instruction_budget=500000)
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
  report.update(status='source_prepared',resource_pins=resource_pins,literal_instructions=r['steps'],draws=r['draws'],queue_requests=r['queue_requests'],calls=r['calls'],source_pins={str(p.relative_to(P)):sha(p)for p in model.files},input_pins={str(p):sha(p)for p in a.before.glob('*.bin')})
  if a.expected:
   for name,data in result.items():
    actual=(a.expected/name).read_bytes();assert len(data)==len(actual),(name,'length');diff=next((i for i,(x,y)in enumerate(zip(data,actual))if x!=y),None);assert diff is None,(name,diff,data[diff],actual[diff])
   report.update(status='passed',production_comparison='passed',compared_bytes=sum(map(len,result.values())))
 except Exception as e:report.update(status='error',reason=repr(e))
 (out/'result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k not in ['source_pins','input_pins']},indent=2))
if __name__=='__main__':main()

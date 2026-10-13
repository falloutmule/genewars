"""Full literal Mule5859 including the numeric water-effect factory.

This publishes private afterimages for a later native captured-pass comparison.
It does not claim that the runtime capture has already been compared.
"""
from pathlib import Path
import importlib.util,sys,struct,json,hashlib,shutil
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('mule',Path(__file__).with_name('creature-hungry-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
ORIGIN=P/'private/review/family8-trajectory/07e0a0d3f4944407b813480131408781/frame5859/boundary'
W=P/'private/review/mule-deep-water-world';START=W/'before';Original=m.CapturedSource;instances=[]
class WithFactory(Original):
 def __init__(self):
  super().__init__();files=[P/'private/ghidra'/x for x in ['mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt']];self.files+=files;self.code.update(m.m.m.Model(files,b'').code);order=sorted(self.code);self.next=dict(zip(order,order[1:]));instances.append(self)
 def run(self,*args,**kwargs):
  state=kwargs['state_before'].copy();db=bytearray((P/'private/integration/game-runtime/effects/EFF.ANB').read_bytes());frames,animations=struct.unpack_from('<II',db,10)
  for i in range(struct.unpack_from('<H',db)[0]):
   for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x610000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
  state.update({0x610000+i:b for i,b in enumerate(db)});state.update({0x116818+i:b for i,b in enumerate(struct.pack('<I',0x610000+animations))})
  for i in range(200):
   record=0x1688ae+i*81;code=state[record+69]
   state.update({record+35+j:b for j,b in enumerate(struct.pack('<I',0x112254))})
   if state[record+6]:state.update({record+31+j:b for j,b in enumerate(struct.pack('<I',0x610000+animations+code*10))})
  kwargs['state_before']=state;return super().run(*args,**kwargs)

def main():
 START.mkdir(parents=True,exist_ok=True)
 for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']:shutil.copyfile(ORIGIN/name,START/name)
 m.CapturedSource=WithFactory;sys.argv=['source','5859','--snapshot='+str(START.relative_to(P))];m.main();model=instances[0]
 assert all(bytes(model.load(0x1688ae+i,1)for i in range(16200))==bytes(other.load(0x1688ae+i,1)for i in range(16200))for other in instances[1:]),'factory stack-poison mismatch'
 before={p.name:p.read_bytes()for p in START.glob('*.bin')if not p.name.startswith('source-')}
 out={name:bytearray(data)for name,data in before.items()};backing=out['backing.bin']
 backing[:0x355e]=(START/'source-prefix-after5859.bin').read_bytes();backing[0x355e:0x3b55e]=(START/'source-map-after5859.bin').read_bytes();backing[0x43d4e:0x48f4e]=(START/'source-plants-after5859.bin').read_bytes()
 for name,address,stride in [('creatures.bin',0x1542be,124),('actors.bin',0x16360e,200),('effects.bin',0x1688ae,81),('family8.bin',0x16da36,102)]:
  if name=='family8.bin':continue
  data=bytearray(model.load(address+i,1)for i in range(len(out[name])))
  for offset in range(0,len(data),stride):data[offset+31:offset+39]=before[name][offset+31:offset+39]
  out[name]=data
 out['players.bin']=bytes(model.load(0x118d74+i,1)for i in range(7220));out['rng.bin']=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
 for name,data in out.items():(W/name).write_bytes(data)
 trace=json.loads((START/'source-trace5859.json').read_text());report=dict(status='literal-whole-mule-pass-afterimages',frame=5859,literal_instructions=trace['report']['steps'],origin=ORIGIN.relative_to(P).as_posix(),inputs=trace['snapshot_inputs'],listing_pins=trace['listing_pins'],outputs={name:hashlib.sha256(data).hexdigest()for name,data in out.items()},scope='Complete Mule callback with48790 factory, pointer bytes normalized to owned input. No native comparison yet; no subsequent effects scheduler is included.',original_execution=False)
 (W/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','frame','literal_instructions']}))
if __name__=='__main__':main()

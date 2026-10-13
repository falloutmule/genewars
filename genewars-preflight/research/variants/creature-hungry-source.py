"""Read-only listing replay of the explicit combined before-Mule snapshot."""
from pathlib import Path
import importlib.util,struct,json,hashlib,sys
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('creature_source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class CapturedSource(m.Source):
 def store(self,a,n,v):
  if getattr(self,'pc',None)==0x5ed20 and 0x2f0000<=a<0x300000 and not hasattr(self,'prefix_before'):
   self.prefix_before=bytes(self.load(0x118d60+i,1)for i in range(0x355e))
  if getattr(self,'pc',None)==0x5b580 and not hasattr(self,'acquisition_before'):
   self.acquisition_before=bytes(self.load(0x1542be+i,1)for i in range(124));self.target_pointer=self.r['EDX']
  if getattr(self,'pc',None)==0x5c730 and not hasattr(self,'feed_before'):
   self.feed_before=bytes(self.load(0x1542be+i,1)for i in range(124));self.nutrition=self.r['EDX']&255
  super().store(a,n,v)
 def load(self,a,n):
  if getattr(self,'pc',None)==0x5b7d7 and not hasattr(self,'acquisition_after'):
   self.acquisition_after=b''
   self.acquisition_after=bytes(super(CapturedSource,self).load(0x1542be+i,1)for i in range(124))
  if getattr(self,'pc',None)==0x5c7cd and not hasattr(self,'feed_after'):
   self.feed_after=b'';self.feed_after=bytes(super(CapturedSource,self).load(0x1542be+i,1)for i in range(124))
  return super().load(a,n)
def main():
 frame=int(sys.argv[1])if len(sys.argv)>1 else 365
 prefix='combined-seed0-before-mule'if '--seed0'in sys.argv else'combined-before-mule';w=P/f'private/integration/{prefix}{frame}'
 b=(w/'backing.bin').read_bytes();c=(w/'creatures.bin').read_bytes();seed,ring,count=struct.unpack('<HBI',(w/'rng.bin').read_bytes())
 state={0x118d60+i:x for i,x in enumerate(b[:0x355e])};state.update({0x11c2be+i:x for i,x in enumerate(b[0x355e:0x3b55e])})
 for i in range(300):
  raw=c[i*124:(i+1)*124];state.update({0x1542be+i*124+j:x for j,x in enumerate(raw)if not 31<=j<39})
 for i in range(512):
  raw=b[0x43d4e+i*41:0x43d4e+(i+1)*41];raw=raw[:31]+bytes(8)+raw[31:];state.update({0x15d40e+i*49+j:x for j,x in enumerate(raw)if not 31<=j<39})
 #Explicit common native pools from the parent-owned snapshot. Actor and
 #effect bytes include their real runtime gaps, not guessed packed priors.
 for name,base in [('actors.bin',0x16360e),('effects.bin',0x1688ae),('players.bin',0x118d74)]:
  state.update({base+i:x for i,x in enumerate((w/name).read_bytes())})
 state[0x118770]=b[0x52466];state[0x11872c]=ring
 for i,x in enumerate(count.to_bytes(4,'little')):state[0x118782+i]=x
 model=CapturedSource();extra=[P/'private/ghidra/terrain-map-refs/instructions-0005b580.txt',P/'private/ghidra/activation-rng-refs/instructions-0005c920.txt',P/'private/ghidra/initial-creature/instructions-0005d360.txt',P/'private/ghidra/terrain-map-refs/instructions-00074a50.txt',P/'private/ghidra/initial-creature/instructions-0006c900.txt',P/'private/ghidra/initial-creature/instructions-0005c730.txt',P/'private/ghidra/player-creation/instructions-0001e810.txt',P/'private/ghidra/landing-contact/instructions-000417e0.txt',P/'private/ghidra/terrain-map-refs/instructions-0006c780.txt',P/'private/ghidra/panel-activation-refs/instructions-000724c0.txt',P/'private/ghidra/entity-plant/instructions-0007af00.txt',P/'private/ghidra/display-geometry/instructions-000979e8.txt',P/'private/ghidra/plant-startup/instructions-000409f0.txt',P/'private/ghidra/plant-startup/audio-shared-tail.txt',P/'private/ghidra/plant-startup/instructions-0002c200.txt']
 full=next(p for p in extra if p.name=='instructions-0006c900.txt');trim=P/'private/review/creature-motion/listing-6c900-closed.txt';trim.write_text('\n'.join(line for line in full.read_text().splitlines()if int(line[:8],16)<=0x6c927)+'\n');extra=[trim if p==full else p for p in extra]
 ext=m.m.Model(extra,b'');model.code.update(ext.code);model.next=dict(zip(sorted(model.code),sorted(model.code)[1:]));model.files+=extra+[full];model.code[0x2c200]=('RET',[])
 report,after,trace=model.run(frame,seed,state_before=state)
 for fill in [0x55,0xff]:
  other=CapturedSource();other.code.update(ext.code);other.next=dict(zip(sorted(other.code),sorted(other.code)[1:]));other.code[0x2c200]=('RET',[])
  _,poisoned,_=other.run(frame,seed,state_before=state,stack_fill=fill)
  assert after==poisoned,('stack-poison',frame,fill)
  assert bytes(model.saved_state[0x11c2be+i]for i in range(229376))==bytes(other.saved_state[0x11c2be+i]for i in range(229376))
 (w/f'source-after{frame}.bin').write_bytes(after)
 prefix_after=bytearray(b[:0x355e])
 for i,old in enumerate(model.prefix_before):
  if model.saved_state[0x118d60+i]!=old:prefix_after[i]=model.saved_state[0x118d60+i]
 (w/f'source-prefix-after{frame}.bin').write_bytes(prefix_after)
 (w/f'source-map-after{frame}.bin').write_bytes(bytes(model.saved_state[0x11c2be+i]for i in range(229376)))
 packed_plants=bytearray()
 for i in range(512):
  native=bytes(model.saved_state[0x15d40e+i*49+j]for j in range(49));packed_plants+=native[:31]+native[39:]
 (w/f'source-plants-after{frame}.bin').write_bytes(packed_plants)
 if hasattr(model,'acquisition_after'):
  before=bytearray(model.acquisition_before);before[0x61]&=~0x40
  target=model.target_pointer
  fixture=before+model.acquisition_after+struct.pack('<hhhHBI',model.load(target+15,2),m.signed(model.load(target+7,2),16),m.signed(model.load(target+9,2),16),seed,ring,count)
  (w/'acquisition-fixture.bin').write_bytes(fixture)
 if hasattr(model,'feed_after'):
  (w/'feed-fixture.bin').write_bytes(model.feed_before+model.feed_after+struct.pack('<BHBI',model.nutrition,seed,ring,count))
 result=dict(report=report,deferred_ui_calls=trace.count(0x2c200),ui_context_policy="Module2C200 UI cleanup is an explicit pending request; its preserved registers are retained. No native UI root prior is invented.",snapshot_inputs=[dict(path=x.relative_to(P).as_posix(),sha256=hashlib.sha256(x.read_bytes()).hexdigest())for x in w.glob('*.bin')if not x.name.startswith('source-')],listing_pins=[dict(path=x.relative_to(P).as_posix(),sha256=hashlib.sha256(x.read_bytes()).hexdigest())for x in model.files],trace=[hex(x)for x in trace])
 (w/f'source-trace{frame}.json').write_text(json.dumps(result,indent=2)+'\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

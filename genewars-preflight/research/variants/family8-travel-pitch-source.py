"""Literal selector1 travel at the second-cycle pitch-down boundary.

The actual captured before12887 slot6 is followed by an explicit flags/parity
sweep of the same record. Particle allocation is a typed port, not executed.
"""
from pathlib import Path
import importlib.util,struct,hashlib,json
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
START=P/'private/review/family8-trajectory/929a16220fe64789a7597b748c49bc44/frame12887/before'
W=P/'private/review/family8-travel-pitch'

def main():
 W.mkdir(parents=True,exist_ok=True)
 backing=(START/'backing.bin').read_bytes();map_data=backing[0x355e:0x3b55e];(W/'map.bin').write_bytes(map_data)
 state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
 for name,base in [('creatures.bin',0x1542be),('actors.bin',0x16360e),('effects.bin',0x1688ae),('family8.bin',m.BASE),('players.bin',0x118d74)]:state.update({base+i:b for i,b in enumerate((START/name).read_bytes())})
 for i in range(512):
  raw=backing[0x43d4e+i*41:0x43d4e+(i+1)*41];native=raw[:31]+bytes(8)+raw[31:];state.update({0x15d40e+i*49+j:b for j,b in enumerate(native)})
 state[0x118770]=backing[0x52466]
 pool=(START/'family8.bin').read_bytes();initial=pool[6*102:7*102];base=m.BASE+6*102;rng=struct.unpack('<HBI',(START/'rng.bin').read_bytes())
 assert struct.unpack_from('<H',initial,0x49)[0]==0x4210
 model=m.Model();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['13108','11400','73a50']]+[P/'private/ghidra/landing-contact/instructions-0001f030.txt'];model.files+=files
 model.code.update(m.m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 cases=[('captured',12887,initial)]
 for frame in range(12884,12888):
  for bits in range(32):
   before=bytearray(initial);flags=0x210
   for bit,mask in enumerate([4,0x20,0x2000,0x4000,0x2004]):
    if bits&(1<<bit):flags|=mask
   struct.pack_into('<H',before,0x49,flags);cases.append(('explicit_flags_parity',frame,bytes(before)))
 inputs=bytearray();expected=bytearray();particles=bytearray();timeline=[];steps=0
 for label,frame,a in cases:
  after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=state,base=base)
  if label=='captured':
   for fill in [85,255]:
    other=m.Model();other.code.update(m.m.m.Model(files,b'').code);order=sorted(other.code);other.next=dict(zip(order,order[1:]));other.next[0x97830]=0x97831
    check,nr,_,q,_=other.step(a,map_data,frame,rng,fill,extra_state=state,base=base);assert (check,nr,q)==(after,nrng,requests)
  normalized=bytearray(after);normalized[:6]=a[:6]
  inputs+=a+struct.pack('<IHBI',frame,*rng);expected+=normalized+struct.pack('<HBI',*nrng)
  if requests['particle']:
   q=model.particle;particles+=struct.pack('<BBIi3H',1,q['code'],q['point'],q['height'],*q['angles'])
  else:particles+=bytes(16)
  steps+=n;timeline.append(dict(label=label,frame=frame,before_flags=struct.unpack_from('<H',a,0x49)[0],after_flags=struct.unpack_from('<H',after,0x49)[0],literal_instructions=n))
 for name,data in [('input.bin',inputs),('expected.bin',expected),('particles.bin',particles)]: (W/name).write_bytes(data)
 paths=model.files+[Path(__file__),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']+list(START.glob('*.bin'))
 report=dict(status='literal-travel-pitch-fixtures',cases=len(cases),literal_instructions=steps,timeline=timeline,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (W/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','cases','literal_instructions']}))
if __name__=='__main__':main()

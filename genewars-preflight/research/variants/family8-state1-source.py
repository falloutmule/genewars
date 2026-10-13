"""Literal selector1 travel from an actual combined-world before6144 capture.

Later callbacks are an isolated numeric context: particle factories are typed
ports, not silently claimed to consume the same RNG as the full scheduler.
Intrusive links are interpreted and persisted, then normalized only for the
separate pure-record comparison; complete source records remain private.
"""
from pathlib import Path
import importlib.util,struct,hashlib,json
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def main():
 start=P/'private/integration/family8-combined010-seed5428-frame6144/before';work=P/'private/review/family8-state1';work.mkdir(parents=True,exist_ok=True)
 backing=(start/'backing.bin').read_bytes();map_data=backing[0x355e:0x3b55e];(work/'map.bin').write_bytes(map_data)
 state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
 for name,base in [('creatures.bin',0x1542be),('actors.bin',0x16360e),('effects.bin',0x1688ae),('family8.bin',m.BASE),('players.bin',0x118d74)]:state.update({base+i:b for i,b in enumerate((start/name).read_bytes())})
 for i in range(512):
  raw=backing[0x43d4e+i*41:0x43d4e+(i+1)*41];native=raw[:31]+bytes(8)+raw[31:];state.update({0x15d40e+i*49+j:b for j,b in enumerate(native)})
 state[0x118770]=backing[0x52466]
 pool=(start/'family8.bin').read_bytes();a=pool[4*102:5*102];base=m.BASE+4*102;rng=struct.unpack('<HBI',(start/'rng.bin').read_bytes())
 model=m.Model();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['13108','11400','73a50']]+[P/'private/ghidra/landing-contact/instructions-0001f030.txt'];model.files+=files
 model.code.update(m.m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 inputs=bytearray();expected=bytearray();particles=bytearray();timeline=[];steps=0
 for frame in range(6144,7144):
  inputs+=a+struct.pack('<IHBI',frame,*rng)
  after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=state,base=base)
  normalized=bytearray(after);normalized[:6]=a[:6]
  expected+=normalized+struct.pack('<HBI',*nrng)
  if requests['particle']:
   q=model.particle;particles+=struct.pack('<BBIi3H',1,q['code'],q['point'],q['height'],*q['angles'])
  else:particles+=bytes(16)
  (work/f'source-after{frame}.bin').write_bytes(after)
  steps+=n;timeline.append(dict(frame=frame,point=struct.unpack_from('<hh',after,7),height=struct.unpack_from('<i',after,0x27)[0],flags=struct.unpack_from('<H',after,0x49)[0],timer=struct.unpack_from('<h',after,0x64)[0],animation=after[0x19],animation_frame=after[0x1b],particle=requests['particle'],draws=(nrng[2]-rng[2])&0xffffffff))
  state=model.saved_state;map_data=bytes(state[0x11c2be+i]for i in range(229376));a=after;rng=nrng
  if struct.unpack_from('<h',a,0x64)[0]>0:break
 else:raise AssertionError('travel did not arrive within1000callbacks')
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected);(work/'particles.bin').write_bytes(particles)
 paths=model.files+[Path(__file__),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']+list(start.glob('*.bin'))
 report=dict(status='literal-selector1-state1-fixtures',cases=len(timeline),literal_instructions=steps,timeline=timeline,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','cases','literal_instructions']}));print(timeline[-1])
if __name__=='__main__':main()

"""Literal positive-timer state1 callbacks from actual combined slot7."""
from pathlib import Path
import importlib.util,struct,hashlib,json
P=Path.cwd()
s=importlib.util.spec_from_file_location('family8_source',P/'research/variants/family8-state4-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def main():
 start=P/'private/review/family8-trajectory/579c9bdb947e4419926cb281b7bf0f38/frame6332/before';work=P/'private/review/family8-idle';work.mkdir(parents=True,exist_ok=True)
 backing=(start/'backing.bin').read_bytes();map_data=backing[0x355e:0x3b55e];(work/'map.bin').write_bytes(map_data)
 state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
 for name,base in [('creatures.bin',0x1542be),('actors.bin',0x16360e),('effects.bin',0x1688ae),('family8.bin',m.BASE),('players.bin',0x118d74)]:state.update({base+i:b for i,b in enumerate((start/name).read_bytes())})
 for base,stride,count,vtable in [(0x1542be,124,300,0x111fec),(0x16360e,200,20,0x112194),(0x1688ae,81,200,0x112254)]:
  for i in range(count):
   for j,b in enumerate(struct.pack('<I',vtable)):state[base+i*stride+0x23+j]=b
 for i in range(512):
  raw=backing[0x43d4e+i*41:0x43d4e+(i+1)*41];native=raw[:31]+bytes(8)+raw[31:];state.update({0x15d40e+i*49+j:b for j,b in enumerate(native)})
  for j,b in enumerate(struct.pack('<I',0x1120ac)):state[0x15d40e+i*49+0x23+j]=b
 buildings=bytearray()
 for i in range(256):
  raw=backing[301630+i*59:301630+(i+1)*59];native=raw[:31]+bytes(8)+raw[31:];state.update({0x1645ae+i*67+j:b for j,b in enumerate(native)})
  if 0<=i<64:buildings+=native
 (work/'buildings.bin').write_bytes(buildings);assert all(buildings[67*i+6]==0 for i in range(64));assert backing[0x34b8]==0
 for i in range(16):
  for j,b in enumerate(struct.pack('<I',0x11242c)):state[m.BASE+i*102+0x23+j]=b
 state[0x118770]=backing[0x52466]
 pool=(start/'family8.bin').read_bytes();a=pool[4*102:5*102];base=m.BASE+4*102;rng=struct.unpack('<HBI',(start/'rng.bin').read_bytes())
 model=m.Model();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['13108','11400','73a50','11720','114b0','13e60','1ea80']]
 files +=[P/'private/ghidra/landing-contact/instructions-0001f030.txt',P/'private/ghidra/actor-activation/instructions-00073e00.txt']
 files +=[P/'private/ghidra/initial-creature/instructions-0006d230.txt',P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt']
 model.files+=files;model.code.update(m.m.m.Model(files,b'').code)
 #Old overlapping exports contain now-corrected mid-instruction ghosts.
 #The repaired11720 export is authoritative for this exact helper range.
 model.code={a:v for a,v in model.code.items()if not 0x11720<=a<=0x1188d};model.code.update(m.m.m.Model([P/'private/ghidra/countdown-expiry/instructions-00011720.txt'],b'').code)
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 model.code[0x6fc52]=('JMP',['0x00070790'])
 inputs=bytearray();expected=bytearray();particles=bytearray();timeline=[];steps=0
 for frame in range(6333,6334):
  a=bytearray(a);a[0x53]=255;a[0x49]|=32
  inputs+=a+struct.pack('<IHBI',frame,*rng)
  after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=state,base=base,entry=0x6fb60)
  expected_prefix=bytearray(a);expected_prefix[0x49]&=0x9f
  assert after==expected_prefix and nrng==rng
  assert bytes(model.saved_state[0x11c2be+i]for i in range(229376))==map_data
  print('PREFIX',json.dumps(dict(instructions=n,queries=trace.count(0x6d230),search=0x305f0 in trace,callback=trace.count(0x1ea80),draws=(nrng[2]-rng[2])&0xffffffff)))
  normalized=bytearray(after);normalized[:6]=a[:6];expected+=normalized+struct.pack('<HBI',*nrng)
  if requests['particle']:
   q=model.particle;particles+=struct.pack('<BBIi3H',1,q['code'],q['point'],q['height'],*q['angles'])
  else:particles+=bytes(16)
  (work/f'source-after{frame}.bin').write_bytes(after)
  steps+=n;timeline.append(dict(frame=frame,state=after[0x56],timer=struct.unpack_from('<h',after,0x64)[0],point=struct.unpack_from('<hh',after,7),flags=struct.unpack_from('<H',after,0x49)[0],accels=struct.unpack_from('<hhh',after,0x33),particle=requests['particle'],idle_search=0x305f0 in trace))
  state=model.saved_state;map_data=bytes(state[0x11c2be+i]for i in range(229376));a=after;rng=nrng
  
 
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected);(work/'particles.bin').write_bytes(particles)
 continuation=work/'continuation';continuation.mkdir(exist_ok=True)
 for name,base,length in [('prefix',0x118d60,0x355e),('map',0x11c2be,229376),('creatures',0x1542be,300*124),('plants',0x15d40e,512*49),('actors',0x16360e,20*200),('buildings',0x1645ae,256*67),('effects',0x1688ae,200*81),('family8',m.BASE,16*102)]:
  (continuation/f'{name}.bin').write_bytes(bytes(model.load(base+i,1)for i in range(length)))
 (continuation/'rng.bin').write_bytes(struct.pack('<HBI',*rng))
 paths=model.files+[Path(__file__),P/'research/variants/family8-state4-source.py',P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']+list(start.glob('*.bin'))
 report=dict(status='literal-selector1-hover-fixtures',cases=len(timeline),literal_instructions=steps,timeline=timeline,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','cases','literal_instructions']}));print(timeline[-1])
if __name__=='__main__':main()

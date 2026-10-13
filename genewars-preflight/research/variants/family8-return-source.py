"""Literal selector1/state3 homeward phases; retirement is a typed port."""
from pathlib import Path
import importlib.util,struct,hashlib,json,sys
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def main():
 actual='--actual' in sys.argv
 start=P/'private/review/family8-trajectory/e1ec8cf35b6e4f8fbaa276e4a0fee8f1/frame6290/before' if actual else P/'private/review/family8-hover/continuation'
 work=P/'private/review/family8-return'/('actual6290' if actual else 'isolated6408');work.mkdir(parents=True,exist_ok=True)
 state={}
 segments=[('prefix',0x118d60),('map',0x11c2be),('creatures',0x1542be),('plants',0x15d40e),('actors',0x16360e),('buildings',0x1645ae),('effects',0x1688ae),('family8',m.BASE)]
 if actual:
  backing=(start/'backing.bin').read_bytes();map_data=backing[0x355e:0x3b55e]
  state.update({0x118d60+i:b for i,b in enumerate(backing[:0x355e])})
  for name,base in [('creatures',0x1542be),('actors',0x16360e),('effects',0x1688ae),('family8',m.BASE),('players',0x118d74)]:state.update({base+i:b for i,b in enumerate((start/f'{name}.bin').read_bytes())})
  for base,stride,count,vtable in [(0x1542be,124,300,0x111fec),(0x16360e,200,20,0x112194),(0x1688ae,81,200,0x112254)]:
   for i in range(count):
    state.update({base+i*stride+0x23+j:b for j,b in enumerate(struct.pack('<I',vtable))})
  for filebase,base,stride,count,vtable in [(0x43d4e,0x15d40e,41,512,0x1120ac),(301630,0x1645ae,59,256,0)]:
   for i in range(count):
    raw=backing[filebase+i*stride:filebase+(i+1)*stride];native=raw[:31]+bytes(8)+raw[31:];state.update({base+i*(stride+8)+j:b for j,b in enumerate(native)})
    if vtable:state.update({base+i*(stride+8)+0x23+j:b for j,b in enumerate(struct.pack('<I',vtable))})
  for i in range(16):state.update({m.BASE+i*102+0x23+j:b for j,b in enumerate(struct.pack('<I',0x11242c))})
  state[0x118770]=backing[0x52466]
 else:
  for name,base in segments:state.update({base+i:b for i,b in enumerate((start/f'{name}.bin').read_bytes())})
  map_data=(start/'map.bin').read_bytes()
 (work/'map.bin').write_bytes(map_data);pool=(start/'family8.bin').read_bytes();slot=6 if actual else 7;first_frame=6290 if actual else 6408;parent_slot=2 if actual else 3
 a=pool[slot*102:(slot+1)*102];base=m.BASE+slot*102;rng=struct.unpack('<HBI',(start/'rng.bin').read_bytes())
 model=m.Model();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['132fa','13108','11400','73a50','11720','114b0','13e60','1ea80']]
 files +=[P/'private/ghidra/landing-contact/instructions-0001f030.txt',P/'private/ghidra/actor-activation/instructions-00073e00.txt',P/'private/ghidra/initial-creature/instructions-0006d230.txt',P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt']
 model.files+=files;model.code.update(m.m.m.Model(files,b'').code)
 model.code={at:v for at,v in model.code.items()if not 0x11720<=at<=0x1188d};model.code.update(m.m.m.Model([P/'private/ghidra/countdown-expiry/instructions-00011720.txt'],b'').code)
 #Source103F0 is separately compared by root. Here its clear-kind effect
 #preserves the callback's epilogue decision; no audio/map cleanup is invented.
 model.code[0x103f0]=('MOV',['byte ptr [EAX + 0x6]','0x0']);model.code[0x103f1]=('RET',[])
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 inputs=bytearray();expected=bytearray();particles=bytearray();parents=bytearray();timeline=[];steps=0
 for frame in range(first_frame,first_frame+1000):
  inputs+=a+struct.pack('<IHBI',frame,*rng);parents+=bytes(state[m.BASE+i]for i in range(16*102))
  after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=state,base=base)
  normalized=bytearray(after);normalized[:6]=a[:6];normalized[6]=a[6];retired=0x103f0 in trace
  expected+=normalized+struct.pack('<HBI',*nrng)+bytes([retired,parent_slot if retired else 255,0x13765 in trace])
  if requests['particle']:
   q=model.particle;particles+=struct.pack('<BBIi3H',1,q['code'],q['point'],q['height'],*q['angles'])
  else:particles+=bytes(16)
  (work/f'source-after{frame}.bin').write_bytes(after)
  steps+=n;timeline.append(dict(frame=frame,active=after[6],state=after[0x56],substate=after[0x57],point=struct.unpack_from('<hh',after,7),height=struct.unpack_from('<i',after,0x27)[0],flags=struct.unpack_from('<H',after,0x49)[0],particle=requests['particle'],idle_search=0x305f0 in trace,parent57=model.load(m.BASE+parent_slot*102+0x57,1),retired=retired))
  state=model.saved_state;map_data=bytes(state[0x11c2be+i]for i in range(229376));a=after;rng=nrng
  if retired:break
 else:raise AssertionError('return did not finish within1000callbacks')
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected);(work/'particles.bin').write_bytes(particles);(work/'parents.bin').write_bytes(parents)
 continuation=work/'continuation';continuation.mkdir(exist_ok=True)
 for name,base,length in [('prefix',0x118d60,0x355e),('map',0x11c2be,229376),('creatures',0x1542be,300*124),('plants',0x15d40e,512*49),('actors',0x16360e,20*200),('buildings',0x1645ae,256*67),('effects',0x1688ae,200*81),('family8',m.BASE,16*102),('players',0x118d74,4*1805)]:
  (continuation/f'{name}.bin').write_bytes(bytes(model.load(base+i,1)for i in range(length)))
 (continuation/'rng.bin').write_bytes(struct.pack('<HBI',*rng))
 paths=model.files+[Path(__file__),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'private/ghidra/countdown-expiry/instructions-000103f0.txt']+list(start.glob('*.bin'))
 if actual:paths+=[start.parent.parent/'compilation.json',start.parent.parent/'result.json']
 report=dict(status='literal-selector1-return-fixtures',context='actual-combined-before6290-slot6' if actual else 'isolated-hover-continuation-after6407-slot7',cases=len(timeline),literal_instructions=steps,timeline=timeline,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','cases','literal_instructions']}));print(timeline[-1])
if __name__=='__main__':main()

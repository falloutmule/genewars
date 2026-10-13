"""Literal selector0/state9 ascent from source-proved state8 transitions."""
from pathlib import Path
import importlib.util,struct,hashlib,json,sys
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def main():
 actual='--actual' in sys.argv
 start=P/('private/review/family8-trajectory/a27b449717204a89bd5535536ad2d53f/frame6567/before' if actual else 'private/review/family8-trajectory/3f14d91e20d54928881be92a55cc4e35/frame6442/before')
 work=P/'private/review/family8-departure';work=work/'actual6567' if actual else work;work.mkdir(parents=True,exist_ok=True)
 backing=(start/'backing.bin').read_bytes();map_data=backing[0x355e:0x3b55e];(work/'map.bin').write_bytes(map_data)
 state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
 for name,base in [('creatures',0x1542be),('actors',0x16360e),('effects',0x1688ae),('family8',m.BASE),('players',0x118d74)]:state.update({base+i:b for i,b in enumerate((start/f'{name}.bin').read_bytes())})
 for base,stride,count,vtable in [(0x1542be,124,300,0x111fec),(0x16360e,200,20,0x112194),(0x1688ae,81,200,0x112254),(m.BASE,102,16,0x11242c)]:
  for i in range(count):state.update({base+i*stride+0x23+j:b for j,b in enumerate(struct.pack('<I',vtable))})
 for filebase,base,stride,count,vtable in [(0x43d4e,0x15d40e,41,512,0x1120ac),(301630,0x1645ae,59,256,0)]:
  for i in range(count):
   raw=backing[filebase+i*stride:filebase+(i+1)*stride];native=raw[:31]+bytes(8)+raw[31:];state.update({base+i*(stride+8)+j:b for j,b in enumerate(native)})
   if vtable:state.update({base+i*(stride+8)+0x23+j:b for j,b in enumerate(struct.pack('<I',vtable))})
 state[0x118770]=backing[0x52466]
 model=m.Model();files=[P/'private/ghidra/countdown-expiry/instructions-00012dae.txt',P/'private/ghidra/landing-contact/instructions-0001f030.txt',P/'private/ghidra/initial-creature/instructions-0006d230.txt',P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt']
 model.files+=files;model.code.update(m.m.m.Model(files,b'').code);model.code[0x103f0]=('MOV',['byte ptr [EAX + 0x6]','0x0']);model.code[0x103f1]=('RET',[])
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 inputs=bytearray();expected=bytearray();timeline=[];steps=0;parent=P/'private/review/family8-parent'
 source_records=(parent/'state8-expected.bin').read_bytes();contexts=[]
 if actual:contexts=[('actual-prefixfix-6567',(start/'family8.bin').read_bytes()[:102],struct.unpack('<HBI',(start/'rng.bin').read_bytes()))]
 else:
  for i,seed in enumerate([0,1,5428,65535]):
   record=source_records[(4+i)*109:(5+i)*109];contexts.append((seed,record[:102],struct.unpack('<HBI',record[102:])))
 base=m.BASE if actual else m.BASE+204;first_frame=6567 if actual else 6444
 for seed,a,rng in contexts:
  assert a[0x56]==9
  extra=dict(state)
  for frame in range(first_frame,first_frame+500):
   inputs+=a+struct.pack('<IHBI',frame,*rng)
   after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=extra,base=base)
   retired=0x103f0 in trace;normalized=bytearray(after);normalized[:6]=a[:6];normalized[6]=a[6]
   expected+=normalized+struct.pack('<HBI',*nrng)+bytes([retired,0x13765 in trace,0x11aa8 in trace,0x12ded in trace])
   steps+=n;timeline.append(dict(seed_context=seed,frame=frame,height=struct.unpack_from('<i',after,0x27)[0],speed=struct.unpack_from('<h',after,0x3d)[0],retired=retired))
   (work/('seed'+str(seed)+'-after'+str(frame)+'.bin')).write_bytes(after);a=after;rng=nrng;extra=model.saved_state
   if retired:break
  else:raise AssertionError('parent did not depart within500callbacks')
 for name,data in [('input',inputs),('expected',expected)]:(work/f'{name}.bin').write_bytes(data)
 paths=model.files+[Path(__file__),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'private/ghidra/countdown-expiry/instructions-000103f0.txt',parent/'state8-expected.bin',parent/'source.json']+list(start.glob('*.bin'))
 paths+=[start.parent.parent/'compilation.json',start.parent.parent/'result.json']
 report=dict(status='literal-parent-state9-departure',context='actual6567-before-footstep-correction' if actual else 'four-explicit-state8-ready-seeds',callbacks=len(timeline),literal_instructions=steps,timeline=timeline,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','callbacks','literal_instructions']}));print([t for t in timeline if t['retired']])
if __name__=='__main__':main()

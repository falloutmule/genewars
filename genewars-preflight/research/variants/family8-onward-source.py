"""Literal state5 waiting and newly spawned selector1/state0 investigation."""
from pathlib import Path
import importlib.util,struct,json,hashlib
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def factory(selector,point,height):
 model=m.m.Source();files=[P/'private/ghidra'/f for f in ['mission-array-refs/instructions-000101d0.txt','countdown-expiry/instructions-0001ee80.txt','countdown-expiry/instructions-0001ea90.txt','mission-tail-init/instructions-00010060.txt','terrain-map-refs/instructions-0007b240.txt']]
 model.code.update(m.m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));base=m.BASE
 state={base+i:0 for i in range(16*102)}
 for i in range(16):
  for j,b in enumerate(struct.pack('<I',0x11242c)):state[base+102*i+0x23+j]=b
 state[0x11c2b9]=0;state[0x11872c]=0
 for i in range(4):state[0x118782+i]=0
 for i,b in enumerate(struct.pack('<I',0x500000)):state[0x116820+i]=b
 model.run(state_before=state,entry=0x101d0,registers=dict(EAX=selector,EDX=int.from_bytes(struct.pack('<hh',*point),'little'),EBX=height,ECX=0))
 after=bytearray(model.load(base+i,1)for i in range(102));after[31:39]=bytes(8)
 return bytes(after)
def main():
 mission=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes();map_data=mission[0x355e:0x3b55e]
 work=P/'private/review/family8-onward';work.mkdir(parents=True,exist_ok=True);(work/'map.bin').write_bytes(map_data)
 a=(P/'private/review/family8-state4/state5-start6001-seed5428.bin').read_bytes();rng=(5428*0xe62d&65535,0,0)
 outputs={};total=0
 for name,initial,start in [('parent',a,6047),('child',factory(1,[20704,13536],21982),6046)]:
  a=initial;inputs=bytearray();expected=bytearray();particles=bytearray();timeline=[];model=m.Model()
  for frame in range(start,start+128):
   inputs+=a+struct.pack('<IHBI',frame,*rng)
   after,nrng,steps,requests,trace=model.step(a,map_data,frame,rng)
   expected+=after+struct.pack('<HBI',*nrng)+bytes(requests[k]for k in ['child','ui','moving_audio','query_stop_audio','particle','target_selection'])
   if requests['particle']:
    p=model.particle;particles+=struct.pack('<BBIi3H',1,p['code'],p['point'],p['height'],*p['angles'])
   else:particles+=bytes(16)
   total+=steps;timeline.append(dict(frame=frame,state=after[0x56],animation=after[0x19],animation_frame=after[0x1b],height=struct.unpack_from('<i',after,0x27)[0],speed=struct.unpack_from('<h',after,0x41)[0],requests=requests,draws=(nrng[2]-rng[2])&0xffffffff))
   (work/f'{name}-after{frame}.bin').write_bytes(after)
   a=after;rng=nrng
   if requests['target_selection']:break
  (work/f'{name}-input.bin').write_bytes(inputs);(work/f'{name}-expected.bin').write_bytes(expected);(work/f'{name}-particles.bin').write_bytes(particles);outputs[name]=timeline
 paths=model.files+[Path(__file__),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),m.ANB,P/'private/integration/game-runtime/terrain/CONQ001.DAT',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'private/review/family8-state4/state5-start6001-seed5428.bin']
 paths +=[P/'private/ghidra'/f for f in ['mission-array-refs/instructions-000101d0.txt','countdown-expiry/instructions-0001ee80.txt','countdown-expiry/instructions-0001ea90.txt','mission-tail-init/instructions-00010060.txt','terrain-map-refs/instructions-0007b240.txt']]
 report=dict(status='literal-onward-fixtures',literal_instructions=total,timeline=outputs,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:[len(v),v[-1]]for k,v in outputs.items()},indent=2))
if __name__=='__main__':main()

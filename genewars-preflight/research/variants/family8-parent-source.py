"""Literal11AEB signed-counter decision and bounded stationary12DAE callbacks."""
from pathlib import Path
import importlib.util,struct,hashlib,json
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('family8_source',Path(__file__).with_name('family8-state4-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def main():
 start=P/'private/review/family8-trajectory/3f14d91e20d54928881be92a55cc4e35/frame6442/before';work=P/'private/review/family8-parent';work.mkdir(parents=True,exist_ok=True)
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
 pool=(start/'family8.bin').read_bytes();initial=pool[204:306];assert initial[0x57]==1
 model=m.Model();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['12c29','12c59','12c64','12dae','13e60','1ea80']]
 files +=[P/'private/ghidra/landing-contact/instructions-0001f030.txt',P/'private/ghidra/initial-creature/instructions-0006d230.txt',P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt']
 model.files+=files;model.code.update(m.m.m.Model(files,b'').code)
 #Formatter and UI module invocation are explicit request ports. No text,
 #audio engine or module state is invented; source RNG remains interpreted.
 model.code[0x981d3]=('RET',[]);model.code[0x41ce0]=('RET',['0x14'])
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
 inputs=bytearray();expected=bytearray();timeline=[];steps=0
 counter_ptr=0x118d74+2*1805+0x11
 for counter in [-2147483648,-1,0,1,4999,5000,5001,2147483647]:
  for threshold in [0,1,5000,32768,65535]:
   for seed in [0,1,5428,65535]:
    for signal in [1,255]:
     a=bytearray(initial);a[0x57]=signal;rng=(seed,31,0xffffffff);extra=dict(state)
     extra.update({counter_ptr+i:b for i,b in enumerate(struct.pack('<i',counter))});extra.update({0x11a9dc+i:b for i,b in enumerate(struct.pack('<H',threshold))})
     after,nrng,n,requests,trace=model.step(a,map_data,6442,rng,extra_state=extra,base=m.BASE+204)
     normalized=bytearray(after);normalized[:6]=a[:6];variant=255
     if 0x11c16 in trace:variant=0 if ((nrng[0]>>8)%10)<5 else 1
     inputs+=a+struct.pack('<IHBIiH',6442,*rng,counter,threshold);expected+=normalized+struct.pack('<HBIiB',*nrng,model.load(counter_ptr,4)-(1<<32)if model.load(counter_ptr,4)&(1<<31)else model.load(counter_ptr,4),variant)
     steps+=n;timeline.append(dict(counter=counter,threshold=threshold,seed=seed,signal=signal,state=after[0x56],variant=variant,draws=(nrng[2]-rng[2])&0xffffffff))
 #The actual zero-counter path becomes state8 without a velocity/goal write.
 #State8 waits for other selector0 parents, then starts its own departure.
 a=bytearray(initial);a[0x56]=8;a[0x57]=0;rng=struct.unpack('<HBI',(start/'rng.bin').read_bytes());state8_input=bytearray();state8_expected=bytearray();state8_pools=bytearray();state8_cases=0
 for other_state in [5,8,9,0]:
  for seed in [0,1,5428,65535]:
   group=bytearray(pool)
   for i in range(4):group[i*102+0x56]=8
   group[204:306]=a
   group[0x56]=other_state
   if other_state==0:group[6]=0
   extra=dict(state);extra.update({m.BASE+i:b for i,b in enumerate(group)})
   for i in range(16):extra.update({m.BASE+i*102+0x23+j:b for j,b in enumerate(struct.pack('<I',0x11242c))})
   r=(seed,31,0xffffffff);after,nrng,n,requests,trace=model.step(a,map_data,6443,r,extra_state=extra,base=m.BASE+204)
   normalized=bytearray(after);normalized[:6]=a[:6];state8_input+=a+struct.pack('<IHBI',6443,*r);state8_expected+=normalized+struct.pack('<HBI',*nrng);state8_pools+=group;steps+=n;state8_cases+=1
 state.update({m.BASE+i:b for i,b in enumerate(pool)});state.update({m.BASE+204+i:b for i,b in enumerate(a)})
 for i in range(16):state.update({m.BASE+i*102+0x23+j:b for j,b in enumerate(struct.pack('<I',0x11242c))})
 for frame in range(6443,6571):
  state8_input+=a+struct.pack('<IHBI',frame,*rng)
  state8_pools+=bytes(state[m.BASE+i]for i in range(1632))
  after,nrng,n,requests,trace=model.step(a,map_data,frame,rng,extra_state=state,base=m.BASE+204)
  assert 0x103f0 not in trace and after[0x56]==8
  normalized=bytearray(after);normalized[:6]=a[:6];state8_expected+=normalized+struct.pack('<HBI',*nrng);steps+=n;a=after;rng=nrng;state=model.saved_state;state8_cases+=1
 for name,data in [('input',inputs),('expected',expected),('state8-input',state8_input),('state8-expected',state8_expected),('state8-pools',state8_pools)]:(work/f'{name}.bin').write_bytes(data)
 paths=model.files+[Path(__file__),Path(__file__).with_name('family8-state4-source.py'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),m.ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']+list(start.glob('*.bin'))
 paths+=[start.parent.parent/'compilation.json',start.parent.parent/'result.json']
 report=dict(status='literal-parent-notification-and-state8',notification_cases=len(timeline),state8_callbacks=state8_cases,literal_instructions=steps,timeline=timeline,original_execution=False,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','notification_cases','state8_callbacks','literal_instructions']}))
if __name__=='__main__':main()

"""Literal private-listing replay of selector0/state4; no original execution.

Audio, optional UI and selector1 allocation are typed request boundaries. The
parent record and RNG are interpreted; map linking remains the owning adapter's
transaction. Inputs and full native afterimages remain private.
"""
from pathlib import Path
import importlib.util, struct, hashlib, json
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
BASE=0x16da36
ANB=P/'private/integration/rnc-decoded/7660297d85773f47cacc07dcfdf96e8daa118dc52c8a38afc054c1ef9dfa20bc.bin'
class Model(m.Source):
 def __init__(self):
  super().__init__()
  extra=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt' for x in ['118e0','11943','1eab0','1eac0','1ea90','11270','11380','11aeb','12e11']]
  extra +=[P/'private/ghidra/construction-command/instructions-0001ee40.txt']
  self.files+=extra; self.code.update(m.m.Model(extra,b'').code)
  # No audio engine, UI object, or child allocation is invented. The exact
  # caller argument/state boundaries are captured below.
  for address,instruction in {0x1e810:('RET',['0x8']),0x98210:('RET',[]),0x101d0:('RET',[]),0x48790:('RET',['0x10']),0x114b0:('RET',[]),0x97830:('XOR',['AL','AL']),0x97831:('RET',[]),0x11a18:('JMP',['0x00011a7c'])}.items():self.code[address]=instruction
  order=sorted(self.code);self.next=dict(zip(order,order[1:]));self.next[0x97830]=0x97831
 def load(self,a,n):
  pc=getattr(self,'pc',0)
  if pc==0x101d0 and not hasattr(self,'child'):
   self.child=dict(selector=self.r['EAX'],point=self.r['EDX'],height=m.signed(self.r['EBX']),owner=self.r['ECX']&255)
  if pc==0x48790 and not hasattr(self,'particle'):
   self.particle=dict(code=self.r['EAX'],point=self.r['EDX'],height=m.signed(self.r['EBX']),angles=[super(Model,self).load(self.r['ECX']+2*i,2)for i in range(3)])
  return super().load(a,n)
 def step(self,before,map_data,frame,rng,fill=0,entry=0x118e0,extra_state=None,base=BASE):
  anb=bytearray(ANB.read_bytes());assert hashlib.sha256(anb).hexdigest()=='318dcb26efa6968c34357e67c05b6514efe7d7a8b4052a4e811a851a2ba01bc4'
  frames,animations=struct.unpack_from('<II',anb,10)
  for i in range(9):
   for off in [0,4]:struct.pack_into('<I',anb,animations+10*i+off,0x500000+frames+struct.unpack_from('<I',anb,animations+10*i+off)[0])
  state=dict(extra_state or {});state.update({base+i:b for i,b in enumerate(before)});state.update({0x11c2be+i:b for i,b in enumerate(map_data)})
  state.update({0x500000+i:b for i,b in enumerate(anb)})
  for at,value in [(base+0x23,0x11242c),(base+0x1f,0x500000+animations+(0 if before[0x55]==1 else 80)),(0x118782,rng[2])]:
   state.update({at+i:b for i,b in enumerate(struct.pack('<I',value))})
  state[0x11872c]=rng[1]
  for attr in ['child','particle']:
   if hasattr(self,attr):delattr(self,attr)
  report,_,trace=self.run(frame,rng[0],state_before=state,entry=entry,registers={'EAX':base},stack_fill=fill)
  after=bytearray(self.load(base+i,1)for i in range(102));after[31:39]=before[31:39]
  child=0x101d0 in trace
  requests=dict(child=child,ui=0x11a18 in trace,moving_audio=0x1e810 in trace,query_stop_audio=0x97830 in trace,particle=0x48790 in trace,target_selection=0x114b0 in trace)
  return bytes(after),(report['rng_seed_after'],report['rng_ring_after'],report['rng_draw_count_after']),report['steps'],requests,trace

def main():
 mission=(P/'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes();map_data=mission[0x355e:0x3b55e]
 work=P/'private/review/family8-state4';work.mkdir(parents=True,exist_ok=True);(work/'map.bin').write_bytes(map_data)
 # Literal factory output with its pointer bytes normalized. Use independently
 # generated creation review input/output, whose source comparison is separate.
 import sys
 sys.path.insert(0,str(P/'scripts'))
 factory=m.Source();files=[P/'private/ghidra'/f for f in ['mission-array-refs/instructions-000101d0.txt','countdown-expiry/instructions-0001ee80.txt','countdown-expiry/instructions-0001ea90.txt','mission-tail-init/instructions-00010060.txt','terrain-map-refs/instructions-0007b240.txt']]
 factory.code.update(m.m.Model(files,b'').code);order=sorted(factory.code);factory.next=dict(zip(order,order[1:]))
 state={BASE+i:0 for i in range(16*102)}
 for i in range(16):
  for j,b in enumerate(struct.pack('<I',0x11242c)):state[BASE+102*i+0x23+j]=b
 state[0x11c2b9]=0;state[0x11872c]=0
 for i in range(4):state[0x118782+i]=0
 for i,b in enumerate(struct.pack('<I',0x500000)):state[0x116820+i]=b
 factory.run(state_before=state,entry=0x101d0,registers=dict(EAX=0,EDX=20736|(13568<<16),EBX=3136*2+25600,ECX=0))
 initial=bytearray(factory.load(BASE+i,1)for i in range(102));initial[31:39]=bytes(8)
 model=Model();inputs=bytearray();expected=bytearray();total=0;timeline=[]
 for first_frame in [6001,6002]:
  for first_seed in [0,5428,65535]:
   a=bytes(initial);rng=(first_seed,31,0xffffffff)
   for frame in range(first_frame,first_frame+1000):
    inputs+=a+struct.pack('<IHBI',frame,*rng)
    result,nrng,steps,requests,trace=model.step(a,map_data,frame,rng)
    if frame==first_frame or result[0x56]!=4:
     for fill in [85,255]:
      b,r,_,q,_=Model().step(a,map_data,frame,rng,fill);assert (b,r,q)==(result,nrng,requests)
    expected+=result+struct.pack('<HBI',*nrng)+bytes([requests[k]for k in ['child','ui','moving_audio','query_stop_audio']])
    total+=steps;timeline.append(dict(first_frame=first_frame,first_seed=first_seed,frame=frame,height=struct.unpack_from('<i',result,0x27)[0],speed=struct.unpack_from('<h',result,0x3d)[0],state=result[0x56],animation=result[0x19],animation_frame=result[0x1b],requests=requests))
    a=result;rng=nrng
    if a[0x56]!=4:
     (work/f'state5-start{first_frame}-seed{first_seed}.bin').write_bytes(a)
     break
   else:raise AssertionError('state4 did not finish within1000 callbacks')
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected);(work/'initial.bin').write_bytes(initial)
 paths=model.files+files+[ANB,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'private/integration/game-runtime/terrain/CONQ001.DAT',Path(__file__),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py')]
 report=dict(status='literal-source-fixtures',cases=len(timeline),literal_instructions=total,timeline=timeline,original_execution=False,scope='selector0 state4, full102 parent+RNG; child allocation/UI/audio are explicit request boundaries, no world commit',pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths})
 (work/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','cases','literal_instructions']}));print(timeline[-1])
if __name__=='__main__':main()

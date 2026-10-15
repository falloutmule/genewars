"""Actual1227 family8 callback: nonzero sites do not block positive hover timer."""
from pathlib import Path
import importlib.util,struct,json,hashlib,uuid
P=Path(__file__).resolve().parents[2]
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
m=load('f8',P/'research/variants/family8-state4-source.py');k=load('krell',P/'research/variants/krell-mule-callback-review.py')
before=P/'private/review/krell-mule-callbacks-058/frame-1227-slot-2-family8-before';after=P/'private/review/krell-mule-callbacks-058/frame-1227-slot-2-family8-after';backing=(before/'backing.bin').read_bytes();buffers={p.name:p.read_bytes()for p in before.glob('*.bin')};state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
for start,count,stride,base,width,name,vt in k.LAYOUTS:
 for slot in range(count):
  raw=bytearray(buffers[name][slot*width:(slot+1)*width]if name else backing[start+slot*stride:start+(slot+1)*stride][:31]+bytes(8)+backing[start+slot*stride:start+(slot+1)*stride][31:])
  if vt:struct.pack_into('<I',raw,35,vt)
  state.update({base+slot*width+i:v for i,v in enumerate(raw)})
state.update({0x118d74+i:v for i,v in enumerate(buffers['players.bin'])})
class Source(m.Model):
 def store(self,a,n,v):
  context={0x11a9bd:(2,int.from_bytes(backing[0x1c5d:0x1c5f],'little')),0x11a9c1:(2,int.from_bytes(backing[0x1c61:0x1c63],'little')),0x11a9c7:(4,int.from_bytes(backing[0x1c67:0x1c6b],'little'))}
  if getattr(self,'pc',0)==0x5ed20 and a in context:n,v=context[a]
  super().store(a,n,v)
model=Source();files=[P/'private/ghidra/countdown-expiry'/f'instructions-000{x}.txt'for x in ['13108','11400','73a50','11720','114b0','13e60','1ea80']]+[P/'private/ghidra/landing-contact/instructions-0001f030.txt',P/'private/ghidra/actor-activation/instructions-00073e00.txt',P/'private/ghidra/initial-creature/instructions-0006d230.txt',P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt']
model.files+=files;model.code.update(m.m.m.Model(files,b'').code);model.code={a:v for a,v in model.code.items()if not 0x11720<=a<=0x1188d};model.code.update(m.m.m.Model([P/'private/ghidra/countdown-expiry/instructions-00011720.txt'],b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x97830]=0x97831
record=buffers['family8.bin'][204:306];rng=struct.unpack('<HBI',buffers['rng.bin']);aa,rr,steps,requests,trace=model.step(record,backing[0x355e:0x3b55e],1227,rng,extra_state=state,base=m.BASE+204)
assert not requests['particle']and not requests['child'];assert 0x114b0 not in trace;assert 0x11720 in trace
result={name:bytearray(data)for name,data in buffers.items()}
for i in range(0x355e):
 value=model.load(0x118d60+i,1)
 if value!=state[0x118d60+i]:result['backing.bin'][i]=value
result['backing.bin'][0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
for start,count,stride,base,width,name,vt in k.LAYOUTS:
 for slot in range(count):
  raw=bytearray(model.load(base+slot*width+i,1)for i in range(width))
  if name:raw[31:39]=buffers[name][slot*width+31:slot*width+39];result[name][slot*width:(slot+1)*width]=raw
  else:result['backing.bin'][start+slot*stride:start+(slot+1)*stride]=raw[:31]+raw[39:]
result['players.bin']=bytes(model.load(0x118d74+i,1)for i in range(7220));result['rng.bin']=struct.pack('<HBI',*rr)
work=P/'private/review/krell-family8-1227'/uuid.uuid4().hex;work.mkdir(parents=True)
for name,data in result.items():
 (work/name).write_bytes(data);actual=(after/name).read_bytes();assert bytes(data)==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(data,actual))if a!=b][:10])
r=dict(status='passed',literal_instructions=steps,compared_bytes=sum(map(len,result.values())),site_count=backing[0x34b8],timer_before=struct.unpack_from('<h',record,100)[0],timer_after=struct.unpack_from('<h',aa,100)[0],requests=requests,private_run=str(work.relative_to(P)),source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in [Path(__file__),P/'rust/crates/genewars-runtime/src/family8_motion.rs',*before.glob('*.bin'),*after.glob('*.bin')]},scope='Exact1227slot2 fullworld callback;118E0 positive timer230→229 reaches11720 without114B0/site search. Audio/UI queue remain explicit existing ports. No mission acceptance.')
(P/'research/variants/krell-family8-1227-review.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:r[k]for k in ['status','literal_instructions','compared_bytes','timer_before','timer_after']}))

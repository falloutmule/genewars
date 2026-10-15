"""Literal full effect20 airborne callbacks, registry relocation and retirement."""
from pathlib import Path
import sys,struct,json,uuid,hashlib,importlib.util
P=Path(__file__).resolve().parents[2];root=P/(sys.argv[1]if len(sys.argv)>1 else 'private/review/effect20-lifecycle-001')
sp=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(sp);sp.loader.exec_module(m)
class Source(m.Source):
 def store(self,a,n,value):
  if a==0x11a9c7:value=self.water
  return super().store(a,n,value)
v=Source()
files=[P/'private/ghidra'/f for f in ['plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt','actor-activation/instructions-0004b090.txt','landing-contact/instructions-0001f030.txt','terrain-map-refs/instructions-00049d30.txt','panel-activation-refs/instructions-000724c0.txt']];v.code.update(m.m.Model(files,b'').code)
for address in [0x1e800,0x409f0,0x2c200]:v.code[address]=('RET',[])
order=sorted(v.code);v.next=dict(zip(order,order[1:]));effpath=P/'private/integration/game-runtime/effects/EFF.ANB';eff=bytearray(effpath.read_bytes());f,a=struct.unpack_from('<II',eff,10)
for i in range(struct.unpack_from('<H',eff)[0]):
 for off in [0,4]:struct.pack_into('<I',eff,a+10*i+off,0x610000+f+struct.unpack_from('<I',eff,a+10*i+off)[0])
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
OVER={1:'creatures.bin',3:'actors.bin',4:'buildings.bin',5:'effects.bin'};w=P/'private/review/effect20-world'/uuid.uuid4().hex;w.mkdir(parents=True);cases=[];total=0;digest=hashlib.sha256()
for fixture in sorted(root.glob('*-before')):
 frame=int(fixture.name.split('-')[1]);slot=int(fixture.name.split('-')[3]);after=root/(fixture.name[:-6]+'after');
 if not after.exists()and len(sys.argv)>2:continue
 bs={p.name:p.read_bytes()for p in fixture.glob('*.bin')};expected={p.name:p.read_bytes()for p in after.glob('*.bin')};back=bs['backing.bin'];state={0x118d60+i:b for i,b in enumerate(back[:0x3b55e])};state.update({0x118d74+i:b for i,b in enumerate(bs['players.bin'])})
 for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
  for i in range(count):
   raw=back[start+i*stride:start+(i+1)*stride];native=bytearray(bs[OVER[kind]][i*width:(i+1)*width])if kind in OVER else bytearray(raw[:31]+bytes(8)+raw[31:]);vt={1:0x111fec,2:0x1120ac,3:0x112194,4:0x112120,5:0x112254,8:0x11242c}.get(kind)
   if vt:struct.pack_into('<I',native,35,vt)
   if kind==5:struct.pack_into('<I',native,31,0x610000+a+native[69]*10)
   state.update({address+i*width+j:b for j,b in enumerate(native)})
 state.update({0x610000+i:b for i,b in enumerate(eff)});state.update({0x185ec4+i:0 for i in range(4)});seed,ring,draws=struct.unpack('<HBI',bs['rng.bin']);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
 v.water=struct.unpack_from('<i',back,0x1c67)[0]
 r,_,trace=v.run(counter=frame,seed=seed,state_before=state,entry=0x4b630,registers={'EAX':0x1688ae+slot*81},instruction_budget=100000)
 actual={'map.bin':bytes(v.load(0x11c2be+i,1)for i in range(229376)),'players.bin':bytes(v.load(0x118d74+i,1)for i in range(7220)),'rng.bin':struct.pack('<HBI',v.load(0x11878a,2),v.load(0x11872c,1),v.load(0x118782,4))};backout=bytearray(back);backout[:0x355e]=bytes(v.load(0x118d60+i,1)for i in range(0x355e));backout[0x355e:0x3b55e]=actual['map.bin']
 # Native player overlays are not written into backing by the owned engine.
 backout[20:20+7220]=back[20:20+7220]
 for kind,(start,count,stride,address,width)in enumerate(LAYOUTS,1):
  native=bytearray(v.load(address+i,1)for i in range(count*width))
  if kind in OVER:
   name=OVER[kind]
   for i in range(count):native[i*width+31:i*width+39]=expected[name][i*width+31:i*width+39]
   actual[name]=native
  else:
   for i in range(count):backout[start+i*stride:start+(i+1)*stride]=native[i*width:i*width+31]+native[i*width+39:(i+1)*width]
 actual['backing.bin']=backout;delta={name:[(i,x,y)for i,(x,y)in enumerate(zip(data,expected[name]))if x!=y][:12]for name,data in actual.items()};assert not any(delta.values()),(fixture.name,delta)
 for name,data in actual.items():total+=len(data);digest.update(data)
 cases.append(dict(frame=frame,slot=slot,instructions=r['steps'],draws=r['draws'],retired=0x49d30 in v.calls))
required=int(sys.argv[2])if len(sys.argv)>2 else 63
assert len(cases)==required,(len(cases),'incomplete callback fixture')
if required==63:assert sum(c['retired']for c in cases)==7
assert all(c['draws']==0 for c in cases)
receipt=dict(status='passed',retirements=sum(c['retired']for c in cases),cases=cases,compared_bytes=total,output_sha256=digest.hexdigest(),private_run=str(w.relative_to(P)),fixture_root=str(root.relative_to(P)),fixture_sha256={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest()for p in root.glob('*/*.bin')},pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in files+v.files+[effpath,Path(__file__),P/'rust/crates/genewars-runtime/src/effect20_motion.rs',P/'rust/crates/genewars-runtime/src/actor_world/effect20_updates.rs',P/'rust/crates/genewars-runtime/src/landing_effect_lifecycle.rs',P/'rust/crates/genewars-runtime/src/effect_pool.rs']},scope='Full literal6DF30+4B630 with real collision queries,8ACC0/8AC60 relinking and49D30/724C0 retirement; all captured worldbuffers compared. Only UI1E800/409F0/2C200 explicitports, opaque31:39 normalized. Synthetic terminal fixture, no missionacceptance.')
(P/'research/variants/effect20-world-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(cases),compared_bytes=total)))

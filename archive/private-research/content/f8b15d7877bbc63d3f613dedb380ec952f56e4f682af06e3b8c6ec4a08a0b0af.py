"""Literal-source investigation of the captured Distributor task stall."""
import json,struct,hashlib,importlib.util
from pathlib import Path
P=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 capture=P/'private/review/distributor-stall/0818cb7f7b104aca993a19e9f6eaf366'
 spec=importlib.util.spec_from_file_location('original_model',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);
 class Source(m.Source):
  def get(self,o):
   if o in ['0x000981d3','0x000a6390','0x00084e10']:return 0x2e0000
   if o=='0x000419e0':return 0x2e0008
   return super().get(o)
 model=Source()
 files=[P/'private/ghidra'/p for p in ['terrain-map-refs/instructions-00042b90.txt','terrain-map-refs/instructions-00074e80.txt','terrain-map-refs/instructions-0008c860.txt','construction-command/instructions-000751c0.txt','terrain-map-refs/instructions-000753f0.txt','terrain-map-refs/instructions-0008cba0.txt','terrain-map-refs/instructions-0008b890.txt','terrain-map-refs/instructions-0008d1b0.txt','validated/instructions-0008d580.txt','building-cost-refs/instructions-000534d0.txt','actor-motion/instructions-0001ebb0.txt','actor-motion/instructions-0006f760.txt','actor-motion/instructions-00057a10.txt','actor-motion/instructions-00051930.txt','construction-command/instructions-000526c0.txt','activation-rng-refs/instructions-00052340.txt','construction-setup/instructions-000507d0.txt','actor-motion/instructions-0001ee30.txt','construction-command/instructions-0001ee40.txt']]
 model.code.update(m.m.Model(files,b'').code);model.code[0x585ea]=('CALL',['0x2e0000']);model.code[0x2e0000]=('RET',[]);model.code[0x2e0008]=('RET',['0x10']);order=sorted(model.code);model.next=dict(zip(order,order[1:]));cases=[]
 layouts=[('creatures',0x1542be,124,300,0x111fec),('actors',0x16360e,200,20,0x112194),('buildings',0x1645ae,67,256,0x112120),('effects',0x1688ae,81,200,0x112254),('family8',0x16da36,102,16,0x11242c)]
 for tick in ['1489-assigned','1490','1491','1510']:
  folder=capture/tick;actor=(folder/'actor0.bin').read_bytes();state={0x11c2be+i:b for i,b in enumerate((folder/'map.bin').read_bytes())}
  for name,address,width,count,vt in layouts:
   raw=(folder/f'{name}.bin').read_bytes()
   for slot in range(count):
    r=bytearray(raw[slot*width:(slot+1)*width]);struct.pack_into('<I',r,35,vt)
    if name=='actors':struct.pack_into('<I',r,192,0x2d0000+slot*36)
    state.update({address+slot*width+i:b for i,b in enumerate(r)})
  backing=(folder/'backing.bin').read_bytes();state.update({0x118d60+i:b for i,b in enumerate(backing[:0x355e])})
  jobs=(folder/'jobs.bin').read_bytes();state[0x11bd57]=jobs[0];state.update({0x11ac27+i:b for i,b in enumerate(jobs[1:])})
  patterns=(P/'private/integration/game-runtime/construction/terrain-patterns.bin').read_bytes();table=bytearray([151,0]*4096);assigned=set()
  for tile in range(160):
   a,b,c,d=[v&7 for v in patterns[tile*4:tile*4+4]]
   for corners,o in [((a,b,c,d),0),((b,a,d,c),16),((c,d,a,b),48),((d,c,b,a),32),((b,c,d,a),96),((c,b,a,d),112),((d,a,b,c),80),((a,d,c,b),64)]:
    key=sum(v<<(9-i*3)for i,v in enumerate(corners))
    if key not in assigned:table[key*2:key*2+2]=bytes([tile,o]);assigned.add(key)
  assert hashlib.sha256(table).hexdigest()=='422f124eac806197ba95d2438516284c3bf828a1db8e8aabe330661fff23ef8a'
  state.update({0x112d24+i:b for i,b in enumerate(table)})
  for slot in range(512):
   raw=backing[277838+slot*41:277838+(slot+1)*41];r=bytearray(raw[:31]+bytes(8)+raw[31:]);struct.pack_into('<I',r,35,0x1120ac);state.update({0x15d40e+slot*49+i:b for i,b in enumerate(r)})
  players=(folder/'players.bin').read_bytes();state.update({0x118d74+i:b for i,b in enumerate(players)})
  # Actor0 is initial architect blueprint0; make that explicit in native pointer binding.
  state.update({0x2d0000+i:b for i,b in enumerate(players[0x16:0x16+36])})
  seed,ring,draws=struct.unpack('<HBI',(folder/'rng.bin').read_bytes());state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
  anb=bytearray((P/'private/integration/game-runtime/actors/key-08.anb').read_bytes());count,_,_,frame_at,animation_at=struct.unpack_from('<H4I',anb)
  for a in range(count):
   for offset in [0,4]:struct.pack_into('<I',anb,animation_at+a*10+offset,0x600000+frame_at+struct.unpack_from('<I',anb,animation_at+a*10+offset)[0])
  state.update({0x600000+i:b for i,b in enumerate(anb)});state.update({0x16360e+31+i:b for i,b in enumerate(struct.pack('<I',0x600000+animation_at))})
  for entry in ([0x57a10]if tick=='1489-assigned'else[0x526c0,0x6fb60,0x57a10]):
   report,_,trace=model.run(entry=entry,registers={'EAX':0x16360e},counter=int(tick.split('-')[0])+1,seed=seed,state_before=state)
   after=bytearray(model.load(0x16360e+i,1)for i in range(200));after[31:39]=actor[31:39];after[192:196]=actor[192:196]
   if entry==0x57a10 and tick!='1510':
    next_actor=(capture/str(int(tick.split('-')[0])+1)/'actor0.bin').read_bytes();assert after==next_actor,(tick,[(i,a,b)for i,(a,b)in enumerate(zip(after,next_actor))if a!=b])
   path=folder/f'source-{entry:x}-after.bin';path.write_bytes(after);trace_path=folder/f'source-{entry:x}-trace.json';trace_path.write_text(json.dumps([hex(pc)for pc in trace]));
   cases.append(dict(tick=tick,entry=hex(entry),steps=report['steps'],speed_before=struct.unpack_from('<h',actor,0x3f)[0],speed_after=struct.unpack_from('<h',after,0x3f)[0],position_before=struct.unpack_from('<hh',actor,7),position_after=struct.unpack_from('<hh',after,7),changed_offsets=[i for i,(a,b)in enumerate(zip(actor,after))if a!=b],trace_sha256=sha(trace_path),actor_after_sha256=sha(path),actual_next_actor_compared=entry==0x57a10 and tick!='1510',calls={hex(pc):trace.count(pc)for pc in [0x507d0,0x52340,0x526c0,0x6fb60,0x57a10]}))
 print(json.dumps([{k:v for k,v in c.items()if k!='trace'}for c in cases],indent=2))
 (P/'research/integration/distributor-stall-source.json').write_text(json.dumps(dict(status='passed-captured-callback-comparison',classification='upstream-command-path-fidelity-gap',recommendation='Do not inject target speed: original arrival and architect setup preserve the captured zero-speed behavior. Recover original placement dispatch before changing production.',cases=cases,listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']},capture=str(capture.relative_to(P)),scope='Literal57A10 including507D0 setup and6FB60 physics; queue/audio/UI ports explicit. Captured1489-assigned,1490,1491 actor200B afterimages match next native tick;1510 stationary behavior reproduced.'),indent=2)+'\n')
if __name__=='__main__':main()

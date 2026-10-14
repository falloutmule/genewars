"""Literal-source comparison of captured blocked Photon placement."""
import json,struct,hashlib,importlib.util,uuid,argparse
from pathlib import Path
P=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 parser=argparse.ArgumentParser();parser.add_argument('--expected',type=Path);args=parser.parse_args()
 capture=P/'private/review/blocked-photon-site/2ced9c6f6b4a4484bc21d23a523bb133'
 expected_folder=args.expected or capture/'2524'
 work=P/'private/review/blocked-photon-source'/uuid.uuid4().hex;work.mkdir(parents=True)
 post=P/'private/review/blocked-photon-site/30ec25edea1d412c8fbd6f9ad90c0af6/2523-postmotion'
 spec=importlib.util.spec_from_file_location('original_model',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);
 class Source(m.Source):
  def get(self,o):
   if o in ['0x000981d3','0x000a6390','0x00084e10','0x000273f0']:return 0x2e0000
   if o=='0x000419e0':return 0x2e0008
   value=super().get(o)
   if self.pc==0x57a3e:
    assert value==0x6fb60,hex(value)
    raw=bytearray((post/'actor0.bin').read_bytes())
    for start,end in [(31,39),(192,196)]:
     raw[start:end]=bytes(self.load(0x16360e+i,1)for i in range(start,end))
    for i,b in enumerate(raw):self.store(0x16360e+i,1,b)
    for i,b in enumerate((post/'map.bin').read_bytes()):self.store(0x11c2be+i,1,b)
    seed,ring,draws=struct.unpack('<HBI',(post/'rng.bin').read_bytes())
    self.store(0x11878a,2,seed);self.store(0x11872c,1,ring);self.store(0x118782,4,draws)
    self.r['EAX']=1
    return 0x2e0000
   return value
 model=Source()
 files=[P/'private/ghidra'/p for p in ['terrain-map-refs/instructions-00042b90.txt','terrain-map-refs/instructions-00074e80.txt','terrain-map-refs/instructions-0008c860.txt','construction-command/instructions-000751c0.txt','terrain-map-refs/instructions-000753f0.txt','terrain-map-refs/instructions-0008cba0.txt','terrain-map-refs/instructions-0008b890.txt','terrain-map-refs/instructions-0008d1b0.txt','validated/instructions-0008d580.txt','building-cost-refs/instructions-000534d0.txt','actor-motion/instructions-0001ebb0.txt','actor-motion/instructions-0006f760.txt','actor-motion/instructions-00057a10.txt','actor-motion/instructions-00051930.txt','construction-command/instructions-000526c0.txt','activation-rng-refs/instructions-00052340.txt','construction-setup/instructions-000507d0.txt','actor-motion/instructions-0001ee30.txt','construction-command/instructions-0001ee40.txt','construction-command/instructions-0001ee50.txt','building-loader-refs/instructions-00051e80.txt','validated/instructions-0008d5f0.txt']]
 model.code.update(m.m.Model(files,b'').code);model.code[0x585ea]=('CALL',['0x2e0000']);model.code[0x2e0000]=('RET',[]);model.code[0x2e0008]=('RET',['0x10']);order=sorted(model.code);model.next=dict(zip(order,order[1:]));cases=[]
 layouts=[('creatures',0x1542be,124,300,0x111fec),('actors',0x16360e,200,20,0x112194),('buildings',0x1645ae,67,256,0x112120),('effects',0x1688ae,81,200,0x112254),('family8',0x16da36,102,16,0x11242c)]
 for tick in ['2523']:
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
  for entry in [0x57a10]:
   report,_,trace=model.run(entry=entry,registers={'EAX':0x16360e},counter=int(tick.split('-')[0]),seed=seed,state_before=state)
   after=bytearray(model.load(0x16360e+i,1)for i in range(200));after[31:39]=actor[31:39];after[192:196]=actor[192:196]
   if entry==0x57a10 and tick!='1510':
    next_actor=(expected_folder/'actor0.bin').read_bytes();diff=[(i,a,b)for i,(a,b)in enumerate(zip(after,next_actor))if a!=b];print('ACTOR_DIFF',diff)
   comparisons={}
   values={'actor0':bytes(after),'map':bytes(model.load(0x11c2be+i,1)for i in range(229376)), 'rng':struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4)), 'blueprint0':bytes(model.load(0x2d0000+i,1)for i in range(36))}
   for name,value in values.items():
    expected=(expected_folder/f'{name}.bin').read_bytes()if name!='blueprint0'else(expected_folder/'players.bin').read_bytes()[22:58]
    differences=[{'offset':i,'source':a,'native':b}for i,(a,b)in enumerate(zip(value,expected))if a!=b]
    comparisons[name]={'bytes':len(value),'difference_count':len(differences),'first_differences':differences[:20]}
    (work/f'{name}-source.bin').write_bytes(value)
   path=work/f'source-{entry:x}-after.bin';path.write_bytes(after);trace_path=work/f'source-{entry:x}-trace.json';trace_path.write_text(json.dumps([hex(pc)for pc in trace]));
   cases.append(dict(comparisons=comparisons,tick=tick,entry=hex(entry),steps=report['steps'],speed_before=struct.unpack_from('<h',actor,0x3f)[0],speed_after=struct.unpack_from('<h',after,0x3f)[0],position_before=struct.unpack_from('<hh',actor,7),position_after=struct.unpack_from('<hh',after,7),changed_offsets=[i for i,(a,b)in enumerate(zip(actor,after))if a!=b],trace_sha256=sha(trace_path),actor_after_sha256=sha(path),actual_next_actor_compared=entry==0x57a10 and tick!='1510',calls={hex(pc):trace.count(pc)for pc in [0x507d0,0x52340,0x526c0,0x6fb60,0x57a10]}))
 print(json.dumps([c['comparisons']for c in cases],indent=2))
 result=dict(status='passed'if all(v['difference_count']==0 for c in cases for v in c['comparisons'].values())else'error',classification='captured-callback-comparison',expected=str(expected_folder),cases=cases,listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']},capture_pins={str(f.relative_to(P)):sha(f)for folder in [capture/'2523',post,expected_folder]for f in folder.glob('*.bin')},harness_sha256=sha(Path(__file__)),capture=str(capture.relative_to(P)),motion_port=str(post.relative_to(P)),scope='Literal57A10 wrapper,507D0 site query and51E80 cancellation, idle/tail. Virtual94 movement at57A3E is replaced by captured post-common-motion actor/map/RNG; queue/audio/UI are explicit no-state ports. Whole frame next map and RNG may include other callbacks; differences are retained without attribution.')
 (work/'result.json').write_text(json.dumps(result,indent=2)+'\n')
 print('RESULT',work)
if __name__=='__main__':main()

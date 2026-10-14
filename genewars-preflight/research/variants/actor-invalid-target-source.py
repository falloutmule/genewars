"""Literal6FEAB..6FED3 target repair on an unmodified captured world."""
from pathlib import Path
import argparse,struct,json,hashlib,importlib.util,uuid
P=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 parser=argparse.ArgumentParser();parser.add_argument('capture',type=Path);parser.add_argument('--actor',type=int,default=0);parser.add_argument('--stack-fill',type=int,default=0);parser.add_argument('--route',action='store_true');parser.add_argument('--expected-route',type=Path);args=parser.parse_args();D=args.capture
 W=P/'private/review/actor-invalid-target-source'/uuid.uuid4().hex;W.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 class Model(m.Source):
  def get(self,o):
   if self.pc==0x74ba1 and o=='0x00074270':
    self.queries.append({'point':[m.signed(self.load(self.r['EAX']+i,2),16)for i in [0,2]],'mask':self.r['EDX'],'actor':self.r['EDI']})
   if self.pc==0x70601:self.route_working=bytes(self.load(self.r['EBP']-0x24+i,1)for i in range(4))
   return super().get(o)
 model=Model();model.queries=[]
 files=[P/'private/ghidra/terrain-map-refs/instructions-00074a50.txt',P/'private/ghidra/actor-motion/instructions-0006f760.txt',P/'private/ghidra/actor-motion/instructions-0001ebb0.txt']
 model.code.update(m.m.Model(files,b'').code);model.code[0x70601 if args.route else 0x6fed6]=('JMP',['0x0006fd78'])if args.route else('RET',[]);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 backing=(D/'backing.bin').read_bytes();state={0x118d60+i:b for i,b in enumerate(backing[:0x355e])}
 state.update({0x11c2be+i:b for i,b in enumerate((D/'map.bin').read_bytes())})
 layouts=[('actors',20,200,0x16360e,0x112194),('creatures',300,124,0x1542be,0x111fec),('buildings',256,67,0x1645ae,0x112120),('effects',200,81,0x1688ae,0x112254),('family8',16,102,0x16da36,0x11242c)]
 for name,count,width,address,vt in layouts:
  raw=(D/(name+'.bin')).read_bytes()
  for slot in range(count):
   native=bytearray(raw[slot*width:(slot+1)*width]);struct.pack_into('<I',native,35,vt)
   if name=='actors':struct.pack_into('<I',native,192,0x118d74+native[24]*1805+0x16+native[0x74]*36)
   state.update({address+slot*width+i:b for i,b in enumerate(native)})
 for slot in range(512):
  raw=backing[277838+slot*41:277838+(slot+1)*41];native=bytearray(raw[:31]+bytes(8)+raw[31:]);struct.pack_into('<I',native,35,0x1120ac);state.update({0x15d40e+slot*49+i:b for i,b in enumerate(native)})
 state.update({0x118d74+i:b for i,b in enumerate((D/'players.bin').read_bytes())})
 seed,ring,draws=struct.unpack('<HBI',(D/'rng.bin').read_bytes());state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
 base=0x16360e+args.actor*200;before=bytes(state[base+i]for i in range(200));nominated=0x1645ae+struct.unpack_from('<h',before,0x4d)[0]*67 if before[0x4f]==4 else 0
 assert before[0x4f]in [0,4]
 report,_,trace=model.run(counter=2636,seed=seed,state_before=state,entry=0x6fb60 if args.route else 0x6feab,stack_fill=args.stack_fill,registers={'EAX':base,'ESI':base,'EDI':nominated})
 after=bytes(model.load(base+i,1)for i in range(200));changes=[i for i,(a,b)in enumerate(zip(before,after))if a!=b]
 if not args.route:assert all(i in range(11,15)for i in changes)
 for name,count,width,address,vt in layouts:
  for at in range(address,address+count*width):
   if not base<=at<base+200:assert model.load(at,1)==state[at],hex(at)
 for address,size in [(0x11c2be,229376),(0x118d74,7220),(0x15d40e,512*49)]:
  for at in range(address,address+size):assert model.load(at,1)==state[at],hex(at)
 assert report['draws']==0 and report['rng_seed_after']==seed and report['rng_ring_after']==ring
 if args.route:(W/'route-target.bin').write_bytes(model.route_working)
 (W/'actor-before.bin').write_bytes(before);(W/'actor-after.bin').write_bytes(after)
 result=dict(status='passed-source-execution',capture=str(D),actor=args.actor,route=args.route,stack_fill=args.stack_fill,current=struct.unpack_from('<hh',before,7),requested=struct.unpack_from('<hh',before,11),resolved=struct.unpack_from('<hh',after,11),changed_offsets=changes,queries=model.queries,report=report,nominated_pointer=hex(nominated),production_comparison='not run',scope='Literal6FEAB through6FED3 including74D90/74A50 environment; stops before ray/physics; source changes limited toactor+B; otherboundworld and RNG unchanged',capture_sha256={f.name:sha(f)for f in D.glob('*.bin')},source_pins={str(f.relative_to(P)):sha(f)for f in model.files+files+[Path(__file__),P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']})
 if args.route:
  result['scope']='Literal6FB60 route prefix through virtualB4 steering, stopped before70603 physics; no gameplay ports; otherboundworld and RNG unchanged'
 if args.expected_route:
  assert args.route
  native=bytearray((args.expected_route/'route-actor.bin').read_bytes());native[35:39]=after[35:39];native[192:196]=after[192:196]
  actor_diff=[(i,a,b)for i,(a,b)in enumerate(zip(after,native))if a!=b]
  target=(args.expected_route/'route-target.bin').read_bytes();target_matches=target==model.route_working
  result['production_comparison']={'actor_bytes':200,'actor_differences':actor_diff,'target_bytes':4,'target_matches':target_matches,'normalization':'Source vtable and blueprint pointers only, offsets35..39 and192..196','expected':str(args.expected_route)}
  result['status']='passed'if not actor_diff and target_matches else'error'
 (W/'result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:result[k]for k in ['status','current','requested','resolved','changed_offsets','queries','report']},indent=2));print('RESULT',W)
if __name__=='__main__':main()

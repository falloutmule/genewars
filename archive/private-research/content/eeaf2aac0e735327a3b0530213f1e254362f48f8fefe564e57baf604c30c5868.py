"""Literal projectile42 free-slot factory and postphysics callback fixtures.

No game is executed. Physics, child20 creation and final unlink are explicit
composition ports; immediate child20 damage is deliberately outside this file.
"""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,uuid
P=Path.cwd()
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
BASE=0x1688ae
class Source(m.Source):
 def get(self,o):
  if self.pc==0x48e38:
   self.audio=dict(channel=self.r['EDX'],mode=self.r['EBX'],parameter=self.r['ECX'],stack=[self.load(self.r['ESP']+i*4,4)for i in range(2)])
  if self.pc==0x4b3f2:
   self.child=dict(code=self.r['EAX'],position=self.r['EDX'],height=m.signed(self.r['EBX']),angles=[self.load(self.r['ECX']+i*2,2)for i in range(3)],arguments=[self.load(self.r['ESP']+i*4,4)for i in range(4)])
  if self.pc in [0x4b7be,0x49d30]:
   self.retired=True
  if self.pc==0x4b9ac:
   self.near_query=True
  return super().get(o)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 w=P/'private/review/turret-projectile'/uuid.uuid4().hex;w.mkdir(parents=True)
 files=[P/'private/ghidra'/x for x in ['mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt','plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt','construction-command/instructions-0004f9f0.txt']]
 model=Source();model.files+=files;model.code.update(m.m.Model(files,b'').code)
 model.code[0x48e38]=('CALL',['0x2e0000']);model.code[0x2e0000]=('RET',['0x8'])
 object1=P/'private/ghidra/validated/memory-.object1.bin';object3=P/'private/ghidra/validated/memory-.object3.bin';obj=object3.read_bytes();code=object1.read_bytes()
 assert obj[0x4eb4+42*6:0x4eb4+43*6]==bytes([4,0,9,20,1,1])
 assert struct.unpack_from('<I',code,0x38674+42*4)[0]==0x48e1f
 resource=P/'private/integration/game-runtime/effects/EFF.ANB';db=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',db,10)
 assert struct.unpack_from('<IIBB',db,animations+42*10)==(2996,2996,1,0);assert db[frames+2996+10]==0
 for i in range(struct.unpack_from('<H',db)[0]):
  for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x610000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
 flat=struct.pack('<HhBBBBBhhB',8192,3136,0,7,0,0,0,0,0,0)
 state0={0x610000+i:b for i,b in enumerate(db)};state0.update({0x11c2be+c*14+i:b for c in range(16384)for i,b in enumerate(flat)});state0.update({0x116818+i:b for i,b in enumerate(struct.pack('<I',0x610000+animations))})
 rand=random.Random(0x487902a);inputs=bytearray();expected=bytearray();cases=[];steps=0
 def order():
  keys=sorted(model.code);model.next=dict(zip(keys,keys[1:]))
 order()
 for case in range(64):
  slot=case%8;point=[20864,13696];height=6272+6400+case*31;owner=case%4;angles=[case*61%2049,case*97%2049,6400];prior=bytearray(rand.randbytes(81));prior[6]=0;native=bytearray(prior);struct.pack_into('<I',native,35,0x112254)
  shooter=bytearray(67);shooter[6]=4;struct.pack_into('<h',shooter,15,case*4+owner)
  state=state0.copy();state.update({BASE+i*81+6:int(i<slot)*5 for i in range(200)});state.update({BASE+slot*81+i:b for i,b in enumerate(native)});state.update({0x1645ae+i:b for i,b in enumerate(shooter)})
  for address,data in [(0x220000,struct.pack('<3H',*angles)),(0x11c2b4,struct.pack('<h',slot)),(0x300004,struct.pack('<4I',128,owner,0,0x1645ae)),(0x118782,struct.pack('<I',case))]:state.update({address+i:b for i,b in enumerate(data)})
  state[0x11872c]=case%32;model.audio=None
  report,_,_=model.run(entry=0x48790,seed=5428,state_before=state,registers=dict(EAX=42,EDX=point[0]|point[1]<<16,EBX=height,ECX=0x220000))
  after=bytearray(model.load(BASE+slot*81+i,1)for i in range(81));after[31:35]=bytes(4);after[35:39]=prior[35:39]
  assert report['draws']==0;assert model.audio==dict(channel=33,mode=2,parameter=0,stack=[127,100]),model.audio
  inputs+=prior+struct.pack('<HhhiiB3Hh',slot,*point,6272,height,owner,*angles,case*4+owner);expected+=after;steps+=report['steps'];cases.append(dict(case=case,instructions=report['steps']))
 (w/'factory-input.bin').write_bytes(inputs);(w/'factory-expected.bin').write_bytes(expected)
 # The following matrix executes the entire4B630 lifetime prefix, deliberately
 # returning at physics and observing a NULL child-allocation port.
 model.code[0x6df30]=('RET',[]);model.code[0x4b3f2]=('CALL',['0x2e0010']);model.code[0x2e0010]=('XOR',['EAX','EAX']);model.code[0x2e0011]=('RET',['0x10']);model.code[0x4b7be]=('CALL',['0x2e0020']);model.code[0x2e0020]=('MOV',['byte ptr [EAX + 0x6]','0x0']);model.code[0x2e0021]=('RET',[]);order();model.next[0x2e0020]=0x2e0021;model.next[0x2e0010]=0x2e0011
 model.code[0x49d30]=('JMP',['0x2e0020']);order();model.next[0x2e0020]=0x2e0021;model.next[0x2e0010]=0x2e0011
 inputs=bytearray();expected=bytearray();updates=[]
 for case in range(128):
  before=bytearray((w/'factory-expected.bin').read_bytes()[:81]);struct.pack_into('<II',before,31,0x610000+animations+420,0x112254);struct.pack_into('<h',before,74,128);struct.pack_into('<i',before,39,7000)
  point=[-32760,32760]if case%2 else[20864,13696];struct.pack_into('<hh',before,7,*point)
  state=state0.copy();state.update({BASE+i:b for i,b in enumerate(before)});state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',0xffffffff-case))});state[0x11872c]=case%32;model.retired=False;model.child=None;model.near_query=False
  creature=bytearray(124);creature[6]=1;creature[24]=4;creature[122]=4;creature[118]=30;creature[94]=[8,23,24,25][case//8%4];struct.pack_into('<I',creature,35,0x111fec)
  dx=[0,1,63,64,65,127,256,257][case%8];dy=[0,63,64,65][case//32%4];at=[m.signed(point[0]+dx,16),m.signed(point[1]+dy,16)];struct.pack_into('<hh',creature,7,*at);struct.pack_into('<i',creature,39,[7000,7008,7512,7520][case//16%4]);state.update({0x1542be+i:b for i,b in enumerate(creature)})
  c=((at[1]>>8)&127)*128+((at[0]>>8)&127);state[0x11c2be+c*14+8]=1
  object_bytes=object1.read_bytes()
  for pc in [0x8d81f,0x8d8a3]:
   assert object_bytes[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100');model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
  report,_,_=model.run(entry=0x4b630,state_before=state,registers={'EAX':BASE})
  after=bytearray(model.load(BASE+i,1)for i in range(81));after[31:39]=before[31:39];after[6]=before[6]
  if model.child:assert model.child==dict(code=20,position=int.from_bytes(before[7:11],'little'),height=int.from_bytes(before[39:43],'little',signed=True),angles=[0,0,0],arguments=[before[76],0,0,0x1645ae]),model.child
  assert model.retired==bool(model.child)
  # Independent one-candidate predicate. Coordinates wrap at the original
  # doubled-i16 delta; vertical physical height is scaled by eight.
  wrapped=lambda x:m.signed(x*2,16)>>1
  distance=wrapped(at[0]-point[0])**2+wrapped(at[1]-point[1])**2+((struct.unpack_from('<i',creature,39)[0]-7000)>>3)**2
  hit=distance<=4096 and creature[94]<24
  assert model.retired==hit,(case,distance,creature[94],model.retired)
  seed=5428
  for _ in range(2):seed=(seed*0xe62d)&65535
  assert (report['draws'],report['rng_seed_after'],report['rng_ring_after'],report['rng_draw_count_after'])==(2,seed,(case+2)%32,(0xffffffff-case+2)&0xffffffff)
  expected_after=bytearray(before);struct.pack_into('<h',expected_after,74,127);expected_after[26]=expected_after[27]=expected_after[30]=0
  assert after==expected_after,(case,[(i,a,b)for i,(a,b)in enumerate(zip(after,expected_after))if a!=b])
  assert bytes(model.load(0x1542be+i,1)for i in range(124))==creature
  inputs+=before+creature;expected+=after+bytes([model.retired,model.near_query]);steps+=report['steps'];updates.append(dict(case=case,retired=model.retired,near_query=model.near_query,draws=report['draws'],rng_seed=report['rng_seed_after'],rng_ring=report['rng_ring_after'],rng_count=report['rng_draw_count_after'],instructions=report['steps']))
 (w/'update-input.bin').write_bytes(inputs);(w/'update-expected.bin').write_bytes(expected)
 paths=model.files+[resource,object1,object3,Path(__file__),P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py']
 receipt=dict(status='passed-neutral-projectile-proximity',factory_cases=cases,update_cases=updates,literal_instructions=steps,pins={p.relative_to(P).as_posix():sha(p)for p in paths},private_run=w.relative_to(P).as_posix(),original_execution=False,gui_execution=False,scope='Actual 4B630 postphysics callback including randomized 8D7B0/8D930 traversal and 4F9F0 selection, independently checked against one-neutral-candidate distance/state predicate, 81-byte effect output, unchanged 124-byte candidate and RNG seed/ring/count. Includes wrapped coordinates and state24/25 exclusions. Physics is RET; child20 is a NULL allocation request; final unlink is a kind-clear port and excluded from numeric output. Evasive candidates, multiple-candidate order, actual child damage and whole-world composition remain unproved.')
 (w/'source.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','literal_instructions','private_run']}))
if __name__=='__main__':main()

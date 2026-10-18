"""Literal projectile42 free-slot factory and postphysics callback fixtures.

No game is executed. Physics, child20 creation and final unlink are explicit
composition ports; immediate child20 damage is deliberately outside this file.
"""
from pathlib import Path
import hashlib,importlib.util,json,random,struct,uuid
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
BASE=0x1688ae
class Source(m.Source):
 def get(self,o):
  if self.pc==0x48e38:
   self.audio=dict(channel=self.r['EDX'],mode=self.r['EBX'],parameter=self.r['ECX'],stack=[self.load(self.r['ESP']+i*4,4)for i in range(2)])
  if self.pc==0x4b3f2:
   self.child=dict(code=self.r['EAX'],position=self.r['EDX'],height=m.signed(self.r['EBX']),angles=[self.load(self.r['ECX']+i*2,2)for i in range(3)],arguments=[self.load(self.r['ESP']+i*4,4)for i in range(4)])
  if self.pc==0x4b7be:
   self.retired=True
  if self.pc==0x4b9ac:
   self.near_query=True
  return super().get(o)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 w=P/'private/review/turret-projectile'/uuid.uuid4().hex;w.mkdir(parents=True)
 files=[P/'private/ghidra'/x for x in ['mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt','plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt']]
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
 model.code[0x6df30]=('RET',[]);model.code[0x4b3f2]=('CALL',['0x2e0010']);model.code[0x2e0010]=('XOR',['EAX','EAX']);model.code[0x2e0011]=('RET',['0x10']);model.code[0x4b7be]=('CALL',['0x2e0020']);model.code[0x2e0020]=('MOV',['byte ptr [EAX + 0x6]','0x0']);model.code[0x2e0021]=('RET',[]);model.code[0x4b9ac]=('JMP',['0x0004b7d5']);order();model.next[0x2e0020]=0x2e0021;model.next[0x2e0010]=0x2e0011
 inputs=bytearray();expected=bytearray();updates=[]
 for case in range(240):
  before=bytearray((w/'factory-expected.bin').read_bytes()[:81]);struct.pack_into('<II',before,31,0x610000+animations+420,0x112254);struct.pack_into('<h',before,74,[-32768,-1,0,1,128][case%5]);before[76]=[0,1,255][case//5%3];before[26]=[0,1,127,128,255][case//15%5];before[27]=[0,255][case//75%2];before[30]=[0,127,128,255][case//30%4];struct.pack_into('<i',before,39,6272 if case%2==0 else 7000)
  state=state0.copy();state.update({BASE+i:b for i,b in enumerate(before)});state.update({0x118782+i:0 for i in range(4)});state[0x11872c]=0;model.retired=False;model.child=None;model.near_query=False
  report,_,_=model.run(entry=0x4b630,state_before=state,registers={'EAX':BASE})
  after=bytearray(model.load(BASE+i,1)for i in range(81));after[31:39]=before[31:39];after[6]=before[6]
  if model.child:assert model.child==dict(code=20,position=int.from_bytes(before[7:11],'little'),height=int.from_bytes(before[39:43],'little',signed=True),angles=[0,0,0],arguments=[before[76],0,0,0x1645ae]),model.child
  assert model.retired==bool(model.child)
  inputs+=before;expected+=after+bytes([model.retired,model.near_query]);steps+=report['steps'];updates.append(dict(case=case,retired=model.retired,near_query=model.near_query,instructions=report['steps']))
 (w/'update-input.bin').write_bytes(inputs);(w/'update-expected.bin').write_bytes(expected)
 paths=model.files+[resource,object1,object3,Path(__file__),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py')]
 receipt=dict(status='literal-projectile42-fixtures',factory_cases=cases,update_cases=updates,literal_instructions=steps,pins={p.relative_to(P).as_posix():sha(p)for p in paths},private_run=w.relative_to(P).as_posix(),original_execution=False,gui_execution=False,scope='Free-slot42 factory full81-byte output; audio21 typed port. 4B630 postphysics animation/lifetime prefix with physics RET, child20 NULL-allocation request, retirement clears kind for source flow then numeric output preserves kind. Live42 near-query4B9AC is explicitly a no-hit port, not implemented or counted as full callback. No movement/collision/damage/whole-world claim.')
 (w/'source.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','literal_instructions','private_run']}))
if __name__=='__main__':main()

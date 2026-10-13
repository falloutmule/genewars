"""Independent literal free-slot factories and postphysics water-effect updates.

Physics is a separate composition boundary in the postphysics matrix. The
actual5859 particle factory arguments are included in the factory fixtures.
"""
from pathlib import Path
import importlib.util,struct,random,json,hashlib
P=Path(__file__).resolve().parents[2];W=P/'private/review/water-effect'
s=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
BASE=0x1688ae
class Source(m.Source):
 def get(self,o):
  if o=='0x00049d30' and getattr(self,'pc',0)in[0x4b7be,0x4b83f]:
   self.retired=True;return 0x2e0000
  if o=='0x00048790' and getattr(self,'defer_child',False) and not self.child:
   self.child=dict(code=self.r['EAX'],point=self.r['EDX'],height=m.signed(self.r['EBX']),angles=[self.load(self.r['ECX']+2*i,2)for i in range(3)],arguments=[self.load(self.r['ESP']+4*i,4)for i in range(4)])
  return super().get(o)
 def load(self,a,n):
  if getattr(self,'pc',0)==0x48790 and getattr(self,'defer_child',False)and not self.child:
   self.child=dict(code=self.r['EAX'],point=self.r['EDX'],height=m.signed(self.r['EBX']),angles=[super(Source,self).load(self.r['ECX']+2*i,2)for i in range(3)],arguments=[super(Source,self).load(self.r['ESP']+4+4*i,4)for i in range(4)])
  return super().load(a,n)

def main():
 W.mkdir(parents=True,exist_ok=True)
 files=[P/'private/ghidra'/x for x in ['mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt','plant-command/instructions-0004b630.txt','mission-array-refs/instructions-0004b3b0.txt','actor-activation/instructions-0004b090.txt','landing-contact/instructions-0001e600.txt','landing-contact/instructions-0001f030.txt']]
 model=Source();model.files+=files;model.code.update(m.m.Model(files,b'').code);model.code[0x2e0000]=('RET',[])
 obj=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes();code=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes()
 assert obj[0x4eb4+21*6:0x4eb4+23*6]==bytes.fromhex('030013160000000014ff0000')
 assert struct.unpack_from('<I',code,0x38674+21*4)[0]==0x49d0a and struct.unpack_from('<I',code,0x3b524+15*4)[0]==0x4b81c
 dbpath=P/'private/integration/game-runtime/effects/EFF.ANB';db=bytearray(dbpath.read_bytes());frames,animations=struct.unpack_from('<II',db,10)
 assert struct.unpack_from('<IIBB',db,animations+21*10)==(1890,1904,2,0)
 assert [db[frames+1890+i*14+10]for i in range(2)]==[0,0]
 metadata={str(i):dict(animation=struct.unpack_from('<IIBB',db,animations+i*10),holds=[db[frames+struct.unpack_from('<I',db,animations+i*10)[0]+j*14+10]for j in range(db[animations+i*10+8])])for i in [21,22]}
 for i in range(struct.unpack_from('<H',db)[0]):
  for off in [0,4]:struct.pack_into('<I',db,animations+i*10+off,0x610000+frames+struct.unpack_from('<I',db,animations+i*10+off)[0])
 flat=struct.pack('<HhBBBBBhhB',8192,3136,0,7,0,0,0,0,0,0)
 state0={0x610000+i:b for i,b in enumerate(db)}
 state0.update({0x11c2be+c*14+i:b for c in range(16384)for i,b in enumerate(flat)})
 state0.update({0x116818+i:b for i,b in enumerate(struct.pack('<I',0x610000+animations))})
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));randoms=random.Random(21)
 factory_input=bytearray();factory_expected=bytearray();factory_cases=[];instructions=0
 for code_value in [21,22]:
  for case in range(32):
   slot=case%8;point=[14367,17372]if case==0 else[20864,13696];height=1073 if case==0 else 1000+case*333;owner=case%4;angles=[0,0,128]if case==0 else[case*61%2048,case*97%2048,case*9]
   prior=bytearray(randoms.randbytes(81));prior[6]=0;native=bytearray(prior);struct.pack_into('<I',native,35,0x112254)
   state=state0.copy();state.update({BASE+i*81+6:int(i<slot)*5 for i in range(200)});state.update({BASE+slot*81+i:b for i,b in enumerate(native)})
   for address,data in [(0x220000,struct.pack('<3H',*angles)),(0x11c2b4,struct.pack('<h',slot)),(0x300004,struct.pack('<4I',0,owner,0,0)),(0x118782,struct.pack('<I',0xffffffff-case))]:state.update({address+i:b for i,b in enumerate(data)})
   state[0x11872c]=case%32
   report,_,trace=model.run(entry=0x48790,seed=5428,state_before=state,registers=dict(EAX=code_value,EDX=point[0]|point[1]<<16,EBX=height,ECX=0x220000))
   a=bytearray(model.load(BASE+slot*81+i,1)for i in range(81));a[31:35]=bytes(4);a[35:39]=prior[35:39]
   factory_input+=prior+struct.pack('<BHhhiiB3H',code_value,slot,*point,6272,height,owner,*angles)
   factory_expected+=a;assert report['draws']==0;instructions+=report['steps'];factory_cases.append(dict(code=code_value,case=case,steps=report['steps']))
 # Fully execute4B630 around a deliberate RET physics boundary. This is
 # independent testing of timer/animation/termination and child parameters.
 model.code[0x6df30]=('RET',[]);model.code[0x48790]=('XOR',['EAX','EAX']);model.code[0x48791]=('RET',['0x10']);order=sorted(model.code);model.next=dict(zip(order,order[1:]));model.next[0x48790]=0x48791;model.defer_child=True
 update_input=bytearray();update_expected=bytearray();update_cases=[]
 for code_value in [21,22]:
  template=bytearray(factory_expected[(0 if code_value==21 else 32)*81:(1 if code_value==21 else 33)*81]);struct.pack_into('<I',template,35,0x112254);struct.pack_into('<I',template,31,0x610000+animations+code_value*10)
  for height in [815,816,817,1632,40000]:
   for speed in [0,64,65,128]:
    for frame in sorted(set([0,1,1 if code_value==21 else 2,2 if code_value==21 else 3,255])):
     for hold in [0,1,127,128,255]:
      a=bytearray(template);struct.pack_into('<i',a,0x27,height);struct.pack_into('<i',a,0x11,816);struct.pack_into('<h',a,0x3d,speed);a[0x1a]=hold;a[0x1b]=frame;a[0x1e]=0
      state=state0.copy();state.update({BASE+i:b for i,b in enumerate(a)});state.update({0x11a9c3+i:b for i,b in enumerate(struct.pack('<h',816))});state.update({0x118782+i:0 for i in range(4)});state[0x11872c]=0
      model.retired=False;model.child=None
      report,_,trace=model.run(entry=0x4b630,seed=5428,state_before=state,registers=dict(EAX=BASE))
      after=bytearray(model.load(BASE+i,1)for i in range(81));after[31:39]=a[31:39]
      update_input+=a+struct.pack('<h',816);update_expected+=after+bytes([model.retired,bool(model.child)])
      if model.child:
       assert model.child==dict(code=22,point=int.from_bytes(a[7:11],'little'),height=height,angles=[0,0,0],arguments=[0,0,a[24],0]),model.child
      instructions+=report['steps'];update_cases.append(dict(code=code_value,height=height,speed=speed,frame=frame,hold=hold,retired=model.retired,child=model.child,steps=report['steps']))
 for name,data in [('factory-input.bin',factory_input),('factory-expected.bin',factory_expected),('update-input.bin',update_input),('update-expected.bin',update_expected)]: (W/name).write_bytes(data)
 paths=model.files+[Path(__file__),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),dbpath,P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']
 report=dict(status='literal-water-effect-fixtures',factory_cases=len(factory_cases),update_cases=len(update_cases),literal_instructions=instructions,metadata=metadata,factories=factory_cases,updates=update_cases,pins={x.relative_to(P).as_posix():hashlib.sha256(x.read_bytes()).hexdigest()for x in paths},scope='Free-slot factories and full4B630 postphysics animation/termination with6DF30 a RET boundary; child48790 captured,49D30 deferred. No owned-world integration claim.',original_execution=False)
 (W/'source.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k]for k in ['status','factory_cases','update_cases','literal_instructions','metadata']}))
if __name__=='__main__':main()

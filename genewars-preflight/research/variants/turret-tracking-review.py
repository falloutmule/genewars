"""Literal451F0 target selection and rotation versus bounded native planner."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,random,struct,subprocess,sys,uuid,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 w=P/'private/review/turret-tracking'/uuid.uuid4().hex;w.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);
 class Projectile(Exception):pass
 class Source(m.Source):
  def get(self,operand):
   if self.pc==0x475d5:raise Projectile()
   return super().get(operand)
 model=Source()
 files=[P/'private/ghidra'/f for f in ['building-update/instructions-000451f0.txt','construction-command/instructions-00048540.txt','construction-command/instructions-0008b000.txt']];model.code.update(m.m.Model(files,b'').code)
 object1=P/'private/ghidra/validated/memory-.object1.bin';object3=P/'private/ghidra/validated/memory-.object3.bin'
 for pc in [0x8d81f,0x8d8a3]:
  assert object1.read_bytes()[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100')
  model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
 for pc in [0x47e21,0x47f34]:model.code[pc]=('CALL',['0x2e0010'])
 model.code[0x2e0010]=('RET',[])
 for pc in [0x473e1,0x47446]:model.code[pc]=('CALL',['0x2e0020'])
 model.code[0x2e0020]=('RET',['0x8']);model.code[0x4747b]=('CALL',['0x2e0010'])
 order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 resource=P/'private/reference-run/run-006/drive-c/GAME/DATA/BUILDING/BTURR.ANB';anb=bytearray(resource.read_bytes());assert sha(resource)=='c26df60af3a3166aec54112f197a4334966eb214eeaa48e0d9284e0510b8cd20'
 frames,animations=struct.unpack_from('<II',anb,10);assert struct.unpack_from('<IIBB',anb,animations+10)==(42,140,8,0)
 assert all(anb[frames+42+14*i+10]==0 for i in range(8))
 for i in range(5):
  for off in [0,4]:struct.pack_into('<I',anb,animations+i*10+off,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+off)[0])
 base_state={0x620000+i:v for i,v in enumerate(anb)};base_state.update({0x11c2be+i:0 for i in range(229376)})
 rand=random.Random(0x451F012);data=bytearray();expected=bytearray();instructions=0;cases=[]
 for case in range(256):
  before=bytearray(rand.randbytes(67));before[6]=4;owner=case%4;before[24]=owner;before[45]=1;before[46]=0;before[49:51]=struct.pack('<H',[1,5][case%2]);before[28]=0
  existing=(case//4)%2;before[64:67]=bytes([0,0,existing]);struct.pack_into('<h',before,15,case%20)
  struct.pack_into('<h',before,61,[3,200,32767,2][(case//8)%4]);before[63]=0
  struct.pack_into('<h',before,52,[0,512,1024,1536,2048,-1,-32768,32767][(case//16)%8]);before[51]=[0,1,255][(case//32)%3]
  center=[0,0]if case%2==0 else[32760,-32760];target=[m.signed(center[0]+1024,16),m.signed(center[1]+512,16)];struct.pack_into('<hh',before,7,*center)
  creature=bytearray(124);creature[6]=1;creature[24]=4;creature[0x5e]=[8,18,25][(case//64)%3];struct.pack_into('<h',creature,0x6c,[0,7][(case//128)%2]);struct.pack_into('<hh',creature,7,*target);struct.pack_into('<I',creature,35,0x111fec)
  player=bytearray(rand.randbytes(1805));player[1]=player[0x4a1]=owner;player[0x6e5]=16 if case%5==0 else 0;struct.pack_into('<H',player,0x5ea,[0,1000][case%2])
  seed=rand.randrange(65536);ring=case%32;draws=[0,1,0xffffffff][case%3];frame=[1,64-case%20][(case//2)%2]
  native=bytearray(before);struct.pack_into('<II',native,31,0x620000+animations,0x112120)
  state=base_state.copy();state.update({0x1645ae+i:v for i,v in enumerate(native)});state.update({0x1542be+i:v for i,v in enumerate(creature)});state.update({0x118d74+owner*1805+i:v for i,v in enumerate(player)});state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))})
  target_cell=(((target[1]>>8)&127)*128+((target[0]>>8)&127))*14;state[0x11c2be+target_cell+8]=1
  query_eligible=creature[0x5e]<24 and player[0x6e5]&16==0
  data+=before+player+struct.pack('<HBIIB3shhBhB',seed,ring,draws,frame,existing,bytes([0,0,1]),*target,creature[0x5e],struct.unpack_from('<h',creature,0x6c)[0],query_eligible)
  try:report,_,trace=model.run(counter=frame,seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae},stack_fill=[0,85,255][case%3])
  except Projectile:
   expected+=bytes([1]);cases.append(dict(case=case,projectile_boundary=True));continue
  after=bytearray(model.load(0x1645ae+i,1)for i in range(67));after[31:39]=before[31:39]
  player_after=bytes(model.load(0x118d74+owner*1805+i,1)for i in range(1805));rng_after=struct.pack('<HBI',model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4));cells=trace.count(0x48540);audio=1 if 0x473e1 in trace or 0x47446 in trace else 2 if 0x4747b in trace else 0
  expected+=bytes([0])+after+player_after+rng_after+struct.pack('<IB',cells,audio);instructions+=report['steps'];cases.append(dict(case=case,search_cells=cells,audio=audio,literal_instructions=report['steps'],draws=report['draws']))
 (w/'input.bin').write_bytes(data);(w/'expected.bin').write_bytes(expected)
 (w/'angles.bin').write_bytes((P/'private/review/mule-mortality/angles.bin').read_bytes());(w/'bounce.bin').write_bytes((P/'private/review/mule-mortality/bounce.bin').read_bytes())
 # Compile a private implementation snapshot so the public module uses its
 # actual transitive angle/RNG dependencies.
 implementation_pins={}
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  origin=P/f'rust/crates/genewars-{crate}/src';target=w/f'source/{crate}';shutil.copytree(origin,target)
  if crate=='runtime'and'--draft'in sys.argv:shutil.copyfile(P/'private/review/turret-tracking/candidate.rs',target/'completed_building.rs')
  for file in target.rglob('*.rs'):implementation_pins[(origin/file.relative_to(target)).relative_to(P).as_posix()]=sha(file)
  command=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(target/'lib.rs'),'-L',f'dependency={w}','-o',str(w/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:command+=['--extern',f'genewars_{dep}={w}/libgenewars_{dep}.rlib']
  built=subprocess.run(command,cwd=P,env=environment(),capture_output=True);assert built.returncode==0,built.stderr.decode(errors='replace')
 command=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={w}','--extern',f'genewars_runtime={w}/libgenewars_runtime.rlib','-o',str(w/'probe.exe')]
 r=subprocess.run(command,cwd=P,env=environment(),capture_output=True);assert r.returncode==0,r.stderr.decode(errors='replace')
 subprocess.run([str(w/'probe.exe'),str(w)],check=True);actual=(w/'actual.bin').read_bytes();assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
 paths=model.files+files+[object1,object3,Path(__file__),Path(__file__).with_suffix('.rs'),Path(__file__).with_name('initial-creature-source.py'),Path(__file__).with_name('actor-environment-terrain-target.py'),P/'rust/crates/genewars-runtime/src/completed_building.rs',P/'rust/crates/genewars-runtime/src/blueprint.rs',resource]
 receipt=dict(status='passed-private-draft-turret-tracking'if'--draft'in sys.argv else'passed-bounded-turret-tracking',utc=datetime.now(timezone.utc).isoformat(),callbacks=256,implementation_pins=implementation_pins,compared_bytes=len(actual),literal_instructions=instructions,cases=cases,pins={p.relative_to(P).as_posix():sha(p)for p in paths},private_run=w.relative_to(P).as_posix(),original_execution=False,gui_execution=False,oracle_annotation='8D81F/8D8A3 raw66A3 prove word seed stores; unsized listing text otherwise incorrectly clears adjacent playercount11878C.',
  scope='256 target-tracking cases over full67Bbuilding,1805Bplayer,RNG,querycount andtyped audio. Existing/nulltargets,periodic andrange searches,alliance exclusion,state18timer filter,corpse query exclusion,signedyaw andtorus seams,cooldown andRNGwrap. Actual8B000/48540/73970/7A6C0 and7AC00 interpreted. Alignedzero-cooldown source475D5 boundaries reject; projectilebody not executed or claimed. Audio/queue bodies remain typed ports.')
 out=(w/'draft-result.json')if'--draft'in sys.argv else Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');tmp.replace(out);print(json.dumps({k:receipt[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

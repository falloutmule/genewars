"""Mixed admitted building pass against literal47F80, with explicit audio state."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,shutil,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
ACTORS=343958;CREATURES=ACTORS+4000;BUILDINGS=CREATURES+37200;PLAYERS=BUILDINGS+17152;RNG=PLAYERS+7220;BINDINGS=RNG+7
LAYOUTS=[(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(command):
 r=subprocess.run(command,cwd=P,env=environment(),capture_output=True,timeout=240)
 assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'))
 return r.stdout.decode(errors='replace')
def main():
 work=P/'private/review/admitted-building-pass'/uuid.uuid4().hex;work.mkdir(parents=True)
 pins={}
 # Freeze the exact implementation compiled here; concurrent production work
 # cannot silently change the meaning of these recorded native captures.
 for crate in ['assets','core','runtime']:
  origin=P/f'rust/crates/genewars-{crate}/src';target=work/f'source/{crate}'
  shutil.copytree(origin,target)
  for f in target.rglob('*.rs'):pins[(origin/f.relative_to(target)).relative_to(P).as_posix()]=sha(f)
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  command=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}','-o',str(work/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:command+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
  run(command)
 command=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(P/'scripts/desktop/admitted_building_pass_harness.rs'),'-L',f'dependency={work}','-o',str(work/'probe.exe')]
 for dep in ['assets','runtime']:command+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
 run(command);print(run([str(work/'probe.exe'),str(P),str(work)]),flush=True)
 spec=importlib.util.spec_from_file_location('initial_source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 class TargetFound(Exception):pass
 class Source(m.Source):
  selected=0;reject_target=False;started=[]
  def get(self,operand):
   if self.pc==0x451f0 and operand=='EBX':
    address=self.r['EAX'];self.started.append(address)
    if self.reject_target and self.load(address+45,1)==3 and self.load(address+49,1)&1:raise TargetFound(address)
   if operand=='byte ptr [0x0011873d]':return self.selected
   return super().get(operand)
 model=Source();files=[P/'private/ghidra'/f for f in ['building-update/instructions-000451f0.txt','landing-contact/instructions-0007ac00.txt','construction-command/instructions-00048540.txt','construction-command/instructions-0008b000.txt','mission-array-refs/instructions-00047f80.txt','mission-array-refs/instructions-000437a0.txt']]
 model.code.update(m.m.Model(files,b'').code)
 object1=P/'private/ghidra/validated/memory-.object1.bin';object3=P/'private/ghidra/validated/memory-.object3.bin'
 for pc in [0x8d81f,0x8d8a3]:
  assert object1.read_bytes()[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100')
  model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
 # Presentation services preserve their source calling convention.451F0,
 # 7AC00,47F80 and437A0 numeric bodies remain literal.
 for address,cleanup in [(0x981d3,0),(0x41ce0,20),(0x41910,16),(0x984c0,0)]:model.code[address]=('RET',[hex(cleanup)]if cleanup else[])
 for address in [0x47e21,0x47e34,0x47f34,0x46bac]:model.code[address]=('CALL',['0x2e0010'])
 model.code[0x2e0010]=('RET',[])
 for pc in [0x46bc3,0x46c9d]:model.code[pc]=('CALL',['0x2e0020'])
 model.code[0x2e0020]=('RET',['0x8'])
 model.code[0x97830]=('MOV',['EAX','0']);model.code[0x97831]=('RET',[])
 order=sorted(model.code);model.next=dict(zip(order,order[1:]))
 resources={};resource_pins={}
 for selector,name in [(1,'BTURR'),(3,'BPOWE'),(6,'BSOLA'),(11,'BMINE')]:
  resource=P/f'private/integration/game-runtime/buildings/{name}.ANB';resource_pins[name]=sha(resource)
  anb=bytearray(resource.read_bytes());frames,animations=struct.unpack_from('<II',anb,10);base=0x620000+selector*0x20000
  for i in range(5):
   for offset in [0,4]:struct.pack_into('<I',anb,animations+i*10+offset,base+frames+struct.unpack_from('<I',anb,animations+i*10+offset)[0])
  resources[selector]=(base,animations,anb)
 cases=[];total=steps=0;stream=hashlib.sha256()
 for number,line in enumerate((work/'cases.tsv').read_text().splitlines()):
  name,count,selected,reject,audio,callbacks=line.split('\t');model.code[0x97830]=('MOV',['EAX',audio]);count=int(count);model.selected=int(selected);reject=int(reject);callbacks=int(callbacks);folder=work/name;model.reject_target=bool(reject);model.started=[]
  before=(folder/'before.bin').read_bytes();actual=(folder/'after.bin').read_bytes();assert len(before)==len(actual)==BINDINGS+256
  state={0x118d60+i:b for i,b in enumerate(before[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])})
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
    if base is None:
     raw=before[start+slot*stride:start+(slot+1)*stride];native=bytearray(raw[:31]+bytes(8)+raw[31:])
    else:native=bytearray(before[base+slot*width:base+(slot+1)*width])
    if kind==1:struct.pack_into('<I',native,35,0x111fec)
    if kind==4:
     struct.pack_into('<I',native,35,0x112120)
     if native[6]!=0:
      base_address,animation_table,_=resources[native[45]];struct.pack_into('<I',native,31,base_address+animation_table)
    state.update({address+slot*width+j:b for j,b in enumerate(native)})
  state.update({0x118d74+j:b for j,b in enumerate(before[PLAYERS:RNG])});state[0x11878c]=count
  for base_address,_,resource_bytes in resources.values():state.update({base_address+j:b for j,b in enumerate(resource_bytes)})
  seed,ring,draws=struct.unpack_from('<HBI',before,RNG);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
  try:report,_,trace=model.run(seed=seed,state_before=state,entry=0x47f80,stack_fill=[0,85,255][number%3])
  except TargetFound as found:
   assert reject and found.args[0]==0x1645ae+(3*64+4)*67 and before==actual and len(model.started)==17
   cases.append(dict(case=name,late_unsupported_completed3_rejected=True));print(name,'PASS',flush=True);continue
  assert not reject
  expected=bytearray(before);expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
  for kind,(start,n,stride,address,width)in enumerate(LAYOUTS,1):
   for slot in range(n):
    native=bytearray(model.load(address+slot*width+j,1)for j in range(width));base={1:CREATURES,3:ACTORS,4:BUILDINGS}.get(kind)
    if base is None:expected[start+slot*stride:start+(slot+1)*stride]=native[:31]+native[39:]
    else:
     native[31:39]=before[base+slot*width+31:base+slot*width+39];expected[base+slot*width:base+(slot+1)*width]=native
  expected[PLAYERS:RNG]=bytes(model.load(0x118d74+i,1)for i in range(7220));struct.pack_into('<HBI',expected,RNG,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
  assert expected==actual,(name,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:16])
  assert trace.count(0x451f0)==callbacks and trace.count(0x437a0)==count,(name,'callback/player counts')
  (folder/'source-after.bin').write_bytes(expected);stream.update(before+actual);total+=len(actual);steps+=report['steps']
  cases.append(dict(case=name,player_count=count,selected=model.selected,callbacks=callbacks,literal_instructions=report['steps']));print(name,'PASS',flush=True)
 receipt=dict(status='passed-admitted-building-pass',utc=datetime.now(timezone.utc).isoformat(),cases=cases,compared_bytes=total,literal_instructions=steps,implementation_pins=pins,
  listing_pins={f.relative_to(P).as_posix():sha(f)for f in model.files+files+[object1,object3]},review_pins={p.relative_to(P).as_posix():sha(p)for p in [Path(__file__),P/'scripts/desktop/admitted_building_pass_harness.rs',P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py']},
  private_run=work.relative_to(P).as_posix(),probe_sha256=sha(work/'probe.exe'),resource_sha256=resource_pins,fixture_stream_sha256=stream.hexdigest(),original_execution=False,gui_execution=False,oracle_annotation='8D81F/8D8A3 raw66A3 prove word seed stores; unsized listing text otherwise incorrectly clears adjacent playercount11878C.',
  scope='Eight mixed whole47F80 passes: completed turret and idle Extractor, incomplete Distributor/Photon; exactresources, allowners, selected0..3, count1/4, explicitfalse/true workaudio. Whole backing/native/player/RNG/bindings compared. Late completedDistributor requires wholepassrollback after16earliercallbacks. Fixture only, notordinarygameplayacceptance; poweredharvest remainsoutsideadmission.')
 out=P/'research/integration/admitted-building-pass.json';tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8');tmp.replace(out)
 print(json.dumps({k:receipt[k]for k in ['status','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

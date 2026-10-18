"""Frozen owned Extractor source comparison and rollback fixtures."""
from pathlib import Path
import hashlib,importlib.util,json,os,shutil,struct,subprocess,sys,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
work=P/'private/review/extractor-owned'/uuid.uuid4().hex;work.mkdir(parents=True);pins={}
for crate in ['assets','core','runtime']:
 origin=P/f'rust/crates/genewars-{crate}/src';target=work/f'source/{crate}';shutil.copytree(origin,target)
 for f in target.rglob('*.rs'):pins[(origin/f.relative_to(target)).relative_to(P).as_posix()]=hashlib.sha256(f.read_bytes()).hexdigest()
f=work/'source/runtime/actor_world/extractor_updates.rs';f.write_text(f.read_text()+(P/'scripts/ghidra/mission_contract_extractor_owned_fixture.rs').read_text())
env=environment();env['CARGO_MANIFEST_DIR']=str(P/'rust/crates/genewars-runtime');env['GENEWARS_REVIEW_ROOT']=str(P);env['GENEWARS_REVIEW_OUT']=str(work);rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
def run(cmd):
 r=subprocess.run([str(v)for v in cmd],cwd=P,env=env,capture_output=True,timeout=240)
 assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'))
 return r.stdout.decode(errors='replace')
for crate in ['assets','core','runtime']:
 cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}']
 if crate=='runtime':
  cmd+=['--test','-o',work/'review.exe']
  for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
 else:cmd+=['--crate-type','rlib','-o',work/f'libgenewars_{crate}.rlib']
 run(cmd)
print(run([work/'review.exe','extractor_owned_review','--nocapture']),flush=True)
print('PRIVATE_RUN',work,flush=True)
(P/'private/review/extractor-owned/latest.txt').write_text(str(work))

r={'status':'passed-bounded-owned-integration','cases':9,'passed_callbacks':[0,1,3,6,7,8],'expected_transaction_rollbacks':{'2':'empty-resource notice unsupported','4':'own unprotected kind1 contact','5':'enemy unprotected kind1 contact after smoke allocation'},'implementation_pins':pins,'private_run':work.relative_to(P).as_posix(),'literal_whole_world_comparison':'not run','source_evidence':['extractor-producing-callback.json','extractor-contact-pass.json'],'remaining':['literal451F0 full world comparison including smoke factory and8CBA0','occupied contact damage callback','empty-resource shutdown'],'original_execution':False,'mission_acceptance':False}
(P/'research/campaign/extractor-owned-integration.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:r[k]for k in ['status','cases','remaining']}))
# Full source callback comparison for successful owned cases. Presentation
# services alone are ports; factory, cell queries and terrain cleanup stay literal.
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);model=m.Source()
files=[P/'private/ghidra'/n for n in ['building-update/instructions-000451f0.txt','mission-array-refs/instructions-00048790.txt','player-creation/instructions-0001f070.txt','mission-tail-init/instructions-00049e90.txt','terrain-map-refs/instructions-0007b240.txt','actor-activation/instructions-0008ac60.txt','terrain-map-refs/instructions-0008cba0.txt','terrain-map-refs/instructions-0008b890.txt','terrain-map-refs/instructions-0008d1b0.txt','construction-command/instructions-0008b000.txt','mission-contract-refs/query-8b080.txt','player-creation/instructions-0001ec50.txt','terrain-camera-refs/instructions-00075240.txt']]
model.code.update(m.m.Model(files,b'').code)
for pc in [0x46bac,0x47e34,0x47f34,0x47098]:model.code[pc]=('CALL',['0x2e0010'])
for pc in [0x46bc3,0x46c9d,0x470af,0x470d9]:model.code[pc]=('CALL',['0x2e0020'])
model.code[0x2e0010]=('RET',[]);model.code[0x2e0020]=('RET',['0x8']);model.code[0x97830]=('XOR',['EAX','EAX']);model.code[0x97831]=('RET',[])
model.code[0x84e10]=('RET',[])
for pc in [0x46d55,0x46dc3,0x46e33,0x48bdf]:model.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
order=sorted(model.code);model.next=dict(zip(order,order[1:]));anb=bytearray((P/'private/integration/game-runtime/buildings/BMINE.ANB').read_bytes());frames,animations=struct.unpack_from('<II',anb,10)
for i in range(5):
 for off in [0,4]:struct.pack_into('<I',anb,animations+i*10+off,0x620000+frames+struct.unpack_from('<I',anb,animations+i*10+off)[0])
B=343958;PL=B+17152;EF=PL+7220;CR=EF+16200;RN=CR+37200;steps=total=0
for case in [0,1,3,6,7,8]:
 before=(work/f'case{case}-before.bin').read_bytes();actual=(work/f'case{case}-after.bin').read_bytes();state={0x118d60+i:b for i,b in enumerate(before[:0x355e])};state.update({0x11c2be+i:b for i,b in enumerate(before[0x355e:0x3b55e])});state.update({0x620000+i:b for i,b in enumerate(anb)})
 for start,n,stride,addr,width in [(243038,300,116,0x1542be,124),(277838,512,41,0x15d40e,49),(298830,20,140,0x16360e,200),(301630,256,59,0x1645ae,67),(316734,200,73,0x1688ae,81),(331334,32,41,0x16c7f6,49),(332646,32,89,0x16ce16,97),(335494,16,94,0x16da36,102)]:
  for slot in range(n):
   base={124:CR,67:B,81:EF}.get(width)
   if base is None:native=bytearray(width)
   else:native=bytearray(before[base+slot*width:base+(slot+1)*width])
   if width==124:struct.pack_into('<I',native,35,0x111fec)
   if width==81:struct.pack_into('<I',native,35,0x112254)
   if width==67:struct.pack_into('<II',native,31,0x620000+animations,0x112120)
   state.update({addr+slot*width+i:b for i,b in enumerate(native)})
 state.update({0x118d74+i:b for i,b in enumerate(before[PL:EF])});seed,ring,draws=struct.unpack_from('<HBI',before,RN);state[0x11872c]=ring;state.update({0x118782+i:b for i,b in enumerate(struct.pack('<I',draws))})
 report,_,_=model.run(counter=1 if case==8 else case%2,seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae})
 expected=bytearray(before);expected[0x3554:0x3556]=bytes(model.load(0x11c2b4+i,1)for i in range(2));expected[0x355e:0x3b55e]=bytes(model.load(0x11c2be+i,1)for i in range(229376))
 for base,n,width,addr in [(B,256,67,0x1645ae),(EF,200,81,0x1688ae),(CR,300,124,0x1542be)]:
  for slot in range(n):
   native=bytearray(model.load(addr+slot*width+i,1)for i in range(width));native[31:39]=before[base+slot*width+31:base+slot*width+39];expected[base+slot*width:base+(slot+1)*width]=native
 expected[PL:EF]=bytes(model.load(0x118d74+i,1)for i in range(7220));struct.pack_into('<HBI',expected,RN,model.load(0x11878a,2),model.load(0x11872c,1),model.load(0x118782,4))
 assert expected==actual,(case,[(i,a,b)for i,(a,b)in enumerate(zip(expected,actual))if a!=b][:15])
 (work/f'case{case}-source.bin').write_bytes(expected);steps+=report['steps'];total+=len(actual);print('source case',case,'PASS',flush=True)
r['literal_whole_world_comparison']={'status':'passed','cases':6,'bytes':total,'instructions':steps,'ports':['audio service statefalse and audio calls','render queue','viewport invalidation84E10']};r['remaining']=['occupied contact damage callback','empty-resource shutdown'];r['listing_pins']={f.relative_to(P).as_posix():hashlib.sha256(f.read_bytes()).hexdigest()for f in files};(P/'research/campaign/extractor-owned-integration.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r['literal_whole_world_comparison']))

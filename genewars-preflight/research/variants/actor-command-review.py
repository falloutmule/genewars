"""Independent opcode3 record-write and literal-word resolver audit. No original/GUI."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,itertools,json,random,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/actor-command'
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),size=len(b),sha256=hashlib.sha256(b).hexdigest())
def word(v):return(v+32768)%65536-32768
def distance(a,b):
 # Literal low-word double, subtract, SAR1, sign extension, IMUL/ADD.
 deltas=[word(((x*2)&65535)-((y*2)&65535))//2 for x,y in zip(a,b)];return sum(d*d for d in deltas)
def setters():
 for a,b,c in itertools.product(range(256),range(256),[0,1,255]):yield(19,200,(a*257+b)*13+c,a,b,c,3,[-32768,32767],[32767,-32768])
 for state,flag in itertools.product(range(256),[0,2,128,254,255]):yield(0,200,state,0,flag,state,17,[0,0],[65,-65])
 for raw in range(65536):yield(word(raw),200,raw,16,1,255,0,[0,0],[0,0])
 for length,prefix,id in itertools.product(range(401),[0,3,128],[-1,0,19,20]):yield(id,length,length*7,16,1,255,prefix,[1,2],[3,4])
 # Every opaque offset/value is independently exercised with an admitted
 # applied record pattern; full guards/record are compared, not only fields.
 for offset,value in itertools.product(range(200),range(256)):yield(0,200,offset*65536+value,0,254,0,offset%19,[word(offset*257),word(-offset*511)],[word(value*257),word(-value*257)])
 rng=random.Random(0x80fc074a50)
 for _ in range(4096):yield(rng.randrange(20),200,rng.randrange(2**32),rng.randrange(256),rng.randrange(256),rng.randrange(256),rng.randrange(32),[rng.randrange(-32768,32768)for _ in range(2)],[rng.randrange(-32768,32768)for _ in range(2)])
def setmodel(v):
 id,n,seed,a,b,c,prefix,request,resolved=v;data=bytearray((i*37+seed)%256 for i in range(prefix+n+11))
 for offset,value in [(0x71,a),(0x57,b),(0x82,c)]:
  if offset<n:data[prefix+offset]=value
 if not 0<=id<20:status=2
 elif n!=200:status=3
 elif a&16:status=1
 elif b&1 or c:status=4
 else:
  status=0;data[prefix+0x57]|=2;data[prefix+0x5b]=0;encoded=struct.pack('<hh',*resolved);data[prefix+0x5c:prefix+0x60]=encoded;data[prefix+0xb:prefix+0xf]=encoded
 return status,struct.pack('<4h',*request,*resolved)if status==0 else bytes(8),bytes(data)
def resolvers():
 for raw,delta in itertools.product(range(65536),[0,1,-1,63,64,65,-64,16383,16384,16385,32767]):
  current=[word(raw),word(raw^0xa55a)];request=[word(current[0]+delta),word(current[1]+(1 if delta==64 else 0))];yield(current,request,raw%8,(raw>>8)%256,bool(raw&1),9 if raw&2 else 10)
 for terrain,mask,obstacle,weight,near in itertools.product(range(8),range(256),range(2),[0,9,10,2**31-1],range(2)):yield([0,0],[64 if near else 65,0],terrain,mask,obstacle,weight)
 for terrain,weight in itertools.product(range(256),[-2**31,-1,0,9]):yield([0,0],[1,0],terrain,255,True,weight)
def resolvemodel(v):
 current,request,terrain,mask,obstacle,weight=v
 if terrain>7:return 1,bytes(4)
 if weight<0:return 2,bytes(4)
 if distance(current,request)<=4096:return 0,struct.pack('<2h',*current)
 if obstacle or mask&(1<<terrain)or weight>=10:return 3,bytes(4)
 return 0,struct.pack('<2h',*request)
def main():
 WORK.mkdir(parents=True,exist_ok=True);source=ROOT/'rust/crates/genewars-runtime/src/actor_command.rs';sourcepin=pin(source);proofpath=ROOT/'research/ghidra/move-order-evidence.json';proof=json.loads(proofpath.read_bytes());proofpin=pin(proofpath);pins=[]
 for r in proof['instruction_contracts']:p=pin(ROOT/r['private_path']);assert p['sha256']==r['sha256'];pins.append(p)
 code=(ROOT/'private/ghidra/validated/memory-.object1.bin').read_bytes();table=proof['opcode_table'];raw=code[int(table['va'],16)-0x10000:int(table['va'],16)-0x10000+table['length']];assert hashlib.sha256(raw).hexdigest()==table['sha256']and struct.unpack_from('<I',raw,12)[0]==0x1c528
 text=(ROOT/'private/ghidra/plant-command/instructions-00080fc0.txt').read_text();assert text.index('00080fec TEST byte ptr [ESI + 0x71],0x10')<text.index('00080ffb OR byte ptr [ESI + 0x57],0x2')<text.index('00081044 CALL 0x00074a50')<text.index('00081059 TEST byte ptr [ESI + 0x57],0x1')
 with (WORK/'setters.bin').open('wb')as out:
  for id,n,seed,a,b,c,prefix,request,resolved in setters():out.write(struct.pack('<hHI4B4h',id,n,seed,a,b,c,prefix,*request,*resolved))
 with (WORK/'resolvers.bin').open('wb')as out:
  for current,request,terrain,mask,obstacle,weight in resolvers():out.write(struct.pack('<4h3Bi',*current,*request,terrain,mask,obstacle,weight))
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe';result=subprocess.run([str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],env=environment(),capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace');result=subprocess.run([str(exe),str(WORK)],capture_output=True,timeout=90);assert result.returncode==0,result.stderr.decode(errors='replace')
 counts=[0]*5;rcounts=[0]*4;byteschecked=0;digest=hashlib.sha256()
 with (WORK/'results.bin').open('rb')as native:
  for i,v in enumerate(setters()):
   status,echo,expected=setmodel(v);actual=native.read(9+len(expected));assert actual==bytes([status])+echo+expected,(i,v);counts[status]+=1;byteschecked+=len(expected);digest.update(actual)
  for i,v in enumerate(resolvers()):
   status,point=resolvemodel(v);actual=native.read(5);assert actual==bytes([status])+point,(i,v,actual.hex(),status,point.hex());rcounts[status]+=1;digest.update(actual)
  assert not native.read(1)
 assert sourcepin==pin(source)and proofpin==pin(proofpath)
 report=dict(status='passed-independent-native-actor-command-review',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,gui_execution=False,source=sourcepin,evidence=proofpin,instruction_pins=pins,setter_cases=sum(counts),setter_outcomes=dict(zip(['applied','source_noop','invalid_id','invalid_length','reinitialization_required'],counts)),record_and_guard_bytes_compared=byteschecked,resolver_cases=sum(rcounts),resolver_outcomes=dict(zip(['resolved','invalid_terrain','negative_weight','backward_search_required'],rcounts)),native_output_sha256=digest.hexdigest(),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],harness=pin(exe),limits=['Single supplied record must be externally bound to globalactorID; no pool/owner/type gate inferred','PrecomputedResolvedTarget is an explicit caller value, not proof of world occupancy or a trusted provenance token','Resolver outputs only under admitted explicit environment values; virtual effects and backward search excluded','Invalid environment rejected even fornearpoint as deliberate safe input validation','No travel/animation/landing activation, readybit producer, physical input conversion or wholemission command dispatch executed'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k]for k in ['status','setter_cases','record_and_guard_bytes_compared','resolver_cases']}))
if __name__=='__main__':main()

"""Independent instruction-derived candidate and complete activation after-image review."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,itertools,json,struct,subprocess,sys
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts'))
from cargo import environment
WORK=ROOT/'private/review/actor-activation';SIZE=200;BPSIZE=1152;MAPSIZE=229376
def pin(p):
 b=p.read_bytes();return dict(path=p.relative_to(ROOT).as_posix(),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
def signed_word(v):return (v+32768)%65536-32768
def cases():
 # id,slot,len,seed,cx,cy,flags,pattern,type,avoid,selector,recordid,bplen,category,resource18,count,headid,headkind,envmode,class,weight,maplen,associatedAbsent
 base=[0,0,200,5428,20736,13568,1024,151,3,0,0,0,1152,1,9,0,0,0,1,0,9,229376,1,0]
 for active,flags in itertools.product([0,3],range(65536)):
  v=base.copy();v[0]=flags%20;v[1]=v[0]%5;v[3]=flags;v[6]=flags;v[7]=flags%256;v[8]=active;v[10]=flags%32;v[11]=v[0];v[13]=flags%2;v[15]=[0,1,0xfffffffe,0xffffffff][flags%4];v[16]=signed_word(flags*257);yield v
 for avoid,kind,weight in itertools.product(range(256),range(8),[-1,0,9,10,2**31-1]):
  v=base.copy();v[9]=avoid;v[19]=kind;v[20]=weight;yield v
 for index,values in [(0,[-32768,-1,0,19,20,32767]),(1,[0,1,4,5,255]),(2,[0,6,16,112,113,199,200,201]),(4,[0,32767,32768,65535]),(8,range(256)),(10,range(256)),(11,[-32768,-1,0,1,19,32767]),(12,[0,1151,1152,1153]),(18,[0,2,3,4,5]),(21,[0,229375,229376]),(22,[0,1])]:
  for value in values:v=base.copy();v[index]=value;yield v
 for headid,headkind in itertools.product([-32768,-1,0,19,511,32767],range(256)):
  v=base.copy();v[16]=headid;v[17]=headkind;yield v
 for headkind,resolution in itertools.product([0,2,9],range(5)):
  v=base.copy();v[17]=headkind;v[23]=resolution;yield v
 # Exact retry sequencing, accepted prefix ignores malformed unused suffix.
 v=base.copy();v[9]=1;v[18]=5;yield v
 for center,seed in itertools.product([[0,0],[32767,32767],[16383,16384]],range(256)):
  v=base.copy();v[4:6]=center;v[3]=seed;v[18]=2;yield v
def placement(seed,center,sine,cosine):
 # Literal signed low-word register arithmetic, independent of native helper.
 new=(seed*58925)%65536;angle=(new//256)*8;table=(angle+1536)%2048
 dx=(1024*cosine[table])//65536;dy=(1024*sine[table])//65536
 return [seed,new,angle,*[(signed_word(2*c+d)//2)%32768 for c,d in zip(center,[dx,dy])]]
def oracle(v,sine,cosine):
 id,slot,n,seed,cx,cy,flags,pattern,kind,avoid,sel,rid,bplen,category,r18,count,headid,headkind,mode,terrain,weight,maplen,absent,resolution=v
 actor=bytearray((i*37+pattern)%256 for i in range(n));bp=bytearray((i*53+pattern+11)%256 for i in range(bplen))
 for i,b in [(6,kind),(0x53,avoid),(0x74,sel)]:
  if i<n:actor[i]=b
 if n>=114:struct.pack_into('<H',actor,112,flags)
 if n>=17:struct.pack_into('<h',actor,15,rid)
 if sel<32 and bplen>sel*36+32:
  a=sel*36;bp[a+22]=category;bp[a+24]=r18;struct.pack_into('<I',bp,a+28,count)
 if not 0<=id<20:return bytes([2])
 if slot>=5:return bytes([3])
 if n!=200:return bytes([4])
 if kind==0 or not flags&1024:return bytes([1])
 if kind!=3:return bytes([5])
 if id%5!=slot:return bytes([6])
 if rid!=id:return bytes([7])
 if not absent:return bytes([8])
 if bplen!=1152:return bytes([9])
 if maplen!=229376:return bytes([10])
 if cx>32767 or cy>32767:return bytes([11])
 if mode==3:return bytes([15])
 if sel>=32:return bytes([12])
 envs={0:[],1:[(terrain,weight)],2:[(terrain,10),(terrain,weight)],3:[(0,10)]*4097,4:[(0,10)]*4096,5:[(0,10),(1,9),(255,-1)]}[mode]
 attempts=[];accepted=None
 for t,w in envs:
  if t>7:return bytes([13])
  if w<0:return bytes([14])
  p=placement(seed,[cx,cy],sine,cosine);seed=p[1];ok=not avoid&(1<<t) and w<10;attempts.append(struct.pack('<5HB',*p,ok))
  if ok:accepted=p;break
 if accepted is None:return bytes([15])
 resolved=resolution!=1 and (headkind!=0 or resolution==4)
 if not headkind and resolved:return bytes([17])
 if headkind and not resolved:return bytes([18])
 if headkind and (headid<0 or headkind>8):return bytes([16])
 if headkind and resolution in [2,3]:return bytes([19])
 x,y=accepted[3:];cell=(y//256)*128+x//256;old=struct.pack('<hB',headid,headkind);ref=struct.pack('<hB',id,3);prev=struct.pack('<HB',cell,0);nxt=old if headkind else bytes(actor[3:5])+b'\0'
 after=actor.copy();beforebp=bp[sel*36:sel*36+36];afterbp=beforebp.copy()
 after[:3]=prev;after[3:6]=nxt;struct.pack_into('<HH',after,7,x,y);struct.pack_into('<H',after,112,(flags&0xfbfe)|1);after[22]&=0xf7
 afterbp[27]=beforebp[24];afterbp[32]|=2;newcount=(count+1)%2**32;struct.pack_into('<I',afterbp,28,newcount)
 if newcount==1 and category==1:after[114]=40
 return b'\0'+struct.pack('<4H',seed,len(attempts),sel,cell)+actor+after+beforebp+afterbp+old+prev+nxt+ref+bytes([bool(headkind)])+(ref if headkind else bytes(3))+b''.join(attempts)
def main():
 WORK.mkdir(parents=True,exist_ok=True);source=ROOT/'rust/crates/genewars-runtime/src/actor_activation.rs';proofpath=ROOT/'research/ghidra/actor-activation-evidence.json';proof=json.loads(proofpath.read_bytes());pins=[pin(source),pin(ROOT/'rust/crates/genewars-runtime/src/identity.rs'),pin(proofpath)]
 for r in proof['instruction_contracts']:assert pin(ROOT/r['private_path'])['sha256']==r['sha256']
 listing=(ROOT/'private/ghidra/command-resolver/instructions-00080ae0.txt').read_text();assert '00080c65 MOV EAX,dword ptr [ESI + 0xc0]'in listing and '00080c6e MOV byte ptr [EAX + 0x1b],DL'in listing
 memory=(ROOT/'private/ghidra/validated/memory-.object3.bin').read_bytes();raws=[memory[a:a+8192]for a in [0x7ee8,0x86e8]];sine,cosine=[struct.unpack('<2048i',b)for b in raws]
 for name,b,r in zip(['sin','cos'],raws,proof['tables']):assert hashlib.sha256(b).hexdigest()==r['sha256'];(WORK/(name+'.bin')).write_bytes(b)
 with (WORK/'cases.txt').open('w')as out:
  for v in cases():out.write(' '.join(map(str,v))+'\n')
 rustc=ROOT/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe'
 for cmd in [[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-o',str(exe)],[str(exe),str(WORK)]]:
  r=subprocess.run(cmd,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 digest=hashlib.sha256();total=0;outcomes=[0]*20
 with (WORK/'actual.bin').open('rb')as f:
  for seed in range(65536):
   for center in [[0,0],[32767,32767],[20736,13568],[16383,16384]]:
    expected=struct.pack('<5H',*placement(seed,center,sine,cosine));actual=f.read(len(expected));assert actual==expected,(seed,center);digest.update(actual);total+=1
  count=0;byteschecked=0
  for v in cases():
   expected=oracle(v,sine,cosine);actual=f.read(len(expected));assert actual==expected,(count,v,actual[:32].hex(),expected[:32].hex());digest.update(actual);count+=1;outcomes[expected[0]]+=1;byteschecked+=len(expected)
  assert not f.read(1)
 assert pins==[pin(source),pin(ROOT/'rust/crates/genewars-runtime/src/identity.rs'),pin(proofpath)],'source or proof changed during review'
 report=dict(status='passed-independent-native-actor-activation-review',utc=datetime.now(timezone.utc).isoformat(),source_pins=pins,instruction_pins=proof['instruction_contracts'],candidate_cases=total,planner_cases=count,outcomes=outcomes,whole_native_output_bytes_compared=total*10+byteschecked,native_output_sha256=digest.hexdigest(),harness=pin(exe),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],original_execution=False,gui_execution=False,resolved_findings=['resource+1B write corrected; actor frame preserved','eligible actor restricted to type3','record ID and partition slot validated'],limits=['Non-null head tests assume externally validated family bounds and resolved ID/type identity; no record resolver or world commit executed','Explicit terrain/blocker sequence remains caller supplied; original retries and consumes live RNG without this safe cap','Blueprint selector must externally bind native C0; no pointer reconstruction','No associated type5 route, landing/RNG timeline, batch player49D update or whole startup claim'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:report[k]for k in ['status','candidate_cases','planner_cases','whole_native_output_bytes_compared']}))
if __name__=='__main__':main()

"""Literal72ED0 eligibility/outcome comparison; capability and module switch are typed ports."""
from pathlib import Path
import struct,random,json,hashlib,subprocess,sys,uuid,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
W=P/'private/review/mission-outcome'/uuid.uuid4().hex;W.mkdir(parents=True)
original=P/'research/variants/initial-creature-source.py';text=original.read_text();needle='   if self.pc==0xdeadbeef:break';assert text.count(needle)==1;scope={'__file__':str(original),'__name__':'outcome_source'}
text=text.replace('(0x11873d,1,0),','').replace(needle,'''   if self.pc==0x731b6:
    self.scan=(self.load(self.r['ESP']+4,1),self.load(self.r['ESP'],1))
    if self.prefix_only:break
   if self.pc==0x72f28:break
   if self.pc==0x805e0:
    self.query_ids.append((self.r['EAX']-0x118d74)//1805)
    if self.load(self.r['EAX']+0x6e2,1)==2:break
   if self.pc==0xdeadbeef:break''')
exec(compile(text,'mission-outcome-literal','exec'),scope);s=scope['Source']();m=scope['m'];files=[P/'private/ghidra/font-menu-state/instructions-00072ed0.txt',P/'private/ghidra/construction-command/instructions-0008ab30.txt'];s.code.update(m.Model(files,b'').code);s.code[0x805e0]=('MOV',['AL','byte ptr [EAX + 0x6e2]']);s.code[0x805e1]=('RET',[]);s.code[0x2f810]=('RET',[]);order=sorted(s.code);s.next=dict(zip(order,order[1:]));rng=random.Random(0x72ed0);data=bytearray();expected=bytearray();stats=[]
for case in range(640):
 players=[bytearray(1805)for _ in range(4)];active=case%5;flags=[0,0x20,0x40,0x80,0x3a,0x3e][(case//5)%6];mode=(case//30)%2;target=[0,1,200][case%3];timer=case%2;limit=case%4;terrain=case%2;terrainlimit=rng.choice([-2147483648,-1,0,20,2147483647]);counter=[0,2,256,0xffffffff][case%4];frame=2 if case%7==0 else 1;selected=(case//25)%4;other=(case//2)%2
 for i,b in enumerate(players):
  b[2]=rng.randrange(2);b[3]=rng.choice([0,1,127,128,255]);b[0x498]=rng.randrange(2);b[0x4a2]=rng.choice([0,1,128,255]);b[0x6e2]=rng.choice([0,1,2]);b[0x6e3]=rng.randrange(2);b[0x606]=rng.choice([0,2,3,255]);b[0x6e7]=rng.choice([0,1,14,20,255]);struct.pack_into('<i',b,0x49d,rng.choice([-2147483648,-1,0,1,2147483647]));struct.pack_into('<H',b,0x60e,rng.choice([0,1,65535]));struct.pack_into('<i',b,0x11,rng.randrange(5));struct.pack_into('<i',b,0x4a7,rng.choice([-1,0,1,200]));struct.pack_into('<i',b,0xd,rng.choice([-2147483648,-1,0,10,30,65535,65536,65537,2147483647]))
 if case>=600:
  # First source campaign rule and enclosing gates; no scripted world success.
  active=1;selected=0;flags=0x3e;mode=0;counter=0;target=0;terrain=0;other=0;players=[bytearray(1805)for _ in range(4)];b=players[0];struct.pack_into('<i',b,0x49d,0 if case%8==0 else 1);b[0x4a2]=0 if case%8==1 else 1;b[3]=0 if case%8==1 else 1;b[0x606]=[2,3,255,0][case%4];b[0x6e2]=case%3;b[0x6e7]=0 if case%8!=2 else 1
 state={0x118d74+i*1805+j:v for i,b in enumerate(players)for j,v in enumerate(b)}
 def put(a,fmt,v):state.update({a+i:b for i,b in enumerate(struct.pack(fmt,v))})
 for a,fmt,v in [(0x118770,'<B',flags),(0x11878c,'<B',active),(0x118968,'<B',mode),(0x11ac06,'<H',target),(0x11a9dc,'<H',limit),(0x11c2bc,'<H',timer),(0x11c218,'<B',terrain),(0x11ac0a,'<i',terrainlimit),(0x118d60+0x34b9,'<B',other),(0x11873d,'<B',selected),(0x11ac22,'<I',counter)]:put(a,fmt,v)
 for i in range(4):state[0x18386a+i*0x6be]=2
 s.query_ids=[];s.scan=None;s.prefix_only=True;report,_,trace=s.run(counter=frame,entry=0x72ed0,state_before=state,registers={'EAX':0x118d60});status=None
 if flags&0x40:status=3
 elif s.pc==0x72f28:status=4
 elif s.pc==0x805e0:status=6
 else:
  winner,survivors=s.scan
  if mode&1 and (winner!=255 or flags&0x80==0 and s.load(0x118d74+selected*1805+0x6e3,1)!=0):status=5
 tail=bytearray([255]*12)
 if status is not None:out=b''.join(players);tail[0]=flags;tail[1]=status
 else:
  s.prefix_only=False;s.query_ids=[];full,_,trace=s.run(counter=frame,entry=0x72ed0,state_before=state,registers={'EAX':0x118d60});assert s.pc==0xdeadbeef;out=bytes(s.load(0x118d74+i,1)for i in range(7220));afterflags=s.load(0x118770,1);tail[0]=afterflags;tail[1]=1 if winner==selected else 2 if winner!=255 or afterflags&0x80 else 0;tail[2]=winner;tail[3]=len(s.query_ids);tail[4:4+len(s.query_ids)]=s.query_ids;tail[8]=int(0x2f810 in trace);report['steps']+=full['steps'];assert tail[8]==int(tail[1]==1)
 expected+=out+tail;c=bytearray(32);c[:4]=bytes([flags,mode,active,selected]);struct.pack_into('<IIHHHB',c,4,counter,frame,target,limit,timer,terrain);struct.pack_into('<i',c,19,terrainlimit);c[23]=other;data+=b''.join(players)+c;stats.append(dict(case=case,result=tail[1],instructions=report['steps']))
(W/'input.bin').write_bytes(data);(W/'expected.bin').write_bytes(expected);production=P/'rust/crates/genewars-runtime/src/mission_outcome.rs';shutil.copyfile(production,W/'runtime.rs');shutil.copyfile(Path(__file__).with_suffix('.rs'),W/'probe.rs')
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr)
run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','-O',str(W/'probe.rs'),'-o',str(W/'probe.exe')]);run([str(W/'probe.exe'),str(W)])
a=(W/'actual.bin').read_bytes();assert a==expected,[(i//7232,i%7232,x,y)for i,(x,y)in enumerate(zip(a,expected))if x!=y][:20]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
receipt=dict(status='passed',private_run=W.relative_to(P).as_posix(),cases=stats,result_counts={str(code):sum(c['result']==code for c in stats)for code in range(8)},source_pins={f.relative_to(P).as_posix():sha(f)for f in s.files+files+[original,P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',production,Path(__file__),Path(__file__).with_suffix('.rs')]},scope='Actual72ED0 numeric scan and ordinary terminal flags;805E0 capability boolean and2F810 result request are explicit ports; no campaign advancement or original execution')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(stats),private_run=receipt['private_run'],results=receipt['result_counts'])))

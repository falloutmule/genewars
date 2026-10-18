"""Bounded loss-view numeric entry; all presentation calls are explicit obligations."""
from pathlib import Path
import importlib.util,struct,json,hashlib,subprocess,sys,uuid,random,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
W=P/'private/review/loss-transition'/uuid.uuid4().hex;W.mkdir(parents=True)
original=P/'research/variants/initial-creature-source.py';text=original.read_text().replace('(0x11873d,1,0),','');scope={'__file__':str(original),'__name__':'transition_source'};exec(compile(text,'transition_source','exec'),scope);s=scope['Source']();m=scope['m']
files=[P/'private/ghidra/mission-result-refs'/n for n in ['loss-view.txt','result-thunks.txt','instructions-00082590.txt']];files.append(P/'private/ghidra/ui-actor-action-refs/instructions-0001bbc0.txt');s.code.update(m.Model(files,b'').code)
# Stop at the documented dispatch boundary after the main-loop loss gate.
s.code[0x1bcb5]=('RET',[])
# Ghidra omits the byte qualifier for absolute accumulator stores; restore the
# operand width from AL, without changing the instruction or its value.
s.code[0x7df3e]=('MOV',['byte ptr [0x00118967]','AL'])
ports=[0x8e780,0x154b0,0x154f0,0xa4480,0xa1340,0x2f0c0,0x1be50,0xa1610,0xaea78,0xa2fb8]
for address in ports:s.code[address]=('RET',[])
s.code[0xa1610]=('MOV',['EAX','0x1']);s.code[0xa1611]=('RET',[]);keys=sorted(s.code);s.next=dict(zip(keys,keys[1:]));rng=random.Random(0x7dea0);data=bytearray();expected=bytearray();stats=[]
obj=P/'private/ghidra/validated/memory-.object3.bin';raw=obj.read_bytes();assert struct.unpack_from('<I',raw,0x112810-0x100000)[0]==0x8df10;assert struct.unpack_from('<I',raw,0x112834-0x100000)[0]==24
for case in range(32):
 players=bytearray(rng.randbytes(7220));selected=case%4;flags=[0x80,0xbe,0xe4,0xff][case%4];mode=[0,2,8,10][case%4];frame=[0,1,999,0xffffffff][case%4];players[selected*1805+0x6e7]=[0,1,23,255][case%4];state={0x118d74+i:v for i,v in enumerate(players)}
 def put(a,fmt,v):state.update({a+i:b for i,b in enumerate(struct.pack(fmt,v))})
 for a,fmt,v in [(0x1863e8,'<I',0x112830),(0x11873d,'<B',selected),(0x118770,'<B',flags),(0x118968,'<B',mode),(0x16e0a6,'<B',0),(0x16e0a7,'<B',0),(0x19cf50,'<I',0),(0x19cf6c,'<I',0),(0x19cf70,'<I',0)]:put(a,fmt,v)
 report,_,trace=s.run(counter=frame,state_before=state,entry=0x8df10,registers={'EAX':0x186400});calls=[pc for pc in trace if pc in ports];pa=bytes(s.load(0x118d74+i,1)for i in range(7220));numeric=bytes([s.load(0x118770,1),s.load(0x118968,1),s.load(0x118967,1)])+struct.pack('<II',s.load(0x11877e,4),s.load(0x118786,4));assert s.load(0x16e0a6,1)==0 and s.load(0x16e0a7,1)==0
 data+=players+bytes([selected,flags,mode,0])+struct.pack('<I',frame);expected+=pa+numeric+bytes([len(calls)])+b''.join(struct.pack('<I',p)for p in calls)
 # Literal main-loop1BC8E prefix, stopping before virtual dispatch.
 for gate_mode in [mode,mode|1]:
  for gate_flags in [flags,flags&0x7f]:
   put(0x118968,'<B',gate_mode);put(0x118770,'<B',gate_flags);put(0x116974,'<I',0x185e03);put(0x116978,'<I',0);put(0x11697c,'<I',0)
   s.run(state_before=state,entry=0x1bc8e);assert s.load(0x116978,4)==(0x186400 if gate_flags&0x80 and not gate_mode&1 else 0)
 # Source7E060 edge gate returns title irrespective of campaign progress.
 for pressed in [0,1,2,255]:
  put(0x118779,'<B',pressed);put(0x116978,'<I',0);put(0x11697c,'<I',0)
  s.run(state_before=state,entry=0x7e060,registers={'EAX':0x1863e8});assert s.load(0x116978,4)==(0x185008 if pressed&1 else 0)
 stats.append(dict(case=case,instructions=report['steps'],calls=calls))
(W/'input.bin').write_bytes(data);(W/'expected.bin').write_bytes(expected);production=P/'rust/crates/genewars-runtime/src/loss_transition.rs';shutil.copyfile(production,W/'runtime.rs');shutil.copyfile(Path(__file__).with_suffix('.rs'),W/'probe.rs')
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr)
run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(W/'probe.rs'),'-o',str(W/'probe.exe')]);run([str(W/'probe.exe'),str(W)]);actual=(W/'actual.bin').read_bytes();assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
receipt=dict(status='passed',private_run=W.relative_to(P).as_posix(),cases=stats,source_pins={f.relative_to(P).as_posix():sha(f)for f in s.files+files+[original,P/'research/variants/actor-environment-terrain-target.py',obj,P/'private/ghidra/validated/memory-.object1.bin',production,Path(__file__),Path(__file__).with_suffix('.rs')]},scope='32 loss entry plans compare unchanged fullplayers, flags/mode/stage/frame/savedframe and ordered presentation ports; source7E060 edge routing to title checked;72E50 cleanup not executed')
(W/'result.json').write_text(json.dumps(receipt,indent=2));Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=32,private_run=receipt['private_run'])))


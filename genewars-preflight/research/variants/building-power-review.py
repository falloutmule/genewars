"""Bounded source451F0 numeric prefixes; effect33/audio are explicit requests."""
from pathlib import Path
import importlib.util,struct,json,random,uuid,hashlib,subprocess,sys,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
spec=importlib.util.spec_from_file_location('literal',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Source(m.Source):
 def load(self,a,n):
  if self.pc in [0x2e0010,0x2e0020,0x48790] and a==self.r['ESP'] and n==4:
   if self.pc==0x48790:
    assert self.r['EAX']==33
    ptr=0x280000+len(self.transfers)*81;v=self.r['ECX'];self.transfers.append((ptr,self.r['EDX'],self.r['EBX'],tuple(m.Source.load(self,v+i*2,2)for i in range(3))))
    for i in range(81):self.mem[ptr+i]=0
    self.r['EAX']=ptr
   else:self.audio.append((self.pc,self.r['EDX'],self.r['ECX'],self.r['EBX']))
  return super().load(a,n)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(a):
 r=subprocess.run(a,cwd=P,env=environment(),capture_output=True,text=True,timeout=120);assert r.returncode==0,(r.stdout,r.stderr);return r.stdout
W=P/'private/review/building-power'/uuid.uuid4().hex;W.mkdir(parents=True)
s=Source();files=[P/'private/ghidra/building-update/instructions-000451f0.txt'];s.code.update(m.m.Model(files,b'').code)
obj=P/'private/ghidra/validated/memory-.object1.bin'
for pc,(op,args)in list(s.code.items()):
 if op=='MOV' and args==['[0x0011878a]','AX']:
  assert obj.read_bytes()[pc-0x10000:pc-0x10000+6]==bytes.fromhex('66a38a871100');s.code[pc]=('MOV',['word ptr [0x0011878a]','AX'])
for pc in [0x45f40,0x462b8]:s.code[pc]=('CALL',['0x2e0010'])
s.code[0x4619b]=('CALL',['0x2e0020']);s.code[0x2e0010]=('RET',['8']);s.code[0x2e0020]=('RET',[]);s.code[0x48790]=('RET',['16']);s.code[0x47f2c]=('JMP',['0x47f69']);order=sorted(s.code);s.next=dict(zip(order,order[1:]))
r=random.Random(33);data=bytearray();expected=bytearray();stats=[]
for case in range(96):
 owner=case%4;slot=case%8;selector=[3,6][case%2];grade=(case//2)%3;frame=[0,1,16,32,63,64][case%6];solar=[0,1,4,10,-1][case%5];seed=[0,5428,65535,r.randrange(65536)][case%4];ring=case%32;draws=0xffffffff if case%5==0 else case
 pool=[bytearray(67)for _ in range(64)]
 for i in range(64):
  b=pool[i];struct.pack_into('<h',b,15,owner*64+i);b[24]=owner
  if i<12:
   b[6]=4;b[45]=[0,3,6,8,9,11,1,7,4,13,3,3][i];b[46]=i%3;b[49:51]=struct.pack('<H',[1,5,0x101,0x41][i%4]);struct.pack_into('<hh',b,7,10000+[0,100,1536,2304,3072,5000][i%6],10000)
 b=pool[slot];b[6]=4;b[45]=selector;b[46]=grade;b[49:51]=struct.pack('<H',[1,5][case%2]);b[27]=case%3;b[48]=case%5;b[63]=[0,1,255][case%3];struct.pack_into('<hh',b,7,10000,10000);struct.pack_into('<h',b,61,[-32768,-1,0,198,199,200,1999,2000,32767][case%9]);b[25]=case%4
 player=bytearray(r.randbytes(1805));player[1]=player[0x4a1]=owner
 state={0x1645ae+i:0 for i in range(256*67)};state.update({0x11c2be+i:0 for i in range(229376)})
 for i,b in enumerate(pool):state.update({0x1645ae+(owner*64+i)*67+j:v for j,v in enumerate(b)})
 state.update({0x118d74+owner*1805+i:v for i,v in enumerate(player)});state.update({0x11a9bf+i:v for i,v in enumerate(struct.pack('<h',solar))});state[0x11872c]=ring;state.update({0x118782+i:v for i,v in enumerate(struct.pack('<I',draws))});s.transfers=[];s.audio=[]
 report,_,trace=s.run(counter=frame,seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae+(owner*64+slot)*67});assert all(pc in [0x736a6,0x73986]for pc,_ in s.uninitialized_stack_reads)
 after=bytes(s.load(0x1645ae+owner*64*67+i,1)for i in range(64*67));pa=bytes(s.load(0x118d74+owner*1805+i,1)for i in range(1805));ra=struct.pack('<HBI',s.load(0x11878a,2),s.load(0x11872c,1),s.load(0x118782,4))
 requests=bytearray()
 for ptr,pos,height,angles in s.transfers:requests+=bytes(s.load(ptr+71+i,1)for i in range(3))+bytes([s.load(ptr+76,1)])+struct.pack('<3Hi',*angles,height)
 aud=bytes(1 if pc==0x2e0020 else 0 if code==61 else 2 for pc,code,ecx,ebx in s.audio)
 first_transfers=list(s.transfers);first_audio=list(s.audio)
 s.transfers=[];s.audio=[]
 s.run(counter=frame,seed=seed,state_before=state,entry=0x451f0,registers={'EAX':0x1645ae+(owner*64+slot)*67},stack_fill=255)
 assert after==bytes(s.load(0x1645ae+owner*64*67+i,1)for i in range(64*67)) and pa==bytes(s.load(0x118d74+owner*1805+i,1)for i in range(1805))
 assert first_transfers==s.transfers and first_audio==s.audio
 data+=b''.join(pool)+player+struct.pack('<HBIIBh',seed,ring,draws,frame,slot,solar)
 expected+=after+pa+ra+struct.pack('<I',len(s.transfers))+requests+struct.pack('<I',len(aud))+aud
 stats.append(dict(case=case,selector=selector,transfers=len(s.transfers),steps=report['steps']))
(W/'input.bin').write_bytes(data);(W/'expected.bin').write_bytes(expected)
for n in ['angles.bin','bounce.bin']:shutil.copyfile(P/'private/review/family8-state4'/n,W/n)
delivery_file=P/'private/ghidra/plant-command/instructions-0004b630.txt';files.append(delivery_file)
s.code.update(m.m.Model([delivery_file],b'').code);s.code[0x49d30]=('RET',[]);order=sorted(s.code);s.next=dict(zip(order,order[1:]))
assert struct.unpack_from('<I',obj.read_bytes(),0x4b524-0x10000+(33-6)*4)[0]==0x4c8e6
di=bytearray();de=bytearray();delivery_cases=[]
for case in range(64):
 effect=bytearray(81);effect[6]=5;effect[69]=33;effect[73]=4 if case%4 else 0;effect[71]=7;effect[76]=[0,1,2,200,255][case%5];struct.pack_into('<hh',effect,7,32760,1000)
 target=bytearray(67);target[6]=4;target[15]=7;target[45]=[3,11,6,12][case%4];target[49]=case%2;struct.pack_into('<hh',target,7,m.signed(32760+[0,128,129,255,-128][case%5],16),1000);struct.pack_into('<h',target,61,[-32768,-1,0,199,200,1999,2000,32767][case%8])
 state={0x1688ae+i:v for i,v in enumerate(effect)};state.update({0x1645ae+7*67+i:v for i,v in enumerate(target)});state.update({0x11c2be+i:0 for i in range(229376)});state.update({i:0 for i in range(0x2fff00,0x300001)});state.update({0x2fff22+i:v for i,v in enumerate(struct.pack('<I',0x1688ae))});state.update({0x2fffa6+i:v for i,v in enumerate(struct.pack('<I',0xdeadbeef))})
 report,_,trace=s.run(state_before=state,entry=0x4c8e6,registers={'EBP':0x2fff00});assert report['uninitialized_stack_reads']==0
 ea=bytes(s.load(0x1688ae+i,1)for i in range(81));ta=bytes(s.load(0x1645ae+7*67+i,1)for i in range(67));retired=int(0x49d30 in trace)
 di+=effect+target;de+=ea+ta+bytes([retired]);delivery_cases.append(dict(case=case,retire=bool(retired),instructions=report['steps']))
(W/'delivery-input.bin').write_bytes(di);(W/'delivery-expected.bin').write_bytes(de)
rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');pins={}
for c in ['assets','core','runtime']:
 t=W/c;shutil.copytree(P/f'rust/crates/genewars-{c}/src',t)
 for f in t.rglob('*.rs'):pins[f'{c}/{f.relative_to(t)}']=sha(f)
 if c=='runtime' and 'pub mod building_power;' not in (t/'lib.rs').read_text():(t/'lib.rs').write_text((t/'lib.rs').read_text()+'\npub mod building_power;\n')
 a=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-type','rlib','--crate-name',f'genewars_{c}',str(t/'lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{c}.rlib')]
 if c=='runtime':
  for dep in ['assets','core']:a+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(a)
run([rustc,'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(W/'probe.exe')]);run([str(W/'probe.exe'),str(W)])
assert (W/'delivery-actual.bin').read_bytes()==de
a=(W/'actual.bin').read_bytes();assert a==expected,[(i,x,y)for i,(x,y)in enumerate(zip(a,expected))if x!=y][:20]
receipt=dict(status='passed',cases=stats,delivery_cases=delivery_cases,private_run=W.relative_to(P).as_posix(),production_pins=pins,source_pins={f.relative_to(P).as_posix():sha(f)for f in s.files+files+[P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',obj,P/'private/ghidra/validated/memory-.object3.bin',Path(__file__),Path(__file__).with_suffix('.rs')]},compared_bytes=len(a),boundary='47F2C before queue and animation; 48790 effect33 is captured allocation request, not delivery')
(W/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status='passed',cases=len(stats),requests=sum(c['transfers']for c in stats),private_run=receipt['private_run'])))



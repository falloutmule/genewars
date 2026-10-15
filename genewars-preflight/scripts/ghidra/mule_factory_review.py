"""Compare pure Rust Mule factory against literal5A210 before8AC60."""
from pathlib import Path
import importlib.util,json,struct,sys,uuid,pickle,subprocess,hashlib
P=Path(__file__).resolve().parents[2];G=P/'private/ghidra';W=P/'private/review/mule-factory-review'/uuid.uuid4().hex;W.mkdir(parents=True)
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
files=[G/'activation-rng-refs/instructions-0005a210.txt',G/'genepod-mule-initializer.txt',G/'construction-command/instructions-0001ee50.txt',G/'bones-frame/instructions-0005a1c0.txt',G/'player-creation/instructions-0001f480.txt',G/'terrain-map-refs/instructions-0007b240.txt']
class Stop(BaseException):pass
class Hook(dict):
 def __getitem__(self,k):
  if k==0x8ac60:raise Stop()
  return super().__getitem__(k)
vm=m.Source();vm.code.update(m.m.Model(files,b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]));vm.code=Hook(vm.code)
def put(s,a,n,v):s.update({a+i:b for i,b in enumerate(int(v&((1<<(8*n))-1)).to_bytes(n,'little'))})
def capture(state,seed,ring,count,owner,q,pos,parent,slot,frame):
 before=bytes(state[0x1542be+slot*124+i]for i in range(124));par=bytes(state[parent+i]for i in range(124))if parent else bytes(124)
 put(state,0x300004,4,parent);put(state,0x118782,4,count);put(state,0x11872c,1,ring)
 try:vm.run(counter=frame,seed=seed,state_before=state,entry=0x5a210,registers={'EAX':owner,'EDX':4,'EBX':pos,'ECX':q})
 except Stop:pass
 else:raise AssertionError('factory did not reach map insertion')
 ptr=0x1542be+slot*124;after=bytearray(vm.load(ptr+i,1)for i in range(124));after[31:39]=before[31:39]
 rng=struct.pack('<HBI',vm.load(0x11878a,2),vm.load(0x11872c,1),vm.load(0x118782,4))
 position=struct.unpack('<hh',struct.pack('<I',pos));ground=struct.unpack_from('<i',after,17)[0];shade=after[23]
 data=before+par+bytes([bool(parent)])+struct.pack('<HBhhBiBHBI',slot,owner,*position,q,ground,shade,seed,ring,count)
 return data,bytes(after)+rng
cases=[];inputs=[];expected=[]
for seed in [0,5428,10548,65535]:
 for fill in [0,165]:
  state={0x1542be+i:fill for i in range(300*124)}
  for slot in range(300):put(state,0x1542be+slot*124+6,1,0);put(state,0x1542be+slot*124+35,4,0x111fec)
  state.update({0x11c2be+i:0 for i in range(229376)})
  put(state,0x11c2b0,2,0);put(state,0x180e94,4,0x400000)
  data,out=capture(state,seed,4,19,0,35,(1024<<16)|1024,0,0,1);inputs.append(data);expected.append(out);cases.append({'kind':'synthetic-null','seed':seed,'prior_fill':fill})
actual=P/'private/review/krell-parent-mule-factory/7b74af10077b4051817ce14edd508af3/before-memory.pickle'
state=pickle.loads(actual.read_bytes());data,out=capture(state,1825,0,35139,4,50,1418492787,1393590,4,785);inputs.append(data);expected.append(out);cases.append({'kind':'actual785-parent2','slot':4})
obj=(G/'validated/memory-.object3.bin').read_bytes();trig=obj[0x7ee8:0x7ee8+8192]+obj[0x86e8:0x86e8+8192]
for i,(a,b)in enumerate(zip(inputs,expected)):(W/f'{i}.in').write_bytes(a+trig);(W/f'{i}.expected').write_bytes(b)
module=P/'rust/crates/genewars-runtime/src/mule_factory.rs'
wrapper=W/'review.rs';exe=W/'review.exe'
wrapper.write_text(r'''use std::io::{Read,Write};
mod blueprint{#[derive(Debug,Clone,Copy,PartialEq,Eq)]pub struct BlueprintRng{pub seed:u16,pub ring:u8,pub draw_count:u32}}
mod creature_world{pub type CreatureRecord=[u8;124];}
mod actor_activation{pub struct TrigTables{pub s:Vec<i32>,pub c:Vec<i32>}impl TrigTables{pub fn sine(&self)->&[i32]{&self.s}pub fn cosine(&self)->&[i32]{&self.c}}}
#[path="'''+module.as_posix()+r'''"]mod factory;
fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let prior=b[..124].try_into().unwrap();let p:&[u8;124]=b[124..248].try_into().unwrap();let mut o=249;macro_rules! get{($t:ty)=>{{let n=std::mem::size_of::<$t>();let x=<$t>::from_le_bytes(b[o..o+n].try_into().unwrap());o+=n;x}}}
let input=factory::MuleFactoryInput{slot:get!(u16),owner:get!(u8),position:[get!(i16),get!(i16)],quality:get!(u8),ground:get!(i32),shade:get!(u8)};let rng=blueprint::BlueprintRng{seed:get!(u16),ring:get!(u8),draw_count:get!(u32)};
let s=(0..2048).map(|_|get!(i32)).collect();let c=(0..2048).map(|_|get!(i32)).collect();let t=actor_activation::TrigTables{s,c};let q=factory::plan_mule_factory(prior,input,if b[248]!=0{Some(p)}else{None},rng,&t).unwrap();let mut out=q.record_after.to_vec();out.extend(q.rng_after.seed.to_le_bytes());out.push(q.rng_after.ring);out.extend(q.rng_after.draw_count.to_le_bytes());std::io::stdout().write_all(&out).unwrap();}''')
sys.path.insert(0,str(P/'scripts'));from cargo import environment
subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(wrapper),'-o',str(exe)],env=environment(),check=True)
for i,case in enumerate(cases):
 out=subprocess.check_output([str(exe)],input=(W/f'{i}.in').read_bytes());(W/f'{i}.actual').write_bytes(out)
 assert out==expected[i],(i,case,[(j,a,b)for j,(a,b)in enumerate(zip(expected[i],out))if a!=b][:20])
receipt={'status':'passed','cases':cases,'bytes_compared':sum(map(len,expected)),'source_pins':{p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files+[module]},'actual_input_pin':hashlib.sha256(actual.read_bytes()).hexdigest(),'private_run':W.relative_to(P).as_posix(),'scope':'Full124-byte record plus7-byte RNG, literal stop before8AC60, normalize only pointergap31..39. Syntheticnull35 prior0/A5 andactual785parent50. Map/count/caller outside comparison.','original_execution':False}
(P/'research/campaign/mule-factory-review.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','cases':len(cases),'bytes_compared':receipt['bytes_compared'],'private_run':receipt['private_run']}))

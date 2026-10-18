"""Literal57A10 movement-prefix and arrival dispatch boundary for opcode16."""
from pathlib import Path
import importlib.util,struct,json,subprocess,sys,uuid,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def main():
 w=P/'private/review/queued-architect'/uuid.uuid4().hex;w.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 class Source(m.Source):
  def get(self,o):
   if self.pc==0x57a3e:return 0x2e0000
   if o in ['0x0006fb60','0x000981d3','0x000a6390']:return 0x2e0000
   if o=='0x000419e0':return 0x2e0008
   return super().get(o)
 s=Source();files=[P/'private/ghidra/actor-motion'/f'instructions-000{x}.txt'for x in ['57a10','51930','1ee30']]+[P/'private/ghidra/construction-command'/f'instructions-000{x}.txt'for x in ['526c0','1ee40']]+[P/'private/ghidra/activation-rng-refs/instructions-00052340.txt']
 s.code.update(m.m.Model(files,b'').code);s.code.update({0x2e0000:('RET',[]),0x2e0008:('RET',['0x10'])});original=dict(s.code)
 inp=bytearray();expected=bytearray();details=[]
 for i in range(64):
  arrival=i>=32;a=bytearray(200);b=bytearray(36);a[6]=3;a[0x55]=8 if arrival or i%2 else 1;a[0x57]=1 if arrival or i%2 else 3;a[0x58]=2;a[0x70]=1;a[0x7e]=0;a[0x82]=1;a[0x83]=[1,3,6,11][i%4];b[21]=i%3;b[22]=0;b[25]=9;b[26]=4;b[24]=8;b[27]=2
  a[0x49]=32 if arrival else 16;struct.pack_into('<I',a,35,0x112194);struct.pack_into('<I',a,192,0x2d0000);struct.pack_into('<hhhh',a,7,19839,12927,21119,13439);a[92:96]=a[11:15]
  seed=5428+i;ring=i%32;draws=100+i;frame=[1,128,256,0][i%4]
  state={0x16360e+j:v for j,v in enumerate(a)};state.update({0x2d0000+j:v for j,v in enumerate(b)})
  for addr,data in [(0x11872c,bytes([ring])),(0x118782,struct.pack('<I',draws)),(0x2f0ff8,struct.pack('<I',0x16360e))]:state.update({addr+j:v for j,v in enumerate(data)})
  s.code=dict(original);s.code[0x57e0f if arrival else 0x583d1]=('JMP',['0xdeadbeef']);order=sorted(s.code);s.next=dict(zip(order,order[1:]))
  report,_,trace=s.run(entry=0x57b8f if arrival else 0x57a10,registers={'EAX':0x16360e,'EBP':0x2f1000},counter=frame,seed=seed,state_before=state)
  out=bytes(s.load(0x16360e+j,1)for j in range(200));bp=bytes(s.load(0x2d0000+j,1)for j in range(36));rng=struct.pack('<HBI',s.load(0x11878a,2),s.load(0x11872c,1),s.load(0x118782,4))
  inp+=bytes([arrival])+a+b+struct.pack('<IHBI',frame,seed,ring,draws);expected+=out+bp+rng;details.append(dict(arrival=arrival,steps=report['steps']))
 (w/'input.bin').write_bytes(inp)
 code=(P/'private/ghidra/validated/memory-.object1.bin').read_bytes();(w/'angles.bin').write_bytes(code[0x9e134:0x9e134+514])
 module=(P/'rust/crates/genewars-runtime/src/actor_motion.rs').as_posix()
 (w/'main.rs').write_text(f'#[path="{module}"]mod actor_motion;\n'+r'''
fn main(){let args:Vec<_>=std::env::args().collect();let input=std::fs::read(&args[1]).unwrap();let raw=std::fs::read(&args[2]).unwrap();let angles=std::array::from_fn(|i|u16::from_le_bytes([raw[i*2],raw[i*2+1]]));let mut out=Vec::new();for c in input.chunks_exact(248){let a=c[1..201].try_into().unwrap();let bp=c[201..237].try_into().unwrap();let frame=u32::from_le_bytes(c[237..241].try_into().unwrap());let rng=actor_motion::ActorCallbackRng{seed:u16::from_le_bytes(c[241..243].try_into().unwrap()),ring:c[243],draw_count:u32::from_le_bytes(c[244..248].try_into().unwrap())};let (after,b,r)=if c[0]!=0{let p=actor_motion::plan_actor_arrival_pending_architect(&a,&bp,rng).unwrap();(p.after,p.blueprint_after,p.rng_after)}else{let p=actor_motion::plan_actor_post_motion_prefix(&a,&bp,frame,&[960,576,1088,832],&angles).unwrap();(p.after,p.blueprint_after,rng)};out.extend(after);out.extend(b);out.extend(r.seed.to_le_bytes());out.push(r.ring);out.extend(r.draw_count.to_le_bytes());}std::fs::write(&args[3],out).unwrap();}
''')
 subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(w/'main.rs'),'-o',str(w/'probe.exe')],env=environment(),check=True)
 subprocess.run([str(w/'probe.exe'),str(w/'input.bin'),str(w/'angles.bin'),str(w/'actual.bin')],check=True)
 actual=(w/'actual.bin').read_bytes();assert actual==expected,next((i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b)
 result=dict(status='passed',cases=64,compared_bytes=len(actual),scope='Actual57A10 post6FB60 prefix through583D1; actual526C0 arrival through57E0F; excludes later507D0 composition, tested separately.',work=w.relative_to(P).as_posix(),details=details,pins={p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files+[Path(module)]})
 (P/'research/integration/queued-architect-motion.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='details'}))
if __name__=='__main__':main()

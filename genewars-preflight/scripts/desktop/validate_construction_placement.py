"""Literal 811A0 setter comparison with explicit two-call 74A50 query ports."""
from pathlib import Path
import importlib.util,json,hashlib,struct,subprocess,uuid,sys
P=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 work=P/'private/review/construction-placement'/uuid.uuid4().hex;work.mkdir(parents=True)
 spec=importlib.util.spec_from_file_location('original',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
 class Source(m.Source):
  def get(self,o):
   if o=='0x00074a50':
    self.queries.append((self.r['EAX'],self.r['EDX'],self.r['EBX'],self.r['ECX']))
    self.port_value=self.outputs[len(self.queries)-1];return 0x2e0000
   if o=='PORT_VALUE':return self.port_value
   if o=='0x000273b0':return 0x2e0008
   return super().get(o)
 model=Source();files=[P/'private/ghidra/construction-ui-refs'/f'instructions-000{x}.txt'for x in ['811a0','51960']]
 model.code.update(m.m.Model(files,b'').code);model.code.update({0x2e0000:('MOV',['EAX','PORT_VALUE']),0x2e0001:('RET',['0x4']),0x2e0008:('RET',[])})
 order=sorted(model.code);model.next=dict(zip(order,order[1:]));inputs=bytearray();expected=bytearray();cases=[]
 for i in range(64):
  actor=bytearray((j*37+i*11)&255 for j in range(200));actor[6]=3;actor[24]=0;actor[0x55]=1;actor[0x82]=0;struct.pack_into('<I',actor,192,0x2d0000)
  bp=bytearray(36);bp[22]=0;selector=[1,3,6,11][i%4];mode=i%16
  first=[-30000+i*101,21000-i*77];second=[first[0]+i,first[1]-i];requested=[19839+i,12927-i]
  packet=struct.pack('<HHHhhB',0,mode,selector<<8,*requested,16)
  state={0x16360e+j:b for j,b in enumerate(actor)};state.update({0x2d0000+j:b for j,b in enumerate(bp)});state.update({0x2c0000+j:b for j,b in enumerate(packet)});state.update({0x118740+j:b for j,b in enumerate(struct.pack('<I',0))})
  model.outputs=[struct.unpack('<I',struct.pack('<hh',*v))[0]for v in [first,second]];model.queries=[]
  report,_,trace=model.run(entry=0x811a0,registers={'EDX':0x2c0000},state_before=state)
  after=bytes(model.load(0x16360e+j,1)for j in range(200));assert len(model.queries)==2
  assert model.queries[0][1]==struct.unpack('<I',struct.pack('<hh',*requested))[0]
  assert model.queries[1][1]==model.outputs[0]
  assert all(q[0]==0x163615 and q[2]==actor[0x53] and q[3]==0x16360e for q in model.queries)
  inputs+=actor+bp+struct.pack('<HHhhhh',mode,selector<<8,*first,*second);expected+=after
  cases.append(dict(selector=selector,mode=mode,steps=report['steps'],resolver_calls=2))
 (work/'input.bin').write_bytes(inputs);(work/'expected.bin').write_bytes(expected)
 source=P/'rust/crates/genewars-runtime/src'
 harness='#[path="'+(source/'actor_construction.rs').as_posix()+'"] mod actor_construction;\n#[path="'+(source/'actor_construction_placement.rs').as_posix()+'"] mod actor_construction_placement;\n'+r'''
fn main(){let args:Vec<_>=std::env::args().collect();let input=std::fs::read(&args[1]).unwrap();let mut output=Vec::new();for c in input.chunks_exact(248){let a:[u8;200]=c[..200].try_into().unwrap();let b:[u8;36]=c[200..236].try_into().unwrap();let w=|p|u16::from_le_bytes(c[p..p+2].try_into().unwrap());let plan=actor_construction_placement::plan_architect_placement(&a,&b,w(236),w(238),[[w(240)as i16,w(242)as i16],[w(244)as i16,w(246)as i16]],true,false).unwrap();output.extend_from_slice(&plan.after);}std::fs::write(&args[2],output).unwrap();}
'''
 (work/'main.rs').write_text(harness)
 subprocess.run([str(P/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow',str(work/'main.rs'),'-o',str(work/'probe.exe')],check=True,cwd=P,env=environment())
 subprocess.run([str(work/'probe.exe'),str(work/'input.bin'),str(work/'actual.bin')],check=True)
 assert (work/'actual.bin').read_bytes()==expected
 result=dict(status='passed',cases=64,compared_actor_bytes=len(expected),scope='Literal811A0 and51960;74A50 two ordered query results are explicit ports. Owned-world search and screen placement are separate integration obligations.',pins={p.relative_to(P).as_posix():sha(p)for p in files+[source/'actor_construction.rs',source/'actor_construction_placement.rs']},work=work.relative_to(P).as_posix(),details=cases)
 (P/'research/integration/construction-placement.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k!='details'}))
if __name__=='__main__':main()

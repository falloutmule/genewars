"""Literal305F0/6D230 search comparison with explicit synthetic predicates."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,subprocess,struct,json,hashlib,sys
P=Path(__file__).resolve().parents[2];W=P/'private/review/creature-relocation';W.mkdir(exist_ok=True)
spec=importlib.util.spec_from_file_location('source',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
sys.path.insert(0,str(P/'scripts'));from cargo import environment
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
class Search(m.Source):
 def get(self,op):
  if self.pc==0x6d248 and op=='AL':
   if self.query_half:self.query_half=False;return self.answer
   point=tuple(m.signed(self.load(0x2f1010+2*i,2),16)for i in range(2));self.queries.append(point)
   self.answer=int(len(self.queries)-1>=self.threshold);self.query_half=True;return self.answer
  return super().get(op)
def main():
 files=[P/'private/ghidra/terrain-map-refs/instructions-000305f0.txt',P/'private/ghidra/initial-creature/instructions-0006d230.txt']
 model=Search();ext=m.m.Model(files,b'');model.code.update(ext.code);model.next=dict(zip(sorted(model.code),sorted(model.code)[1:]));model.code[0x74d90]=('RET',[])
 data=bytearray();cases=0;instructions=0;queries=0
 for point in [(0,0),(32767,32767),(-32768,-32768),(-1,12345),(15969,16010)]:
  for threshold in list(range(65))+[65535]:
   context=bytearray(64);struct.pack_into('<hh',context,4,*point);struct.pack_into('<I',context,0x1d,0x400000);struct.pack_into('<I',context,0x25,8);struct.pack_into('<I',context,0x2f,0x11276c);struct.pack_into('<I',context,0x38,0x1542be)
   model.threshold=threshold;model.queries=[];model.query_half=False;state={0x2f1000+i:b for i,b in enumerate(context)}
   report,_,_=model.run(2,state_before=state,entry=0x305f0,registers={'EAX':0x2f1000})
   found=model.saved_state[0x2f1033]!=0;result=tuple(m.signed(sum(model.saved_state[0x2f1034+2*i+j]<<(8*j)for j in range(2)),16)for i in range(2))if found else(0,0)
   data+=struct.pack('<hhHHBhh',*point,threshold,len(model.queries),found,*result)
   for q in model.queries:data+=struct.pack('<hh',*q)
   cases+=1;instructions+=report['steps'];queries+=len(model.queries)
 fixture=W/'search.bin';fixture.write_bytes(data)
 srcs=[p for name in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{name}/src').rglob('*.rs')];pins=[pin(p)for p in srcs];rustc=P/'private/tools/rust/cargo/bin/rustc.exe'
 def run(cmd):
  r=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace');return r.stdout.decode(errors='replace')
 for name in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{name}','--crate-type','rlib',str(P/f'rust/crates/genewars-{name}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{name}.rlib')]
  if name=='runtime':
   for d in ['assets','core']:args+=['--extern',f'genewars_{d}={W}/libgenewars_{d}.rlib']
  run(args)
 exe=W/'review.exe';run([str(rustc),'--edition=2021',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','--extern',f'genewars_runtime={W}/libgenewars_runtime.rlib','-o',str(exe)]);print(run([str(exe),str(fixture)]).strip());assert pins==[pin(p)for p in srcs]
 receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=cases,queries=queries,interpreted_instructions=instructions,fixture=pin(fixture),source_pins=pins,listing_pins=[pin(p)for p in files],review_pins=[pin(p)for p in [Path(__file__),Path(__file__).with_suffix('.rs'),Path(__file__).with_name('initial-creature-source.py')]],limits=['Explicit threshold predicates substitute only74D90 environment output; radial iteration/callback/choice execute through the pinned listing interpreter. No game or GUI execution.','The source completes the current group after success; last accepted in that group wins.','This checks the no75080-rounding, radius2048/eight-ring variant only.'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(receipt,indent=2)+'\n');tmp.replace(out)
if __name__=='__main__':main()

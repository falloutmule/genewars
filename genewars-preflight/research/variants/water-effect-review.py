"""Native water-effect factory/lifetime afterimages vs literal source fixtures."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/water-effect'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));return r.stdout.decode(errors='replace')
def main():
 print(run([sys.executable,'-B',str(Path(__file__).with_name('water-effect-source.py'))]))
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);print(run([str(W/'review.exe'),str(W)]));total=0
 for role in ['factory','update']:
  actual=(W/f'{role}-actual.bin').read_bytes();expected=(W/f'{role}-expected.bin').read_bytes();assert actual==expected,dict(role=role,byte=next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None));total+=len(actual)
 assert pins==[pin(p)for p in sources],'Source changed during native review';source=json.loads((W/'source.json').read_text())
 result=dict(status='passed-water-effect-factory-and-postphysics',utc=datetime.now(timezone.utc).isoformat(),factory_cases=source['factory_cases'],update_cases=source['update_cases'],compared_bytes=total,literal_instructions=source['literal_instructions'],source_pins=pins,listing_and_input_pins=source['pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('water-effect-source.py'))],private_outputs=[pin(W/f'{role}-{kind}.bin')for role in ['factory','update']for kind in ['input','expected','actual']],original_execution=False,gui_execution=False,limits=[source['scope'],'No full-pool replacement, owned-world cleanup, full6DF30 trajectory or rendering claim. Child22 allocation uses a NULL typed port; request arguments are source-checked.'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','factory_cases','update_cases','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

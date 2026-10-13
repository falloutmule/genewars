"""Independent literal/native selector0 state5 and selector1 state0 comparison."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,subprocess,sys,json,hashlib
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-onward'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 run([sys.executable,'-B',str(Path(__file__).with_name('family8-onward-source.py'))])
 base=P/'private/review/family8-state4'
 for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((base/name).read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources]
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)])
 total=0;outputs=[]
 for name in ['parent','child']:
  for actual,expected in [(f'{name}-actual.bin',f'{name}-expected.bin'),(f'{name}-actual-particles.bin',f'{name}-particles.bin')]:
   got=(W/actual).read_bytes();want=(W/expected).read_bytes();assert got==want,dict(output=actual,byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None),actual_length=len(got),expected_length=len(want));total+=len(got);outputs+=[pin(W/actual),pin(W/expected)]
 assert pins==[pin(p)for p in sources],'source changed during review'
 source=json.loads((W/'source.json').read_text());result=dict(status='passed-bounded-family8-onward',utc=datetime.now(timezone.utc).isoformat(),parent_wait_callbacks=len(source['timeline']['parent']),child_descent_callbacks=len(source['timeline']['child']),compared_bytes=total,literal_instructions=source['literal_instructions'],source_pins=pins,listing_pins=source['pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],outputs=outputs,original_execution=False,gui_execution=False,
  limits=['Parent57==0 wait only; nonzero57 player threshold branch not admitted','Source74D90 periodic current-point query compared against literal admitted map; readonly idle search has a separate proof and world binding remains external','Selector1 code1A factory is an explicit port; complete call parameters and two prefactory RNG draws compared; factory RNG/allocations external','114B0 call on arrival is an explicit required port; afterimage must not commit until owner applies that helper','No claim subsequent state1..3 or whole family scheduler; no source child-creation order inferred from isolated callback'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','parent_wait_callbacks','child_descent_callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

"""Production11AEB notification and12C64 group-departure comparison."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-parent'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=300);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 if '--draft' in sys.argv:raise SystemExit('Draft archived privately; this gate compiles production.')
 draft=False
 run([sys.executable,'-B',str(Path(__file__).with_name('family8-parent-source.py'))])
 for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');runtime=P/'rust/crates/genewars-runtime/src'
 for crate in ['assets','core','runtime']:
  lib=runtime/'lib.rs' if crate=='runtime' else P/f'rust/crates/genewars-{crate}/src/lib.rs'
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(lib),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)]);total=0
 for actual,expected in [('actual.bin','expected.bin'),('state8-actual.bin','state8-expected.bin')]:
  got=(W/actual).read_bytes();want=(W/expected).read_bytes();assert got==want,dict(file=actual,byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None));total+=len(got)
 assert pins==[pin(p)for p in sources],'source changed during review';source=json.loads((W/'source.json').read_text())
 result=dict(status='passed-bounded-family8-parent-notification-state8',draft=draft,utc=datetime.now(timezone.utc).isoformat(),notification_cases=source['notification_cases'],state8_callbacks=source['state8_callbacks'],compared_bytes=total,literal_instructions=source['literal_instructions'],source_pins=pins,listing_and_input_pins=source['pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('family8-parent-source.py'))],private_outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','state8-input.bin','state8-expected.bin','state8-actual.bin','state8-pools.bin','source.json','map.bin','angles.bin','bounce.bin','ethereal.anb']],original_execution=False,gui_execution=False,
  limits=['Input parent is actual combined before6442 slot2 after live child retirement. Later callbacks hold other families fixed.', 'Counter is signed32; threshold is zero-extended unsigned16. Negative branch RNG is interpreted; formatter/module work is an explicit variant request.', 'State8 admission is stationary. Allactive selector0 parents must be state8 before transition9 with twoRNG draws. Later state9 has a separate receipt; state6/state7 callbacks remain unsupported.', 'Native world supplies actual player counter, threshold, pool and query; separate idle proof admits both numeric query outcomes.'])
 out=W/'draft-validation.json' if draft else Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','draft','notification_cases','state8_callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

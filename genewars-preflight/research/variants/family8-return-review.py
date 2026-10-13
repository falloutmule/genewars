"""Production selector1 state3 comparison against private literal source."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-return'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 if '--draft' in sys.argv:raise SystemExit('Draft archived privately; this gate compiles production.')
 draft=False
 for args in [[],['--actual']]:run([sys.executable,'-B',str(Path(__file__).with_name('family8-return-source.py'))]+args)
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 runtime=P/'rust/crates/genewars-runtime/src'
 for crate in ['assets','core','runtime']:
  lib=runtime/'lib.rs' if crate=='runtime' else P/f'rust/crates/genewars-{crate}/src/lib.rs'
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(lib),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);total=cases=steps=0;outputs=[];listing_inputs={}
 for context in ['actual6290','isolated6408']:
  directory=W/context
  for name in ['angles.bin','bounce.bin','ethereal.anb']:(directory/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
  run([str(W/'review.exe'),str(directory)])
  for actual,expected in [('actual.bin','expected.bin'),('actual-particles.bin','particles.bin')]:
   got=(directory/actual).read_bytes();want=(directory/expected).read_bytes();assert got==want,dict(context=context,file=actual,byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None));total+=len(got)
  source=json.loads((directory/'source.json').read_text());cases+=source['cases'];steps+=source['literal_instructions'];listing_inputs.update(source['pins'])
  outputs.extend(pin(directory/f)for f in ['input.bin','expected.bin','actual.bin','particles.bin','actual-particles.bin','parents.bin','source.json','map.bin','angles.bin','bounce.bin','ethereal.anb'])
 assert pins==[pin(p)for p in sources],'source changed during review'
 result=dict(status='passed-bounded-family8-selector1-return',draft=draft,utc=datetime.now(timezone.utc).isoformat(),callbacks=cases,compared_bytes=total,literal_instructions=steps,source_pins=pins,listing_and_input_pins=listing_inputs,review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('family8-return-source.py'))],private_outputs=outputs,original_execution=False,gui_execution=False,
  limits=['First context is actual combined frame6290 slot6; second is isolated hover continuation at6408 slot7. Later callbacks hold other families fixed and particle factory is a typed port.', 'Numeric comparison normalizes first6 intrusive links; full native map and link source writes remain private. World adapter owns commit.', '103F0 retirement is an explicit typed port, separately independently reviewed by root. Child retirement must finish before parent57=1; pure plan does neither mutation.', 'Live parent must be active kind8 same owner selector0/state5/57zero; lookup identity checked. Missing/changed parent rejects.', 'Water/damage, arbitrary active references and other child states remain unsupported; readonly idle search has a separate proof.'])
 out=W/'draft-validation.json' if draft else Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','draft','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

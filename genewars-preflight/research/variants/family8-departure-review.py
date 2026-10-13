"""Production state9 ascent/retirement-request comparison to literal source."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-departure'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=300);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 run([sys.executable,'-B',str(Path(__file__).with_name('family8-parent-source.py'))])
 for args in [[],['--actual']]:run([sys.executable,'-B',str(Path(__file__).with_name('family8-departure-source.py'))]+args)
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);total=cases=steps=0;outputs=[];listing_inputs={}
 for directory in [W,W/'actual6567']:
  for name in ['angles.bin','bounce.bin','ethereal.anb']:(directory/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
  run([str(W/'review.exe'),str(directory)]);got=(directory/'actual.bin').read_bytes();want=(directory/'expected.bin').read_bytes();assert got==want,dict(context=str(directory),byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None));total+=len(got)
  source=json.loads((directory/'source.json').read_text());cases+=source['callbacks'];steps+=source['literal_instructions'];listing_inputs.update(source['pins']);outputs.extend(pin(directory/f)for f in ['input.bin','expected.bin','actual.bin','source.json','map.bin','angles.bin','bounce.bin','ethereal.anb'])
 assert pins==[pin(p)for p in sources],'source changed during review'
 result=dict(status='passed-bounded-family8-parent-state9-departure',utc=datetime.now(timezone.utc).isoformat(),callbacks=cases,compared_bytes=total,literal_instructions=steps,source_pins=pins,listing_and_input_pins=listing_inputs,review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('family8-departure-source.py'))],private_outputs=outputs,original_execution=False,gui_execution=False,
  limits=['Fourexplicit state8-ready seed contexts plus actual before6567slot0 from a27b449717204a89bd5535536ad2d53f. The latter is historical beforeactor-footstep RNG correction; no currentcombinedtimingclaim.', 'Later callbacks hold otherfamilies fixed. Retirement is an explicit103F0 port; the pure plan leaves kind8/ownlinks intact for the reviewedworldretirementadapter.', 'Stationary-to-upward selector0state9 only, accelerationZ640/desiredspeed-1 and null targetreferences. Water/damage and other callback states reject; readonly idle search has a separate proof.', 'Audio and queue request presence is compared. Retirement skips common audio/animation tail; selector0-specific103F0 stoprequests belong to owning retirement adapter.'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

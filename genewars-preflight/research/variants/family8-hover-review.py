"""Native positive-timer/empty-search comparison to private source listings."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-hover'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 run([sys.executable,'-B',str(Path(__file__).with_name('family8-hover-source.py'))])
 for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)]);total=0
 for actual,expected in [('actual.bin','expected.bin'),('actual-particles.bin','particles.bin')]:
  got=(W/actual).read_bytes();want=(W/expected).read_bytes();assert got==want,dict(file=actual,byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None));total+=len(got)
 assert pins==[pin(p)for p in sources],'source changed during review';source=json.loads((W/'source.json').read_text())
 result=dict(status='passed-bounded-family8-selector1-hover',utc=datetime.now(timezone.utc).isoformat(),callbacks=source['cases'],compared_bytes=total,literal_instructions=source['literal_instructions'],source_pins=pins,listing_and_input_pins=source['pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs'))],private_outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','particles.bin','actual-particles.bin','source.json','buildings.bin']],original_execution=False,gui_execution=False,
  limits=['First before6234 is actualcombined011 slot7; later isolated callbacks keep otherfamilies fixed and particle factories explicit','Pure numeric comparison normalizes source first6 intrusive-link bytes; sourcefullrecords/maps are privately interpreted, adapter ownscommits','All64 ownerbuildingsinactive and site_count0 required, closing timer expiry state1→state2→state3 with no accepted target','Periodic74D90 remains ownedworld responsibility; separate idle proof admits readonly scratch-work elision for either result','Repaired11720 listing replaces older overlapping misaligned entries in this explicit helper range'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

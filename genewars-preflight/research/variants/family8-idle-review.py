"""Compare the actual blocked-query callback and audit read-only search effects."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/family8-idle'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));print(r.stdout.decode(errors='replace'))
def main():
 run([sys.executable,'-B',str(Path(__file__).with_name('family8-idle-source.py'))])
 for name in ['angles.bin','bounce.bin','ethereal.anb']:(W/name).write_bytes((P/'private/review/family8-state4'/name).read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);run([str(W/'review.exe'),str(W)]);total=0
 for actual,expected in [('actual.bin','expected.bin'),('actual-particles.bin','particles.bin')]:
  got=(W/actual).read_bytes();want=(W/expected).read_bytes();assert got==want,dict(file=actual,byte=next((i for i,(a,b)in enumerate(zip(got,want))if a!=b),None));total+=len(got)
 assert pins==[pin(p)for p in sources],'source changed during review';source=json.loads((W/'source.json').read_text())
 result=dict(status='passed-family8-readonly-idle-search-elision',utc=datetime.now(timezone.utc).isoformat(),full_callbacks=1,prefix_cases=len(source['effect_audit']),compared_bytes=total,literal_instructions=source['literal_instructions']+source['effect_audit_instructions'],prefix_effect_audit=source['effect_audit'],source_pins=pins,listing_and_input_pins=source['pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('family8-idle-source.py'))],private_outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','particles.bin','actual-particles.bin','source.json','buildings.bin','map.bin','angles.bin','bounce.bin','ethereal.anb']],original_execution=False,gui_execution=False,
  limits=['Actual corrected-footstep before6332slot4 from579c9bdb947e4419926cb281b7bf0f38. Prefixaudits also varyavoidmask/transientflags/nonqueryphase.', '305F0/6D230 only query admittedworld and writesearchscratch; foundpoint passedto family8+A8RET. No record/map/RNG effects beyond transient49bitclear.', 'Numericport elidesreadonlyscratchwork for either searchoutcome. Actualcurrentquery stillruns inworldadapter; its unsupportedbindings retainrejection. This is notquerycount/timing parity.', 'First6 intrusive links normalized forpurecallback comparison; worldcommit andotherfamilies remain separate.'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','full_callbacks','prefix_cases','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

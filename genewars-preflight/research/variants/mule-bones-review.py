"""Native water-effect factory/lifetime afterimages vs literal source fixtures."""
from pathlib import Path
from datetime import datetime,timezone
import sys,subprocess,hashlib,json
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
W=P/'private/review/mule-bones'
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));return r.stdout.decode(errors='replace')
def main():
 print(run([sys.executable,'-B',str(Path(__file__).with_name('mule-bones-source.py'))]))
 for name in ['angles.bin','bounce.bin','mule.anb']:(W/name).write_bytes((P/'private/review/mule-mortality'/name).read_bytes())
 (W/'bones.anb').write_bytes((P/'private/integration/sprites/2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f.anb').read_bytes())
 sources=[p for crate in ['assets','core','runtime']for p in(P/f'rust/crates/genewars-{crate}/src').rglob('*.rs')];pins=[pin(p)for p in sources];rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime']:
  args=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={W}','-o',str(W/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
  run(args)
 args=[rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={W}','-o',str(W/'review.exe')]
 for dep in ['assets','core','runtime']:args+=['--extern',f'genewars_{dep}={W}/libgenewars_{dep}.rlib']
 run(args);print(run([str(W/'review.exe'),str(W)]));total=0
 actual=(W/'actual.bin').read_bytes();expected=(W/'expected.bin').read_bytes();assert actual==expected,dict(byte=next((i for i,(a,b)in enumerate(zip(actual,expected))if a!=b),None));total=len(actual)
 assert pins==[pin(p)for p in sources],'Source changed during native review';source=json.loads((W/'source.json').read_text())
 result=dict(status='passed-bounded-mule-bones-transition',utc=datetime.now(timezone.utc).isoformat(),callbacks=source['callbacks'],compared_bytes=total,literal_instructions=source['literal_instructions'],source_pins=pins,listing_and_input_pins=source['listing_pins'],review_pins=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('mule-bones-source.py'))],private_outputs=[pin(W/f)for f in ['input.bin','expected.bin','actual.bin','source.json','map.bin','bones.anb']],original_execution=False,gui_execution=False,limits=[source['scope'],'Full124-byte numeric records, pointer gaps preserved; no owned-world interleaving or rendering claim. Conversion requires explicitly hash-bound BONES metadata; final kind1 retirement remains rejected atomically.'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','callbacks','compared_bytes','literal_instructions']}))
if __name__=='__main__':main()

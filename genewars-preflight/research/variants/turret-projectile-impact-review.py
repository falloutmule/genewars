"""Reproducible bounded impact comparison; not a whole-world combat claim."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,sys,shutil
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,text=True,timeout=120)
 if r.returncode:raise RuntimeError(f'{args[0]} failed: {r.stdout}\n{r.stderr}')
 return r.stdout
def main():
 source=Path(__file__).with_name('turret-projectile-impact-source.py');probe=Path(__file__).with_suffix('.rs')
 result=json.loads(run([sys.executable,'-B',str(source)]).strip().splitlines()[-1]);w=P/result['private_run'];evidence=json.loads((w/'damage-source.json').read_text())
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');implementation_pins={}
 for crate in ['assets','core','runtime']:
  target=w/f'source/{crate}';shutil.copytree(P/f'rust/crates/genewars-{crate}/src',target)
  for p in target.rglob('*.rs'):implementation_pins[f'{crate}/{p.relative_to(target).as_posix()}']=sha(p)
  command=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(target/'lib.rs'),'-L',f'dependency={w}','-o',str(w/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:command+=['--extern',f'genewars_{dep}={w}/libgenewars_{dep}.rlib']
  run(command)
 run([rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(probe),'-L',f'dependency={w}','--extern',f'genewars_runtime={w}/libgenewars_runtime.rlib','-o',str(w/'impact.exe')]);run([str(w/'impact.exe'),str(w)])
 actual=(w/'damage-actual.bin').read_bytes();expected=(w/'damage-expected.bin').read_bytes()
 assert actual==expected,[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:16]
 pins=evidence['pins']|{p.relative_to(P).as_posix():sha(p)for p in [probe,Path(__file__),P/'research/variants/initial-creature-source.py',P/'research/variants/actor-environment-terrain-target.py',P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin']}
 receipt=dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),cases=result['cases'],compared_bytes=len(actual),literal_instructions=result['instructions'],private_run=result['private_run'],pins=pins,implementation_pins=implementation_pins,scope=evidence['scope'],original_execution=False,native_game_execution=False,production_integration='Pure runtime planner only; firing disabled',remaining=['Subsequent health0/state18 mortality update','Other recipient states, species and linked work cancellation','Multiple-recipient traversal and pool saturation','Projectile physics/collision, child creation and retirement in one atomic shared-world callback'])
 Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');(w/'impact-result.json').write_text(json.dumps(receipt,indent=2)+'\n')
 print(json.dumps({k:receipt[k]for k in ['status','cases','compared_bytes','literal_instructions','private_run']}))
if __name__=='__main__':main()

"""Compare private projectile numeric draft with separately executed listings."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,shutil,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(args):
 r=subprocess.run(args,cwd=P,env=environment(),capture_output=True,timeout=240);assert r.returncode==0,(r.stdout.decode(errors='replace'),r.stderr.decode(errors='replace'));return r.stdout.decode(errors='replace')
def main():
 source=Path(__file__).with_name('turret-projectile-source.py');result=json.loads(run([sys.executable,'-B',str(source)]).strip().splitlines()[-1]);w=P/result['private_run'];source_receipt=json.loads((w/'source.json').read_text());draft=P/'private/review/turret-geometry/candidate.rs';rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe');pins={}
 for crate in ['assets','core','runtime']:
  target=w/f'source/{crate}';shutil.copytree(P/f'rust/crates/genewars-{crate}/src',target)
  if crate=='runtime':shutil.copyfile(draft,target/'turret_projectile.rs');lib=target/'lib.rs';lib.write_bytes(lib.read_bytes()+b'\npub mod turret_projectile;\n')
  for p in target.rglob('*.rs'):pins[f'{crate}/{p.relative_to(target).as_posix()}']=sha(p)
  cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','-C','overflow-checks=yes','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(target/'lib.rs'),'-L',f'dependency={w}','-o',str(w/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dep in ['assets','core']:cmd+=['--extern',f'genewars_{dep}={w}/libgenewars_{dep}.rlib']
  run(cmd)
 run([rustc,'--edition=2021','-O','-C','overflow-checks=yes',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={w}','--extern',f'genewars_runtime={w}/libgenewars_runtime.rlib','-o',str(w/'probe.exe')]);print(run([str(w/'probe.exe'),str(w)]))
 total=0
 for role in ['factory','update']:
  a=(w/f'{role}-actual.bin').read_bytes();b=(w/f'{role}-expected.bin').read_bytes();assert a==b,(role,len(a),len(b),[(i,x,y)for i,(x,y)in enumerate(zip(a,b))if x!=y][:16]);total+=len(a)
 receipt=dict(status='passed-private-projectile42-prefix',utc=datetime.now(timezone.utc).isoformat(),factory_cases=64,postphysics_prefixes=240,compared_bytes=total,literal_instructions=source_receipt['literal_instructions'],implementation_pins=pins,pins=source_receipt['pins']|{p.relative_to(P).as_posix():sha(p)for p in [draft,Path(__file__),Path(__file__).with_suffix('.rs')]},private_run=w.relative_to(P).as_posix(),scope=source_receipt['scope'],original_execution=False,gui_execution=False)
 (w/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k]for k in ['status','compared_bytes','literal_instructions','private_run']}))
if __name__=='__main__':main()

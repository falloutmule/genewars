"""Read-only fresh startup capture; frozen code."""
from pathlib import Path
import sys,hashlib,json,shutil,subprocess,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 work=P/'private/review/family8-fresh-stall'/uuid.uuid4().hex;work.mkdir(parents=True);pins={}
 def run(cmd):
  r=subprocess.run(cmd,cwd=P,env=environment(),capture_output=True,text=True,timeout=180)
  assert r.returncode==0,(r.stdout,r.stderr)
  return r.stdout
 for crate in ['assets','core','runtime','render']:
  src=P/f'rust/crates/genewars-{crate}/src';target=work/f'source/{crate}';shutil.copytree(src,target)
  for f in target.rglob('*.rs'):pins[(src/f.relative_to(target)).relative_to(P).as_posix()]=sha(f)
 src=P/'rust/crates/genewars-desktop/src/bin/support';target=work/'support';shutil.copytree(src,target)
 for f in target.rglob('*.rs'):pins[(src/f.relative_to(target)).relative_to(P).as_posix()]=sha(f)
 catalog=P/'research/integration/runtime-resource-catalog.tsv';shutil.copy2(catalog,work/'catalog.tsv');pins[catalog.relative_to(P).as_posix()]=sha(catalog)
 f=target/'runtime_resources.rs';text=f.read_text().replace('../../../../../../research/integration/runtime-resource-catalog.tsv','../catalog.tsv');f.write_text(text)
 with (target/'shared_scene.rs').open('a')as f:f.write('\nimpl SharedMission {pub fn fixture_jobs(&self)->Vec<u8>{let mut v=vec![self.terrain_jobs.count()];for r in self.terrain_jobs.records(){v.extend(r)}v}}\n')
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime','render']:
  cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}','-o',str(work/f'libgenewars_{crate}.rlib')]
  for dep in (['assets','core']if crate=='runtime'else['assets']if crate=='render'else[]):cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
  run(cmd)
 shutil.copy2(P/'research/variants/family8-fresh-stall.rs',work/'main.rs')
 cmd=[rustc,'--edition=2021','--cap-lints=allow','-O',str(work/'main.rs'),'-L',f'dependency={work}','-o',str(work/'probe.exe')]
 for crate in ['assets','core','runtime','render']:cmd+=['--extern',f'genewars_{crate}={work}/libgenewars_{crate}.rlib']
 run(cmd);print('frozen compilation ready',work,flush=True);print(run([str(work/'probe.exe'),str(P),str(work),*(sys.argv[1:])]),flush=True)
 (work/'pins.json').write_text(json.dumps(pins,indent=2))
 (P/'private/review/family8-fresh-stall/latest.txt').write_text(str(work))
if __name__=='__main__':main()

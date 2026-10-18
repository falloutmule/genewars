"""Read-only fresh startup capture; frozen code."""
from pathlib import Path
import sys,hashlib,json,shutil,subprocess,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 work=P/'private/review/fresh-startup-stall'/uuid.uuid4().hex;work.mkdir(parents=True);pins={}
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
 shutil.copy2(P/'scripts/desktop/fresh_startup_stall_probe.rs',work/'main.rs')
 cmd=[rustc,'--edition=2021','--cap-lints=allow','-O',str(work/'main.rs'),'-L',f'dependency={work}','-o',str(work/'probe.exe')]
 for crate in ['assets','core','runtime','render']:cmd+=['--extern',f'genewars_{crate}={work}/libgenewars_{crate}.rlib']
 run(cmd);print('frozen compilation ready',work,flush=True);print(run([str(work/'probe.exe'),str(P),str(work)]),flush=True)
 run([sys.executable,'-B','research/variants/creature-hungry-source.py','95','--snapshot='+str((work/'95').relative_to(P))])
 before=work/'95';after=work/'95-callback'
 expected=bytearray((before/'source-after95.bin').read_bytes());actual=(after/'creatures.bin').read_bytes();expected[31:39]=actual[31:39]
 assert expected==actual[:124], 'Mule native record differs from literal source'
 assert actual[124:]==(before/'creatures.bin').read_bytes()[124:]
 backing=bytearray((before/'backing.bin').read_bytes());backing[:0x355e]=(before/'source-prefix-after95.bin').read_bytes();backing[0x355e:0x3b55e]=(before/'source-map-after95.bin').read_bytes();backing[0x43d4e:0x48f4e]=(before/'source-plants-after95.bin').read_bytes()
 assert backing==(after/'backing.bin').read_bytes()
 for name in ['actors.bin','buildings.bin','effects.bin','family8.bin','players.bin','jobs.bin']:assert (before/name).read_bytes()==(after/name).read_bytes(),name
 import struct
 trace=json.loads((before/'source-trace95.json').read_bytes());r=trace['report']
 assert struct.unpack('<HBI',(after/'rng.bin').read_bytes())==(r['rng_seed_after'],r['rng_ring_after'],r['rng_draw_count_after'])
 result={'status':'passed-fresh-startup95-source-callback','scope':'Actual fresh CD frame95: living injured neutral Mule rechoice. No substituted RNG or internal world edits. Source renderer/portrait ports remain those of the existing literal validator.','source_instructions':r['steps'],'source_trace_sha256':sha(before/'source-trace95.json'),'private_run':work.relative_to(P).as_posix(),'source_pins':pins,'captures':{}}
 for folder in work.iterdir():
  if folder.is_dir()and(folder/'actor0.bin').exists():result['captures'][folder.name]={f.name:sha(f)for f in folder.iterdir()if f.is_file()}
 (P/'research/integration/fresh-startup-stall-capture.json').write_text(json.dumps(result,indent=2)+'\n')
if __name__=='__main__':main()

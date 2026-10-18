"""Read-only full-world capture after ordinary Distributor commands; frozen code."""
from pathlib import Path
import sys,hashlib,json,shutil,subprocess,uuid
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
 work=P/'private/review/blocked-photon-site'/uuid.uuid4().hex;work.mkdir(parents=True);pins={}
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
 with (work/'source/runtime/actor_world.rs').open('a')as f:f.write('\nimpl ActorWorld {pub fn fixture_architect_motion(&mut self,id:u16,frame:u32,context:LandingPhysicsContext,physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables)->Result<ArchitectMotionReceipt,ConstructionWorldError>{self.step_architect_motion(id,frame,context,physics,trig)} pub fn fixture_blocked_callback(&mut self,resource:&crate::actor_resources::ActorAnimationDatabase,physics:&crate::physics_tables::PhysicsTables,trig:&TrigTables)->Result<ArchitectStage0Receipt,ConstructionWorldError>{let idle=crate::actor_motion::ActorIdleContext{runtime_mode:0,override_180d80:0,runtime_count_11c2bc:word(&self.backing,0x355c)as u16,player_mode:self.players[0].bytes()[2]};self.step_blocked_site_callback(0,2523,LandingPhysicsContext{gravity:1024,water_height:1632},physics,trig,&[0;8192],resource,idle,(true,true))}}\n')
 rustc=str(P/'private/tools/rust/cargo/bin/rustc.exe')
 for crate in ['assets','core','runtime','render']:
  cmd=[rustc,'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(work/f'source/{crate}/lib.rs'),'-L',f'dependency={work}','-o',str(work/f'libgenewars_{crate}.rlib')]
  for dep in (['assets','core']if crate=='runtime'else['assets']if crate=='render'else[]):cmd+=['--extern',f'genewars_{dep}={work}/libgenewars_{dep}.rlib']
  run(cmd)
 shutil.copy2(P/'scripts/desktop/blocked_photon_site_probe.rs',work/'main.rs')
 cmd=[rustc,'--edition=2021','--cap-lints=allow','-O',str(work/'main.rs'),'-L',f'dependency={work}','-o',str(work/'probe.exe')]
 for crate in ['assets','core','runtime','render']:cmd+=['--extern',f'genewars_{crate}={work}/libgenewars_{crate}.rlib']
 run(cmd);print('frozen compilation ready',work,flush=True);print(run([str(work/'probe.exe'),str(P),str(work)]),flush=True)
 result={'status':'captured','scope':'Ordinary-command fresh Photon capture; additional2523-postmotion diagnostic applies one isolated cloned-world physics step then restores original world before replay continuation','private_run':work.relative_to(P).as_posix(),'source_pins':pins,'captures':{}}
 for folder in work.iterdir():
  if folder.is_dir()and(folder/'actor0.bin').exists():result['captures'][folder.name]={f.name:sha(f)for f in folder.iterdir()if f.is_file()}
 (P/'research/integration/blocked-photon-site-capture.json').write_text(json.dumps(result,indent=2)+'\n')
if __name__=='__main__':main()

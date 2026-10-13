"""Independent comparison of owned actual environment query APIs; no GUI."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,importlib.util,json,shutil,subprocess,sys
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'))
from cargo import environment
spec=importlib.util.spec_from_file_location('environment_oracle',Path(__file__).with_name('actor-environment-contract.py'));oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
WORK=P/'private/review/actor-environment-native'
def pin(path):return dict(path=path.relative_to(P).as_posix(),sha256=hashlib.sha256(path.read_bytes()).hexdigest())
def main():
 WORK.mkdir(parents=True,exist_ok=True);mission=P/'private/integration/game-runtime/terrain/CONQ001.DAT';data=mission.read_bytes();assert hashlib.sha256(data).hexdigest()=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2';region=data[0x355e:0x355e+229376]
 sources=[P/'rust/crates/genewars-runtime/src'/s for s in ['actor_world.rs','actor_pool.rs','world.rs','actor_activation.rs','player_state.rs','actor_initialization.rs','blueprint.rs','identity.rs']];before=[pin(p)for p in sources]
 shutil.copyfile(mission,WORK/'mission.bin');rustc=P/'private/tools/rust/cargo/bin/rustc.exe';exe=WORK/'review.exe'
 for crate in ['assets','core','runtime']:
  args=[str(rustc),'--edition=2021','--cap-lints=allow','-O','--crate-name',f'genewars_{crate}','--crate-type','rlib',str(P/f'rust/crates/genewars-{crate}/src/lib.rs'),'-L',f'dependency={WORK}','-o',str(WORK/f'libgenewars_{crate}.rlib')]
  if crate=='runtime':
   for dependency in ['assets','core']:args+=['--extern',f'genewars_{dependency}={WORK}/libgenewars_{dependency}.rlib']
  r=subprocess.run(args,env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 args=[str(rustc),'--edition=2021','--cap-lints=allow','-O',str(Path(__file__).with_suffix('.rs')),'-L',f'dependency={WORK}','-o',str(exe)]
 for dependency in ['assets','core','runtime']:args+=['--extern',f'genewars_{dependency}={WORK}/libgenewars_{dependency}.rlib']
 for cmd in [args,[str(exe),str(WORK)]]:
  r=subprocess.run(cmd,env=environment(),capture_output=True,timeout=90);assert r.returncode==0,r.stderr.decode(errors='replace')
 expected=bytearray();terraincases=0;blockercases=0
 for y in range(128):
  for x in range(128):
   for fx,fy in [(0,0),(127,127),(128,0),(0,128),(128,128),(255,255),(127,128),(128,127),(255,0)]:expected.extend([oracle.terrain_class(region,[x*256+fx,y*256+fy]),0]);terraincases+=1
   direct=oracle.initial_blocker(data,y*128+x);terrain=oracle.terrain_class(region,[x*256+128,y*256+127]);envweight=0 if terrain==0 else direct
   for _ in range(2):expected.extend(direct.to_bytes(4,'little')+bytes([terrain])+envweight.to_bytes(4,'little'));blockercases+=1
 for raw in range(65536):
  point=[oracle.struct.unpack('<h',(raw&65535).to_bytes(2,'little'))[0],oracle.struct.unpack('<h',(raw^0xa55a).to_bytes(2,'little'))[0]];expected.extend([oracle.terrain_class(region,point),0]);terraincases+=1
 actual=(WORK/'query-output.bin').read_bytes();assert actual==expected, next((i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b)
 assert before==[pin(p)for p in sources],'production changed during query review'
 result=dict(status='passed-independent-native-actor-environment-review',utc=datetime.now(timezone.utc).isoformat(),terrain_cases=terraincases,obstacle_null_cases=terraincases,blocker_cases=blockercases,placement_environment_cases=blockercases,invalid_actor_cases=6,query_output_bytes=len(actual),query_output_sha256=hashlib.sha256(actual).hexdigest(),source_pins=before,mission=pin(mission),harness=pin(exe),review_sources=[pin(Path(__file__)),pin(Path(__file__).with_suffix('.rs')),pin(Path(__file__).with_name('actor-environment-contract.py'))],original_execution=False,gui_execution=False,limits=['Exact initial map/list fields with two independently prepared native actors/categories0/2','Fresh progress0 admission creates only categories0/2; category1 target exemption, actor-ignore bit and nonclear building flag require separately admitted fixtures; root unit tests separate','No true terrain unprojection or original consumption claim','Queries proved readonly by native whole ActorWorld equality'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,indent=2)+'\n');tmp.replace(out);print(json.dumps({k:result[k]for k in ['status','terrain_cases','blocker_cases','placement_environment_cases','query_output_bytes']}))
if __name__=='__main__':main()

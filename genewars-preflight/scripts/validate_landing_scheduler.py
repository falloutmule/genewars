"""Actual-map composition regression; component oracles remain separate."""
from pathlib import Path
from datetime import datetime,timezone
import csv,io,json,shutil,sys
from validate_landing_world import ROOT,sha,pin,run
from physics_inputs import stage
from cargo import environment

WORK=ROOT/'private/integration/landing-scheduler-validation'
def main():
    WORK.mkdir(parents=True,exist_ok=True)
    for path in stage().glob('*.bin'):shutil.copyfile(path,WORK/path.name)
    source=ROOT/'private/integration/game-runtime/terrain/CONQ001.DAT'
    assert sha(source.read_bytes())=='72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    shutil.copyfile(source,WORK/'mission.bin')
    build=run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime','--offline','--locked','--message-format=json'],cwd=ROOT)
    artifacts={}
    for line in build.stdout.splitlines():
        item=json.loads(line)
        if item.get('reason')=='compiler-artifact' and 'lib' in item['target']['kind']:
            artifacts[item['target']['name']]=next(p for p in item['filenames'] if p.endswith('.rlib'))
    harness=ROOT/'scripts/landing_scheduler_probe.rs';exe=WORK/'probe.exe';env=environment()
    cmd=[str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021',str(harness),'-L',f'dependency={ROOT}/private/build/rust-target/debug/deps','-o',str(exe)]
    for name in ['genewars_assets','genewars_runtime']:cmd+=['--extern',f'{name}={artifacts[name]}']
    run(cmd,cwd=ROOT,env=env);result=run([str(exe),str(WORK)],cwd=ROOT,env=env,timeout=120)
    (WORK/'passes.csv').write_bytes(result.stdout);rows=list(csv.DictReader(io.StringIO(result.stdout.decode())))
    assert len(rows)==5 and [int(r['seed'])for r in rows]==[0,1,5428,32768,65535]
    for r in rows:
        assert [int(r[k])for k in ['passes','created','retired','contact_pass','rng_draws','quiet_timer']]==[289,188,188,189,752,50]
        assert r['rollback']=='true' and int(r['max_active'])<60
    sources=[ROOT/'rust/crates/genewars-runtime/src'/name for name in ['actor_world.rs','actor_world/effect_updates.rs','effect_pool.rs','landing.rs','landing_effect_lifecycle.rs','physics_tables.rs']]
    report=dict(status='passed',verified_at=datetime.now(timezone.utc).isoformat(),sessions=rows,total_passes=1445,created_and_retired=940,callback_draws=3760,source_pins=[pin(source)],implementation_pins=[pin(p)for p in sources+[harness,Path(__file__)]],output_sha256=sha(result.stdout),scope='Actual CONQ map, explicit synthetic player/native-prior/RNG inputs, native-empty UI. Complete bounded family4 numeric primary/newborn passes through quiet timer50, preserving shared links and cleanup. Players have no actors; contact proves the zero-slot activation counter branch. Kind1/plant callbacks, render admission, palette/audio execution and full startup are outside this regression. Timer49 failure is atomic; no original executable ran.')
    (ROOT/'research/integration/landing-scheduler-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:report[k] for k in ['status','total_passes','created_and_retired','callback_draws']}))
if __name__=='__main__':main()

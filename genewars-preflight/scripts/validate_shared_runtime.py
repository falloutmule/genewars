"""Native owned landing, rendering and pending-command integration; no original execution."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, shutil, subprocess, sys, uuid
from runtime_inputs import ROOT, stage

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--window-smoke',action='store_true');parser.add_argument('--release',action='store_true');args=parser.parse_args()
    bundle, _ = stage()
    work = ROOT/'private/integration/shared-runtime-validation'/uuid.uuid4().hex
    work.mkdir(parents=True)
    paths=sorted((ROOT/'rust/crates').rglob('*.rs'))+sorted((ROOT/'rust').rglob('Cargo.toml'))
    paths += [ROOT/'rust/Cargo.lock',ROOT/'research/integration/runtime-resource-catalog.tsv',Path(__file__)]
    pins=[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p))for p in paths]
    build_command=[sys.executable,'-B','scripts/cargo.py','build','-p','genewars-desktop','--bin','genewars-game','--offline','--locked']
    if args.release:build_command+=['--release']
    build=subprocess.run(build_command,cwd=ROOT,capture_output=True,text=True,timeout=180)
    assert build.returncode==0,build.stderr
    assert pins==[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p))for p in paths],'sources changed during build; rerun with a stable compilation snapshot'
    profile='release' if args.release else 'debug'
    binary=work/'genewars-game.exe';shutil.copyfile(ROOT/f'private/build/rust-target/{profile}/genewars-game.exe',binary)
    cases = []
    def run(name, replay, seed=5428, bmp=False, window=False):
        folder=work/name;folder.mkdir()
        script=folder/'replay.txt';script.write_text(replay,encoding='utf-8')
        command=[str(binary),str(bundle),'--landing-seed',str(seed),'--window-replay' if window else '--headless',str(script),'--report',str(folder/'report.json')]
        if window:command+=['--frames','3','--seconds','20']
        if bmp:command+=['--bmp',str(folder/'frame.bmp')]
        result=subprocess.run(command,capture_output=True,text=True,timeout=240)
        assert result.returncode==0,(name,result.stderr)
        report=json.loads(result.stdout)
        if window:assert report['presented_frames']==3,report
        assert report==json.loads((folder/'report.json').read_bytes())
        assert report['scene']=='Mission',(name,report)
        mission=report['mission']
        assert mission['world_mode']=='landing-study' and mission['blocked'] is None,(name,mission)
        assert not report['gameplay'] and not report['full_world_tick']
        assert mission['other_families_frozen'] and mission['actor_movement'] and mission['plant_updates'] and mission['creature_ai']
        cases.append(dict(name=name,mission=mission,error=report['error'],private_output=folder.relative_to(ROOT).as_posix()))
        print(json.dumps(dict(case=name,steps=mission['steps'],effects=mission['active_effects'],error=report['error'])),flush=True)
        return report
    start='click 312 140\nload\n'
    initial=run('initial',start,bmp=True)
    assert initial['mission']['rng_draws']==64 and initial['mission']['active_effects']==1
    one=run('one',start+'tick 1\n',bmp=True)
    assert one['mission']['steps']==1 and one['mission']['active_effects']==2
    assert one['mission']['world_sha256']!=initial['mission']['world_sha256']
    early=run('early-selection',start+'select 0\n')
    assert early['error']=='actor is not active for this player'
    assert early['mission']==initial['mission']
    contact=run('contact',start+'tick 189\nselect 0\n',bmp=True,window=args.window_smoke)
    assert contact['error'] is None and contact['mission']['selected_actor']==0
    command=run('command',start+'tick 189\nselect 0\norder 320 240\n',bmp=True)
    assert command['error'] is None and command['mission']['issued_move_commands']==1
    assert command['mission']['world_sha256']!=contact['mission']['world_sha256']
    assert command['mission']['rgb_sha256']==contact['mission']['rgb_sha256']
    moved=run('command-arrival',start+'tick 189\nselect 0\norder 320 240\ntick 100\n',bmp=True)
    actor=next(a for a in moved['mission']['actor_states']if a['slot']==0)
    requested=next(a for a in command['mission']['actor_states']if a['slot']==0)['pending_destination']
    assert actor['state']==1 and actor['position']==requested and not actor['pending_move']
    assert moved['mission']['issued_move_commands']==1
    assert moved['mission']['rgb_sha256']!=command['mission']['rgb_sha256']
    full=run('full',start+'tick 450\n',bmp=True)
    assert full['mission']['steps']==450 and full['mission']['active_effects']==0
    assert full['mission']['changed_world_frames']>100
    mule=full['mission']['creature_states'][0]
    assert mule['state']==7 and mule['animation']==19 and mule['target_kind']==2
    assert mule['position']!=initial['mission']['creature_states'][0]['position']
    assert all(a['state']==1 and a['position']==a['destination'] for a in full['mission']['actor_states'])
    assert any(a['position']!=[20736,13568]for a in full['mission']['actor_states'])
    repeat=run('repeat',start+'tick 450\n')
    assert repeat['mission']==full['mission']
    pan=run('pan-return',start+'pan 256 0\npan -256 0\n')
    assert pan['mission']==initial['mission']
    for seed in [0,65535]:
        result=run('seed-'+str(seed),start+'tick 450\n',seed)
        assert result['mission']['steps']==450 and result['mission']['active_effects']==0
        mule=result['mission']['creature_states'][0]
        if seed==0:
            assert mule['state']==8 and mule['target_kind']==0 and result['mission']['plants']==26
        else:
            assert mule['state']==7 and mule['target_kind']==2
    reject=subprocess.run([str(binary),str(bundle),'--landing-seed','1','--world-output',str(work/'incomplete-save.bin')],capture_output=True,text=True,timeout=10)
    assert reject.returncode!=0 and 'full-state export' in reject.stderr and not (work/'incomplete-save.bin').exists()
    receipt=dict(status='passed',verified_at=datetime.now(timezone.utc).isoformat(),binary_sha256=sha(binary),build_profile=profile,
        cases=cases,case_count=len(cases)+1,full_state_export_rejected=True,mission_window_presented_three_frames=args.window_smoke,
        implementation_pins=pins,
        scope='Explicit seeded world study: numeric reveal, source factory/contact activates actors; player-list actor callbacks consume ground orders, move, arrive and idle; plants, empty-building bookkeeping, mission countdown and full family4 lifecycle run in admitted source order. The neutral Mule shares RNG in source family order, searches for food, approaches, feeds and retires consumed plants; its admitted states are rendered through the verified wrapper. Compilation snapshot pinned before/after build and binary copied privately before tests. Other creature kinds, other families and presentation requests remain pending. No full-game/startup parity or original execution.')
    (ROOT/'research/integration/shared-runtime-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',cases=receipt['case_count'],private_output=str(work))))
if __name__=='__main__':main()

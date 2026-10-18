"""Native construction chooser: real pointer/menu dispatch, not campaign acceptance."""
import hashlib,json,shutil,subprocess,sys,uuid
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
INPUT='''pointer 312 140
left down
left up
wait
key c
observe no-actor
pointer 200 445
left down
left up
observe rejected
key escape
key space 189
key tab
pointer 320 240
right down
right up
key space 150
key c
observe choices
pointer 200 445
left down
left up
observe assigned
'''
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    work=ROOT/'private/integration/building-menu'/uuid.uuid4().hex;work.mkdir(parents=True)
    paths=list((ROOT/'rust/crates').glob('*/src/**/*.rs'));pins={str(p.relative_to(ROOT)):sha(p) for p in paths}
    build=subprocess.run([sys.executable,'scripts/cargo.py','build','-p','genewars-desktop','--bin','genewars-game','--release','--offline','--locked'],cwd=ROOT,capture_output=True,text=True)
    (work/'build.log').write_text(build.stdout+build.stderr);build.check_returncode()
    assert pins=={str(p.relative_to(ROOT)):sha(p) for p in paths},'sources changed during build'
    binary=work/'genewars-game.exe';shutil.copy2(ROOT/'private/build/rust-target/release/genewars-game.exe',binary)
    (work/'input.txt').write_text(INPUT);runs=[]
    for name in ['first','repeat']:
        folder=work/name
        run=subprocess.run([str(binary),str(ROOT/'private/integration/game-runtime'),'--landing-seed','5428','--native-input-replay',str(work/'input.txt'),'--observations',str(folder),'--seconds','120'],cwd=ROOT,capture_output=True,text=True,timeout=140)
        (work/f'{name}.stderr').write_text(run.stderr);run.check_returncode()
        frames={p.stem:json.loads(p.read_text()) for p in folder.glob('*.json')}
        assert set(frames)=={'no-actor','rejected','choices','assigned'}
        assert frames['no-actor']['construction_menu_open'] and frames['no-actor']['construction_choices']==[11]
        assert frames['rejected']['construction_menu_open'] and 'select an architect' in frames['rejected']['error']
        assert frames['no-actor']['mission']['world_sha256']==frames['rejected']['mission']['world_sha256']
        assert frames['choices']['construction_menu_open'] and frames['choices']['construction_choices']==[11]
        actor=frames['assigned']['mission']['actor_states'][0]
        assert actor['assignment_pending'] and actor['building_selector']==11 and not actor['pending_move']
        assert not frames['assigned']['construction_menu_open'] and frames['assigned']['error'] is None
        assert frames['assigned']['mission']['steps']==339 and not frames['assigned']['mission']['buildings']
        runs.append({key:{'bmp_sha256':sha(folder/f'{key}.bmp'),'world_sha256':value['mission']['world_sha256']} for key,value in frames.items()})
    assert runs[0]==runs[1]
    receipt={'status':'passed','scope':'port-owned construction UI with source43960 availability/MENU_ORDER; authentic campaign acceptance remains0',
             'result':'no-architect click rejects unchanged; movement arrives; real chooser click dispatches selector11 opcode5 assignment without creating a building directly',
             'cases':runs,'source_pins':pins,'binary_sha256':sha(binary),'work':str(work.relative_to(ROOT)),
             'source_2ab70_sha256':sha(ROOT/'private/ghidra/actor-name-trace/instructions-0002ab70.txt'),
             'limitations':['C key and chooser rectangles/typography are port-owned, not recovered original controls','Does not establish placement, completed construction or mission completion']}
    (ROOT/'research/integration/native-building-menu.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','work':str(work)}))
if __name__=='__main__':main()

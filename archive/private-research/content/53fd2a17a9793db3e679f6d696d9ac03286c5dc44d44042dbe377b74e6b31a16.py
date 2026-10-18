"""Native-host input replay; development route, not authentic mission acceptance."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[2]
INPUT = """observe menu
pointer 312 140
left down
left up
wait
observe loaded
key space 189
key tab
observe selected
pointer 320 240
right down
right up
observe ordered
key space 150
observe moved
"""

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    work = ROOT / 'private/integration/native-input' / uuid.uuid4().hex
    work.mkdir(parents=True)
    sources = sorted((ROOT / 'rust/crates').glob('*/src/**/*.rs'))
    pins = {str(p.relative_to(ROOT)): sha(p) for p in sources}
    result = subprocess.run([sys.executable, '-B', 'scripts/cargo.py', 'build', '-p',
                             'genewars-desktop', '--bin', 'genewars-game', '--release',
                             '--offline', '--locked'], cwd=ROOT, capture_output=True, text=True)
    (work / 'build.log').write_text(result.stdout + result.stderr)
    result.check_returncode()
    assert pins == {str(p.relative_to(ROOT)): sha(p) for p in sources}, 'sources changed during build'
    binary = work / 'genewars-game.exe'
    shutil.copy2(ROOT / 'private/build/rust-target/release/genewars-game.exe', binary)
    (work / 'input.txt').write_text(INPUT)
    runs = []
    for case in ['first', 'repeat']:
        observations = work / case
        command = [str(binary), str(ROOT / 'private/integration/game-runtime'), '--landing-seed', '5428',
                   '--native-input-replay', str(work / 'input.txt'), '--observations', str(observations),
                   '--seconds', '120', '--report', str(work / f'{case}.json')]
        run = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=140)
        (work / f'{case}.stderr').write_text(run.stderr)
        run.check_returncode()
        frames = {p.stem: json.loads(p.read_text()) for p in observations.glob('*.json')}
        assert set(frames) == {'menu', 'loaded', 'selected', 'ordered', 'moved'}
        assert frames['menu']['scene'] == 'Menu'
        for name in ['loaded', 'selected', 'ordered', 'moved']:
            f = frames[name]
            assert f['scene'] == 'Mission' and f['error'] is None and f['mission']['blocked'] is None
        assert frames['loaded']['mission']['steps'] == 0
        assert frames['selected']['mission']['selected_actor'] == 0
        ordered = frames['ordered']['mission']['actor_states'][0]
        moved = frames['moved']['mission']['actor_states'][0]
        assert ordered['pending_move'] and ordered['destination'] != ordered['position']
        assert moved['position'] == ordered['destination'] == moved['destination']
        assert moved['state'] == 1 and not moved['pending_move']
        assert frames['moved']['mission']['steps'] == 339
        assert all(a['presented_frames'] < b['presented_frames'] for a,b in
                   zip([frames[n] for n in ['menu','loaded','selected','ordered']],
                       [frames[n] for n in ['loaded','selected','ordered','moved']]))
        runs.append({n: {'bmp_sha256': sha(observations / f'{n}.bmp'),
                         'world_sha256': None if f['mission'] is None else f['mission']['world_sha256']}
                     for n,f in frames.items()})
    assert runs[0] == runs[1], 'native input replay must repeat exactly'
    interrupted = subprocess.run([str(binary), str(ROOT / 'private/integration/game-runtime'),
                                 '--landing-seed', '5428', '--native-input-replay', str(work / 'input.txt'),
                                 '--observations', str(work / 'interrupted'), '--seconds', '20', '--frames', '2'],
                                cwd=ROOT, capture_output=True, text=True, timeout=30)
    assert interrupted.returncode != 0 and 'interrupted before completion' in interrupted.stderr
    (work / 'interrupted.stderr').write_text(interrupted.stderr)
    receipt = {'status': 'passed', 'scope': 'development native-host inputs, not original mission acceptance',
               'authentic_missions_completable': 0, 'work': str(work.relative_to(ROOT)),
               'binary_sha256': sha(binary), 'source_pins': pins, 'input_sha256': sha(work / 'input.txt'),
               'cases': runs, 'steps': 339, 'observations_per_run': 5,
               'premature_frame_limit_rejected': True,
               'input_path': 'frame-paced shared native window dispatch; menu and terrain hit testing; development Tab selection',
               'limitations': ['Not OS event injection', 'Does not establish original selection controls or mission startup']}
    output = ROOT / 'research/integration/native-input-development.json'
    output.write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({'status': 'passed', 'receipt': str(output), 'work': str(work)}))

if __name__ == '__main__':
    main()

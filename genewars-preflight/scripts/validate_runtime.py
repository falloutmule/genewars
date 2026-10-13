"""Exercise actual menu packets, typed mission load, updates, rendering and failure paths."""
from pathlib import Path
import argparse
import hashlib
import json
import shutil
import subprocess
import uuid
from datetime import datetime, timezone
from runtime_inputs import ROOT, CATALOG, stage


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--window-smoke', action='store_true')
    args = parser.parse_args()
    bundle, staged = stage()
    binary = ROOT / 'private/build/rust-target/debug/genewars-game.exe'
    scratch = ROOT / 'private/integration/runtime-validation' / uuid.uuid4().hex
    scratch.mkdir(parents=True)
    cases = []

    def run(name, script, root=bundle, export=False, failure=False):
        directory = scratch / name
        directory.mkdir()
        replay = directory / 'input.txt'
        replay.write_text(script, encoding='utf-8')
        command = [str(binary), str(root), '--headless', str(replay), '--report', str(directory / 'report.json')]
        if export:
            command += ['--world-output', str(directory / 'world.bin'), '--bmp', str(directory / 'image.bmp')]
        proc = subprocess.run(command, capture_output=True, text=True, timeout=180)
        if failure:
            assert proc.returncode != 0 and 'mismatch' in proc.stderr, (name, proc.stderr)
            cases.append(dict(name=name, passed=True, outcome='rejected before menu'))
            return None
        assert proc.returncode == 0, (name, proc.stderr)
        result = json.loads(proc.stdout)
        assert result == json.loads((directory / 'report.json').read_bytes())
        assert result['scene_dispatch'] and not result['gameplay'] and not result['full_world_tick']
        cases.append(dict(name=name, passed=True, scene=result['scene'], mission=result['mission']))
        return result

    for name, script in [
        ('transparent-corner', 'click 140 115\n'),
        ('release-outside', 'move 312 140\ndown\nmove 1 1\nup\n'),
        ('press-only', 'move 312 140\ndown\nload\n'),
        ('focus-cancel', 'move 312 140\ndown\ncancel\nmove 312 140\nup\n'),
        ('extreme-coordinate-miss', 'click -2147483648 -2147483648\nclick 2147483647 2147483647\n')]:
        assert run(name, script)['scene'] == 'Menu'
    options = run('options-return-quit', 'click 312 210\ntoggle\nescape\nclick 312 282\n')
    assert options['scene_history'] == ['Menu','Options','Menu','Quit']
    initial = run('initial', 'click 312 140\nload\n', export=True)
    assert initial['scene_history'] == ['Menu','Loading','Mission'] and initial['mission']['plants'] == 40
    assert (scratch / 'initial/world.bin').read_bytes() == (bundle / 'terrain/CONQ001.DAT').read_bytes()
    one = run('one-step', 'click 312 140\nload\ntick 1\n', export=True)
    assert (one['mission']['steps'], one['mission']['plants'], one['mission']['cursor']) == (1,38,24)
    assert one['mission']['rgb_sha256'] == initial['mission']['rgb_sha256']  # offscreen retirements
    two = run('two-steps', 'click 312 140\nload\ntick 2\n', export=True)
    assert (two['mission']['steps'], two['mission']['plants'], two['mission']['cursor']) == (2,37,40)
    assert two['mission']['changed_world_frames'] == 1
    assert two['mission']['rgb_sha256'] != initial['mission']['rgb_sha256']
    assert 32 in initial['mission']['queue'] and 32 not in two['mission']['queue']
    assert two['mission'] == run('repeat-two-steps', 'click 312 140\nload\ntick 2\n')['mission']
    longer = run('longer-run', 'click 312 140\nload\ntick 128\n', export=True)
    assert longer['mission'] == run('repeat-longer-run', 'click 312 140\nload\ntick 128\n')['mission']
    pan = run('pan-roundtrip', 'click 312 140\nload\npan 256 0\npan -256 0\n')
    assert pan['mission'] == initial['mission']

    damaged = scratch / 'damaged'
    shutil.copytree(bundle, damaged)
    for i, resource in enumerate(staged['resources']):
        path = damaged / resource['path']
        original = path.read_bytes()
        path.write_bytes(bytes([original[0] ^ 1]) + original[1:])
        is_menu = resource['path'].startswith('menu/')
        result = run('corrupt-%02d' % i, 'click 312 140\nload\n', damaged, failure=is_menu)
        if not is_menu:
            assert result['scene'] == 'Error' and result['mission'] is None and 'mismatch' in result['error']
        path.write_bytes(original)
    missing = damaged / 'terrain/CONQ001.DAT'
    moved = damaged / 'terrain/missing.dat'
    missing.rename(moved)
    result = run('missing-mission-return', 'click 312 140\nload\nescape\n', damaged)
    assert result['scene_history'] == ['Menu','Loading','Error','Menu']
    moved.rename(missing)
    if args.window_smoke:
        output = scratch / 'window.json'
        proc = subprocess.run([str(binary), str(bundle), '--frames', '3', '--seconds', '20', '--report', str(output)], capture_output=True, text=True, timeout=30)
        assert proc.returncode == 0, proc.stderr
        result = json.loads(output.read_bytes())
        assert result['presented_frames'] == 3 and result['scene'] == 'Menu'
        cases.append(dict(name='native-three-frame-smoke', passed=True))
    for r in staged['resources']:
        data = (bundle / r['path']).read_bytes()
        assert len(data) == r['size'] and sha(data) == r['sha256']
    sources = [*sorted((ROOT/'rust/crates/genewars-runtime/src').glob('*.rs')),
               ROOT/'rust/crates/genewars-desktop/src/bin/genewars-game.rs',
               *[ROOT/'rust/crates/genewars-desktop/src/bin/support'/name for name in ['runtime_resources.rs','runtime_scene.rs','menu_art.rs','game_window.rs','status_text.rs']],
               CATALOG,Path(__file__)]
    receipt = dict(status='passed', utc=datetime.now(timezone.utc).isoformat(), binary_sha256=sha(binary.read_bytes()),
                   sources=[dict(path=p.relative_to(ROOT).as_posix(),sha256=sha(p.read_bytes())) for p in sources],
                   cases=cases, case_count=len(cases), private_output=scratch.relative_to(ROOT).as_posix(),
                   milestones=dict(A='bounded native menu-to-loaded-world route', B='transactional plant steps with actual pixel change', C='not implemented'),
                   original_execution=False, full_world_tick=False, gameplay=False)
    (ROOT/'research/integration/runtime-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status='passed',cases=len(cases),initial=initial['mission'],two_steps=two['mission'],longer=longer['mission'])))


if __name__ == '__main__':
    main()

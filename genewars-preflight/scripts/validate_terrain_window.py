"""Bounded native presentation and frozen supervised-control observations."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]


def record(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), bytes=len(data), sha256=hashlib.sha256(data).hexdigest())


def main():
    exe = ROOT / 'private/build/rust-target/debug/genewars-terrain.exe'
    inputs = ROOT / 'private/integration/terrain-inputs'
    output = ROOT / 'private/integration' / ('terrain-window-' + uuid.uuid4().hex + '.json')
    result = subprocess.run([str(exe), str(inputs), '--target', '20736', '13568', '--policy', 'raise4096',
                             '--frames', '3', '--report', str(output)], capture_output=True, text=True, timeout=20)
    assert result.returncode == 0, result.stderr
    report = json.loads(output.read_bytes())
    assert report['presented_frames'] == 3 and report['exit_reason'] == 'frame_limit'
    assert report['final_target'] == [20736, 13568] and report['final_policy'] == 'raise4096'
    assert report['camera_words'] == report['view_camera_cells'] == [66, 48]
    assert report['pairs'] == 366 and report['pixels_written'] == 310224
    assert report['state_history'] == [dict(target=[20736, 13568], policy='raise4096')]
    assert not report['history_truncated']
    # Preserve the actual earlier supervised result. This verifier checks its
    # artifact; it does not pretend to reinject the historical input sequence.
    observed_path = ROOT / 'private/integration/terrain-interactive-20261013-0414.json'
    observed = json.loads(observed_path.read_bytes())
    sequence = [(20736, 13568, 'raise4096'), (20992, 13568, 'raise4096'),
                (20992, 13312, 'raise4096'), (20992, 13312, 'raw'),
                (20992, 13312, 'raise4096'), (20736, 13312, 'raise4096'), (20736, 13568, 'raise4096')]
    assert observed['state_history'] == [dict(target=[x, y], policy=policy) for x, y, policy in sequence]
    assert observed['presented_frames'] == 7 and observed['exit_reason'] == 'escape'
    assert observed['final_target'] == [20736, 13568] and not observed['history_truncated']
    earlier = ROOT / 'private/integration/terrain-interactive-20261013-0410.json'
    failed = json.loads(earlier.read_bytes())
    assert failed['state_history'] == [dict(target=[20736, 13568], policy='raise4096'), dict(target=[20736, 13568], policy='raw')]
    receipt = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(),
                   current_binary=record(exe), current_bounded_run=record(output), bounded_result=report,
                   supervised_observation=record(observed_path), supervised_result=observed,
                   supervised_report_written_utc=datetime.fromtimestamp(observed_path.stat().st_mtime, timezone.utc).isoformat(),
                   supervised_binary_sha256='01261a744739ed6d5639756ccd016cfc52740579557afbc6d17d9d0337cd3d16',
                   observed_inputs=['Right', 'Up', 'r', 'r', 'Left', 'Down', 'Escape'],
                   visual_observation='Terrain moved with each arrow, raw showed black, raised snapshot restored terrain; window closed on Escape.',
                   superseded_navigation_observation=record(earlier),
                   superseded_limit='Physical-key Right was not recognized; R and Escape worked. That binary hash was not recorded. Logical-key mapping corrected the issue.',
                   original_execution=False, gameplay=False,
                   limits='Historical supervised input is not replayed by this script; fresh bounded3frame presentation is executed. No original-game input/cadence or framebuffer parity is claimed.')
    (ROOT / 'research/integration/native-terrain-window.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k not in ['supervised_result', 'bounded_result']}))


if __name__ == '__main__':
    main()

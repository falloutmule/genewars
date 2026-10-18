"""Check the recorded native scene interaction; does not replay UI input."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIRECTORY = ROOT / 'private/integration/plant-scene-example/b933f14ea57b40d7bd31d6046b62d4af'


def validate_launch_binding(launch, process):
    for key in ('pid', 'command', 'binary_sha256', 'snapshot_sha256', 'started_utc',
                'timeout_seconds', 'original_execution'):
        assert launch[key] == process[key], ('launch/process mismatch', key)
    assert launch['original_execution'] is False
    expected_command = [str(ROOT / 'private/build/rust-target/debug/genewars-terrain.exe'),
        str(ROOT / 'private/integration/terrain-inputs'), '--source-player', '0',
        '--policy', 'raise4096', '--layout', 'initial-genepanl', '--plant-snapshot',
        str(DIRECTORY / 'explicit.snapshot'), '--plant-resources',
        str(ROOT / 'private/integration/plant-inputs'), '--report', str(DIRECTORY / 'supervised.json')]
    assert launch['command'] == expected_command, 'unexpected supervised command or output/input path'
    assert datetime.fromisoformat(process['ended_utc']) >= datetime.fromisoformat(launch['started_utc'])


def record(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), bytes=len(data),
                sha256=hashlib.sha256(data).hexdigest())


def main():
    launch = json.loads((DIRECTORY / 'supervised-launch.json').read_bytes())
    process = json.loads((DIRECTORY / 'supervised-process.json').read_bytes())
    observed = json.loads((DIRECTORY / 'supervised.json').read_bytes())
    cli = json.loads((DIRECTORY / 'report.json').read_bytes())
    validate_launch_binding(launch, process)
    # Exercise metadata mismatches without changing historical evidence files.
    negative_bindings = 0
    for key in ('binary_sha256', 'snapshot_sha256', 'command'):
        bad = dict(process)
        bad[key] = ['wrong-output-path'] if key == 'command' else '0' * 64
        try:
            validate_launch_binding(launch, bad)
        except AssertionError:
            negative_bindings += 1
        else:
            raise AssertionError(('mismatched historical metadata admitted', key))
    bad_launch, bad_process = dict(launch), dict(process)
    bad_launch['command'] = bad_process['command'] = [*launch['command'][:-1], str(DIRECTORY / 'other.json')]
    try:
        validate_launch_binding(bad_launch, bad_process)
    except AssertionError:
        negative_bindings += 1
    else:
        raise AssertionError('consistent but wrong historical output path admitted')
    assert launch['binary_sha256'] == '5571390d4789f5fc3cff158573a5f55de770b1b7e95d1ed08df25e4ef129db0c'
    assert launch['snapshot_sha256'] == record(DIRECTORY / 'explicit.snapshot')['sha256']
    assert process['pid'] == launch['pid'] and process['exit_code'] == 0 and not process['host_timeout']
    assert 0 < process['elapsed_seconds'] < launch['timeout_seconds'] == 180
    assert observed['exit_reason'] == 'escape' and observed['presented_frames'] == 5
    assert observed['state_history'] == [dict(target=target, policy=policy) for target, policy in [
        ([20736, 13568], 'raise4096'), ([20992, 13568], 'raise4096'),
        ([20992, 13568], 'raw'), ([20736, 13568], 'raw'), ([20736, 13568], 'raise4096')]]
    assert observed['final_target'] == [20736, 13568] and observed['final_policy'] == 'raise4096'
    assert observed['rgb_crc16'] == cli['rgb_crc16'] == 41527
    scene = observed['plant_scene']
    assert scene['queue_order'] == cli['plant_scene']['queue_order'] == [29, 34, 35, 30, 33, 31, 32]
    for field in ('foreground_pixels', 'shadow_pixels', 'foreground_clipped', 'shadow_clipped'):
        assert scene[field] == cli['plant_scene'][field]
    assert not scene['initial_state_inferred'] and not scene['admission_callbacks_applied']
    receipt = dict(status='passed-recorded-native-scene-controls', checked_utc=datetime.now(timezone.utc).isoformat(),
        original_execution=False, input_replayed_by_checker=False,
        observed_inputs=['Right', 'r', 'Left', 'r', 'Escape'],
        observed_behavior='Terrain target moved; raw policy hid terrain; explicit plant positions stayed fixed; raised policy restored view; Escape closed window.',
        supervised_binary_sha256=launch['binary_sha256'], presented_frames=5,
        final_rgb_crc16=41527, queue_order=scene['queue_order'], rejected_metadata_mutations=negative_bindings,
        evidence=[record(DIRECTORY / name) for name in ['supervised-launch.json', 'supervised-process.json',
            'supervised.json', 'report.json', 'explicit.snapshot', 'scene.bmp', 'native-initial.jpg', 'native-restored.jpg']],
        limits=['Historical input observation, not a new UI replay',
                'CRC16 and draw receipts are not cryptographic framebuffer equivalence',
                'Explicit screen-space plant snapshot remains fixed while terrain target changes',
                'Window captures include the native host; no original game state or timing claim'])
    (ROOT / 'research/integration/scene-controls-observed.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(dict(status=receipt['status'], presented_frames=5, final_rgb_crc16=41527)))


if __name__ == '__main__':
    main()

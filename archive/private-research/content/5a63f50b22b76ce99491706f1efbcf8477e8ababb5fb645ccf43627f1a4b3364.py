"""Validate native multi-plant snapshot presentation against the scene CLI.

The default run is headless: it checks pinned fixtures, re-renders scene CLI
cases, and verifies malformed inputs fail before creating reports. Pass
--run-window-smokes only when the native-window slot is clear; each valid case
then opens a bounded two-frame diagnostic window.
"""
from datetime import datetime, timezone
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import uuid

ROOT = Path(__file__).resolve().parents[1]
sha = lambda data: hashlib.sha256(data).hexdigest()
SOURCE_PATHS = [
    'rust/crates/genewars-render/src/plant_snapshot.rs',
    'rust/crates/genewars-render/src/plant_scene.rs',
    'rust/crates/genewars-desktop/src/bin/genewars-terrain.rs',
    'rust/crates/genewars-core/src/entity_draw_queue.rs',
    'rust/crates/genewars-tools/src/plant_scene.rs',
]


def code_pins():
    return [dict(path=name, bytes=(ROOT / name).stat().st_size,
        sha256=sha((ROOT / name).read_bytes())) for name in SOURCE_PATHS]


def verify_code_pins(payload, current):
    recorded = payload.get('code_sources')
    if recorded is None:
        raise ValueError('private receipt has no historical production code-source pins')
    if recorded != current:
        raise ValueError('production code sources changed since the private run receipt')


def source_pin_self_test():
    baseline = [dict(path='render.rs', bytes=12, sha256='a' * 64)]
    verify_code_pins({'code_sources': baseline}, baseline)
    mutated = [dict(baseline[0], sha256='b' * 64)]
    for payload in ({'code_sources': mutated}, {}):
        try:
            verify_code_pins(payload, baseline)
        except ValueError:
            continue
        raise AssertionError('stale or missing source pins were accepted')


def write_public_receipt(payload, private_path):
    sources = code_pins()
    verify_code_pins(payload, sources)
    binaries = payload['binaries']
    for binary in binaries.values():
        path = ROOT / binary['path']
        if not path.is_file() or path.stat().st_size != binary['bytes'] or sha(path.read_bytes()) != binary['sha256']:
            raise ValueError('a binary no longer matches the private run receipt')
    private_bytes = private_path.read_bytes()
    public = dict(status=payload['status'], checked_utc=payload['checked_utc'],
        private_receipt=dict(path=private_path.relative_to(ROOT).as_posix(), sha256=sha(private_bytes)),
        native_windows_launched=payload['native_windows_launched'], case_count=len(payload['cases']),
        rejected_count=len(payload['rejected']),
        cases=[dict(case=item['case'], snapshot_sha256=item['snapshot_sha256'],
            cli_bmp_sha256=item['cli_bmp_sha256'], expected_rgb_crc16=item['expected_rgb_crc16'],
            expected_queue_order=item['expected_queue_order'], presented_frames=item.get('presented_frames'),
            native_rgb_crc16=item.get('rgb_crc16'), native_report_sha256=item.get('native_report_sha256'))
            for item in payload['cases']],
        binaries=binaries, code_sources=sources,
        staged_source_unchanged=payload['staged_source_unchanged'],
        existing_fixtures_unchanged=payload['existing_fixtures_unchanged'],
        rejected_snapshots_unchanged=payload['rejected_snapshots_unchanged'],
        code_source_pin_binding=payload.get('code_source_pin_binding', 'production source hashes captured with this private run receipt'),
        comparison_scope=payload.get('comparison_scope',
            'Native comparison checks CRC16 plus queue order and per-record/pixel receipts. The native window does not expose raw framebuffer bytes, so this is not cryptographic or raw-framebuffer equality. Each CLI BMP matched its pinned prior full SHA-256.'),
        scope=payload['scope'])
    target = ROOT / 'research/integration/plant-snapshot-window.json'
    fd, temporary = tempfile.mkstemp(dir=target.parent, prefix='plant-snapshot-window-', suffix='.tmp')
    try:
        with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as stream:
            stream.write(json.dumps(public, indent=2) + '\n')
        os.replace(temporary, target)
    finally:
        if os.path.exists(temporary): os.unlink(temporary)
    return target


def run(args, timeout=60):
    return subprocess.run([str(value) for value in args], cwd=ROOT, capture_output=True,
        text=True, timeout=timeout)


def resolve_fixture(case, receipt, roots):
    snapshot_hash = case['snapshot_sha256']
    bmp_hash = case['bmp_sha256']
    for directory in roots:
        snapshot = directory / f"{case['case']}.snapshot.txt"
        bmp = directory / f"{case['case']}.bmp"
        if snapshot.is_file() and bmp.is_file() and sha(snapshot.read_bytes()) == snapshot_hash:
            assert sha(bmp.read_bytes()) == bmp_hash, case['case']
            return snapshot, bmp
    raise AssertionError(f"no pinned private fixture for {case['case']}")


def main():
    parser = argparse.ArgumentParser(description='Validate the native explicit plant snapshot viewer.')
    group = parser.add_mutually_exclusive_group()
    group.add_argument('--run-window-smokes', action='store_true',
        help='launch six bounded two-frame native windows; use only when the GUI slot is clear')
    group.add_argument('--summarize-existing', type=Path,
        help='regenerate compact metadata from a successful private receipt without launching windows')
    group.add_argument('--self-test-source-pins', action='store_true',
        help='verify stale and missing historical source pins are rejected')
    options = parser.parse_args()
    if options.self_test_source_pins:
        source_pin_self_test()
        print(json.dumps(dict(status='passed', check='stale-and-missing-source-pins-rejected')))
        return
    if options.summarize_existing is not None:
        private_path = options.summarize_existing.resolve()
        if not private_path.is_relative_to((ROOT / 'private/integration/plant-snapshot-window').resolve()):
            raise ValueError('private receipt must be under private/integration/plant-snapshot-window')
        payload = json.loads(private_path.read_bytes())
        if payload.get('status') != 'passed' or payload.get('native_windows_launched') is not True:
            raise ValueError('summary requires a successful native-window receipt')
        try:
            verify_code_pins(payload, code_pins())
        except ValueError as error:
            parser.error(str(error))
        target = write_public_receipt(payload, private_path)
        print(json.dumps(dict(status=payload['status'], case_count=len(payload['cases']),
            receipt=target.relative_to(ROOT).as_posix(), private_receipt=private_path.relative_to(ROOT).as_posix())))
        return
    run_windows = options.run_window_smokes
    scene_receipt_path = ROOT / 'research/integration/mission-plant-scene.json'
    scene_receipt = json.loads(scene_receipt_path.read_bytes())
    assert scene_receipt['status'] == 'passed' and scene_receipt['original_execution'] is False
    terrain_dir = ROOT / 'private/integration/terrain-inputs'
    plant_dir = ROOT / 'private/integration/plant-inputs'
    terrain_inputs = json.loads((ROOT / 'research/integration/terrain-inputs.json').read_bytes())
    plant_inputs = json.loads((ROOT / 'research/integration/plant-inputs.json').read_bytes())
    for item in terrain_inputs['prepared']:
        data = (ROOT / item['private_path']).read_bytes()
        assert len(data) == item['staged_bytes'] and sha(data) == item['staged_sha256']
    for item in plant_inputs['prepared']:
        data = (ROOT / item['private_path']).read_bytes()
        assert len(data) == item['staged_bytes'] and sha(data) == item['staged_sha256']
    pinned_sources = [ROOT / item['private_path'] for item in terrain_inputs['prepared'] + plant_inputs['prepared']]

    tools = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    desktop = ROOT / 'private/build/rust-target/debug/genewars-terrain.exe'
    assert tools.is_file() and desktop.is_file(), 'build genewars-tools and genewars-desktop first'
    roots = list((ROOT / 'private/integration/mission-plant-scene').iterdir())
    outroot = ROOT / 'private/integration/plant-snapshot-window' / uuid.uuid4().hex
    outroot.mkdir(parents=True)

    cases = {row['case']: row for row in scene_receipt['cases']}
    selected = ['overlap-forward', 'overlap-reverse', 'edges-capacity',
        'gate-open-mode-zero', 'gate-closed', 'unsupported-context-skipped']
    prepared = []
    native_results = []
    fixture_pins = []
    for name in selected:
        case = cases[name]
        snapshot, prior_bmp = resolve_fixture(case, scene_receipt, roots)
        cli_bmp = outroot / f'{name}.cli.bmp'
        cli = run([tools, 'mission-plant-scene', terrain_dir, plant_dir, '0', 'raw', snapshot, cli_bmp])
        assert cli.returncode == 0, (name, cli.stderr)
        cli_json = json.loads(cli.stdout)
        assert cli_json['plant_scene']['queue_order'] == case['draw_order']
        assert cli_json['rgb_crc16'] == case['rgb_crc16'] and sha(cli_bmp.read_bytes()) == sha(prior_bmp.read_bytes())
        entry = dict(case=name, snapshot_path=snapshot.relative_to(ROOT).as_posix(),
            snapshot_sha256=sha(snapshot.read_bytes()), cli_bmp_sha256=sha(cli_bmp.read_bytes()),
            fixture_bmp_path=prior_bmp.relative_to(ROOT).as_posix(), fixture_bmp_sha256=sha(prior_bmp.read_bytes()),
            expected_rgb_crc16=cli_json['rgb_crc16'], expected_queue_order=cli_json['plant_scene']['queue_order'])
        fixture_pins.append((snapshot, sha(snapshot.read_bytes()), prior_bmp, sha(prior_bmp.read_bytes())))
        if run_windows:
            report = outroot / f'{name}.native.json'
            command = [desktop, terrain_dir, '--source-player', '0', '--policy', 'raw',
                '--layout', 'initial-genepanl', '--plant-snapshot', snapshot,
                '--plant-resources', plant_dir, '--frames', '2', '--report', report]
            native = run(command, timeout=90)
            assert native.returncode == 0, (name, native.stderr, native.stdout)
            native_json = json.loads(report.read_bytes())
            scene = native_json['plant_scene']
            assert native_json['presented_frames'] == 2 and native_json['exit_reason'] == 'frame_limit'
            assert native_json['rgb_crc16'] == cli_json['rgb_crc16'], (name, native_json['rgb_crc16'], cli_json['rgb_crc16'])
            assert scene['queue_order'] == cli_json['plant_scene']['queue_order']
            assert scene['candidate_count'] == cli_json['plant_scene']['candidate_count']
            assert scene['final_count'] == cli_json['plant_scene']['final_count']
            expected_records = cli_json['plant_scene']['records']
            assert len(scene['records']) == len(expected_records)
            for actual, expected in zip(scene['records'], expected_records):
                for key in ('id', 'slot', 'animation', 'frame', 'shade', 'flags_before', 'flags_after', 'drawn',
                    'foreground_pixels', 'foreground_clipped', 'shadow_pixels', 'shadow_clipped'):
                    assert actual[key] == expected[key], (name, key, actual[key], expected[key])
                assert actual['origin'] == expected['origin'], (name, actual['origin'], expected['origin'])
            entry['native_report'] = report.relative_to(ROOT).as_posix()
            entry['native_report_sha256'] = sha(report.read_bytes())
            entry['presented_frames'] = native_json['presented_frames']
            entry['rgb_crc16'] = native_json['rgb_crc16']
            entry['command'] = [Path(part).relative_to(ROOT).as_posix() if Path(part).is_absolute() and Path(part).is_relative_to(ROOT) else str(part)
                for part in command]
            native_results.append(entry)
        prepared.append(entry)

    invalid_root = outroot / 'invalid-plants'
    shutil.copytree(plant_dir, invalid_root)
    bad_anb = invalid_root / 'slot-00.anb'
    bad_anb.write_bytes(bad_anb.read_bytes()[:-1])
    fixtures = next(root for root in roots if (root / 'malformed-line.txt').is_file()
        and (root / 'duplicate-id.txt').is_file() and (root / 'later-bad.snapshot.txt').is_file())
    invalids = [
        ('malformed-snapshot', fixtures / 'malformed-line.txt', plant_dir),
        ('duplicate-id', fixtures / 'duplicate-id.txt', plant_dir),
        ('bad-queued-resource', fixtures / 'later-bad.snapshot.txt', invalid_root),
    ]
    rejected = []
    rejected_snapshot_pins = []
    for name, snapshot, resources in invalids:
        rejected_snapshot_pins.append((snapshot, sha(snapshot.read_bytes())))
        report = outroot / f'{name}.report.json'
        result = run([desktop, terrain_dir, '--source-player', '0', '--policy', 'raw',
            '--layout', 'initial-genepanl', '--plant-snapshot', snapshot,
            '--plant-resources', resources, '--report', report])
        assert result.returncode != 0 and not report.exists(), (name, result.returncode, result.stderr)
        rejected.append(dict(case=name, exit_code=result.returncode, report_absent=True))

    # Compare every pinned staged asset to the hashes captured before the runs.
    source_unchanged = all(sha(path.read_bytes()) == (terrain_inputs['prepared'] + plant_inputs['prepared'])[index]['staged_sha256']
        for index, path in enumerate(pinned_sources))
    assert source_unchanged
    fixtures_unchanged = all(sha(snapshot.read_bytes()) == snapshot_hash and sha(bmp.read_bytes()) == bmp_hash
        for snapshot, snapshot_hash, bmp, bmp_hash in fixture_pins)
    rejected_inputs_unchanged = all(sha(snapshot.read_bytes()) == digest for snapshot, digest in rejected_snapshot_pins)
    assert fixtures_unchanged and rejected_inputs_unchanged

    binaries = {name: dict(path=path.relative_to(ROOT).as_posix(), bytes=path.stat().st_size, sha256=sha(path.read_bytes()))
        for name, path in [('genewars-tools', tools), ('genewars-terrain', desktop)]}
    source_paths = SOURCE_PATHS
    code_sources = code_pins()
    payload = dict(status='passed' if run_windows else 'prepared',
        checked_utc=datetime.now(timezone.utc).isoformat(), native_windows_launched=run_windows,
        cases=prepared, rejected=rejected, code_sources=code_sources,
        code_source_pin_binding='production source hashes captured with this private run receipt',
        binaries=binaries,
        staged_source_unchanged=source_unchanged,
        existing_fixtures_unchanged=fixtures_unchanged,
        rejected_snapshots_unchanged=rejected_inputs_unchanged,
        comparison_scope='Native comparison checks CRC16 plus queue order and per-record/pixel receipts. The native window does not expose raw framebuffer bytes, so this is not cryptographic or raw-framebuffer equality. Each CLI BMP matched its pinned prior full SHA-256.',
        scope='Explicit GWPLANT1 post-admission diagnostics only; no original state or callback claim.',
        source_pins=dict(scene_receipt_sha256=sha(scene_receipt_path.read_bytes()),
            terrain_receipt_sha256=sha((ROOT / 'research/integration/terrain-inputs.json').read_bytes()),
            plant_receipt_sha256=sha((ROOT / 'research/integration/plant-inputs.json').read_bytes())))
    receipt = outroot / 'receipt.json'
    encoded = json.dumps(payload, indent=2) + '\n'
    receipt.write_text(encoded, encoding='utf-8', newline='\n')
    target = write_public_receipt(payload, receipt)
    print(json.dumps(dict(status=payload['status'], cli_cases=len(prepared), native_cases=len(native_results),
        rejected=len(rejected), receipt=receipt.relative_to(ROOT).as_posix(), public_receipt=target.relative_to(ROOT).as_posix())))


if __name__ == '__main__':
    main()

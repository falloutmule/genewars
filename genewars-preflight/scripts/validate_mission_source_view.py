"""Source-table target + initial viewport integration, not live mission startup.

Uses the independent lane's camera/strip/raster models. The explicit GENEPANL
choice and factor snapshot remain presentation policies, not inferred UI state.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import uuid
from validate_terrain_cell_render import load
from terrain_inputs import SOURCES

ROOT = Path(__file__).resolve().parents[1]
sha = lambda data: hashlib.sha256(data).hexdigest()


def record(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), bytes=len(data), sha256=sha(data))


def main():
    model = load('independent_scene', 'research/variants/terrain-view-review.py')
    camera_model = load('independent_camera', 'research/variants/camera-target-review.py')
    shade_model = load('independent_shade', 'research/variants/entity-shade-review.py')
    inputs = ROOT / 'private/integration/terrain-inputs'
    resources = {name: (inputs / name).read_bytes() for name in SOURCES}
    for name, (_, source_digest, decoded_digest) in SOURCES.items():
        assert sha(resources[name]) == (decoded_digest or source_digest), name
    mission = resources['CONQ001.DAT']
    original = mission[0x355e:0x3b55e]
    panel = resources['GENEPANL.DAT']
    palette = resources['MAIN.PAL']
    colors = [bytes(c * 255 // 63 for c in palette[i:i + 3]) for i in range(0, 768, 3)]
    work = ROOT / 'private/integration/mission-source-views' / uuid.uuid4().hex
    work.mkdir(parents=True)
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    window = ROOT / 'private/build/rust-target/debug/genewars-terrain.exe'
    records = []
    metadata_records = []
    for player in range(4):
        target = struct.unpack_from('<hh', mission, 0x1c7e + player * 4)
        cx, cy = target[0] // 256, target[1] // 256
        def height(x, y):
            return struct.unpack_from('<h', original, ((y % 128) * 128 + x % 128) * 14 + 2)[0]
        h2 = camera_model.height_oracle.weighted(
            [height(cx, cy), height(cx + 1, cy), height(cx, cy + 1), height(cx + 1, cy + 1)],
            target[0] % 256, target[1] % 256)
        words = camera_model.expected(*target, h2)
        camera = [v % 128 for v in words]
        run = subprocess.run([str(exe), 'mission-source', str(inputs / 'CONQ001.DAT'), str(player)],
                             capture_output=True, text=True, timeout=15)
        assert run.returncode == 0, run.stderr
        info = json.loads(run.stdout)
        assert info['camera_words'] == list(words) and info['target'] == list(target)
        assert info['source_player'] == player and info['source_bytes'] == len(mission)
        assert info['flags_byte'] == 56 and info['active_count_byte'] == 1
        layouts = [(243038, 300, 116), (277838, 512, 41), (298830, 20, 140), (301630, 256, 59),
                   (316734, 200, 73), (331334, 32, 41), (332646, 32, 89), (335494, 16, 94)]
        expected_groups = []
        for family, (offset, count, stride) in enumerate(layouts):
            expected_groups.append(dict(family=family, file_offset=offset, count=count, packed_stride=stride,
                                        nonzero_byte6_records=sum(mission[offset + i * stride + 6] != 0 for i in range(count))))
        assert info['groups'] == expected_groups
        expected_points = []
        signed = lambda value, bits: (value + (1 << (bits - 1))) % (1 << bits) - (1 << (bits - 1))
        for index in range(512):
            data = mission[0x43d4e + index * 41:0x43d4e + (index + 1) * 41]
            if not data[6]:
                continue
            x, y = struct.unpack_from('<hh', data, 7)
            xx, yy = x // 256, y // 256
            points = [(xx, yy), (xx + 1, yy), (xx, yy + 1), (xx + 1, yy + 1)]
            height2 = camera_model.height_oracle.weighted([height(a, b) for a, b in points], x % 256, y % 256)
            dx, dy = [signed(2 * value - (origin << 9), 16) >> 1 for value, origin in zip([x, y], words)]
            products = []
            for a, b in points:
                offset = ((b % 128) * 128 + a % 128) * 14
                products.append((struct.unpack_from('<h', original, offset)[0] * original[offset + 12]) // 64)
            expected_points.append(dict(record=index, position=[x, y],
                local_projection=[((dx - dy) * 32 >> 8) - 128, ((dx + dy) * 16 >> 8) - (height2 >> 6)],
                source_shade=shade_model.expected(products, x % 256, y % 256),
                resource_slot=data[0x23], animation=data[0x19], frame=data[0x1b]))
        assert info['family1_nonzero_byte6_records'] == expected_points and len(expected_points) == 40
        assert not any(info[key] for key in ['runtime_objects_constructed', 'visibility_inferred', 'sort_order_inferred'])
        path = work / f'player-{player}-source.json'
        path.write_text(json.dumps(info, indent=2) + '\n')
        metadata_records.append(record(path))
        for policy in ['raw', 'raise4096']:
            region = bytearray(original)
            changes = [] if policy == 'raw' else model.factor.plan(region, target)
            for _, _, x, y, old, value in changes:
                assert region[(y * 128 + x) * 14 + 12] == old
                region[(y * 128 + x) * 14 + 12] = value
            composed, error = model.compose(region, camera, 0, resources['B32A.DAT'], resources['FADE.DAT'], None, True)
            assert error is None
            pairs, prepared, writes, counts, dirty, frame = composed
            indices = bytearray(frame)
            assert all(indices[y * 640 + x] == 0 for y in range(480) for x in range(128))
            for y in range(480):
                indices[y * 640:y * 640 + 128] = panel[y * 128:(y + 1) * 128]
            bmp_path = work / f'player-{player}-{policy}.bmp'
            run = subprocess.run([str(exe), 'mission-source-view', str(inputs), str(player), policy, str(bmp_path)],
                                 capture_output=True, text=True, timeout=20)
            assert run.returncode == 0, run.stderr
            native = json.loads(run.stdout)
            for key, value in dict(target=list(target), camera_words=list(words), wrapped_camera=camera,
                                   source_player=player, factor_changes=len(changes), pairs=pairs, columns=16,
                                   requested_modes=counts, prepared_triangles=prepared, pixel_writes=writes,
                                   dirty_requests=len(dirty), layout='initial-viewport', panel_policy='explicit-GENEPANL').items():
                assert native[key] == value, (player, policy, key, native[key], value)
            rgb = b''.join(colors[i] for i in indices)
            bmp = bmp_path.read_bytes()
            assert bmp[:2] == b'BM' and struct.unpack_from('<ii', bmp, 18) == (640, 480)
            offset = struct.unpack_from('<I', bmp, 10)[0]
            actual = b''.join(bmp[offset + y * 1920:offset + (y + 1) * 1920] for y in range(479, -1, -1))
            expected = bytearray(len(rgb))
            expected[0::3], expected[1::3], expected[2::3] = rgb[2::3], rgb[1::3], rgb[0::3]
            assert len(bmp) == offset + len(rgb) and actual == expected
            records.append(native | dict(bmp=record(bmp_path), index_sha256=sha(indices), rgb_sha256=sha(rgb)))
    bad = work / 'invalid.bmp'
    for player, policy in [('4', 'raw'), ('-1', 'raw'), ('0', 'guessed')]:
        result = subprocess.run([str(exe), 'mission-source-view', str(inputs), player, policy, str(bad)], capture_output=True, timeout=15)
        assert result.returncode == 2 and not bad.exists()
    output = work / 'window.json'
    result = subprocess.run([str(window), str(inputs), '--source-player', '0', '--layout', 'initial-genepanl',
                             '--policy', 'raise4096', '--frames', '3', '--report', str(output)],
                            capture_output=True, text=True, timeout=20)
    assert result.returncode == 0, result.stderr
    native = json.loads(output.read_bytes())
    expected = records[1]
    assert native['presented_frames'] == 3 and native['exit_reason'] == 'frame_limit'
    assert native['initial_source_player'] == 0 and native['layout'] == 'initial-genepanl'
    assert native['final_target'] == expected['target'] == [20736, 13568]
    assert native['pairs'] == expected['pairs'] and native['pixels_written'] == expected['pixel_writes']
    assert native['state_history'] == [dict(target=[20736, 13568], policy='raise4096')]
    assert not native['original_initial_panel_inferred'] and not native['live_callbacks_applied']
    for name, data in resources.items():
        assert (inputs / name).read_bytes() == data
    receipt = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(), frames=len(records),
                   pixels_compared=len(records) * 307200, invalid_requests_rejected=3, input_bytes_unchanged=True,
                   source_metadata_records=metadata_records, source_family1_points_compared=160,
                   cli=record(exe), native_window=record(window), bounded_window=record(output), window_result=native,
                   resources=[record(inputs / name) for name in resources], records=records,
                   dependencies=[record(ROOT / name) for name in ['research/variants/terrain-view-review.py',
                       'research/variants/camera-target-review.py', 'scripts/validate_mission_source_view.py']],
                   original_execution=False, gameplay=False,
                   limits='Explicit source-player selection, zero inherited context flags, initialized pitch640, GENEPANL choice and raw/raise4096 snapshot. No live mission, entity callbacks, panel-selector inference or original framebuffer comparison.')
    (ROOT / 'research/integration/mission-source-view.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k not in ['records', 'resources', 'dependencies', 'window_result']}))


if __name__ == '__main__':
    main()

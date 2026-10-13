"""Compare the explicit single-record plant view with independent scene models.

This exercises a caller-selected source record over an independently assembled
terrain/panel background. It is not a complete mission renderer or a live-game
ordering, visibility, timing, or callback claim.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import uuid

from validate_terrain_cell_render import load
from terrain_inputs import SOURCES as TERRAIN_SOURCES

ROOT = Path(__file__).resolve().parents[1]
sha = lambda data: hashlib.sha256(data).hexdigest()


def pin(path):
    data = path.read_bytes()
    return dict(path=path.relative_to(ROOT).as_posix(), bytes=len(data), sha256=sha(data))


def crc16_rnc(data):
    crc = 0
    for byte in data:
        crc ^= byte
        for _ in range(8):
            crc = (crc >> 1) ^ (0xA001 if crc & 1 else 0)
    return crc


def signed(value, bits):
    return (value + (1 << (bits - 1))) % (1 << bits) - (1 << (bits - 1))


def trunc_zero(value, divisor):
    return value // divisor if value >= 0 else -((-value) // divisor)


def bmp_rgb(data):
    assert data[:2] == b'BM'
    offset = struct.unpack_from('<I', data, 10)[0]
    width, height = struct.unpack_from('<ii', data, 18)
    planes, depth = struct.unpack_from('<HH', data, 26)
    assert (width, height, planes, depth) == (640, 480, 1, 24)
    stride = (width * 3 + 3) & ~3
    assert len(data) == offset + stride * height
    rgb = bytearray(width * height * 3)
    for y in range(height):
        source = offset + (height - 1 - y) * stride
        target = y * width * 3
        for x in range(width):
            blue, green, red = data[source + x * 3:source + x * 3 + 3]
            rgb[target + x * 3:target + x * 3 + 3] = bytes((red, green, blue))
    return bytes(rgb)


def independently_compose_plant(frame, indices, fade, gla, origin, flags, shade):
    """Apply the reviewed shadow-first and indexed8 foreground models."""
    width, height, pitch = 512, 480, 640
    before = bytearray(indices)
    shadow_written = shadow_clipped = foreground_written = foreground_clipped = 0
    origin16 = [signed(value, 16) for value in origin]

    # 6C230 calls the complete frame shadow pass before its color pass.
    for component in frame['components']:
        sprite_width, sprite_height = component['w'], component['h']
        left = origin16[0] + component['x']
        top = origin16[1] + trunc_zero(component['y'], 2)
        for shadow_row in range(sprite_height // 2):
            source_y = shadow_row * 2 + 1
            y = top + shadow_row
            for x in range(sprite_width):
                source = source_y * sprite_width + x
                if not component['mask'][source]:
                    continue
                screen_x = left + x
                if 0 <= screen_x < width and 0 <= y < height:
                    at = y * pitch + 128 + screen_x
                    before[at] = gla[before[at]]
                    shadow_written += 1
                else:
                    shadow_clipped += 1

    direct = (flags & 0x02) != 0 or shade >= 30
    for component in frame['components']:
        sprite_width, sprite_height = component['w'], component['h']
        left = signed(origin16[0] + component['x'], 16)
        top = signed(origin16[1] + component['y'], 16)
        for y in range(sprite_height):
            screen_y = top + y
            for x in range(sprite_width):
                source = y * sprite_width + x
                if not component['mask'][source]:
                    continue
                screen_x = left + x
                if 0 <= screen_x < width and 0 <= screen_y < height:
                    at = screen_y * pitch + 128 + screen_x
                    value = component['indices'][source]
                    before[at] = value if direct else fade[shade * 256 + value]
                    foreground_written += 1
                else:
                    foreground_clipped += 1
    return bytes(before), dict(foreground_pixels=foreground_written,
        foreground_clipped=foreground_clipped, shadow_pixels=shadow_written,
        shadow_clipped=shadow_clipped, foreground_route='direct' if direct else 'fade')


def target_and_projection(mission, map_region, player, camera_model, point_model):
    target = struct.unpack_from('<hh', mission, 0x1C7E + player * 4)

    def height(cx, cy):
        return struct.unpack_from('<h', map_region, ((cy % 128) * 128 + cx % 128) * 14 + 2)[0]

    cx, fx = divmod(target[0], 256)
    cy, fy = divmod(target[1], 256)
    doubled = camera_model.height_oracle.weighted(
        [height(cx, cy), height(cx + 1, cy), height(cx, cy + 1), height(cx + 1, cy + 1)], fx, fy)
    camera_words = camera_model.expected(target[0], target[1], doubled)
    return target, camera_words, [value % 128 for value in camera_words]


def position_and_shade(mission, map_region, record, camera_words, point_model, shade_model):
    raw = mission[0x43D4E + record * 41:0x43D4E + (record + 1) * 41]
    position = struct.unpack_from('<hh', raw, 7)
    x, y = position
    dx = point_model.ring(x, camera_words[0])
    dy = point_model.ring(y, camera_words[1])
    doubled = point_model.height_model.weighted(
        [struct.unpack_from('<h', map_region, (((y // 256 + oy) % 128) * 128 + (x // 256 + ox) % 128) * 14 + 2)[0]
         for ox, oy in [(0, 0), (1, 0), (0, 1), (1, 1)]], x & 255, y & 255)
    origin = [((dx - dy) * 32 >> 8) - 128, ((dx + dy) * 16 >> 8) - (doubled >> 6)]
    products = []
    for ox, oy in [(0, 0), (1, 0), (0, 1), (1, 1)]:
        offset = (((y // 256 + oy) % 128) * 128 + (x // 256 + ox) % 128) * 14
        light = struct.unpack_from('<h', map_region, offset)[0]
        products.append((light * map_region[offset + 12]) // 64)
    sampled_shade = shade_model.expected(products, x & 255, y & 255)
    return raw, position, origin, sampled_shade


def main():
    terrain_model = load('mission_plant_terrain_model', 'research/variants/terrain-view-review.py')
    plant_model = load('mission_plant_pass_model', 'research/variants/plant-passes-review.py')
    camera_model = load('mission_plant_camera_model', 'research/variants/camera-target-review.py')
    point_model = load('mission_plant_point_model', 'research/variants/entity-point-review.py')
    shade_model = load('mission_plant_shade_model', 'research/variants/entity-shade-review.py')

    terrain_dir = ROOT / 'private/integration/terrain-inputs'
    plant_dir = ROOT / 'private/integration/plant-inputs'
    terrain_receipt = json.loads((ROOT / 'research/integration/terrain-inputs.json').read_bytes())
    plant_receipt = json.loads((ROOT / 'research/integration/plant-inputs.json').read_bytes())
    proof_path = ROOT / 'research/ghidra/plant-color-evidence.json'
    proof = json.loads(proof_path.read_bytes())
    source_pins = []
    terrain_files = {}
    staged_terrain = {row['private_path']: row for row in terrain_receipt['prepared']}
    for name, (entry_id, source_hash, decoded_hash) in TERRAIN_SOURCES.items():
        path = terrain_dir / name
        data = path.read_bytes()
        row = next(row for row in terrain_receipt['prepared'] if row['entry_id'] == entry_id)
        expected = decoded_hash or source_hash
        assert row['source_sha256'] == source_hash and row['staged_sha256'] == expected
        assert len(data) == row['staged_bytes'] and sha(data) == expected, name
        terrain_files[name] = data
        source_pins.append(pin(path))
    assert set(staged_terrain) == {row['private_path'] for row in terrain_receipt['prepared']}
    mission = terrain_files['CONQ001.DAT']
    assert sha(mission) == proof['conq001_fixture']['sha256']
    map_region = mission[0x355E:0x3B55E]
    atlas, fade, palette, gla = [terrain_files[name] for name in
        ['B32A.DAT', 'FADE.DAT', 'MAIN.PAL', 'MAIN.GLA']]
    assert len(palette) == 768 and max(palette) <= 63

    plant_proof = {row['slot']: row for row in proof['resources']}
    plant_staged = {row['slot']: row for row in plant_receipt['prepared']}
    models_by_selection = {}
    plant_resources = {}
    for slot in range(12):
        path = plant_dir / f'slot-{slot:02}.anb'
        data = path.read_bytes()
        expected = plant_proof[slot]
        staged = plant_staged[slot]
        assert staged['entry_id'] == expected['entry_id']
        assert staged['source_sha256'] == expected['input_sha256']
        assert staged['staged_sha256'] == expected['decoded_sha256'] == sha(data)
        assert staged['staged_bytes'] == expected['decoded_size'] == len(data)
        plant_resources[slot] = data
        source_pins.append(pin(path))

    output_dir = ROOT / 'private/integration/mission-plant-views' / uuid.uuid4().hex
    output_dir.mkdir(parents=True)
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    if not exe.is_file():
        raise FileNotFoundError(f'Build genewars-tools before running this validator: {exe}')

    # Exercise all seven adjacent source records in the viewport where they
    # are actually projected onscreen, plus several fully/partly clipped cases
    # and explicit target records for every source player index.
    cases = [(0, record, 'raw', 'source') for record in range(29, 36)]
    cases.extend([(0, 33, 'raw', 'sampled'), (0, 34, 'raise4096', 'sampled')])
    cases.extend((0, record, 'raw', 'source') for record in (0, 14, 36))
    for player in (1, 2, 3):
        cases.extend((player, record, 'raw', 'source') for record in (29, 33, 35))

    baseline_cache = {}
    records = []
    colors = [bytes(component * 255 // 63 for component in palette[i:i + 3]) for i in range(0, 768, 3)]
    for case_number, (player, record, map_policy, shade_policy) in enumerate(cases):
        target, camera_words, camera = target_and_projection(mission, map_region, player, camera_model, point_model)
        region = bytearray(map_region)
        changes = [] if map_policy == 'raw' else terrain_model.factor.plan(region, target)
        for _, _, x, y, old, value in changes:
            at = (y * 128 + x) * 14 + 12
            assert region[at] == old
            region[at] = value
        cache_key = (player, map_policy)
        if cache_key not in baseline_cache:
            composed, error = terrain_model.compose(bytes(region), camera, 0, atlas, fade, None, True)
            assert error is None
            pairs, prepared, writes, requested, dirty, indices = composed
            indices = bytearray(indices)
            panel = terrain_files['GENEPANL.DAT']
            assert len(panel) == 128 * 480
            for y in range(480):
                indices[y * 640:y * 640 + 128] = panel[y * 128:(y + 1) * 128]
            baseline_cache[cache_key] = (pairs, prepared, writes, requested, dirty, bytes(indices))

        raw, position, origin, sampled_shade = position_and_shade(
            mission, bytes(region), record, camera_words, point_model, shade_model)
        assert raw[6] != 0, (player, record, 'inactive source record')
        slot = signed(raw[0x23], 8)
        assert 0 <= slot < 12, (player, record, slot)
        animation, frame_selector = raw[0x19], raw[0x1B]
        selection = (slot, animation, frame_selector)
        if selection not in models_by_selection:
            models_by_selection[selection] = plant_model.frame(plant_resources[slot], animation, frame_selector)
        selected_frame = models_by_selection[selection]
        shade = raw[0x17] if shade_policy == 'source' else sampled_shade
        indices, plant_stats = independently_compose_plant(
            selected_frame, baseline_cache[cache_key][-1], fade, gla, origin, raw[0x15], shade)
        rgb = b''.join(colors[index] for index in indices)

        bmp_path = output_dir / f'player-{player}-record-{record}-{map_policy}-{shade_policy}.bmp'
        run = subprocess.run([str(exe), 'mission-plant-view', str(terrain_dir), str(plant_dir),
            str(player), str(record), map_policy, shade_policy, str(bmp_path)],
            capture_output=True, text=True, timeout=90)
        assert run.returncode == 0, (player, record, run.stderr)
        native = json.loads(run.stdout)
        pairs, prepared, terrain_writes, requested, dirty, _ = baseline_cache[cache_key]
        for key, expected in dict(target=list(target), camera_words=list(camera_words),
            wrapped_camera=camera, map_policy=map_policy, factor_changes=len(changes),
            columns=16, pairs=pairs, requested_modes=requested,
            prepared_triangles=prepared, pixel_writes=terrain_writes,
            dirty_requests=len(dirty), source_player=player,
            layout='initial-viewport', panel_policy='explicit-GENEPANL',
            entity_callbacks_applied=False, gameplay=False,
            initial_mission_state_inferred=False).items():
            assert native[key] == expected, (player, record, key, native[key], expected)
        plant = native['plant']
        expected_plant = dict(record=record, slot=slot, position=list(position), local_origin=origin,
            animation=animation, frame=frame_selector, shade_policy=shade_policy, shade=shade,
            foreground_pixels=plant_stats['foreground_pixels'],
            foreground_clipped=plant_stats['foreground_clipped'],
            shadow_pixels=plant_stats['shadow_pixels'], shadow_clipped=plant_stats['shadow_clipped'],
            selection_policy='explicit-single-source-record', live_visibility_inferred=False,
            live_order_inferred=False)
        assert plant == expected_plant, (player, record, plant, expected_plant)
        assert bmp_rgb(bmp_path.read_bytes()) == rgb, (player, record, map_policy, shade_policy)
        assert native['index_crc16'] == crc16_rnc(indices)
        assert native['rgb_crc16'] == crc16_rnc(rgb)
        if player == 0 and 29 <= record <= 35:
            assert plant_stats['foreground_pixels'] > 0, (record, 'expected onscreen foreground')
        if player in (1, 2, 3) and record in (29, 33, 35):
            assert plant_stats['foreground_pixels'] == 0 and plant_stats['foreground_clipped'] > 0
        if player == 0 and record in (0, 14, 36):
            assert plant_stats['foreground_pixels'] == 0 and plant_stats['foreground_clipped'] > 0
        records.append(native | dict(bmp=pin(bmp_path), index_sha256=sha(indices), rgb_sha256=sha(rgb),
            foreground_route=plant_stats['foreground_route']))

    # Malformed source, invalid signed slot, and bad selector/index requests
    # must fail before writing a BMP. Mutations are explicitly synthetic copies
    # under private storage and never replace the staged evidence inputs.
    invalid_root = ROOT / 'private/integration/mission-plant-view-invalid' / uuid.uuid4().hex
    invalid_root.mkdir(parents=True)
    def copy_input_directory(target):
        target.mkdir(parents=True)
        for name in terrain_files:
            shutil.copyfile(terrain_dir / name, target / name)
        return target
    bad_source_dir = copy_input_directory(invalid_root / 'bad-source')
    (bad_source_dir / 'CONQ001.DAT').write_bytes(mission[:-1])
    bad_slot_dir = copy_input_directory(invalid_root / 'bad-slot')
    bad_slot = bytearray(mission)
    bad_slot[0x43D4E + 29 * 41 + 0x23] = 0x80
    (bad_slot_dir / 'CONQ001.DAT').write_bytes(bad_slot)
    invalid_cases = [
        ('truncated mission source', bad_source_dir, '0', '29', 'raw', 'source', False),
        ('invalid signed resource slot', bad_slot_dir, '0', '29', 'raw', 'source', True),
        ('record index out of bounds', terrain_dir, '0', '512', 'raw', 'source', False),
        ('source player out of bounds', terrain_dir, '4', '29', 'raw', 'source', True),
        ('inactive source record', terrain_dir, '0', '511', 'raw', 'source', False),
    ]
    rejections = []
    for index, (label, input_dir, player, record, policy, shade_policy, sentinel) in enumerate(invalid_cases):
        output = invalid_root / f'invalid-{index}.bmp'
        if sentinel:
            output.write_bytes(b'preserve existing output')
            before = output.read_bytes()
        run = subprocess.run([str(exe), 'mission-plant-view', str(input_dir), str(plant_dir),
            player, record, policy, shade_policy, str(output)], capture_output=True, text=True, timeout=90)
        assert run.returncode == 2, (label, run.returncode, run.stderr)
        if sentinel:
            assert output.read_bytes() == before, (label, 'clobbered sentinel')
        else:
            assert not output.exists(), (label, 'created output on failure')
        rejections.append(dict(case=label, exit_code=run.returncode, output_absent=not output.exists(),
            existing_output_preserved=sentinel))

    # A bounded native-window launch checks that the event host presents the
    # same selected plant snapshot as the CLI. It does not test game input or
    # original timing/cadence.
    window = ROOT / 'private/build/rust-target/debug/genewars-terrain.exe'
    matching_cli = next(row for row in records if row['source_player'] == 0 and row['plant']['record'] == 33
        and row['plant']['shade_policy'] == 'sampled' and row['map_policy'] == 'raw')
    report_path = output_dir / 'native-3-frame-report.json'
    run = subprocess.run([str(window), str(terrain_dir), '--source-player', '0', '--policy', 'raw',
        '--layout', 'initial-genepanl', '--frames', '3', '--plant-record', '33',
        '--plant-resources', str(plant_dir), '--plant-shade', 'sampled', '--report', str(report_path)],
        capture_output=True, text=True, timeout=45)
    assert run.returncode == 0, run.stderr
    window_report = json.loads(report_path.read_bytes())
    assert window_report['presented_frames'] == 3 and window_report['exit_reason'] == 'frame_limit'
    assert window_report['layout'] == 'initial-genepanl' and window_report['initial_source_player'] == 0
    assert window_report['final_target'] == matching_cli['target']
    assert window_report['final_policy'] == matching_cli['map_policy']
    assert window_report['rgb_crc16'] == matching_cli['rgb_crc16']
    assert window_report['plant'] == dict(record=33, slot=0, shade_policy='sampled',
        selection_policy='explicit-single-source-record', live_visibility_inferred=False,
        live_order_inferred=False)
    assert not window_report['live_callbacks_applied'] and not window_report['original_initial_panel_inferred']

    # Both original staged input sets are source-identity checked again after
    # the exercise, guarding against accidental mutation by the harness.
    for name, data in terrain_files.items():
        assert (terrain_dir / name).read_bytes() == data
    for slot, data in plant_resources.items():
        assert (plant_dir / f'slot-{slot:02}.anb').read_bytes() == data

    receipt = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(),
        original_execution=False, gameplay=False, cases=len(records),
        full_rgb_pixels_compared=len(records) * 640 * 480,
        records=records, invalid_requests=rejections,
        native_smoke=dict(executable=pin(window), report=pin(report_path),
            presented_frames=window_report['presented_frames'], exit_reason=window_report['exit_reason'],
            rgb_crc16=window_report['rgb_crc16'], matched_cli_rgb_crc16=matching_cli['rgb_crc16']),
        source_resources=[pin(terrain_dir / name) for name in terrain_files],
        plant_resources=[pin(plant_dir / f'slot-{slot:02}.anb') for slot in range(12)],
        executable=pin(exe), terrain_staging=pin(ROOT / 'research/integration/terrain-inputs.json'),
        plant_staging=pin(ROOT / 'research/integration/plant-inputs.json'),
        plant_evidence=pin(proof_path), model_sources=[pin(ROOT / name) for name in [
            'research/variants/terrain-view-review.py', 'research/variants/plant-passes-review.py',
            'research/variants/camera-target-review.py', 'research/variants/entity-point-review.py',
            'research/variants/entity-shade-review.py', 'scripts/validate_mission_plant_view.py']],
        coverage=dict(player_indices=[0, 1, 2, 3], onscreen_records=list(range(29, 36)),
            explicitly_clipped_records=[0, 14, 36], map_policies=['raw', 'raise4096'],
            shade_policies=['source', 'sampled']),
        limits=['A caller explicitly chooses a source player, one source record, map snapshot, and shade input.',
            'This does not reproduce record admission, source order, a full entity queue, live visibility, callbacks, or animation timing.',
            'All invalid mutation fixtures are private copies; staged game inputs are unchanged.'])
    target = ROOT / 'research/integration/mission-plant-view.json'
    target.parent.mkdir(parents=True, exist_ok=True)
    temporary = target.with_name(target.name + '.' + uuid.uuid4().hex + '.tmp')
    temporary.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8', newline='\n')
    temporary.replace(target)
    print(json.dumps({key: value for key, value in receipt.items() if key not in ['records', 'source_resources', 'plant_resources', 'model_sources']}))


if __name__ == '__main__':
    main()

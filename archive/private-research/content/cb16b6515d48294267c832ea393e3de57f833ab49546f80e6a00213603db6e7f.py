"""Independent validation of the explicit family-1 post-admission scene CLI.

The snapshots below are diagnostic inputs authored by this validator. They do
not claim to reconstruct the game's initial queue, callbacks, or live state.
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

from validate_mission_plant_view import (ROOT, bmp_rgb, crc16_rnc, independently_compose_plant,
    load, sha)
from terrain_inputs import SOURCES as TERRAIN_SOURCES


def queue_reference(active, initial_count, plants):
    """Independent model of admission, bucket placement, and linked-list ties."""
    if not 0 <= initial_count <= 256:
        raise ValueError('initial count outside safe domain')
    buckets = [[] for _ in range(480)]
    accepted_count = 0
    capacity = 256 - initial_count
    if active:
        for plant in plants:
            x, y = plant['x'], plant['y']
            if -40 <= x < 640 - 128 + 40 and -10 <= y < 520 and accepted_count < capacity:
                if y <= 0:
                    bucket, residual, draw_y = 0, y, y
                    placement = 'ascending'
                elif y < 480:
                    bucket, residual, draw_y = y, 0, y
                    placement = 'head'
                else:
                    bucket, residual, draw_y = 479, y - 480, y - 1
                    placement = 'ascending'
                queued = dict(**plant, bucket=bucket, residual=residual,
                    draw_y=draw_y, placement=placement)
                chain = buckets[bucket]
                if placement == 'head':
                    chain.insert(0, queued)
                else:
                    # Stable ascending insertion; equal residuals remain in
                    # admission order, including mixed bucket-479 entries.
                    at = next((i for i, old in enumerate(chain)
                        if old['residual'] > residual), len(chain))
                    chain.insert(at, queued)
                accepted_count += 1
    order = [entry for chain in buckets for entry in chain]
    return order, initial_count + accepted_count


def draw_packet(gate, mode, flags):
    draws = gate and (mode != 0 or (flags & 8) != 0)
    return dict(draws=draws, flags_after=(flags & 0xF3) if draws else flags,
        dirty_class=(3 if flags & 4 else 1) if draws else None)


def write_snapshot(path, active, initial_count, gate, mode, context_flags, plants):
    lines = ['GWPLANT1', f'context {int(active)} {initial_count} 128 {int(gate)} {mode} {context_flags}']
    for p in plants:
        lines.append('plant ' + ' '.join(str(p[k]) for k in
            ('id', 'slot', 'animation', 'frame', 'x', 'y', 'shade', 'flags', 'jitter')))
    path.write_text('\n'.join(lines) + '\n', encoding='utf-8', newline='\n')


def independent_scene(base_indices, ordered, gate, mode, context_flags, fade, gla, frame_cache):
    indices = bytes(base_indices)
    expected_rows = []
    sums = dict(foreground_pixels=0, foreground_clipped=0, shadow_pixels=0, shadow_clipped=0)
    for p in ordered:
        packet = draw_packet(gate, mode, p['flags'])
        frame = frame_cache[(p['slot'], p['animation'], p['frame'])]
        metrics = dict(foreground_pixels=0, foreground_clipped=0, shadow_pixels=0, shadow_clipped=0)
        if packet['draws']:
            indices, metrics = independently_compose_plant(frame, indices, fade, gla,
                [p['x'], p['draw_y']], p['flags'], p['shade'])
            metrics.pop('foreground_route')
            for name in sums:
                sums[name] += metrics[name]
        expected_rows.append(dict(id=p['id'], projected=[p['x'], p['y']],
            origin=[p['x'], p['draw_y']], bucket=p['bucket'], residual=p['residual'],
            slot=p['slot'], animation=p['animation'], frame=p['frame'], shade=p['shade'],
            flags_before=p['flags'], flags_after=packet['flags_after'],
            dirty_class=packet['dirty_class'], drawn=packet['draws'], **metrics))
    return indices, expected_rows, sums


def main():
    terrain_model = load('scene_terrain_model', 'research/variants/terrain-view-review.py')
    camera_model = load('scene_camera_model', 'research/variants/camera-target-review.py')
    point_model = load('scene_point_model', 'research/variants/entity-point-review.py')
    plant_model = load('scene_plant_model', 'research/variants/plant-passes-review.py')

    terrain_dir = ROOT / 'private/integration/terrain-inputs'
    plant_dir = ROOT / 'private/integration/plant-inputs'
    terrain_receipt = json.loads((ROOT / 'research/integration/terrain-inputs.json').read_bytes())
    plant_receipt = json.loads((ROOT / 'research/integration/plant-inputs.json').read_bytes())
    proof_path = ROOT / 'research/ghidra/plant-color-evidence.json'
    proof = json.loads(proof_path.read_bytes())
    mission = (terrain_dir / 'CONQ001.DAT').read_bytes()
    assert sha(mission) == proof['conq001_fixture']['sha256']
    map_region = mission[0x355E:0x3B55E]
    terrain_data = {name: (terrain_dir / name).read_bytes() for name in TERRAIN_SOURCES}
    for name, (entry_id, source_hash, decoded_hash) in TERRAIN_SOURCES.items():
        data = terrain_data[name]
        row = next(row for row in terrain_receipt['prepared'] if row['entry_id'] == entry_id)
        expected = decoded_hash or source_hash
        assert row['source_sha256'] == source_hash and row['staged_sha256'] == expected
        assert len(data) == row['staged_bytes'] and sha(data) == expected, name
    atlas, fade, palette, gla = (terrain_data[n] for n in ['B32A.DAT', 'FADE.DAT', 'MAIN.PAL', 'MAIN.GLA'])
    assert len(palette) == 768 and max(palette) <= 63
    plant_proof = {r['slot']: r for r in proof['resources']}
    plant_staged = {r['slot']: r for r in plant_receipt['prepared']}
    resources = {}
    for slot in range(12):
        data = (plant_dir / f'slot-{slot:02}.anb').read_bytes()
        assert sha(data) == plant_proof[slot]['decoded_sha256'] == plant_staged[slot]['staged_sha256']
        resources[slot] = data

    player = 0
    target = struct.unpack_from('<hh', mission, 0x1C7E + player * 4)
    height = lambda cx, cy: struct.unpack_from('<h', map_region, ((cy % 128) * 128 + cx % 128) * 14 + 2)[0]
    cx, fx = divmod(target[0], 256)
    cy, fy = divmod(target[1], 256)
    doubled = camera_model.height_oracle.weighted(
        [height(cx, cy), height(cx + 1, cy), height(cx, cy + 1), height(cx + 1, cy + 1)], fx, fy)
    camera_words = camera_model.expected(target[0], target[1], doubled)
    camera = [v % 128 for v in camera_words]
    composed, error = terrain_model.compose(map_region, camera, 0, atlas, fade, None, True)
    assert error is None
    pairs, prepared, terrain_writes, requested, dirty, indices = composed
    indices = bytearray(indices)
    panel = terrain_data['GENEPANL.DAT']
    assert len(panel) == 128 * 480
    for y in range(480):
        indices[y * 640:y * 640 + 128] = panel[y * 128:(y + 1) * 128]
    colors = [bytes(c * 255 // 63 for c in palette[i:i + 3]) for i in range(0, 768, 3)]

    source_rows = {}
    frame_cache = {}
    for record in range(29, 37):
        raw = mission[0x43D4E + record * 41:0x43D4E + (record + 1) * 41]
        slot = raw[0x23] if raw[0x23] < 128 else raw[0x23] - 256
        assert raw[6] != 0 and 0 <= slot < 12
        anim, frame = raw[0x19], raw[0x1B]
        selection = (slot, anim, frame)
        if selection not in frame_cache:
            frame_cache[selection] = plant_model.frame(resources[slot], anim, frame)
        source_rows[record] = dict(slot=slot, animation=anim, frame=frame,
            shade=raw[0x17], flags=raw[0x15], jitter=raw[0x26])

    outroot = ROOT / 'private/integration/mission-plant-scene' / uuid.uuid4().hex
    outroot.mkdir(parents=True)
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    assert exe.is_file(), f'Build genewars-tools before validation: {exe}'
    reports = []

    def run_case(label, plants, active=True, initial=0, gate=True, mode=1, context_flags=0):
        snapshot = outroot / f'{label}.snapshot.txt'
        output = outroot / f'{label}.bmp'
        write_snapshot(snapshot, active, initial, gate, mode, context_flags, plants)
        admitted, final_count = queue_reference(active, initial, plants)
        for p in admitted:
            key = (p['slot'], p['animation'], p['frame'])
            if key not in frame_cache:
                frame_cache[key] = plant_model.frame(resources[p['slot']], p['animation'], p['frame'])
        expected_indices, rows, totals = independent_scene(bytes(indices), admitted, gate, mode,
            context_flags, fade, gla, frame_cache)
        rgb = b''.join(colors[i] for i in expected_indices)
        run = subprocess.run([str(exe), 'mission-plant-scene', str(terrain_dir), str(plant_dir),
            '0', 'raw', str(snapshot), str(output)], capture_output=True, text=True, timeout=120)
        assert run.returncode == 0, (label, run.returncode, run.stderr)
        native = json.loads(run.stdout)
        assert bmp_rgb(output.read_bytes()) == rgb, label
        scene = native['plant_scene']
        expected_scene = dict(policy='explicit-post-admission-snapshot', candidate_count=len(plants),
            initial_count=initial, final_count=final_count, active_module=active, context_gate=gate,
            draw_mode=mode, context_flags=context_flags, margin=128,
            queue_order=[p['id'] for p in admitted], records=rows,
            foreground_pixels=totals['foreground_pixels'], foreground_clipped=totals['foreground_clipped'],
            shadow_pixels=totals['shadow_pixels'], shadow_clipped=totals['shadow_clipped'],
            pixels_written=totals['foreground_pixels'] + totals['shadow_pixels'],
            pixels_clipped=totals['foreground_clipped'] + totals['shadow_clipped'],
            admission_callbacks_applied=False, dirty_prepass_applied=False, initial_state_inferred=False)
        assert scene == expected_scene, (label, scene, expected_scene)
        expected_base = dict(target=list(target), camera_words=list(camera_words), wrapped_camera=camera,
            map_policy='raw', factor_changes=0, columns=16, pairs=pairs, requested_modes=requested,
            prepared_triangles=prepared, pixel_writes=terrain_writes,
            index_crc16=crc16_rnc(expected_indices), rgb_crc16=crc16_rnc(rgb),
            layout='initial-viewport', panel_policy='explicit-GENEPANL', source_player=0,
            dirty_requests=len(dirty), plant=None, entity_callbacks_applied=False,
            gameplay=False, initial_mission_state_inferred=False)
        for key, value in expected_base.items():
            assert native[key] == value, (label, key, native[key], value)
        reports.append(dict(case=label, candidates=len(plants), accepted=len(admitted),
            draw_order=[p['id'] for p in admitted], drawn=sum(row['drawn'] for row in rows),
            pixels_written=expected_scene['pixels_written'], bmp_sha256=sha(output.read_bytes()),
            rgb_crc16=native['rgb_crc16'], snapshot_sha256=sha(snapshot.read_bytes())))
        return native, output

    def item(record, ident, x, y, **override):
        data = dict(id=ident, **source_rows[record], x=x, y=y)
        data.update(override)
        return data

    # Same-row insertion is head-first, so admission order reverses. Different
    # real source slots make overlapping pixel order visible in the BMP.
    overlap = [item(29, 100, 220, 220), item(36, 101, 220, 220, flags=2)]
    first, first_bmp = run_case('overlap-forward', overlap)
    reverse, reverse_bmp = run_case('overlap-reverse', list(reversed(overlap)))
    assert first['plant_scene']['queue_order'] == [101, 100]
    assert reverse['plant_scene']['queue_order'] == [100, 101]
    assert first_bmp.read_bytes() != reverse_bmp.read_bytes()

    # Exercise edge buckets, boundary admission, bottom draw_y=y-1, stable
    # residual ties, and capacity limiting in one compact queue.
    edge = [item(29, 200, -40, -10), item(33, 201, 10, 0), item(34, 202, 30, 480),
        item(35, 203, 40, 480), item(29, 204, -41, 20), item(33, 205, 10, 520),
        item(34, 206, 12, 519)]
    edge_result, _ = run_case('edges-capacity', edge, initial=254)
    assert edge_result['plant_scene']['queue_order'] == [200, 201]
    assert edge_result['plant_scene']['final_count'] == 256
    unclipped_edges = [item(29, 210, 22, 480), item(33, 211, 22, 481),
        item(34, 213, 23, 479), item(35, 212, 24, -10)]
    bottom_result, _ = run_case('bottom-residual-ties', unclipped_edges)
    rows = {r['id']: r for r in bottom_result['plant_scene']['records']}
    assert rows[210]['origin'][1] == 479 and rows[211]['origin'][1] == 480
    assert rows[212]['origin'][1] == -10
    assert bottom_result['plant_scene']['queue_order'] == [212, 213, 210, 211]

    # Packet skip/draw and post-draw flag clearing are tested independently of
    # the input source fields by explicit snapshot values.
    packets = [item(29, 300, 240, 240, flags=0x0C), item(33, 301, 240, 240, flags=0),
        item(34, 302, 245, 240, flags=8)]
    run_case('gate-open-mode-zero', packets, gate=True, mode=0)
    run_case('gate-closed', packets, gate=False, mode=1)
    inactive, _ = run_case('module-inactive', packets, active=False, gate=True, mode=1)
    assert inactive['plant_scene']['queue_order'] == []

    # A later invalid queued resource must not create or replace the BMP.
    # The queued final record is skipped by the packet to confirm resources are
    # still parsed before draw gating.
    bad_root = outroot / 'bad-plants'
    shutil.copytree(plant_dir, bad_root)
    bad_slot = int(source_rows[36]['slot'])
    bad_file = bad_root / f'slot-{bad_slot:02}.anb'
    bad_file.write_bytes(bad_file.read_bytes()[:-1])
    bad_plants = [item(29, 400, 200, 200), item(36, 401, 205, 205, flags=0)]
    bad_snapshot = outroot / 'later-bad.snapshot.txt'
    bad_output = outroot / 'later-bad.bmp'
    write_snapshot(bad_snapshot, True, 0, False, 1, 0, bad_plants)
    bad_run = subprocess.run([str(exe), 'mission-plant-scene', str(terrain_dir), str(bad_root),
        '0', 'raw', str(bad_snapshot), str(bad_output)], capture_output=True, text=True, timeout=120)
    assert bad_run.returncode == 2 and not bad_output.exists(), (bad_run.returncode, bad_run.stderr)
    sentinel = outroot / 'later-bad-sentinel.bmp'
    sentinel.write_bytes(b'keep this existing output')
    sentinel_before = sentinel.read_bytes()
    bad_run2 = subprocess.run([str(exe), 'mission-plant-scene', str(terrain_dir), str(bad_root),
        '0', 'raw', str(bad_snapshot), str(sentinel)], capture_output=True, text=True, timeout=120)
    assert bad_run2.returncode == 2 and sentinel.read_bytes() == sentinel_before

    # Parser-level boundaries: duplicate IDs, malformed line shape, and
    # unsupported draw-only context flags are rejected without output.
    rejected = []
    invalid_snapshots = [
        ('duplicate-id', [item(29, 500, 220, 220), item(33, 500, 221, 221)], True, 1, 0),
        ('bad-context-flags-drawn', [item(29, 501, 220, 220)], True, 1, 2),
        ('bad-jitter-drawn', [item(29, 502, 220, 220, jitter=2)], True, 1, 0),
    ]
    for ix, (label, plants, gate, mode, ctxflags) in enumerate(invalid_snapshots):
        snap = outroot / f'{label}.txt'
        out = outroot / f'{label}.bmp'
        write_snapshot(snap, True, 0, gate, mode, ctxflags, plants)
        run = subprocess.run([str(exe), 'mission-plant-scene', str(terrain_dir), str(plant_dir),
            '0', 'raw', str(snap), str(out)], capture_output=True, text=True, timeout=120)
        assert run.returncode == 2 and not out.exists(), (label, run.returncode, run.stderr)
        rejected.append(dict(case=label, exit_code=run.returncode, output_absent=True))
    malformed = outroot / 'malformed-line.txt'
    malformed.write_text('GWPLANT1\ncontext 1 0 128 1 1 0\nplant 504 0 0\n', encoding='utf-8')
    malformed_output = outroot / 'malformed-line.bmp'
    malformed_run = subprocess.run([str(exe), 'mission-plant-scene', str(terrain_dir), str(plant_dir),
        '0', 'raw', str(malformed), str(malformed_output)], capture_output=True, text=True, timeout=120)
    assert malformed_run.returncode == 2 and not malformed_output.exists()
    rejected.append(dict(case='malformed-line', exit_code=malformed_run.returncode, output_absent=True))
    # The same unsupported flags are harmless when the gate skips drawing.
    skipped_bad_context = [item(29, 503, 220, 220, jitter=2)]
    run_case('unsupported-context-skipped', skipped_bad_context, gate=False, mode=1, context_flags=2)

    # Ensure staged inputs used by the reference path remain immutable.
    for name, data in terrain_data.items():
        assert (terrain_dir / name).read_bytes() == data
    for slot, data in resources.items():
        assert (plant_dir / f'slot-{slot:02}.anb').read_bytes() == data

    receipt = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(),
        original_execution=False, gameplay=False, cases=reports, rejected=rejected,
        bad_later_resource=dict(exit_code=bad_run.returncode, output_absent=True,
            existing_output_preserved=sentinel.read_bytes() == sentinel_before),
        executable=dict(path=exe.relative_to(ROOT).as_posix(), bytes=exe.stat().st_size, sha256=sha(exe.read_bytes())),
        terrain_inputs=[dict(path=(terrain_dir/n).relative_to(ROOT).as_posix(), bytes=len(d), sha256=sha(d))
            for n, d in terrain_data.items()],
        plant_inputs=[dict(path=(plant_dir/f'slot-{s:02}.anb').relative_to(ROOT).as_posix(),
            bytes=len(d), sha256=sha(d)) for s, d in resources.items()],
        evidence=dict(queue=dict(path='research/ghidra/entity-queue-evidence.json',
            sha256=sha((ROOT/'research/ghidra/entity-queue-evidence.json').read_bytes())),
            plant=dict(path=proof_path.relative_to(ROOT).as_posix(), sha256=sha(proof_path.read_bytes()))),
        limitations=['Snapshots are caller-authored explicit post-admission diagnostics, not reconstructed initial state.',
            'This validator independently models family-1 queue order and draw packets; it omits admission callbacks and dirty prepass.',
            'All resources and synthetic corruptions are confined to private integration storage.'])
    target_path = ROOT / 'research/integration/mission-plant-scene.json'
    temporary = target_path.with_name(target_path.name + '.' + uuid.uuid4().hex + '.tmp')
    temporary.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8', newline='\n')
    temporary.replace(target_path)
    print(json.dumps(dict(status=receipt['status'], cases=len(reports), rejected=len(rejected),
        bad_later_resource=receipt['bad_later_resource'], receipt=target_path.relative_to(ROOT).as_posix())))


if __name__ == '__main__':
    main()

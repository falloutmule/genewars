"""Build a reproducible explicit plant inspection scene, not first-game state.

Selects raw CONQ001 source plants in record order, projects them against player0,
samples shade after the optional radius4096 factor plan, and supplies gate1,
mode1 and capacity0 as explicit diagnostic choices. No original code executes.
"""
from pathlib import Path
import hashlib
import json
import subprocess
import uuid

from validate_mission_plant_view import load, target_and_projection, position_and_shade
from validate_mission_plant_scene import write_snapshot

ROOT = Path(__file__).resolve().parents[1]


def main():
    terrain = ROOT / 'private/integration/terrain-inputs'
    plants = ROOT / 'private/integration/plant-inputs'
    mission = (terrain / 'CONQ001.DAT').read_bytes()
    assert hashlib.sha256(mission).hexdigest() == '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    region = bytearray(mission[0x355E:0x3B55E])
    camera = load('example_camera', 'research/variants/camera-target-review.py')
    point = load('example_point', 'research/variants/entity-point-review.py')
    shade = load('example_shade', 'research/variants/entity-shade-review.py')
    factor = load('example_factor', 'research/variants/terrain-factor-review.py')
    target, words, _ = target_and_projection(mission, region, 0, camera, point)
    for _, _, x, y, old, value in factor.plan(region, target):
        at = (y * 128 + x) * 14 + 12
        assert region[at] == old
        region[at] = value
    rows = []
    for record in range(512):
        if not mission[0x43D4E + record * 41 + 6]:
            continue
        raw, position, origin, sampled_shade = position_and_shade(
            mission, region, record, words, point, shade)
        assert raw[0x23] < 12
        rows.append(dict(id=record, slot=raw[0x23], animation=raw[0x19], frame=raw[0x1B],
                         x=origin[0], y=origin[1], shade=sampled_shade,
                         flags=raw[0x15], jitter=raw[0x26]))
    directory = ROOT / 'private/integration/plant-scene-example' / uuid.uuid4().hex
    directory.mkdir(parents=True)
    snapshot, output = directory / 'explicit.snapshot', directory / 'scene.bmp'
    write_snapshot(snapshot, True, 0, True, 1, 0, rows)
    executable = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    result = subprocess.run([str(executable), 'mission-plant-scene', str(terrain), str(plants),
        '0', 'raise4096', str(snapshot), str(output)], capture_output=True, text=True, timeout=90)
    if result.returncode:
        raise RuntimeError(result.stderr)
    report = json.loads(result.stdout)
    (directory / 'report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'snapshot': str(snapshot), 'image': str(output),
        'source_candidates': len(rows), 'draw_order': report['plant_scene']['queue_order'],
        'policy': 'explicit diagnostic: raw record order/frames, sampled shade, gate1/mode1/count0',
        'gameplay': False, 'initial_mission_state_inferred': False}))


if __name__ == '__main__':
    main()

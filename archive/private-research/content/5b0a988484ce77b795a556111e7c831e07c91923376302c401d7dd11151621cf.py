"""Check the native terrain CLI against pinned first-map bytes and tile hashes."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    proof_path = ROOT / 'research/ghidra/terrain-evidence.json'
    proof = json.loads(proof_path.read_bytes())
    projection_path = ROOT / 'research/ghidra/terrain-projection-evidence.json'
    projection = json.loads(projection_path.read_bytes())
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    mission = ROOT / 'private/integration/campaign-first/CONQ001.DAT'
    assert sha(mission) == '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    corpus = json.loads((ROOT / 'research/integration/rnc-corpus.json').read_bytes())
    row = next(r for r in corpus['results'] if r['entry_id'] == proof['atlas']['entry_id'])
    atlas = (ROOT / row['private_output']).resolve()
    assert atlas.is_relative_to((ROOT / 'private').resolve())
    assert row['status'] == 'passed' and sha(atlas) == proof['atlas']['decoded_sha256']

    def run(*args, passed=True):
        result = subprocess.run([str(exe), *map(str, args)], capture_output=True, text=True, timeout=5)
        assert result.returncode == (0 if passed else 2), result.stderr
        return json.loads(result.stdout) if passed else result.stdout

    actual = run('terrain-info', mission)
    expected = dict(width_cells=128, height_cells=128, record_bytes=14,
                    min_height=0, max_height=4800, min_texture=0, max_texture=118, gameplay=False)
    assert actual == expected
    raw = mission.read_bytes()[0x355E:0x3B55E]
    assert hashlib.sha256(raw).hexdigest() == proof['map']['region_sha256']

    def height(cx, cy):
        offset = ((cy % 128) * 128 + cx % 128) * 14 + 2
        return int.from_bytes(raw[offset:offset + 2], 'little', signed=True)

    # Signed boundaries and a varied deterministic set through the actual map.
    points = [(-32768, -32768), (-1, -1), (32767, 32767), (0, 0), (-256, 0)]
    points += [((i * 7919) % 65536 - 32768, (i * 1049) % 65536 - 32768) for i in range(32)]
    for x, y in points:
        cx, fx = divmod(x, 256)
        cy, fy = divmod(y, 256)
        a, b, c, d = height(cx, cy), height(cx + 1, cy), height(cx, cy + 1), height(cx + 1, cy + 1)
        numerator = ((d - c) * fx + (c - a) * fy) if fx < fy else ((b - a) * fx + (d - b) * fy)
        expected_height = 2 * a + numerator // 128
        assert run('terrain-height', mission, x, y)['doubled_height'] == expected_height
        shifted = expected_height // 64
        correction = (abs(shifted) // 32) * (-1 if shifted < 0 else 1)
        camera = run('terrain-camera', mission, x, y)
        assert camera == dict(target=[x, y], camera_cells=[((x >> 8) - correction - 12) & 255,
                                                          ((y >> 8) - correction - 2) & 255],
                              function_va='0x82E20', initial_mission_target_inferred=False)
        def light(cx, cy):
            offset = ((cy % 128) * 128 + cx % 128) * 14
            return (int.from_bytes(raw[offset:offset + 2], 'little', signed=True) * raw[offset + 12]) // 64
        aa, bb, cc, dd = light(cx, cy), light(cx + 1, cy), light(cx, cy + 1), light(cx + 1, cy + 1)
        # Barycentric weights are an independent expression of the two
        # triangular difference formulas. Quantize each vertex first.
        doubled = ((256 - fy) * aa + (fy - fx) * cc + fx * dd) // 128 if fx < fy else (
            (256 - fx) * aa + (fx - fy) * bb + fy * dd) // 128
        shade = max(0, min(64, (doubled - 7680) // 256 + 16))
        assert run('terrain-shade', mission, x, y) == dict(target=[x, y], entity_shade=shade,
                                                         function_va='0x7B240', state_modified=False)
    # Actual cells plus seam neighbors; placement is supplied, never inferred
    # as the original camera. UV expectations come from the pinned binary table.
    placements = [(0, 0, 0, 0, 0, 0, 0), (127, 127, 125, 126, 7, 320, 240),
                  (-1, -1, -3, -2, -17, 0, 480), (-128, 128, -127, 127, 2, -32, 16)]
    placements += [(i * 11 - 180, i * 7 - 130, i - 30, 2 * i - 12, i - 15, 320, 240) for i in range(32)]
    for x, y, cx, cy, top, ax, ay in placements:
        cell_offset = ((y % 128) * 128 + x % 128) * 14
        orientation = (raw[cell_offset + 5] & 0x70) >> 4
        corners = proof['uv_orientation_table']['corners_u16_16'][orientation]
        expected_vertices = []
        for i, (ox, oy) in enumerate([(0, 0), (1, 0), (1, 1), (0, 1)]):
            mx, my = x + ox, y + oy
            h = height(mx, my)
            expected_vertices.append(dict(map=[mx, my], source_cell=[mx % 128, my % 128], height=h,
                buffer=[ax + 32 * ((mx - cx) - (my - cy)), ay + 16 * ((mx - cx) + (my - cy)) - 2 * (h // 64) - top],
                uv_16_16=corners[i * 2:i * 2 + 2]))
        expected_cell = dict(source_cell=[x % 128, y % 128], texture=raw[cell_offset + 4], orientation=orientation,
                             camera=[cx, cy], vertical_top=top, anchor=[ax, ay], vertices=expected_vertices,
                             triangles=[[0, 2, 3], [0, 1, 2]], scene_rendered=False)
        assert run('terrain-cell', mission, x, y, cx, cy, top, ax, ay) == expected_cell
    output = ROOT / 'private/integration/terrain' / uuid.uuid4().hex
    output.mkdir(parents=True)
    for vector in proof['atlas']['tile_vectors']:
        path = output / f"tile-{vector['tile']:03}.idx"
        result = run('terrain-tile', atlas, vector['tile'], path)
        assert result['bytes'] == 1024 and path.stat().st_size == 1024
        assert sha(path) == vector['index_sha256']
    rejected = output / 'rejected.idx'
    assert run('terrain-tile', atlas, 160, rejected, passed=False) == '' and not rejected.exists()
    assert run('terrain-height', mission, 32768, 0, passed=False) == ''
    assert run('terrain-cell', mission, 2147483647, 0, 2147483647, 0, 0, 0, 0, passed=False) == ''
    assert run('terrain-cell', mission, 0, 0, -2147483648, 0, 0, 0, 0, passed=False) == ''
    assert run('terrain-cell', mission, 0, 0, 0, 0, 2147483648, 0, 0, passed=False) == ''
    record = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(),
                  binary_sha256=sha(exe), proof_sha256=sha(proof_path), map_info=actual,
                  projection_proof_sha256=sha(projection_path), explicit_cell_placements=len(placements),
                  cell_vertices_compared=len(placements) * 4,
                  height_points=len(points), tiles_compared=160, tile_bytes_compared=163840,
                  explicit_camera_targets=len(points),
                  entity_shade_targets=len(points),
                  invalid_inputs_rejected=5, private_tile_outputs=output.relative_to(ROOT).as_posix(),
                  original_execution=False, limits='Native resources and explicitly placed cell vertices only; no terrain frame, inferred camera, draw order, visibility or shading claim.')
    (ROOT / 'research/integration/terrain-cli.json').write_text(json.dumps(record, indent=2) + '\n')
    print(json.dumps(record))


if __name__ == '__main__':
    main()

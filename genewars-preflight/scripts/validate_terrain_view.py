"""Full explicit-target terrain previews checked against composed source models.

This tests integration, not original runtime parity or mission initialization.
The separately reviewed strip/triangle/span models are intentionally reused.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import uuid
from validate_terrain_cell_render import load

ROOT = Path(__file__).resolve().parents[1]
sha = lambda data: hashlib.sha256(data).hexdigest()


def main():
    strip = load('strip_spec', 'scripts/ghidra/validate_terrain_strip_bridge.py')
    triangle = load('triangle_spec', 'scripts/ghidra/validate_mode5_triangle_bridge.py')
    span = load('span_spec', 'research/variants/mode5-span-review.py')
    modes = load('modes_spec', 'scripts/ghidra/validate_terrain_modes_bridge.py')
    factor = load('factor_spec', 'research/variants/terrain-factor-review.py')
    inputs = ROOT / 'private/integration/terrain-inputs'
    mission, atlas, fade, palette = [(inputs / name).read_bytes() for name in
                                    ['CONQ001.DAT', 'B32A.DAT', 'FADE.DAT', 'MAIN.PAL']]
    expected = ['72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2',
                '8206e4be65dd7f1fa12df131cf2d1aab5f5b47f7181c8ace86facbb1787bcdb8',
                'b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392',
                '8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05']
    assert [sha(b) for b in [mission, atlas, fade, palette]] == expected
    original_map = mission[0x355E:0x3B55E]
    uv = json.loads((ROOT / 'research/ghidra/terrain-evidence.json').read_bytes())['uv_orientation_table']['corners_u16_16']
    colors = [bytes(c * 255 // 63 for c in palette[i:i + 3]) for i in range(0, 768, 3)]
    overlays = [i for i in range(16384) if original_map[i * 14 + 13] > 1]
    assert len(overlays) == 36
    targets = [(-32768, -32768), (10240, 12800), (32767, -1), (12345, 8317)]
    targets += [((i % 128) * 256, (i // 128) * 256) for i in [overlays[0], overlays[len(overlays) // 2], overlays[-1]]]
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    output = ROOT / 'private/integration/terrain-views' / uuid.uuid4().hex
    output.mkdir(parents=True)
    records = []
    for target in targets:
        x, y = target
        def height(cx, cy):
            return struct.unpack_from('<h', original_map, ((cy % 128) * 128 + cx % 128) * 14 + 2)[0]
        cx, fx = divmod(x, 256)
        cy, fy = divmod(y, 256)
        h2 = factor.height_oracle.weighted([height(cx, cy), height(cx + 1, cy), height(cx, cy + 1), height(cx + 1, cy + 1)], fx, fy)
        shifted = h2 // 64
        correction = abs(shifted) // 32 * (-1 if shifted < 0 else 1)
        camera_words = [(cx - correction - 12) % 256, (cy - correction - 2) % 256]
        camera = [value % 128 for value in camera_words]
        for policy in ['raw', 'raise4096']:
            region = bytearray(original_map)
            changes = [] if policy == 'raw' else factor.plan(region, target)
            for _, _, sx, sy, _, value in changes:
                region[(sy * 128 + sx) * 14 + 12] = value
            indices = bytearray(307200)
            counts = [0] * 16
            pairs = prepared_count = writes = transparent = 0
            for column in range(20):
                plan = strip.strip(region, camera, column, 0, 480, 0)
                pairs += plan['pairs']
                assert plan['dirty_and_masks'] == []
                for call in plan['calls']:
                    mode = call['mode']
                    counts[mode] += 1
                    assert mode in [5, 6], 'Actual preview needs an unsupported mode'
                    tile = call['texture'] if mode == 5 else 157 + call['overlay_table_index']
                    base = tile // 8 * 8192 + tile % 8 * 32
                    vertices = [[*point, *uv[call['orientation']][corner * 2:corner * 2 + 2], shade]
                                for point, corner, shade in zip(call['positions'], call['corner_indices'], call['shades'])]
                    prepared = triangle.prepare(vertices, 480)
                    if prepared is None:
                        continue
                    prepared_count += 1
                    for yy, left, right, u, v, shade in prepared['rows']:
                        if mode == 5:
                            case = (1, len(atlas), base, 640, 640, left, right, u, v, shade, *prepared['gradients'])
                            (status, start, count), pixels = span.oracle(case, atlas, fade)
                            assert status == 0
                            if count:
                                indices[yy * 640 + start:yy * 640 + start + count] = pixels[4:4 + count]
                            writes += count
                        else:
                            row = modes.mode6(atlas, base, fade, 640, left, right, [u, v, shade], prepared['gradients'])
                            for xx, (pixel, opaque) in enumerate(zip(row['pixels'], row['mask']), row['start']):
                                if opaque:
                                    indices[yy * 640 + xx] = pixel
                                    writes += 1
                                else:
                                    transparent += 1
            file = output / f'view-{x}-{y}-{policy}.bmp'
            run = subprocess.run([str(exe), 'terrain-view', str(inputs), str(x), str(y), policy, str(file)],
                                 capture_output=True, text=True, timeout=20)
            assert run.returncode == 0, run.stderr
            native = json.loads(run.stdout)
            for key, value in dict(camera_words=camera_words, wrapped_camera=camera, factor_changes=len(changes),
                                   pairs=pairs, requested_modes=counts, prepared_triangles=prepared_count, pixel_writes=writes).items():
                assert native[key] == value, (target, policy, key, native[key], value)
            rgb = b''.join(colors[i] for i in indices)
            bmp = file.read_bytes()
            assert bmp[:2] == b'BM' and struct.unpack_from('<ii', bmp, 18) == (640, 480)
            offset = struct.unpack_from('<I', bmp, 10)[0]
            assert len(bmp) == offset + len(rgb)
            got = b''.join(bmp[offset + row * 1920:offset + (row + 1) * 1920] for row in range(479, -1, -1))
            bgr = bytearray(len(rgb))
            bgr[0::3], bgr[1::3], bgr[2::3] = rgb[2::3], rgb[1::3], rgb[0::3]
            assert got == bgr, (target, policy)
            records.append(native | dict(rgb_sha256=sha(rgb), index_sha256=sha(indices), bmp_sha256=sha(bmp),
                                         transparent_overlay_samples=transparent, private_bmp=file.relative_to(ROOT).as_posix()))
    invalid = output / 'invalid.bmp'
    for x, y, policy in [('32768', '0', 'raw'), ('0', '0', 'guessed'), ('0', '-32769', 'raise4096')]:
        result = subprocess.run([str(exe), 'terrain-view', str(inputs), x, y, policy, str(invalid)], capture_output=True, timeout=10)
        assert result.returncode == 2 and not invalid.exists()
    receipt = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(), frames=len(records),
                   rgb_pixels_compared=307200 * len(records), binary_sha256=sha(exe.read_bytes()), records=records,
                   input_rejections=3, original_execution=False, gameplay=False,
                   policies='Explicit target; source camera helper modulo128; full640x480 columns0..19/context0; raw map or pure factor-plan snapshot; no entity callbacks or inferred mission startup.')
    (ROOT / 'research/integration/terrain-view.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k != 'records'}))


if __name__ == '__main__':
    main()

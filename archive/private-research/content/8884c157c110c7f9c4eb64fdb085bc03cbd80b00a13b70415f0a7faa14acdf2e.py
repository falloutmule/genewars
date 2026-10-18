"""Real-resource cell integration against previously verified row/span models.

Explicit centering and factor overrides are diagnostic policies. No original
camera, runtime map state, full scene order, or mission gameplay is inferred.
"""
from contextlib import redirect_stdout
from datetime import datetime, timezone
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import struct
import subprocess
import sys
import uuid

ROOT = Path(__file__).resolve().parents[1]


def load(name, relative):
    parent = str((ROOT / relative).parent)
    if parent not in sys.path:
        sys.path.append(parent)
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    with redirect_stdout(io.StringIO()):
        spec.loader.exec_module(module)
    return module


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    tri = load('triangle_spec', 'scripts/ghidra/validate_mode5_triangle_bridge.py')
    span = load('span_review', 'research/variants/mode5-span-review.py')
    inputs = ROOT / 'private/integration/terrain-inputs'
    staging = json.loads((ROOT / 'research/integration/terrain-inputs.json').read_bytes())
    mission, atlas, fade, palette = [
        (inputs / name).read_bytes() for name in ['CONQ001.DAT', 'B32A.DAT', 'FADE.DAT', 'MAIN.PAL']]
    assert sha(mission) == '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    assert sha(atlas) == '8206e4be65dd7f1fa12df131cf2d1aab5f5b47f7181c8ace86facbb1787bcdb8'
    assert sha(fade) == 'b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392'
    assert sha(palette) == '8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05'
    raw = mission[0x355E:0x3B55E]
    table = json.loads((ROOT / 'research/ghidra/terrain-evidence.json').read_bytes())['uv_orientation_table']['corners_u16_16']
    colors = [bytes(c * 255 // 63 for c in palette[i:i + 3]) for i in range(0, 768, 3)]
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    output = ROOT / 'private/integration/terrain-cells' / uuid.uuid4().hex
    output.mkdir(parents=True)
    points = [(0, 0), (127, 127), (-1, 128), (-32768, 32767)]
    # First occurrence of every distinct texture/orientation pair, then sample
    # across that deterministic list to include real slopes and all UV rows.
    seen = {}
    for i in range(16384):
        seen.setdefault((raw[i * 14 + 4], raw[i * 14 + 5] & 0x70), (i % 128, i // 128))
    values = list(seen.values())
    points += values[::max(1, len(values) // 24)][:24]
    records = []
    for x, y in points:
        owner = ((y % 128) * 128 + x % 128) * 14
        tile = raw[owner + 4]
        uv = table[(raw[owner + 5] >> 4) & 7]
        base = tile // 8 * 8192 + tile % 8 * 32
        for factor in [0, 31, 63]:
            vertices = []
            for n, (dx, dy) in enumerate([(0, 0), (1, 0), (1, 1), (0, 1)]):
                offset = (((y + dy) % 128) * 128 + (x + dx) % 128) * 14
                light, height = struct.unpack_from('<hh', raw, offset)
                vertices.append([320 + (dx - dy) * 32, 240 + (dx + dy) * 16 - 2 * (height // 64),
                                 *uv[n * 2:n * 2 + 2], 4 * light * factor + 32768])
            indices = bytearray(640 * 480)
            draws = writes = 0
            error = None
            for corners in [(0, 2, 3), (0, 1, 2)]:
                try:
                    prepared = tri.prepare([vertices[i] for i in corners], 480)
                except ValueError as e:
                    error = str(e)
                    break
                if prepared is None:
                    continue
                draws += 1
                for yy, left, right, u, v, shade in prepared['rows']:
                    case = (1, len(atlas), base, 640, 640, left, right, u, v, shade, *prepared['gradients'])
                    (status, start, count), pixels = span.oracle(case, atlas, fade)
                    if status:
                        error = 'sampler bounds'
                        break
                    if count:
                        indices[yy * 640 + start:yy * 640 + start + count] = pixels[4:4 + count]
                    writes += count
                if error:
                    break
            target = output / f'cell-{x}-{y}-{factor}.bmp'
            result = subprocess.run([str(exe), 'terrain-cell-render', str(inputs), str(x), str(y), str(factor), str(target)],
                                    capture_output=True, text=True, timeout=10)
            if error:
                assert result.returncode == 2 and not target.exists(), (x, y, factor, result)
                records.append(dict(cell=[x, y], factor=factor, status='safely-rejected', reason=error))
                continue
            assert result.returncode == 0, result.stderr
            receipt = json.loads(result.stdout)
            assert receipt['triangles_drawn'] == draws and receipt['pixel_writes'] == writes
            rgb = b''.join(colors[i] for i in indices)
            bmp = target.read_bytes()
            assert bmp[:2] == b'BM' and struct.unpack_from('<ii', bmp, 18) == (640, 480)
            offset = struct.unpack_from('<I', bmp, 10)[0]
            assert len(bmp) == offset + len(rgb)
            # BMP stores BGR in bottom-up rows; compare every RGB byte.
            got = b''.join(bmp[offset + y * 1920:offset + (y + 1) * 1920] for y in range(479, -1, -1))
            expected = bytearray(len(rgb))
            expected[0::3], expected[1::3], expected[2::3] = rgb[2::3], rgb[1::3], rgb[0::3]
            assert got == expected, (x, y, factor)
            records.append(dict(cell=[x, y], factor=factor, texture=tile, status='passed', pixel_writes=writes,
                                index_sha256=sha(indices), rgb_sha256=sha(rgb), bmp_sha256=sha(bmp)))
    rejection = output / 'invalid.bmp'
    for x, y, factor in [('32768', '0', '0'), ('0', '0', '256'), ('0', '-32769', '0')]:
        result = subprocess.run([str(exe), 'terrain-cell-render', str(inputs), x, y, factor, str(rejection)],
                                capture_output=True, timeout=10)
        assert result.returncode == 2 and not rejection.exists()
    receipt = dict(status='passed', checked_utc=datetime.now(timezone.utc).isoformat(),
                   binary_sha256=sha(exe.read_bytes()), cells=len(points), cases=len(records),
                   rendered_frames=sum(r['status'] == 'passed' for r in records),
                   rgb_pixels_compared=307200 * sum(r['status'] == 'passed' for r in records),
                   input_rejections=3, outputs=output.relative_to(ROOT).as_posix(), records=records,
                   original_execution=False, full_scene=False,
                   model='Integration uses the separately verified triangle specification and independent 72-bit span oracle.',
                   policies='640x480 zero-index canvas; explicit [320,240] anchor and caller factor override; per-cell triangle topology, not full scene order.')
    (ROOT / 'research/integration/terrain-cell-render.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k != 'records'}))


if __name__ == '__main__':
    main()

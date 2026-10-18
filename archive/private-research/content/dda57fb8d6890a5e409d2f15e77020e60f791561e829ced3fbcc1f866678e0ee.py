"""Reproduce the traced three-button CD menu in Rust, keeping art private."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import struct
import subprocess
import uuid
import zlib

from menu_inputs import ROOT, stage
from render_panel import png_chunk


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read_bmp(data):
    if data[:2] != b'BM' or len(data) != 54 + 640 * 480 * 3:
        raise ValueError('Unexpected native menu BMP signature/length')
    size, _, _, start = struct.unpack_from('<IHHI', data, 2)
    header, width, height, planes, bpp, compression = struct.unpack_from('<IiiHHI', data, 14)
    if (size, start, header, width, abs(height), planes, bpp, compression) != (len(data), 54, 40, 640, 480, 1, 24, 0):
        raise ValueError('Unexpected native menu BMP encoding')
    rgb = bytearray(640 * 480 * 3)
    for y in range(480):
        source_y = 479 - y if height > 0 else y
        row = data[54 + source_y * 1920:54 + (source_y + 1) * 1920]
        for x in range(640):
            rgb[(y * 640 + x) * 3:(y * 640 + x + 1) * 3] = row[x * 3:x * 3 + 3][::-1]
    return bytes(rgb)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--frames', type=int, help='Also present a bounded native window')
    args = parser.parse_args()
    if args.frames is not None and not 1 <= args.frames <= 600:
        parser.error('--frames must be between 1 and 600')
    sources = stage()
    scratch = ROOT / 'private/integration/menu'
    basename = 'menu-' + uuid.uuid4().hex
    bmp_path = scratch / (basename + '.bmp')
    report_path = scratch / (basename + '.json')
    exe = ROOT / 'private/build/rust-target/debug/genewars-menu.exe'
    command = [str(exe), str(scratch), str(scratch / 'titlescr.pal'), str(bmp_path),
               '--report', str(report_path)]
    if args.frames is not None:
        command += ['--frames', str(args.frames)]
    result = subprocess.run(command, capture_output=True, text=True, timeout=30)
    if result.returncode:
        raise RuntimeError(result.stderr)
    report = json.loads(report_path.read_bytes())
    if report != json.loads(result.stdout) or report['width'] != 640 or report['height'] != 480 or report['gameplay']:
        raise ValueError('Native menu report mismatch')
    if args.frames is not None and (report['presented_frames'] != args.frames or report['exit_reason'] != 'frame_limit'):
        raise ValueError('Native menu did not reach the requested frame count')
    bmp = bmp_path.read_bytes()
    rgb = read_bmp(bmp)
    scanlines = b''.join(b'\0' + rgb[y * 1920:(y + 1) * 1920] for y in range(480))
    png = (b'\x89PNG\r\n\x1a\n' + png_chunk(b'IHDR', struct.pack('>IIBBBBB', 640, 480, 8, 2, 0, 0, 0))
           + png_chunk(b'IDAT', zlib.compress(scanlines, 9)) + png_chunk(b'IEND', b''))
    preview = scratch / (basename + '.png')
    with preview.open('xb') as stream:
        stream.write(png)
    record = {
        'schema_version': 1, 'completed_utc': datetime.now(timezone.utc).isoformat(),
        'sources': sources, 'candidate_sha256': '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
        'bridge': 'research/ghidra/font-menu-evidence.json',
        'bridge_sha256': sha((ROOT / 'research/ghidra/font-menu-evidence.json').read_bytes()),
        'native_report': report, 'native_command': command,
        'native_executable_sha256': sha(exe.read_bytes()),
        'bmp_sha256': sha(bmp), 'rgb_sha256': sha(rgb), 'png_sha256': sha(png),
        'private_bmp': bmp_path.relative_to(ROOT).as_posix(),
        'private_preview': preview.relative_to(ROOT).as_posix(),
        'private_report': report_path.relative_to(ROOT).as_posix(),
        'display_conversion': 'Explicit floor(RGB6*255/63); original monitor transfer not measured.',
        'scope': 'Static initial non-network three-button normal-frame composition. No interaction, command dispatch, mission loading, timing or original-game parity claim. Independent pixel comparison has a separate receipt.',
    }
    (ROOT / 'research/integration/menu-render.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'native_report': report, 'private_preview': record['private_preview'],
                      'rgb_sha256': record['rgb_sha256']}))


if __name__ == '__main__':
    main()

"""Render the CD candidate's exact GENEPANL/MAIN.PAL pair locally."""
import binascii
import hashlib
import json
import struct
import subprocess
import zlib
from source_member import ROOT, SourceReader


def digest(data):
    return hashlib.sha256(data).hexdigest()


def png_chunk(kind, data):
    return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', binascii.crc32(kind + data) & 0xffffffff)


def bmp_rgb(data):
    if data[:2] != b'BM':
        raise ValueError('Missing BMP signature')
    size, _, _, start = struct.unpack_from('<IHHI', data, 2)
    header, width, height, planes, bpp, compression = struct.unpack_from('<IiiHHI', data, 14)
    if (size != len(data) or header != 40 or width != 128 or abs(height) != 480
            or planes != 1 or bpp != 24 or compression != 0):
        raise ValueError('Unexpected BMP dimensions or encoding')
    stride = (width * 3 + 3) & ~3
    if start < 54 or start + stride * abs(height) != len(data):
        raise ValueError('Invalid BMP pixel bounds')
    rgb = bytearray()
    for y in range(abs(height)):
        source_y = abs(height) - 1 - y if height > 0 else y
        row = data[start + source_y * stride:start + source_y * stride + width * 3]
        for x in range(width):
            blue, green, red = row[x * 3:x * 3 + 3]
            rgb.extend((red, green, blue))
    return width, abs(height), bytes(rgb)


def main():
    scratch = ROOT / 'private/integration/panel'
    scratch.mkdir(parents=True, exist_ok=True)
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    with SourceReader() as reader:
        panel = reader.read('root/2/disc/86')
        palette = reader.read('root/2/disc/108')
        sources = [{key: reader.entries[eid][key] for key in ('entry_id', 'path', 'size', 'sha256')}
                   for eid in ('root/2/disc/86', 'root/2/disc/108')]
    panel_path, palette_path = scratch / 'GENEPANL.RNC', scratch / 'MAIN.PAL'
    for path, data in ((panel_path, panel), (palette_path, palette)):
        if path.exists() and path.read_bytes() != data:
            raise ValueError('Refusing to overwrite different cached input: ' + str(path))
        path.write_bytes(data)
    target = scratch / 'panel.bmp'
    command = [str(exe), 'panel-render', str(panel_path), str(palette_path), str(target)]
    process = subprocess.run(command, capture_output=True, text=True, timeout=10)
    if process.returncode:
        raise RuntimeError(process.stderr)
    bmp = target.read_bytes()
    width, height, rgb = bmp_rgb(bmp)
    # Read the independently corpus-validated index output, without invoking a
    # second decoder or inferring dimensions from its byte count.
    indices = (ROOT / 'private/integration/rnc-decoded' / (digest(panel) + '.bin')).read_bytes()
    if len(indices) != 128 * 480 or len(palette) != 768 or max(palette) > 63:
        raise ValueError('Source geometry or RGB6 palette evidence mismatch')
    expected = bytes(component * 255 // 63 for index in indices for component in palette[index * 3:index * 3 + 3])
    if rgb != expected:
        raise ValueError('Exported BMP RGB values differ from decoded index/palette reference')
    scanlines = b''.join(b'\x00' + rgb[y * width * 3:(y + 1) * width * 3] for y in range(height))
    png = (b'\x89PNG\r\n\x1a\n' + png_chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 2, 0, 0, 0))
           + png_chunk(b'IDAT', zlib.compress(scanlines, 9)) + png_chunk(b'IEND', b''))
    preview = scratch / 'panel.png'
    preview.write_bytes(png)
    record = {
        'schema_version': 1, 'sources': sources, 'candidate_binary_sha256': '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
        'width': width, 'height': height, 'index_sha256': digest(indices), 'bmp_sha256': digest(bmp),
        'rgb_sha256': digest(rgb), 'png_sha256': digest(png), 'all_pixels_verified': True,
        'native_command': command, 'native_stdout': json.loads(process.stdout),
        'private_preview': str(preview.relative_to(ROOT)),
        'evidence': 'Panel draw VA0x24AD0 width128; display init0x1CAA0 binds480rows; MAIN.PAL init chain0x979A0->0xA13F0->0xA6D2C.',
        'display_conversion': 'Explicit RGB6-to-RGB8 floor(component*255/63). Source channel order is binary-verified; this presentation transfer is not a measured original DAC/monitor response.',
        'scope': 'Static original UI panel rendering only; no original execution, interactive game, overlay widgets or gameplay parity.',
    }
    (ROOT / 'research/integration/panel-render.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(record))


if __name__ == '__main__':
    main()

"""Read-only panel/palette checks. Prints metadata, never palette/image bytes."""
import hashlib
import json
import struct
from pathlib import Path

root = Path(__file__).resolve().parents[2]
image = (root / 'private/ghidra/mode-functions/memory-.object3.bin').read_bytes()
modes = []
for index in (1, 13):
    start = 0x107a98 - 0x100000 + index * 38
    width, height = struct.unpack_from('<HH', image, start)
    modes.append(dict(mode_index=index, table_width=width, table_height=height))
corpus = json.loads((root / 'research/integration/rnc-corpus.json').read_text())
palettes = {}
for row in corpus['results']:
    if row['status'] == 'passed' and row['source_path'].lower().endswith(('.pal', '.pal;1')):
        palettes[row['input_sha256']] = row
palette_lengths = set()
channel_values = set()
for row in palettes.values():
    data = (root / row['private_output']).read_bytes()
    assert hashlib.sha256(data).hexdigest() == row['output_sha256']
    palette_lengths.add(len(data))
    channel_values.update(data)
target_rows = []
for entry_id in ('root/2/disc/86',):
    row = next(row for row in corpus['results'] if row['entry_id'] == entry_id)
    data = (root / row['private_output']).read_bytes()
    assert hashlib.sha256(data).hexdigest() == row['output_sha256']
    target_rows.append(dict(entry_id=entry_id, source_path=row['source_path'],
                            input_sha256=row['input_sha256'], output_size=len(data),
                            output_sha256=row['output_sha256']))
manifest = json.loads((root / 'research/game-files-manifest.json').read_text())
main_palette = next(row for row in manifest['entries'] if row['entry_id'] == 'root/2/disc/108')
target_rows.append(dict(entry_id=main_palette['entry_id'], source_path=main_palette['path'],
                        input_sha256=main_palette['sha256'], raw_size=main_palette['size'],
                        local_bytes_reviewed=True))
panel_dir = root / 'private/integration/panel'
palette = (panel_dir / 'MAIN.PAL').read_bytes()
assert len(palette) == 768 and max(palette) <= 63
assert hashlib.sha256(palette).hexdigest() == main_palette['sha256']
indices = (root / 'private/integration/rnc-decoded/8116b2b883320b8573c50496a570f0f3fa978e4c20b9c85a284339ff1f920e24.bin').read_bytes()
assert len(indices) == 128 * 480
assert hashlib.sha256(indices).hexdigest() == 'af9741b4545fb6f28a1dd0e50da0570e39992e75c872bc2249891f3330f59951'
bmp = (panel_dir / 'panel.bmp').read_bytes()
assert bmp[:2] == b'BM' and len(bmp) == 184374
assert struct.unpack_from('<I', bmp, 2)[0] == len(bmp)
pixel_offset = struct.unpack_from('<I', bmp, 10)[0]
assert pixel_offset == 54
assert struct.unpack_from('<IiiHHI', bmp, 14) == (40, 128, 480, 1, 24, 0)
expected_rgb = bytearray()
for y in range(480):
    for x in range(128):
        index = indices[y * 128 + x]
        rgb = bytes(palette[index * 3 + channel] * 255 // 63 for channel in range(3))
        pos = 54 + ((479 - y) * 128 + x) * 3
        assert bmp[pos:pos + 3] == rgb[::-1]
        expected_rgb.extend(rgb)
pixel_review = dict(pixels_verified=128*480, channels_verified=128*480*3,
                    bmp_sha256=hashlib.sha256(bmp).hexdigest(),
                    rgb_sha256=hashlib.sha256(expected_rgb).hexdigest(),
                    orientation='bottom-up BMP BGR independently checked against top-down source RGB',
                    main_palette_min=min(palette), main_palette_max=max(palette))
scaling = []
differences = []
for value in range(64):
    normalized_floor = value * 255 // 63
    bit_replication = (value << 2) | (value >> 4)
    if normalized_floor != bit_replication:
        differences.append(value)
    if value in (0, 1, 15, 16, 21, 31, 32, 42, 47, 48, 62, 63):
        scaling.append(dict(vga6=value, normalized_floor=normalized_floor,
                             bit_replication=bit_replication))
print(json.dumps(dict(modes=modes, unique_palette_inputs=len(palettes),
                      palette_lengths=sorted(palette_lengths), channel_min=min(channel_values),
                      channel_max=max(channel_values), distinct_channel_values=len(channel_values),
                      exact_cd_inputs=target_rows,
                      independent_pixel_review=pixel_review,
                      logical_frame=dict(width=640, height=480, bytes=640*480),
                      panel=dict(width=128, height=480, source_pitch=128, bytes=128*480,
                                 destination_pitch=640, last_write_exclusive=479*640+128),
                      scaling_vectors=scaling, scaling_convention_difference_inputs=differences), indent=2))

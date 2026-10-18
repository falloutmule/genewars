"""Boundary tests use synthesized bytes, never proprietary fixtures."""
import contextlib
import hashlib
import io
import json
from pathlib import Path
import struct
import tempfile
import unittest
import warnings
import zipfile

import build_manifest
from inventory_common import analyze_bytes

WORKSPACE = Path(__file__).resolve().parents[1]
TEMP_ROOT = WORKSPACE / 'private' / 'tests'
assert TEMP_ROOT.resolve().is_relative_to(WORKSPACE.resolve())
TEMP_ROOT.mkdir(parents=True, exist_ok=True)


def le_image(offset=128):
    b = bytearray(offset + 512)
    b[:2] = b'MZ'
    struct.pack_into('<I', b, 60, offset)
    b[offset:offset + 2] = b'LE'
    for field, value in ((24, 1), (28, 16), (40, 4096), (64, 196), (68, 1), (128, offset + 300)):
        struct.pack_into('<I', b, offset + field, value)
    struct.pack_into('<6I', b, offset + 196, 512, 65536, 0x2045, 1, 1, 0)
    return b


class Headers(unittest.TestCase):
    def test_le_header_is_followed_at_actual_offset(self):
        for offset in (128, 256):
            e = analyze_bytes(bytes(le_image(offset)), 'GW.EXE')['executable']
            self.assertEqual(e['format'], 'LE')
            self.assertEqual(e['new_header_offset'], offset)
            self.assertEqual(e['entry_virtual_address_candidate'], 65552)
            self.assertEqual(e['validation'], [])

    def test_truncated_le_and_oversized_object_table_are_flagged(self):
        b = le_image()
        self.assertIn('truncated LE/LX header', analyze_bytes(bytes(b[:140]), 'GW.EXE')['executable']['validation'])
        struct.pack_into('<I', b, 128 + 68, 0xffffffff)
        self.assertIn('object table overlaps header, outside file, or implausible count', analyze_bytes(bytes(b), 'GW.EXE')['executable']['validation'])

    def test_object_table_cannot_overlap_header(self):
        b = le_image()
        struct.pack_into('<I', b, 128 + 64, 0)
        self.assertIn('object table overlaps header, outside file, or implausible count', analyze_bytes(bytes(b), 'GW.EXE')['executable']['validation'])

    def test_invalid_pointer_does_not_scan_for_incidental_le(self):
        b = le_image()
        struct.pack_into('<I', b, 60, 0xfffffff0)
        e = analyze_bytes(bytes(b), 'DOS4GW.EXE')['executable']
        self.assertEqual(e['format'], 'DOS MZ')
        self.assertIn('out of bounds', e['new_header_interpretation'])

    def test_entry_outside_object_is_flagged(self):
        b = le_image()
        struct.pack_into('<I', b, 128 + 28, 512)
        self.assertIn('entry offset exceeds object virtual size', analyze_bytes(bytes(b), 'GW.EXE')['executable']['validation'])


class Archives(unittest.TestCase):
    def make_fixture(self, root, stale=False):
        payload = io.BytesIO()
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', UserWarning)
            with zipfile.ZipFile(payload, 'w') as z:
                z.writestr('same.txt', b'one')
                z.writestr('same.txt', b'two')
        # A ZIP with a prepended MZ stub still has an enumerable central directory.
        sfx = b'MZ' + bytes(62) + payload.getvalue()
        seven = b'7z\xbc\xaf\x27\x1c' + b'synthetic test fixture'
        source = root / 'input.zip'
        with zipfile.ZipFile(source, 'w') as z:
            z.writestr('patch.exe', sfx)
            with warnings.catch_warnings():
                warnings.simplefilter('ignore', UserWarning)
                z.writestr('patch.exe', sfx)
            z.writestr('broken.zip', b'PK\x03\x04incomplete')
            z.writestr(build_manifest.DISC_NAME, seven)
        (root / 'research').mkdir()
        disc = {'source_archive_sha256': 'wrong' if stale else hashlib.sha256(seven).hexdigest(),
                'entries': [], 'self_check': {'all_archive_members_accounted_for': True},
                'image_metadata': {'inspection_status': 'blocked', 'inspection_error': 'synthetic unsupported image'}}
        (root / 'research' / 'disc-inventory.json').write_text(json.dumps(disc))
        return source

    def test_sfx_members_duplicate_names_and_blocked_image_are_preserved(self):
        with tempfile.TemporaryDirectory(dir=TEMP_ROOT) as tmp:
            assert Path(tmp).resolve().is_relative_to(TEMP_ROOT.resolve())
            root = Path(tmp)
            source = self.make_fixture(root)
            with contextlib.redirect_stdout(io.StringIO()):
                build_manifest.build(source, root)
            manifest = json.loads((root / 'research' / 'game-files-manifest.json').read_text())
            leaves = [e for e in manifest['entries'] if e.get('member_name') == 'same.txt']
            self.assertEqual(len(leaves), 4)
            self.assertNotEqual(leaves[0]['sha256'], leaves[1]['sha256'])
            self.assertNotEqual(leaves[0]['member_ordinal'], leaves[1]['member_ordinal'])
            self.assertEqual(len({e['entry_id'] for e in leaves}), 4)
            self.assertNotEqual(leaves[0]['parent_id'], leaves[2]['parent_id'])
            broken = next(e for e in manifest['entries'] if e.get('member_name') == 'broken.zip')
            self.assertEqual(broken['container_inspection_status'], 'blocked')
            self.assertEqual(broken['inspection_status'], 'inspected')
            self.assertTrue(broken['sha256'])
            seven = next(e for e in manifest['entries'] if e.get('member_name') == build_manifest.DISC_NAME)
            self.assertEqual(seven['container_inspection_status'], 'partially_inspected')
            self.assertEqual(manifest['disc_evidence']['image_metadata']['inspection_status'], 'blocked')

    def test_stale_disc_inventory_is_rejected(self):
        with tempfile.TemporaryDirectory(dir=TEMP_ROOT) as tmp:
            assert Path(tmp).resolve().is_relative_to(TEMP_ROOT.resolve())
            root = Path(tmp)
            source = self.make_fixture(root, stale=True)
            with self.assertRaisesRegex(ValueError, 'SHA-256 does not match'):
                build_manifest.build(source, root)


if __name__ == '__main__':
    unittest.main()

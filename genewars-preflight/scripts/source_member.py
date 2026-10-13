"""Targeted, hash-bound reads from the established manifest. Never runs source code."""
from __future__ import annotations

import argparse
import hashlib
import io
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
SYNC = b'\0' + b'\xff' * 10 + b'\0'


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


class SourceReader:
    def __init__(self, root=ROOT):
        self.root = Path(root).resolve()
        self.manifest = json.loads((self.root / 'research/game-files-manifest.json').read_text(encoding='utf-8'))
        self.entries = {e['entry_id']: e for e in self.manifest['entries']}
        self.source = Path(self.manifest['source']['path'])
        if digest(self.source) != self.manifest['source']['sha256_before']:
            raise ValueError('Original archive does not match the manifest')
        self.archives = {'root': zipfile.ZipFile(self.source)}
        self.disc = None

    def __enter__(self):
        return self

    def __exit__(self, *_):
        for archive in self.archives.values():
            archive.close()
        if self.disc:
            self.disc.close()

    def _archive(self, entry_id):
        if entry_id not in self.archives:
            data = self.read(entry_id, max_bytes=128 * 1024 * 1024)
            self.archives[entry_id] = zipfile.ZipFile(io.BytesIO(data))
        return self.archives[entry_id]

    def read(self, entry_id, max_bytes=64 * 1024 * 1024):
        entry = self.entries[entry_id]
        if entry['kind'] != 'file' or entry['inspection_status'] != 'inspected':
            raise ValueError('Entry is not an inspected file')
        if not 0 <= entry['size'] <= max_bytes:
            raise ValueError('Entry exceeds the caller-selected input limit')
        if 'iso9660' in entry:
            layout = entry['iso9660']
            if layout['flags'] & 0x80 or layout['file_unit_size'] or layout['interleave_gap'] or layout['extended_attribute_blocks']:
                raise ValueError('This targeted reader does not support multi-extent/interleaved/extended-attribute files')
            if self.disc is None:
                image_path = self.root / 'private/disc/member-004.img'
                expected = self.entries[entry['parent_id']]
                if digest(image_path) != expected['sha256']:
                    raise ValueError('Private disc image does not match the manifest')
                self.disc = image_path.open('rb')
            remaining = entry['size']
            chunks = []
            lba = layout['extent_lba']
            while remaining:
                self.disc.seek(lba * 2352)
                sector = self.disc.read(2352)
                if len(sector) != 2352 or sector[:12] != SYNC or sector[15] != 1:
                    raise ValueError('Invalid raw MODE1 sector')
                take = min(2048, remaining)
                chunks.append(sector[16:16 + take])
                remaining -= take
                lba += 1
            data = b''.join(chunks)
        elif '/disc/' in entry_id:
            scratch = {'Gene_Wars_ISO/.DS_Store': 'member-001', 'Gene_Wars_ISO/GENEWARS.ccd': 'member-002.ccd',
                       'Gene_Wars_ISO/GENEWARS.cue': 'member-003.cue', 'Gene_Wars_ISO/GENEWARS.img': 'member-004.img',
                       'Gene_Wars_ISO/GENEWARS.log': 'member-005.log', 'Gene_Wars_ISO/GENEWARS.sfv': 'member-006.sfv',
                       'Gene_Wars_ISO/GENEWARS.sub': 'member-007.sub'}
            member = entry['path'].rsplit('!/', 1)[-1]
            data = (self.root / 'private/disc' / scratch[member]).read_bytes()
        else:
            archive = self._archive(entry['parent_id'])
            info = archive.infolist()[entry['member_ordinal']]
            if info.filename != entry['member_name'] or info.file_size != entry['size']:
                raise ValueError('ZIP directory metadata does not match manifest ancestry')
            data = archive.read(info)
        if len(data) != entry['size'] or hashlib.sha256(data).hexdigest() != entry['sha256']:
            raise ValueError('Read bytes do not match the source entry SHA-256/size')
        return data


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('entry_id', help='Stable ancestry ID from game-files-manifest.json')
    parser.add_argument('--output', type=Path, required=True, help='Destination beneath project private directory')
    parser.add_argument('--max-bytes', type=int, default=64 * 1024 * 1024)
    args = parser.parse_args()
    target = args.output.resolve()
    if not target.is_relative_to((ROOT / 'private').resolve()):
        raise SystemExit('Original-resource output must remain inside project private/')
    with SourceReader() as reader:
        data = reader.read(args.entry_id, args.max_bytes)
        entry = reader.entries[args.entry_id]
        target.parent.mkdir(parents=True, exist_ok=True)
        if target.exists():
            if target.read_bytes() != data:
                raise SystemExit('Existing output differs; refusing to overwrite')
        else:
            with target.open('xb') as stream:
                stream.write(data)
        print(json.dumps({'entry_id': args.entry_id, 'source_path': entry['path'],
                          'sha256': entry['sha256'], 'size': len(data), 'output': str(target)}))


if __name__ == '__main__':
    main()

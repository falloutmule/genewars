"""Stage only the sixteen hash-pinned actor animation resources."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import tempfile

from source_member import ROOT, SourceReader


PROOF = ROOT / 'research/ghidra/blueprint-init-evidence.json'
MANIFEST = ROOT / 'research/game-files-manifest.json'
SPRITES = ROOT / 'research/integration/sprite-corpus.json'
CATALOG = ROOT / 'research/integration/actor-resource-catalog.tsv'
DEST = ROOT / 'private/integration/actor-resources'
EXPECTED_CANDIDATE = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'
EXPECTED_EVIDENCE_SHA256 = 'cbff4131044fb3586efb35c031b97540336f14993c3e56e21d7bd8b31d085bb4'
EXPECTED_KEYS = tuple(range(16))
FIELDS = ('key', 'style', 'category', 'descriptor_va', 'resource_slot_va', 'animation_slot_va',
          'source_name', 'entry_id', 'source_sha256', 'decoded_sha256', 'decoded_bytes',
          'animation_count', 'encoding_evidence', 'private_path')


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _atomic_public(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists():
        old = path.read_bytes()
        if old != data:
            raise ValueError(f'refusing to replace changed public catalog: {path}')
        return
    fd, name = tempfile.mkstemp(dir=path.parent, prefix=path.name + '-', suffix='.tmp')
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data)
        try:
            os.link(name, path)
        except FileExistsError:
            if path.read_bytes() != data:
                raise ValueError(f'catalog appeared with different content: {path}')
    finally:
        try:
            os.unlink(name)
        except FileNotFoundError:
            pass


def stage() -> dict:
    proof_bytes = PROOF.read_bytes()
    proof = json.loads(proof_bytes)
    if sha(proof_bytes) != EXPECTED_EVIDENCE_SHA256 or proof.get('candidate_sha256') != EXPECTED_CANDIDATE:
        raise ValueError('actor resource evidence is not bound to the pinned candidate')
    resources = proof.get('resources')
    if not isinstance(resources, list) or [r.get('key') for r in resources] != list(EXPECTED_KEYS):
        raise ValueError('expected the sixteen ordered actor resource keys')

    manifest = json.loads(MANIFEST.read_bytes())
    entries = {e['entry_id']: e for e in manifest['entries']}
    corpus = json.loads(SPRITES.read_bytes())
    sprite_rows = corpus['results']
    prepared = []
    with SourceReader() as source:
        for item in resources:
            key = item['key']
            if (item['style'] != key // 4 or item['category'] != key % 4
                    or item['descriptor_va'] != hex(0x104074 + 44 * key)
                    or item['raw_slot'] != hex(0x1168F4 + 4 * key)
                    or item['animation_slot'] != hex(0x116934 + 4 * key)):
                raise ValueError(f'instruction-derived table binding changed for key {key}')
            entry = entries[item['entry_id']]
            if (not item['entry_id'].startswith('root/2/disc/')
                    or entry.get('kind') != 'file' or entry.get('inspection_status') != 'inspected'
                    or entry.get('sha256') != item['input_sha256']
                    or entry.get('size') != item['size']
                    or entry.get('normalized_iso_path', '').lower().removeprefix('game/') != item['source_name'].lower()):
                raise ValueError(f'manifest binding mismatch for key {key}')
            if item['decoded_sha256'] != item['input_sha256'] or item['decoded_size'] != item['size']:
                raise ValueError(f'expected raw ANB bytes for key {key}; no RNC transform is admitted here')
            matches = [r for r in sprite_rows if r['entry_id'] == item['entry_id']]
            if len(matches) != 1:
                raise ValueError(f'expected one existing sprite-corpus record for key {key}')
            row = matches[0]
            if (row.get('input_sha256') != item['input_sha256']
                    or row.get('decoded_sha256') != item['decoded_sha256']
                    or row.get('decoded_size') != item['decoded_size']
                    or row.get('encoding') != 'indexed8'
                    or row.get('native_result', {}).get('status') != 'verified'):
                raise ValueError(f'existing indexed8 sprite evidence mismatch for key {key}')
            # Validate the exact manifest member and existing targeted sprite
            # corpus file independently before staging the bounded set.
            source_bytes = source.read(item['entry_id'], max_bytes=48 * 1024)
            if len(source_bytes) != item['decoded_size'] or sha(source_bytes) != item['decoded_sha256']:
                raise ValueError(f'source bytes differ from actor evidence for key {key}')
            existing = (ROOT / row['private_database']).resolve()
            if not existing.is_relative_to((ROOT / 'private').resolve()):
                raise ValueError('sprite corpus database escaped private storage')
            corpus_bytes = existing.read_bytes()
            if len(corpus_bytes) != len(source_bytes) or sha(corpus_bytes) != sha(source_bytes):
                raise ValueError(f'existing sprite corpus file differs from source for key {key}')
            target = DEST / f'key-{key:02}.anb'
            if target.exists() and target.read_bytes() != source_bytes:
                raise ValueError(f'refusing to overwrite changed staged resource: {target}')
            prepared.append((item, entry, source_bytes, target))

    records = []
    for item, entry, data, target in prepared:
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists():
            with target.open('xb') as stream:
                stream.write(data)
        records.append({
            'key': item['key'], 'style': item['style'], 'category': item['category'],
            'descriptor_va': item['descriptor_va'], 'resource_slot_va': item['raw_slot'],
            'animation_slot_va': item['animation_slot'],
            'source_name': item['source_name'], 'entry_id': item['entry_id'],
            'source_path': entry['path'], 'source_sha256': item['input_sha256'],
            'decoded_sha256': item['decoded_sha256'], 'decoded_bytes': item['decoded_size'],
            'animation_count': item['animation_count'],
            'encoding_evidence': 'indexed8 RLE selected by actor wrapper; color/fade variants still bounded',
            'private_path': target.relative_to(ROOT).as_posix(),
        })
    lines = ['\t'.join(FIELDS)]
    for row in records:
        lines.append('\t'.join(str(row[k]) for k in FIELDS))
    _atomic_public(CATALOG, ('\n'.join(lines) + '\n').encode('utf-8'))
    return {'proof_sha256': sha(proof_bytes), 'catalog_sha256': sha(CATALOG.read_bytes()),
            'resources': records, 'source_read_count': len(records),
            'scope': 'Sixteen exact CD actor animation database keys; raw ANB source bytes; no broad extraction, codec selection, animation timing, or palette inference.'}


if __name__ == '__main__':
    print(json.dumps(stage(), indent=2))

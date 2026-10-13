"""Stage the one source-bound MULE.ANB resource needed by the initial creature lane."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

from source_member import ROOT, SourceReader

ENTRY_ID = 'root/2/disc/335'
SOURCE_SHA256 = 'f6c62221b52a62088f91ae8d00becfef513282cbf98a5990c1a0859cd8b2d346'
EVIDENCE_SHA256 = '9f06ff9bafc7dc3ba4389e0ae214971b671ec29908e83b9c2fbcac3cfbd5b365'
OUTPUT = ROOT / 'private/integration/creature-resources/MULE.ANB'


def stage() -> dict:
    proof_path = ROOT / 'research/ghidra/creature-evidence.json'
    proof_bytes = proof_path.read_bytes()
    if hashlib.sha256(proof_bytes).hexdigest() != EVIDENCE_SHA256:
        raise ValueError('creature packed4 proof changed')
    proof = json.loads(proof_bytes)
    entry = proof['admitted_sources'][4]
    if (entry.get('loader_index') != 4 or entry.get('filename') != 'mule.anb'
            or entry.get('source_sha256') != SOURCE_SHA256
            or entry.get('decoded_sha256') != SOURCE_SHA256
            or [item.get('entry_id') for item in entry.get('sources', [])] != ['root/1/214', ENTRY_ID]):
        raise ValueError('proof no longer binds the CD MULE selector-4 resource')

    manifest = json.loads((ROOT / 'research/game-files-manifest.json').read_bytes())
    manifest_entry = next((item for item in manifest['entries'] if item.get('entry_id') == ENTRY_ID), None)
    if (manifest_entry is None or manifest_entry.get('kind') != 'file'
            or manifest_entry.get('size') != 57038 or manifest_entry.get('sha256') != SOURCE_SHA256
            or manifest_entry.get('normalized_iso_path', '').lower() != 'game/data/creature/mule.anb'):
        raise ValueError('CD MULE manifest identity changed')

    corpus = json.loads((ROOT / 'research/integration/sprite-corpus.json').read_bytes())
    rows = [row for row in corpus.get('results', []) if row.get('entry_id') == ENTRY_ID]
    if len(rows) != 1:
        raise ValueError('expected one exact CD MULE sprite-corpus result')
    row = rows[0]
    if (row.get('input_sha256') != SOURCE_SHA256 or row.get('decoded_sha256') != SOURCE_SHA256
            or row.get('decoded_size') != 57038 or row.get('encoding') != 'packed4-high-first'
            or row.get('status') != 'passed' or row.get('native_result', {}).get('status') != 'verified'):
        raise ValueError('existing packed4 validation does not bind this MULE input')

    with SourceReader() as source:
        if source.entries[ENTRY_ID].get('sha256') != SOURCE_SHA256:
            raise ValueError('SourceReader manifest identity changed')
        data = source.read(ENTRY_ID, max_bytes=57038)
    if len(data) != 57038 or hashlib.sha256(data).hexdigest() != SOURCE_SHA256:
        raise ValueError('CD MULE source bytes do not match the pinned resource')
    corpus_path = (ROOT / row['private_database']).resolve()
    if not corpus_path.is_relative_to((ROOT / 'private').resolve()):
        raise ValueError('sprite corpus database escaped private storage')
    if corpus_path.read_bytes() != data:
        raise ValueError('existing private sprite database differs from the hash-bound CD source')
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    if OUTPUT.exists():
        if OUTPUT.read_bytes() != data:
            raise ValueError('refusing to overwrite changed staged MULE resource')
    else:
        with OUTPUT.open('xb') as stream:
            stream.write(data)
    return {'path': OUTPUT.relative_to(ROOT).as_posix(), 'entry_id': ENTRY_ID,
            'sha256': SOURCE_SHA256, 'size': len(data), 'codec': 'explicit packed4-high-first',
            'proof_sha256': EVIDENCE_SHA256, 'source_reads': 1}


if __name__ == '__main__':
    print(json.dumps(stage(), indent=2))

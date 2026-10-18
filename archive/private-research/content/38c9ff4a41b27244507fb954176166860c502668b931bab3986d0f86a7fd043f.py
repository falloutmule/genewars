"""Stage a hash-pinned set of CD first-mission resources into private storage.

This reads only explicit manifest members and already validated RNC outputs.
The emitted integration receipt contains metadata and private paths, never
resource bytes or decoded text/image content.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import tempfile

from source_member import ROOT, SourceReader


# The CD image entries are the selected sources. Other entries with the same
# complete SHA-256 are listed as aliases in the receipt, not selected by name.
SOURCES = {
    'HDRCO1.DAT': ('root/2/disc/190', '5e1f382ea68f57f1cec03a3f7de2c4903ca127519481da98232a7f1c3a46aa11', 'raw'),
    'CONQ001.DAT': ('root/2/disc/152', '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2', 'raw'),
    'PL1.DAT': ('root/2/disc/117', '4283af614d54519e2a9f6c59a881ba58ad356b8844c9e489a51501c51e2aeeac', 'rnc'),
    'LEVSCR01.DAT': ('root/2/disc/93', 'fa70fb20318dceab350402fe962744248f9127bb91b64cb0b11526f9bb7f88d6', 'rnc'),
    'LEVSCR01.PAL': ('root/2/disc/94', '678f226e78ab61eb7ec30afa6c54f2686dca550850e0267a4824e79e7ec1e44d', 'rnc'),
    'TOPLNT0.DAT': ('root/2/disc/137', '678b8ea509bc5e0c8a609b74c77293604c89e293351ab5f62ff4b6a3a078ffd0', 'rnc'),
    'TOPLNT0.PAL': ('root/2/disc/138', 'fb584c8f2c241f3dabd94a9a5e30f21ede0f37bed08c18ecbde8ebf972ee5ff4', 'rnc'),
}
EXPECTED_EXECUTABLE_SHA256 = '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba'


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _write_identical_or_new(path: Path, data: bytes) -> None:
    if path.exists():
        if not path.is_file() or path.read_bytes() != data:
            raise ValueError(f'refusing to overwrite changed output: {path}')
        return
    with path.open('xb') as stream:
        stream.write(data)


def _write_receipt(path: Path, data: bytes) -> None:
    """Atomically refresh the public metadata receipt, never resource bytes."""
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.is_symlink():
        raise ValueError(f'refusing to replace receipt symlink: {path}')
    fd, temporary_name = tempfile.mkstemp(prefix=path.name + '.', suffix='.tmp', dir=path.parent)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(data)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary_name, path)
    finally:
        if os.path.exists(temporary_name):
            os.unlink(temporary_name)


def stage() -> dict:
    manifest_path = ROOT / 'research/game-files-manifest.json'
    corpus_path = ROOT / 'research/integration/rnc-corpus.json'
    campaign_evidence_path = ROOT / 'research/ghidra/campaign-entry-evidence.json'
    manifest_digest = sha256(manifest_path.read_bytes())
    corpus_bytes = corpus_path.read_bytes()
    corpus = json.loads(corpus_bytes)
    campaign_evidence_bytes = campaign_evidence_path.read_bytes()
    campaign_evidence = json.loads(campaign_evidence_bytes)
    if campaign_evidence.get('candidate_sha256') != EXPECTED_EXECUTABLE_SHA256:
        raise ValueError('campaign-entry evidence does not match the admitted CD executable')
    initial_vector = next((row for row in campaign_evidence.get('vectors', [])
                           if row.get('progress') == 0 and row.get('prior_system') == 0), None)
    required_paths = {
        'header': ('HDRCO1.DAT', 'LEVELS/HDRCO1.DAT'),
        'mission': ('CONQ001.DAT', 'LEVELS/CONQ001.DAT'),
        'texture': ('PL1.DAT', 'DATA/PL1.DAT'),
        'background': ('LEVSCR01.DAT', 'DATA/LEVSCR01.DAT'),
        'palette': ('LEVSCR01.PAL', 'DATA/LEVSCR01.PAL'),
        'transition_background': ('TOPLNT0.DAT', 'DATA/TOPLNT0.DAT'),
        'transition_palette': ('TOPLNT0.PAL', 'DATA/TOPLNT0.PAL'),
    }
    if (initial_vector is None
            or {filename for filename, _ in required_paths.values()} != set(SOURCES)
            or any(initial_vector.get(field) != path for field, (_, path) in required_paths.items())):
        raise ValueError('campaign-entry evidence does not bind the seven requested first-mission inputs')
    decoded_by_entry = {row['entry_id']: row for row in corpus['results']}
    output_dir = ROOT / 'private/integration/campaign-first'
    output_dir.mkdir(parents=True, exist_ok=True)
    output_dir = output_dir.resolve()
    if not output_dir.is_relative_to((ROOT / 'private').resolve()):
        raise ValueError('campaign output escaped private storage')

    records = []
    selected_rnc_records = []
    with SourceReader() as source:
        by_hash: dict[str, list[dict]] = {}
        for entry in source.entries.values():
            by_hash.setdefault(entry['sha256'], []).append(entry)

        for filename, (entry_id, expected_sha, storage_kind) in SOURCES.items():
            entry = source.entries[entry_id]
            if entry['sha256'] != expected_sha:
                raise ValueError(f'manifest identity changed for {entry_id}')
            original = source.read(entry_id)
            if sha256(original) != expected_sha:
                raise ValueError(f'source bytes changed for {entry_id}')

            if storage_kind == 'raw':
                data = original
                decode = None
            else:
                decode = decoded_by_entry.get(entry_id)
                if (decode is None or decode.get('status') != 'passed'
                        or decode.get('input_sha256') != expected_sha
                        or decode.get('input_size') != len(original)):
                    raise ValueError(f'no matching passed RNC record for {entry_id}')
                selected_rnc_records.append({
                    'entry_id': entry_id,
                    'status': decode['status'],
                    'input_sha256': decode['input_sha256'],
                    'input_size': decode['input_size'],
                    'output_size': decode['output_size'],
                    'output_sha256': decode['output_sha256'],
                })
                decoded_path = (ROOT / decode['private_output']).resolve()
                if not decoded_path.is_relative_to((ROOT / 'private').resolve()):
                    raise ValueError(f'decoded output path escaped private storage for {entry_id}')
                data = decoded_path.read_bytes()
                if (len(data) != decode.get('output_size')
                        or sha256(data) != decode.get('output_sha256')):
                    raise ValueError(f'validated RNC output changed for {entry_id}')

            target = output_dir / filename
            _write_identical_or_new(target, data)
            aliases = sorted(
                ({'entry_id': alias['entry_id'], 'path': alias['path']}
                 for alias in by_hash[expected_sha]),
                key=lambda alias: alias['entry_id'],
            )
            record = {
                'filename': filename,
                'selected_entry_id': entry_id,
                'source_path': entry['path'],
                'source_aliases_same_sha256': aliases,
                'source_size': len(original),
                'source_sha256': expected_sha,
                'storage': 'raw_source' if storage_kind == 'raw' else 'validated_rnc_decode',
                'staged_size': len(data),
                'staged_sha256': sha256(data),
                'private_path': target.relative_to(ROOT).as_posix(),
            }
            if decode is not None:
                record['rnc_corpus_output'] = decode['private_output']
                record['rnc_output_size'] = decode['output_size']
                record['rnc_output_sha256'] = decode['output_sha256']
            records.append(record)

    receipt = {
        'schema_version': 1,
        'purpose': 'hash-bound first-mission input staging; no game execution or resource interpretation',
        'manifest_sha256': manifest_digest,
        'campaign_entry_evidence_sha256': sha256(campaign_evidence_bytes),
        'campaign_entry_candidate_sha256': campaign_evidence['candidate_sha256'],
        'selected_rnc_records': selected_rnc_records,
        'selected_rnc_records_sha256': sha256(json.dumps(selected_rnc_records, sort_keys=True, separators=(',', ':')).encode('utf-8')),
        'staged_resources': records,
        'scope': 'Metadata only. Staged resource bytes remain beneath private/. Selected source IDs are explicit; aliases require exact source SHA-256 equality.',
    }
    receipt_path = ROOT / 'research/integration/campaign-inputs.json'
    receipt_bytes = (json.dumps(receipt, indent=2) + '\n').encode('utf-8')
    _write_receipt(receipt_path, receipt_bytes)
    return receipt


if __name__ == '__main__':
    print(json.dumps(stage(), indent=2))

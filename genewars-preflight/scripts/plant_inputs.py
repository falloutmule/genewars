"""Bind the twelve traced CD plant slots to existing validated private decodes.

No extraction, original execution, or runtime-state inference takes place.
"""
import hashlib
import json
import os
from pathlib import Path
import tempfile

ROOT = Path(__file__).resolve().parents[1]
sha = lambda data: hashlib.sha256(data).hexdigest()


def stage():
    proof_path = ROOT / 'research/ghidra/plant-color-evidence.json'
    proof_bytes = proof_path.read_bytes()
    proof = json.loads(proof_bytes)
    manifest = json.loads((ROOT / 'research/game-files-manifest.json').read_bytes())
    entries = {row['entry_id']: row for row in manifest['entries']}
    corpus = json.loads((ROOT / 'research/integration/rnc-corpus.json').read_bytes())
    decoded = {row['entry_id']: row for row in corpus['results']}
    resources = proof['resources']
    if [row['slot'] for row in resources] != list(range(12)):
        raise ValueError('Expected twelve distinct ordered plant slots')
    private = (ROOT / 'private').resolve()
    destination = private / 'integration/plant-inputs'
    destination.mkdir(parents=True, exist_ok=True)
    prepared = []
    # Check every input before writing any staged file.
    for resource in resources:
        entry_id = resource['entry_id']
        entry, row = entries[entry_id], decoded[entry_id]
        if (not entry_id.startswith('root/2/disc/')
                or entry['sha256'] != resource['input_sha256']
                or entry['kind'] != 'file' or entry['inspection_status'] != 'inspected'):
            raise ValueError('Plant manifest binding differs: ' + entry_id)
        if (row['status'] != 'passed' or row['input_sha256'] != resource['input_sha256']
                or row['output_sha256'] != resource['decoded_sha256']
                or row['output_size'] != resource['decoded_size']):
            raise ValueError('Plant decode receipt differs: ' + entry_id)
        source = (ROOT / row['private_output']).resolve()
        if not source.is_relative_to(private):
            raise ValueError('Plant decode escaped private storage')
        data = source.read_bytes()
        if len(data) != resource['decoded_size'] or sha(data) != resource['decoded_sha256']:
            raise ValueError('Plant decoded bytes changed: ' + entry_id)
        target = destination / ('slot-%02d.anb' % resource['slot'])
        if target.exists() and target.read_bytes() != data:
            raise ValueError('Refusing to overwrite changed plant input: ' + str(target))
        prepared.append((resource, entry, data, target))
    records = []
    for resource, entry, data, target in prepared:
        if not target.exists():
            with target.open('xb') as stream:
                stream.write(data)
        records.append(dict(slot=resource['slot'], entry_id=resource['entry_id'],
                            source_path=entry['path'], source_sha256=resource['input_sha256'],
                            staged_sha256=sha(data), staged_bytes=len(data),
                            representation='rnc1_decoded', private_path=target.relative_to(ROOT).as_posix()))
    return dict(prepared=records, proof_sha256=sha(proof_bytes), original_execution=False,
                scope='Twelve exact CD plant resources; no live mission, visibility, order, or animation timing inferred.')


if __name__ == '__main__':
    receipt = stage()
    target = ROOT / 'research/integration/plant-inputs.json'
    fd, temporary = tempfile.mkstemp(dir=target.parent, prefix='plant-inputs-', suffix='.tmp')
    try:
        with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as stream:
            stream.write(json.dumps(receipt, indent=2) + '\n')
        os.replace(temporary, target)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    print(json.dumps(receipt))

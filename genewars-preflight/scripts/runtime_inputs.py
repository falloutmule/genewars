"""Stage only the exact, previously decoded CD runtime resources. No execution."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / 'research/integration/runtime-resource-catalog.tsv'


def stage():
    # Reproduce the two small mapped physics inputs before catalog admission.
    from physics_inputs import stage as stage_physics
    stage_physics()
    from creature_inputs import stage as stage_creature
    stage_creature()
    from construction_inputs import stage as stage_construction
    stage_construction()
    private = (ROOT / 'private').resolve()
    destination = private / 'integration/game-runtime'
    # These raw ANBs were not part of the earlier decoder corpus. Reproduce
    # missing private copies through the manifest-bound disc reader.
    new_buildings = {'buildings/BSOLA.ANB', 'buildings/BMINE.ANB', 'buildings/BPOWE.ANB'}
    missing = []
    for line in CATALOG.read_text(encoding='utf-8').splitlines():
        fields = line.split('\t')
        if len(fields) == 5 and fields[0] in new_buildings and not (ROOT / fields[3]).exists():
            missing.append(fields)
    if missing:
        from source_member import SourceReader
        with SourceReader() as reader:
            for name, digest, length, origin, provenance in missing:
                data = reader.read(provenance.split(';')[0], max_bytes=1024*1024)
                if len(data) != int(length) or hashlib.sha256(data).hexdigest() != digest:
                    raise ValueError('Building source differs: ' + name)
                output = (ROOT / origin).resolve()
                if not output.is_relative_to(private):
                    raise ValueError('Building staging escaped private storage')
                output.parent.mkdir(parents=True, exist_ok=True)
                with output.open('xb') as stream:
                    stream.write(data)
    pending = []
    for line in CATALOG.read_text(encoding='utf-8').splitlines():
        if not line or line.startswith('#'):
            continue
        name, digest, length, origin, provenance = line.split('\t')
        source = (ROOT / origin).resolve()
        target = (destination / name).resolve()
        if not source.is_relative_to(private) or not target.is_relative_to(destination):
            raise ValueError('Runtime resource escaped private storage')
        data = source.read_bytes()
        if len(data) != int(length) or hashlib.sha256(data).hexdigest() != digest:
            raise ValueError('Runtime resource differs: ' + origin)
        if target.exists() and target.read_bytes() != data:
            raise ValueError('Refusing changed runtime input: ' + name)
        pending.append((target, data, name, digest, provenance))
    for target, data, *_ in pending:
        target.parent.mkdir(parents=True, exist_ok=True)
        if not target.exists():
            with target.open('xb') as stream:
                stream.write(data)
    receipt = dict(resources=[dict(path=n, sha256=h, size=len(d), provenance=p)
                              for _, d, n, h, p in pending],
                   original_execution=False, catalog_sha256=hashlib.sha256(CATALOG.read_bytes()).hexdigest())
    (ROOT / 'research/integration/runtime-inputs.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    return destination, receipt


if __name__ == '__main__':
    directory, receipt = stage()
    print(json.dumps(dict(directory=str(directory), verified_resources=len(receipt['resources']))))

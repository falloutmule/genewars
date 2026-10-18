"""Stage exact CD map/B32A/FADE/MAIN inputs for native terrain rendering."""
import hashlib
import json
import os
import tempfile

from source_member import ROOT, SourceReader


# Explicit manifest identity, source digest, and final decoded digest.
SOURCES = {
    'CONQ001.DAT': ('root/2/disc/152', '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2', None),
    'B32A.DAT': ('root/2/disc/40', '481677e31b05d9e4ba428932a4eaf9003fae4fe320f90daf22a947cbd3e4ef9c',
                 '8206e4be65dd7f1fa12df131cf2d1aab5f5b47f7181c8ace86facbb1787bcdb8'),
    'FADE.DAT': ('root/2/disc/61', 'ddef38a3e63fbb3680aac15569f12e02f141036ffa270ddd48cdc82bce361503',
                 'b1297e2c767573164d8f2f9266a0bfd9923bb1608d0d1a22f3f3682da03d5392'),
    'MAIN.PAL': ('root/2/disc/108', '8d9a27bca8ed203629da4ee72bb982150fae5dd876743b50a99ad90dba0f4b05', None),
    'MAIN.GLA': ('root/2/disc/107', 'c5e12a5b4325c8302cab0fb3ba3aaa654e2e044131763298815766d8a6f47a58',
                 'bf841a1c4fca2a80fbef3c2fd6b502c4af34ede8e67b41ff69f64836726ea5b3'),
    'GENEPANL.DAT': ('root/2/disc/86', '8116b2b883320b8573c50496a570f0f3fa978e4c20b9c85a284339ff1f920e24',
                     'af9741b4545fb6f28a1dd0e50da0570e39992e75c872bc2249891f3330f59951'),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def stage():
    corpus = json.loads((ROOT / 'research/integration/rnc-corpus.json').read_bytes())
    decoded = {row['entry_id']: row for row in corpus['results']}
    destination = ROOT / 'private/integration/terrain-inputs'
    destination.mkdir(parents=True, exist_ok=True)
    records = []
    with SourceReader() as source:
        for filename, (entry_id, expected_source, expected_decoded) in SOURCES.items():
            entry = source.entries[entry_id]
            if entry['sha256'] != expected_source:
                raise ValueError('Terrain source identity changed: ' + entry_id)
            if expected_decoded is None:
                data = source.read(entry_id)
                expected = expected_source
            else:
                row = decoded[entry_id]
                if row['status'] != 'passed' or row['input_sha256'] != expected_source:
                    raise ValueError('Missing validated terrain RNC record: ' + entry_id)
                path = (ROOT / row['private_output']).resolve()
                if not path.is_relative_to((ROOT / 'private').resolve()):
                    raise ValueError('Decoded terrain resource escaped private storage')
                data = path.read_bytes()
                expected = expected_decoded
                if row['output_sha256'] != expected or len(data) != row['output_size']:
                    raise ValueError('Terrain decoded receipt differs: ' + entry_id)
            if sha(data) != expected:
                raise ValueError('Terrain bytes changed: ' + entry_id)
            target = destination / filename
            if target.exists():
                if target.read_bytes() != data:
                    raise ValueError('Refusing to overwrite changed terrain input: ' + filename)
            else:
                with target.open('xb') as stream:
                    stream.write(data)
            records.append(dict(entry_id=entry_id, source_path=entry['path'], source_sha256=expected_source,
                                staged_sha256=expected, staged_bytes=len(data),
                                representation='rnc1_decoded' if expected_decoded else 'raw_source',
                                private_path=target.relative_to(ROOT).as_posix()))
    return records


if __name__ == '__main__':
    receipt = dict(prepared=stage(), original_execution=False,
                   scope='Exact resource binding for explicit-input native rendering; not a loaded live mission.')
    target = ROOT / 'research/integration/terrain-inputs.json'
    fd, temporary = tempfile.mkstemp(dir=target.parent, prefix='terrain-inputs-', suffix='.tmp')
    try:
        with os.fdopen(fd, 'w', encoding='utf-8', newline='\n') as stream:
            stream.write(json.dumps(receipt, indent=2) + '\n')
        os.replace(temporary, target)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)
    print(json.dumps(receipt))

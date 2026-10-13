"""Stage only the CD menu's hash-bound inputs, using existing RNC outputs.

No original executable is launched and no archive tree is extracted. These
decoded resources remain private; the returned record contains only metadata.
"""
import hashlib
import json

from source_member import ROOT, SourceReader


# Input hashes bind the exact CD candidate instead of choosing by basename.
SOURCES = {
    'font2.dat': ('root/2/disc/75', '2f3d275f88f2ff6671562a819cbc7a791b16ad4510a4020f3b6078c917f60dba'),
    'font2.tab': ('root/2/disc/76', 'f54057e1b19cc1711a1596b6459cfdfb359a170f5ed739df71fd852befba4bab'),
    'iftitle.anb': ('root/2/disc/92', 'c7d2dfc5d774c69337436249ec74e4d0fa6ae1d06bd5ae785f9f73ee2ac0f707'),
    'titlescr.dat': ('root/2/disc/135', '2548b024fede333b49fc4011541939fc8ce294cb95b48cdb41ed7d0cec54a0b9'),
    'titlescr.pal': ('root/2/disc/136', 'd5ec91e731c2ff7e8d74dbbde6a3fe0410ec602ff0da52eb1e6f6bb1a44307c7'),
    'text.dat': ('root/2/disc/592', '23e60305a9fa028a0f6541456ab453b7e7cc439f787a2b75e4b8d088e0b01786'),
}


def sha(data):
    return hashlib.sha256(data).hexdigest()


def stage():
    corpus = json.loads((ROOT / 'research/integration/rnc-corpus.json').read_bytes())
    decoded = {row['entry_id']: row for row in corpus['results']}
    destination = ROOT / 'private/integration/menu'
    destination.mkdir(parents=True, exist_ok=True)
    records = []
    with SourceReader() as source:
        for filename, (entry_id, expected) in SOURCES.items():
            entry = source.entries[entry_id]
            if entry['sha256'] != expected:
                raise ValueError('CD menu source identity changed: ' + entry_id)
            if filename == 'text.dat':
                data = source.read(entry_id)
            else:
                row = decoded[entry_id]
                if row['status'] != 'passed' or row['input_sha256'] != expected:
                    raise ValueError('Missing validated RNC menu input: ' + entry_id)
                path = (ROOT / row['private_output']).resolve()
                if not path.is_relative_to((ROOT / 'private').resolve()):
                    raise ValueError('Decoded resource path escaped private storage')
                data = path.read_bytes()
                if len(data) != row['output_size'] or sha(data) != row['output_sha256']:
                    raise ValueError('Decoded menu resource changed: ' + entry_id)
            target = destination / filename
            if target.exists():
                if target.read_bytes() != data:
                    raise ValueError('Refusing to overwrite changed private menu input: ' + filename)
            else:
                with target.open('xb') as stream:
                    stream.write(data)
            records.append({
                'entry_id': entry_id, 'source_path': entry['path'],
                'source_sha256': expected, 'source_size': entry['size'],
                'decoded_size': len(data), 'decoded_sha256': sha(data),
                'private_path': target.relative_to(ROOT).as_posix(),
            })
    return records


if __name__ == '__main__':
    print(json.dumps({'prepared': stage(), 'original_execution': False}, indent=2))

"""Validate ANB embedded sprite tables; record failures without assuming pairing."""
import hashlib
import json
import struct
import subprocess
from source_member import ROOT, SourceReader


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    corpus = json.loads((ROOT / 'research/integration/rnc-corpus.json').read_text(encoding='utf-8'))
    decoded = {row['entry_id']: row for row in corpus['results']}
    codec_path = ROOT / 'research/ghidra/creature-evidence.json'
    codec_evidence = json.loads(codec_path.read_text(encoding='utf-8'))
    packed4 = set(codec_evidence['packed4_source_sha256'])
    admissions = {source['entry_id']: (row['source_sha256'], source['source_path'])
                  for row in codec_evidence['admitted_sources'] for source in row['sources']}
    if len(packed4) != 25 or {value[0] for value in admissions.values()} != packed4:
        raise ValueError('Unexpected creature admission contract; review before extending coverage')
    bones_path = ROOT / 'research/ghidra/bones-evidence.json'
    bones = json.loads(bones_path.read_text(encoding='utf-8'))
    bones_hash = '2106ce79deaafe53d5fbb6292c78a8c71fb6920b08d1a11b5c44ea3b0ab75e9f'
    if bones['status'] != 'verified_packed4' or bones['packed4_source_sha256'] != [bones_hash]:
        raise ValueError('Unexpected separate BONES admission contract')
    bones_admissions = {row['entry_id']: (row['sha256'], row['path']) for row in bones['source_aliases']}
    if set(bones_admissions) != {'root/1/221', 'root/2/disc/319'} or any(v[0] != bones_hash for v in bones_admissions.values()):
        raise ValueError('Unexpected BONES source provenance')
    admissions.update(bones_admissions)
    packed4.add(bones_hash)
    scratch = ROOT / 'private/integration/sprites'
    scratch.mkdir(parents=True, exist_ok=True)
    exe = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    cache, results = {}, []
    with SourceReader() as reader:
        entries = [e for e in reader.entries.values() if e['kind'] == 'file'
                   and e['path'].upper().split(';')[0].endswith('.ANB')]
        for entry in entries:
            key = entry['sha256']
            if key in packed4 and admissions.get(entry['entry_id']) != (key, entry['path']):
                raise ValueError('Packed4 source provenance is not explicitly admitted')
            if key not in cache:
                original = reader.read(entry['entry_id'])
                if original.startswith(b'RNC'):
                    record = decoded[entry['entry_id']]
                    data_path = ROOT / record['private_output']
                    data = data_path.read_bytes()
                    if record['status'] != 'passed' or sha(data) != record['output_sha256']:
                        raise ValueError('Decoded ANB cache is not validated')
                else:
                    data = original
                    data_path = scratch / (key + '.anb')
                    data_path.write_bytes(data)
                encoding = 'packed4-high-first' if key in packed4 else 'indexed8'
                info = {'decoded_size': len(data), 'decoded_sha256': sha(data), 'encoding': encoding,
                        'codec_admission': ('hash-and-caller-bound BONES evidence' if key == bones_hash else
                                            'hash-and-caller-bound creature evidence' if key in packed4 else
                                            'existing indexed8 structural probe')}
                try:
                    if len(data) < 18:
                        raise ValueError('Short ANB header')
                    count, *offsets = struct.unpack_from('<HIIII', data)
                    if not (18 <= offsets[0] <= offsets[1] <= offsets[2] <= offsets[3] <= len(data)):
                        raise ValueError('ANB sections are not ordered within file bounds')
                    table = data[offsets[0]:offsets[1]]
                    if len(table) % 6:
                        raise ValueError('Embedded sprite section is not divisible into six-byte records')
                    table_path = scratch / (key + '.tab')
                    table_path.write_bytes(table)
                    # Offsets remain relative to byte18 of the original file.
                    # Retain that prefix but exclude all table/animation metadata.
                    bounded_path = scratch / (key + '-sprite-prefix.bin')
                    bounded_path.write_bytes(data[:offsets[0]])
                    # Base zero exposes raw nibbles for structural diagnostics.
                    # It is not a claim about runtime creature palette selection.
                    command = [str(exe), 'sprite4-verify' if key in packed4 else 'sprite-verify', str(bounded_path), str(table_path)]
                    if key in packed4:
                        command.append('0')
                    proc = subprocess.run(command,
                                          capture_output=True, text=True, timeout=10)
                    info.update(header_word=count, section_offsets=offsets, embedded_table_size=len(table),
                                embedded_table_sha256=sha(table), private_database=str(data_path.relative_to(ROOT)),
                                private_bounded_database=str(bounded_path.relative_to(ROOT)),
                                private_table=str(table_path.relative_to(ROOT)), exit_code=proc.returncode)
                    if proc.returncode:
                        raise ValueError(proc.stderr.strip())
                    info.update(status='passed', native_result=json.loads(proc.stdout))
                except (ValueError, subprocess.TimeoutExpired) as error:
                    info.update(status='blocked', reason=str(error))
                cache[key] = info
            results.append({'entry_id': entry['entry_id'], 'source_path': entry['path'],
                            'input_sha256': key, **cache[key]})
    summary = {'records': len(results), 'unique_inputs': len(cache),
               'passed': sum(r['status'] == 'passed' for r in results),
               'blocked': sum(r['status'] != 'passed' for r in results),
               'packed4_source_records': sum(r['encoding'] == 'packed4-high-first' for r in results),
               'packed4_unique_inputs': len(packed4),
               'packed4_creature_unique_inputs': len(packed4) - 1,
               'packed4_bones_source_records': sum(r['input_sha256'] == bones_hash for r in results),
               'unique_sprite_records': sum(len(r['native_result']['records']) for r in cache.values() if r['status'] == 'passed')}
    report = {'schema_version': 1, 'summary': summary, 'results': results,
              'decoder_executable_sha256': sha(exe.read_bytes()),
              'codec_evidence': {'path': str(codec_path.relative_to(ROOT)), 'sha256': sha(codec_path.read_bytes())},
              'bones_codec_evidence': {'path': str(bones_path.relative_to(ROOT)), 'sha256': sha(bones_path.read_bytes())},
              'scope': 'ANB header and embedded sprite records checked with RLE strictly bounded before metadata. Packed4 is selected only for 25 hash-and-provenance-bound creature inputs plus one separately proved BONES input admitted by CD caller evidence; base0 is a structural diagnostic, not runtime palette or lighting. Other inputs retain the indexed8 structural probe. This supersedes the unsafe full-file-bound result. Parsing is not cross-build semantic equivalence, complete animation reconstruction, or runtime verification.',
              'source_archive_sha256': reader.manifest['source']['planning_sha256']}
    (ROOT / 'research/integration/sprite-corpus.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'summary': summary, 'blocked': [{k: r[k] for k in ('entry_id', 'reason')} for r in results if r['status'] != 'passed']}))
    return 0 if summary['blocked'] == 0 else 1


if __name__ == '__main__':
    raise SystemExit(main())

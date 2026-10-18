"""Validate every inventoried RNC record through the compiled Rust decoder."""
import hashlib
import json
from pathlib import Path
import subprocess

from source_member import ROOT, SourceReader

# These are observed layouts for exact file hashes, not a generic signature scan.
COMPOUND = {
    '9385b0e7d1e073daf2ff36725e2e53c541b73ac30a6a70199d0ab4d7e9b3d898':
        {'first_end': 114753, 'tail_kind': 'rnc1', 'tail_size': 50604},
    '055403c8dd9041a937862a792210be2da82cc1955a5a601ace7aa93eaa60cf40':
        {'first_end': 334383, 'tail_kind': 'raw', 'tail_size': 66552},
}
SHARED_TAIL = '154574cc30f55e1a386cc005c7eaf74cd0933edcabd1e4b0368b6ce4a1c9946c'


def decode_one(executable, scratch, data, name, prefix=False):
    target = scratch / (name + '.bin')
    command = [str(executable), 'rnc-prefix' if prefix else 'rnc-decode', '-', str(target), str(64 * 1024 * 1024)]
    try:
        proc = subprocess.run(command, input=data, capture_output=True, timeout=10)
        item = {'exit_code': proc.returncode, 'stderr': proc.stderr.decode('utf-8', 'replace')}
        if proc.returncode == 0:
            output = target.read_bytes()
            item.update(status='passed', output_size=len(output), output_sha256=hashlib.sha256(output).hexdigest(),
                        decoder_result=json.loads(proc.stdout), private_output=str(target.relative_to(ROOT)))
        else:
            item.update(status='failed', stdout=proc.stdout.decode('utf-8', 'replace'))
    except subprocess.TimeoutExpired:
        item = {'status': 'failed', 'reason': '10-second decode limit exceeded'}
    return item


def decode_compound(executable, scratch, data, key):
    layout = COMPOUND[key]
    first = decode_one(executable, scratch, data, key + '-first', prefix=True)
    if first['status'] != 'passed':
        return first
    consumed = first['decoder_result']['consumed_bytes']
    if consumed != layout['first_end'] or len(data) - consumed != layout['tail_size']:
        return {'status': 'failed', 'reason': 'Observed compound boundary changed', 'first_stream': first}
    tail = data[consumed:]
    if layout['tail_kind'] == 'rnc1':
        tail_result = decode_one(executable, scratch, tail, key + '-tail')
    else:
        tail_result = {'status': 'passed', 'output_size': len(tail),
                       'output_sha256': hashlib.sha256(tail).hexdigest(), 'validation': 'raw tail, not decompressed'}
    matches = tail_result.get('output_sha256') == SHARED_TAIL and tail_result.get('output_size') == 66552
    return {'status': 'passed' if tail_result['status'] == 'passed' and matches else 'failed',
            'layout': 'rnc1_prefix_with_' + layout['tail_kind'] + '_tail', 'first_stream': first,
            'tail_offset': consumed, 'tail_input_size': len(tail), 'tail_input_sha256': hashlib.sha256(tail).hexdigest(),
            'tail': tail_result, 'shared_tail_relation_verified': matches,
            'interpretation': 'Stream boundaries and decoded-byte equality verified; tail resource semantics unknown.'}


def main():
    executable = ROOT / 'private/build/rust-target/debug/genewars-tools.exe'
    scratch = ROOT / 'private/integration/rnc-decoded'
    scratch.mkdir(parents=True, exist_ok=True)
    if not executable.is_file():
        raise SystemExit('Build genewars-tools first with scripts/cargo.py build -p genewars-tools --offline')
    results, cache = [], {}
    with SourceReader() as reader:
        entries = [e for e in reader.entries.values() if e['classification'] == 'compressed_resource_not_decoded']
        for entry in entries:
            data = reader.read(entry['entry_id'])
            key = entry['sha256']
            if key not in cache:
                cache[key] = (decode_compound(executable, scratch, data, key) if key in COMPOUND
                              else decode_one(executable, scratch, data, key))
            results.append({'entry_id': entry['entry_id'], 'source_path': entry['path'], 'input_sha256': key,
                            'input_size': len(data), 'method_byte': data[3], **cache[key]})
    summary = {'records': len(results), 'unique_inputs': len(cache),
               'passed': sum(e['status'] == 'passed' for e in results),
               'failed': sum(e['status'] != 'passed' for e in results),
               'compound_records': sum(e['input_sha256'] in COMPOUND for e in results)}
    record = {'schema_version': 2, 'source_archive_sha256': reader.manifest['source']['sha256_before'],
              'decoder_executable_sha256': hashlib.sha256(executable.read_bytes()).hexdigest(),
              'command_template': 'genewars-tools rnc-decode - private/integration/rnc-decoded/<source-sha256>.bin 67108864',
              'validation': 'Each source record hash checked; Rust verifies packed length/CRC, bounded decode, decoded length/CRC. Equal input hashes share one decode. Exact-hash compound layouts use an explicit prefix API and independently account for every tail byte.',
              'summary': summary, 'results': results,
              'scope': 'Resource decompression only, not image/sprite interpretation or observed original game execution.'}
    destination = ROOT / 'research/integration/rnc-corpus.json'
    destination.write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'summary': summary, 'failures': [e for e in results if e['status'] != 'passed']}))
    return 0 if summary['failed'] == 0 else 1


if __name__ == '__main__':
    raise SystemExit(main())

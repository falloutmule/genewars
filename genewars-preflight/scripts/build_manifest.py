"""Reproducible static inventory; stdlib only, never executes archive members."""
from __future__ import annotations

import argparse
import collections
import hashlib
import io
import json
from pathlib import Path, PurePosixPath
import sys
import zipfile

from inventory_common import analyze_bytes

EXPECTED_SOURCE_SHA256 = '8153f36276f46acfb356f3199c7c2e89c9fa936b2afdf1b37039f143e2657906'
DISC_NAME = 'Genewars_DOS_EN_ISO-Version.7z'


def hash_file(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def build(source, root):
    before = hash_file(source)
    entries, containers, warnings = [], [], []

    def walk_zip(zf, parent, depth=0, parent_id='root'):
        container = {'path': parent, 'container_id': parent_id, 'member_count': len(zf.infolist()), 'format': 'ZIP',
                     'inspection_status': 'inspected', 'crc_checks': 'all read file members verified by zipfile'}
        containers.append(container)
        for ordinal, info in enumerate(zf.infolist()):
            path = parent + '!/' + info.filename
            record = {'path': path, 'parent_container': parent, 'member_name': info.filename,
                      'entry_id': parent_id + '/' + str(ordinal), 'parent_id': parent_id,
                      'member_ordinal': ordinal, 'kind': 'directory' if info.is_dir() else 'file',
                      'size': info.file_size, 'compressed_size': info.compress_size,
                      'zip_crc32': f'{info.CRC:08x}', 'zip_timestamp_untrusted': list(info.date_time),
                      'inspection_status': 'inspected'}
            norm = info.filename.replace('\\', '/')
            record['path_safety'] = 'unsafe_for_extraction' if (norm.startswith('/') or '..' in PurePosixPath(norm).parts or ':' in norm) else 'relative'
            if info.is_dir():
                record.update(sha256=None, signature=None, classification='directory')
                entries.append(record)
                continue
            if info.file_size > 1024 * 1024 * 1024:
                record.update(sha256=None, signature=None, classification='uninspected',
                              inspection_status='blocked', reason='member exceeds 1 GiB in-memory inspection budget')
                entries.append(record)
                continue
            try:
                data = zf.read(info)  # Uses ZipInfo, so duplicate names retain their identity.
                record.update(analyze_bytes(data, info.filename))
                if len(data) != info.file_size:
                    raise ValueError('decoded size differs from archive directory')
            except Exception as exc:
                record.update(sha256=None, signature=None, classification='uninspected',
                              inspection_status='blocked', reason=f'{type(exc).__name__}: {exc}')
                entries.append(record)
                continue
            entries.append(record)
            if zipfile.is_zipfile(io.BytesIO(data)):
                if depth >= 8:
                    record.update(container_inspection_status='blocked', container_reason='depth exceeds 8')
                else:
                    try:
                        with zipfile.ZipFile(io.BytesIO(data)) as inner:
                            walk_zip(inner, path, depth + 1, record['entry_id'])
                        record['container_inspection_status'] = 'inspected'
                    except Exception as exc:
                        record.update(container_inspection_status='blocked', container_reason=f'{type(exc).__name__}: {exc}')
            elif record['signature']['detected'] == 'ZIP':
                record.update(container_inspection_status='blocked', container_reason='ZIP signature present but no readable ZIP central directory')
            elif record['signature']['detected'] == '7z':
                record['container_inspection_status'] = 'pending_disc_inventory'

    with zipfile.ZipFile(source) as zf:
        walk_zip(zf, source.name)
    disc_path = root / 'research' / 'disc-inventory.json'
    if disc_path.exists():
        disc = json.loads(disc_path.read_text(encoding='utf-8'))
        source_record = next((e for e in entries if e['path'] == source.name + '!/' + DISC_NAME), None)
        if source_record is None or disc.get('source_archive_sha256') != source_record.get('sha256'):
            raise ValueError('Disc inventory SHA-256 does not match the actual nested 7z; rerun disc_inventory.py')
        if not disc.get('self_check', {}).get('all_archive_members_accounted_for'):
            raise ValueError('Disc inventory did not account for every listed 7z member')
        prefix = source.name + '!/' + DISC_NAME + '!/'
        disc_ids = {item['path']: source_record['entry_id'] + '/disc/' + str(i) for i, item in enumerate(disc['entries'])}
        for i, item in enumerate(disc['entries']):
            entry = dict(item)
            entry['path'] = prefix + item['path']
            entry.setdefault('parent_container', entry['path'].rsplit('!/', 1)[0])
            entry['entry_id'] = source_record['entry_id'] + '/disc/' + str(i)
            entry['parent_id'] = disc_ids.get(item['path'].rsplit('!/', 1)[0], source_record['entry_id'])
            entries.append(entry)
        for entry in entries:
            if entry['path'] == source.name + '!/' + DISC_NAME:
                incomplete = (any(x.get('inspection_status') == 'blocked' for x in disc['entries']) or
                              disc.get('image_metadata', {}).get('inspection_status') != 'inspected')
                entry['container_inspection_status'] = 'partially_inspected' if incomplete else 'inspected'
        containers.append({'path': source.name + '!/' + DISC_NAME, 'format': '7z',
                           'inspection_status': source_record['container_inspection_status'],
                           'details_file': 'disc-inventory.json'})
        warnings.extend(disc.get('warnings', []))
    else:
        disc = None
        warnings.append('disc-inventory.json missing; 7z contents not inventoried')
    hashes = collections.defaultdict(list)
    names = collections.defaultdict(list)
    for e in entries:
        if e.get('sha256'):
            hashes[e['sha256']].append({'entry_id': e['entry_id'], 'path': e['path']})
        if e['kind'] == 'file':
            names[PurePosixPath(e['path']).name.casefold()].append({'entry_id': e['entry_id'], 'path': e['path']})
    after = hash_file(source)
    assertions = {
        'source_hash_unchanged': before == after,
        'source_matches_planning_hash': before == EXPECTED_SOURCE_SHA256,
        'all_inspected_files_have_sha256': all(e.get('sha256') and len(e['sha256']) == 64 for e in entries if e['kind'] == 'file' and e['inspection_status'] == 'inspected'),
        'all_entries_have_required_fields': all(all(k in e for k in ('path', 'size', 'sha256', 'signature', 'classification', 'inspection_status', 'kind')) for e in entries),
        'disc_inventory_present': disc is not None,
        'no_pending_containers': not any(e.get('container_inspection_status') == 'pending_disc_inventory' for e in entries),
        'disc_source_hash_matches': disc is not None and disc.get('source_archive_sha256') == source_record.get('sha256'),
        'entry_ids_unique': len({e['entry_id'] for e in entries}) == len(entries),
    }
    manifest = {
        'schema_version': 1,
        'source': {'path': str(source), 'size': source.stat().st_size, 'sha256_before': before,
                   'sha256_after': after, 'planning_sha256': EXPECTED_SOURCE_SHA256},
        'method': 'Static bytes only; Python stdlib ZIP/header analysis and separate tar/ISO9660 disc inventory. No game code execution.',
        'entries': entries, 'containers': containers,
        'disc_evidence': {'source_archive_sha256': disc.get('source_archive_sha256'),
                          'image_metadata': disc.get('image_metadata'),
                          'self_check': disc.get('self_check')} if disc else None,
        'duplicate_content_groups': [{'sha256': digest, 'references': paths} for digest, paths in sorted(hashes.items()) if len(paths) > 1],
        'repeated_basename_groups': [{'basename_casefolded': name, 'references': paths} for name, paths in sorted(names.items()) if len(paths) > 1],
        'counts': {'entries': len(entries), 'files': sum(e['kind'] == 'file' for e in entries),
                   'directories': sum(e['kind'] == 'directory' for e in entries),
                   'blocked_entries': sum(e['inspection_status'] == 'blocked' for e in entries),
                   'blocked_or_partial_containers': sum(e.get('container_inspection_status') in ('blocked', 'partially_inspected') for e in entries),
                   'duplicate_content_groups': sum(len(paths) > 1 for paths in hashes.values()),
                   'classifications': dict(sorted(collections.Counter(e['classification'] for e in entries).items()))},
        'checks': assertions, 'warnings': warnings,
        'scope_limits': ['File inventory is not asset decoding or game behavior validation.',
                         'LE header/table bounds do not establish successful Ghidra import or relocations.',
                         'Build strings, dates, filenames and bundled prose are unverified provenance claims.',
                         'Opaque resources may contain logical records not enumerated by this filesystem inventory.'],
    }
    output = root / 'research' / 'game-files-manifest.json'
    output.write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(json.dumps({'output': str(output), 'counts': manifest['counts'], 'checks': assertions}))
    return 0 if all(assertions.values()) else 2


if __name__ == '__main__':
    ap = argparse.ArgumentParser()
    ap.add_argument('--source', type=Path, default=Path(r'C:\Users\fallo\Downloads\genewars.zip'))
    ap.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    args = ap.parse_args()
    sys.exit(build(args.source, args.root))

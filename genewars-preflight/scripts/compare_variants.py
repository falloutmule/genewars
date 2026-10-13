"""Compare file identities and the disc/repack executable bytes without running them."""
import collections
import hashlib
import io
import itertools
import json
from pathlib import Path
import re
import zipfile

from build_manifest import hash_file

ROOT = Path(__file__).resolve().parents[1]


def main():
    manifest = json.loads((ROOT / 'research/game-files-manifest.json').read_text(encoding='utf-8'))
    source = Path(manifest['source']['path'])
    expected = manifest['source']['sha256_before']
    if hash_file(source) != expected:
        raise ValueError('Source archive no longer matches manifest')
    roots = {
        'repack': 'genewars.zip!/Genewars_DOS_EN_RIP-Version (1).zip!/Gene_Wars_RePack/',
        'larger_rip': 'genewars.zip!/Genewars_DOS_EN_RIP-Version.zip!/',
        'patch_payload': 'genewars.zip!/Genewars_Patch_DOS_EN_Patch.zip!/Gene_Wars_Patch/Gwp.exe!/patch/',
        'disc_game_tree': 'genewars.zip!/Genewars_DOS_EN_ISO-Version.7z!/Gene_Wars_ISO/GENEWARS.img!/GAME/',
    }
    groups, maps = {}, {}
    for label, prefix in roots.items():
        es = [e for e in manifest['entries'] if e['kind'] == 'file' and e['path'].startswith(prefix)]
        keyed = {}
        for e in es:
            key = re.sub(r';\d+$', '', e['path'][len(prefix):]).casefold()
            if key in keyed:
                raise ValueError('Normalization collision: ' + key)
            keyed[key] = e
        maps[label] = keyed
        groups[label] = {
            'source_prefix': prefix, 'file_count': len(es), 'total_file_bytes': sum(e['size'] for e in es),
            'classifications': dict(sorted(collections.Counter(e['classification'] for e in es).items())),
            'directory_groups': dict(sorted(collections.Counter(k.split('/')[0] if '/' in k else '(root)' for k in keyed).items())),
            'video_count': sum(e['classification'] == 'video' for e in es),
            'scope_note': 'Patch payload only; not an installed game' if label == 'patch_payload' else 'Files in this package/tree; completeness not established',
        }
    pairs = []
    for a, b in itertools.combinations(roots, 2):
        left, right = maps[a], maps[b]
        common = left.keys() & right.keys()
        identical = sorted(k for k in common if left[k]['sha256'] == right[k]['sha256'])
        changed = sorted(k for k in common if left[k]['sha256'] != right[k]['sha256'])
        pairs.append({'left': a, 'right': b, 'normalization': 'casefold, package root removed, terminal ISO ;version removed; no language/layout remapping',
                      'common_paths': len(common), 'identical_paths': len(identical), 'changed_paths': len(changed),
                      'left_only_count': len(left.keys() - right.keys()), 'right_only_count': len(right.keys() - left.keys()),
                      'identical': identical, 'changed': changed, 'left_only': sorted(left.keys() - right.keys()), 'right_only': sorted(right.keys() - left.keys())})
    game_binaries = []
    for label, files in maps.items():
        if 'gw.exe' in files:
            entry = files['gw.exe']
            game_binaries.append({'variant': label, **{k: entry[k] for k in ('path', 'size', 'sha256', 'executable')}})
    target = maps['disc_game_tree']['gw.exe']
    iso = target['iso9660']
    image_path = ROOT / 'private/disc/member-004.img'
    image_record = next(e for e in manifest['entries'] if e['path'].endswith('!/Gene_Wars_ISO/GENEWARS.img'))
    if hash_file(image_path) != image_record['sha256']:
        raise ValueError('Scratch disc image differs from inventoried bytes')
    payload = bytearray()
    with image_path.open('rb') as stream:
        for sector in range(iso['extent_lba'], iso['extent_lba'] + (target['size'] + 2047) // 2048):
            stream.seek(sector * 2352 + 16)
            payload.extend(stream.read(min(2048, target['size'] - len(payload))))
    if hashlib.sha256(payload).hexdigest() != target['sha256']:
        raise ValueError('Disc GW executable range does not match inventoried SHA-256')
    with zipfile.ZipFile(source) as outer:
        with zipfile.ZipFile(io.BytesIO(outer.read('Genewars_DOS_EN_RIP-Version (1).zip'))) as inner:
            repack = inner.read('Gene_Wars_RePack/GW.EXE')
    if hashlib.sha256(repack).hexdigest() != maps['repack']['gw.exe']['sha256']:
        raise ValueError('Repack executable does not match manifest')
    documented = {0x4e033: (116, 144), 0x4e034: (28, 144), 0x5619f: (117, 235),
                  0xae6ad: (117, 235), 0x12cb20: (44, 0), 0x12cb21: (1, 0)}
    differences = [{'offset': n, 'offset_hex': hex(n), 'disc_byte': x, 'repack_byte': y,
                    'matches_bundled_patcher_documentation': documented.get(n) == (x, y)}
                   for n, (x, y) in enumerate(zip(payload, repack)) if x != y]
    result = {'schema_version': 1, 'source_sha256': expected, 'groups': groups, 'pairwise_file_comparison': pairs,
              'game_binaries': game_binaries,
              'disc_to_repack_executable_comparison': {
                  'same_length': len(payload) == len(repack), 'differing_byte_count': len(differences),
                  'all_documented_changes_present': all((payload[n], repack[n]) == values for n, values in documented.items()),
                  'documented_change_count': sum(d['matches_bundled_patcher_documentation'] for d in differences),
                  'additional_change_count': sum(not d['matches_bundled_patcher_documentation'] for d in differences),
                  'differences': differences,
                  'interpretation': 'Observed byte differences only; author, date, motive and behavior of additional changes are unverified.'},
              'baseline_decision': 'No authoritative retail/reference build established. Disc candidate supplies source context; its QA BETA V5 label and all other provenance claims remain unverified.',
              'source_hash_unchanged': hash_file(source) == expected}
    (ROOT / 'research/variant-comparison.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'groups': groups, 'pair_counts': [{k: p[k] for k in ('left', 'right', 'common_paths', 'identical_paths', 'changed_paths', 'left_only_count', 'right_only_count')} for p in pairs],
                      'gw_binaries': [{'variant': e['variant'], 'size': e['size'], 'sha256': e['sha256'], 'le_offset': e['executable'].get('new_header_offset')} for e in game_binaries],
                      'disc_repack_diff_count': len(differences), 'source_hash_unchanged': result['source_hash_unchanged']}))


if __name__ == '__main__':
    main()

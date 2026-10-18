"""Validate the delivered evidence and reproducibility without running original code."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = ROOT.parent


def digest(path):
    with path.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def main():
    commands = []
    def run(args, stdin=None):
        result = subprocess.run(args, cwd=ROOT, input=stdin, capture_output=True, text=True, timeout=60)
        commands.append({'command': args, 'exit_code': result.returncode, 'stdout': result.stdout, 'stderr': result.stderr})
        return result
    artifacts = [ROOT / 'research/game-files-manifest.json', ROOT / 'research/variant-comparison.json']
    before = {p.name: digest(p) for p in artifacts}
    tests = run([sys.executable, '-B', '-m', 'unittest', 'discover', '-s', str(ROOT / 'scripts'), '-p', 'test_inventory.py', '-v'])
    build = run([sys.executable, '-B', str(ROOT / 'scripts/build_manifest.py')])
    compare = run([sys.executable, '-B', str(ROOT / 'scripts/compare_variants.py')])
    after = {p.name: digest(p) for p in artifacts}
    manifest = json.loads(artifacts[0].read_text(encoding='utf-8'))
    disc = json.loads((ROOT / 'research/disc-inventory.json').read_text(encoding='utf-8'))
    copies = sorted(p.relative_to(WORKSPACE).as_posix() for p in (ROOT / 'private').rglob('*') if p.is_file())
    ignored = run(['git', '-C', str(WORKSPACE), '-c', 'safe.directory=' + WORKSPACE.as_posix(), 'check-ignore', '-z', '--stdin'], '\0'.join(copies) + '\0')
    tracked = run(['git', '-C', str(WORKSPACE), '-c', 'safe.directory=' + WORKSPACE.as_posix(), 'ls-files', '--', 'genewars-preflight/private'])
    checks = {
        'seven_boundary_tests_pass': tests.returncode == 0,
        'manifest_rebuild_passes': build.returncode == 0,
        'variant_comparison_rebuild_passes': compare.returncode == 0,
        'deterministic_manifest_and_comparison': before == after,
        'manifest_assertions_pass': all(manifest['checks'].values()),
        'zero_blocked_file_records': manifest['counts']['blocked_entries'] == 0,
        'zero_blocked_or_partial_containers': manifest['counts']['blocked_or_partial_containers'] == 0,
        'disc_scope_inspected': disc['image_metadata']['inspection_status'] == 'inspected',
        'disc_source_unchanged': disc.get('source_archive_unchanged', False),
        'every_private_file_is_git_ignored': ignored.returncode == 0 and set(ignored.stdout.rstrip('\0').split('\0')) == set(copies),
        'no_private_files_git_tracked': tracked.returncode == 0 and not tracked.stdout.strip(),
        'original_archive_hash_unchanged': digest(Path(manifest['source']['path'])) == manifest['source']['planning_sha256'],
    }
    report = {'schema_version': 1, 'checks': checks, 'all_passed': all(checks.values()),
              'artifact_sha256_before': before, 'artifact_sha256_after': after,
              'private_files_checked': len(copies), 'counts': manifest['counts'], 'commands': commands,
              'limits': 'Static inventory and parser checks only; no asset decode, Ghidra import, patch execution, game execution, or behavioral tests.'}
    (ROOT / 'research/verification.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'checks': checks, 'all_passed': report['all_passed']}))
    return 0 if report['all_passed'] else 1


if __name__ == '__main__':
    sys.exit(main())

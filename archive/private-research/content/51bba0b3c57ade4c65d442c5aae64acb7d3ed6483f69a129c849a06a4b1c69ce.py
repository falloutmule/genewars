"""Read saved evidence and current hashes without building or running a game.

Exit zero means recovery completed, not that verification passed. Historical
failures, standalone repairs and checks added later remain separate fields.
"""
import argparse
import ast
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import subprocess

from verify import ROOT, WORKSPACE, BASELINE_HASHES, production_pins


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def registered_checks():
    tree = ast.parse((ROOT / 'scripts/verify.py').read_text(encoding='utf-8'))
    return {node.args[0].value for node in ast.walk(tree)
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name)
            and node.func.id == 'run' and node.args
            and isinstance(node.args[0], ast.Constant)
            and isinstance(node.args[0].value, str)}


def recover(check_archive=False):
    receipt_path = ROOT / 'research/integration/verification.json'
    receipt = json.loads(receipt_path.read_text(encoding='utf-8'))
    checks = receipt['checks']
    old = receipt.get('production_source_pins_after', {})
    current = production_pins()
    changed = sorted(p for p in old.keys() | current.keys()
                     if old.get(p) != current.get(p))
    commands = {c['label']: c for c in receipt['commands']}
    tests = commands.get('rust_workspace_tests', {}).get('stdout', '')
    git = ['git', '-c', 'safe.directory=' + WORKSPACE.as_posix()]

    def git_read(*args):
        return subprocess.run(git + list(args), cwd=ROOT, check=True,
                              capture_output=True, text=True).stdout.strip()

    status = git_read('status', '--short', '--untracked-files=all', '--', '.')
    rows = status.splitlines() if status else []
    record = {
        'schema_version': 1,
        'recovered_utc': datetime.now(timezone.utc).isoformat(),
        'scope': 'Read-only recovery; no checks rerun and no original program executed.',
        'git': {'branch': git_read('branch', '--show-current'),
                'head': git_read('rev-parse', 'HEAD'),
                'tracked_changed_paths': sum(not s.startswith('??') for s in rows),
                'untracked_paths': sum(s.startswith('??') for s in rows),
                'private_tracked': bool(git_read('ls-files', '--', 'private'))},
        'aggregate': {'path': receipt_path.relative_to(ROOT).as_posix(),
                      'sha256': digest(receipt_path),
                      'completed_utc': receipt.get('completed_utc'),
                      'passed': sum(v is True for v in checks.values()),
                      'total': len(checks), 'all_passed': receipt['all_passed'],
                      'failed_checks': [k for k, v in checks.items() if not v],
                      'rust_tests_passed': sum(int(n) for n in re.findall(
                          r'test result: ok\. (\d+) passed;', tests)),
                      'production_stable_during_run': checks.get('production_sources_stable_during_run'),
                      'private_files_checked_then': receipt.get('private_files_checked')},
        'current_production': {'pinned_paths': len(current),
                               'matches_aggregate': bool(old) and not changed,
                               'different_paths': changed},
        'registered_checks_absent_from_aggregate': sorted(registered_checks() - commands.keys()),
        'baseline_hashes': {name: {'expected': expected,
                                 'actual': digest(ROOT / 'research' / name)}
                            for name, expected in BASELINE_HASHES.items()},
        'failure_details': [{
            'label': label, 'exit_code': command.get('exit_code'),
            'timed_out': command.get('timed_out', False),
            'stderr_tail': command.get('stderr', '')[-1800:]}
            for label, command in commands.items() if checks.get(label) is False],
    }
    if check_archive:
        manifest = json.loads((ROOT / 'research/game-files-manifest.json').read_text(encoding='utf-8'))
        original = Path(manifest['source']['path'])
        actual = digest(original)
        record['archive'] = {'path': str(original), 'sha256': actual,
                             'unchanged': actual == manifest['source']['planning_sha256']}
    else:
        record['archive'] = {'status': 'not rehashed; use --check-archive',
                             'unchanged_at_aggregate': checks.get('original_archive_unchanged')}
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check-archive', action='store_true')
    parser.add_argument('--output', type=Path, help='Optional recovery JSON; source receipts are never rewritten.')
    args = parser.parse_args()
    record = recover(args.check_archive)
    output = json.dumps(record, indent=2) + '\n'
    if args.output:
        if args.output.resolve() == (ROOT / 'research/integration/verification.json').resolve():
            parser.error('recovery must not replace the aggregate receipt')
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(output, encoding='utf-8')
    print(output, end='')


if __name__ == '__main__':
    main()

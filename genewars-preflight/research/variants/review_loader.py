"""Rerun the reviewed SOL-01 static validator without writing its owned output."""
import ast
import json
from pathlib import Path

root = Path(__file__).resolve().parents[2]
source_path = root / 'scripts/ghidra/validate_le.py'
tree = ast.parse(source_path.read_text(), filename=str(source_path))
removed_writes = 0
removed_prints = 0
kept = []
for statement in tree.body:
    if isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Call):
        call = statement.value
        if isinstance(call.func, ast.Attribute) and call.func.attr == 'write_text':
            removed_writes += 1
            continue
        if isinstance(call.func, ast.Name) and call.func.id == 'print':
            removed_prints += 1
            continue
    kept.append(statement)
assert removed_writes == 1 and removed_prints == 1, 'Review source changed; inspect before rerun'
tree.body = kept
namespace = {'__file__': str(source_path), '__name__': '__review__'}
exec(compile(tree, str(source_path), 'exec'), namespace)
summary = namespace['summary']
print(json.dumps({key: summary[key] for key in [
    'candidate_sha256', 'format', 'entry_point', 'pages_verified',
    'fixup_instances', 'unique_fixup_addresses', 'cross_object_fixup_instances',
    'block_checks', 'all_checks_passed', 'limitations']}, indent=2))

"""Read-only805E0 comparison, preserving lazy actor lookup and signed counts."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import importlib.util
import itertools
import json
import random
import struct
import subprocess
import uuid
from cargo import environment

P = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    work = P / 'private/review/mission-status' / uuid.uuid4().hex
    work.mkdir(parents=True)
    module = P / 'rust/crates/genewars-runtime/src/mission_status.rs'
    (work / 'mission_status.rs').write_bytes(module.read_bytes())
    spec = importlib.util.spec_from_file_location('mission_status_source', P / 'research/variants/initial-creature-source.py')
    source_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source_module)

    class Source(source_module.Source):
        def get(self, operand):
            if self.pc == 0x80643 and operand == 'dword ptr [EDX + 0xc0]':
                self.queries.append((self.r['EDX'] - 0x16360e) // 200)
            return super().get(operand)

    source = Source()
    listing = P / 'private/ghidra/mission-array-refs/instructions-000805e0.txt'
    source.code.update(source_module.m.Model([listing], b'').code)
    order = sorted(source.code)
    source.next = dict(zip(order, order[1:]))
    rng = random.Random(0x805e0)
    data = bytearray()
    expected = bytearray()
    steps = 0
    cases = list(itertools.product([-2147483648, -1, 0, 1, 2147483647], [0, 1],
        [0, 1], [-128, -1, 0, 1, 3, 5], range(3)))
    coverage = set()
    for number, (field616, field600, field605, count, category_case) in enumerate(cases):
        player = bytearray(rng.randbytes(1805))
        player[3] = count & 255
        player[4:9] = bytes(rng.sample(range(20), 5))
        player[0x600] = field600
        player[0x605] = field605
        struct.pack_into('<i', player, 0x616, field616)
        categories = bytearray([1 + i % 3 for i in range(20)])
        if count > 0 and category_case != 0:
            categories[player[4 + (0 if category_case == 1 else count - 1)]] = 0
        for index in range(32):
            player[0x16 + index * 36] = 0
            player[0x16 + index * 36 + 22] = 1
        if number % 3 != 0:
            index = 0 if number % 3 == 1 else 31
            player[0x16 + index * 36] = 255
            player[0x16 + index * 36 + 22] = 0
        state = {0x118d74 + i: value for i, value in enumerate(player)}
        for index, category in enumerate(categories):
            pointer = 0x240000 + index * 36
            state.update({0x16360e + index * 200 + 0xc0 + j: value for j, value in enumerate(struct.pack('<I', pointer))})
            state[pointer + 22] = category
        source.queries = []
        report, _, trace = source.run(entry=0x805e0, state_before=state, registers={'EAX': 0x118d74})
        result = source.r['EAX'] & 255
        assert result in [0, 1]
        assert bytes(source.load(0x118d74 + i, 1) for i in range(1805)) == player
        expected.extend(bytes([result, len(source.queries)] + source.queries + [255] * (5 - len(source.queries))))
        data.extend(player + categories)
        steps += report['steps']
        coverage.update(trace)
    assert {0x805f7, 0x8060a, 0x80643, 0x80650, 0x80683, 0x80690} <= coverage
    (work / 'input.bin').write_bytes(data)
    (work / 'expected.bin').write_bytes(expected)
    harness = r'''mod player_state{pub const PLAYER_RECORD_BYTES:usize=1805;}
mod mission_status;
use std::{fs,io::Write,path::Path};
fn main(){let root=std::env::args().nth(1).unwrap();let root=Path::new(&root);let data=fs::read(root.join("input.bin")).unwrap();let mut out=fs::File::create(root.join("actual.bin")).unwrap();
for case in data.chunks_exact(1825){let player=case[..1805].try_into().unwrap();let mut visits=Vec::new();
let exhausted=mission_status::construction_capability_exhausted(player,|id|{visits.push(id as u8);Some(case[1805+id as usize])}).unwrap();
out.write_all(&[exhausted as u8,visits.len()as u8]).unwrap();visits.resize(5,255);out.write_all(&visits).unwrap();}}
'''
    (work / 'probe.rs').write_text(harness, encoding='utf-8')
    compiled = subprocess.run([str(P / 'private/tools/rust/cargo/bin/rustc.exe'), '--edition=2021', '--cap-lints=allow',
        '-O', str(work / 'probe.rs'), '-o', str(work / 'probe.exe')], env=environment(), capture_output=True, timeout=60)
    assert compiled.returncode == 0, compiled.stderr.decode(errors='replace')
    subprocess.run([str(work / 'probe.exe'), str(work)], check=True, timeout=30)
    actual = (work / 'actual.bin').read_bytes()
    assert actual == expected, [(i, a, b) for i, (a, b) in enumerate(zip(actual, expected)) if a != b][:20]
    receipt = dict(status='passed-read-only-predicate', utc=datetime.now(timezone.utc).isoformat(),
        cases=len(cases), compared_result_and_lookup_bytes=len(actual), unchanged_player_bytes=len(cases) * 1805,
        literal_instructions=steps, source_listing_sha256=sha(listing), module_sha256=sha(work / 'mission_status.rs'),
        validator_sha256=sha(Path(__file__)), private_run=work.relative_to(P).as_posix(), original_execution=False,
        scope='Complete805E0 literal body, signed live counts, lazy typed actor category lookup and all32 stored blueprints. Does not establish field616/600/605 semantic names, the enclosing72ED0 mission rules, UI transitions or complete win/loss.')
    destination = P / 'research/integration/mission-status-validation.json'
    destination.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()

"""Literal47F80 admitted building history/allocation maintenance over post-callback records.

The virtual building callback is an explicit no-mutation return0 port. This
checks the independent player arithmetic, not target search or building AI.
"""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import random
import struct
import subprocess
import uuid
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
from cargo import environment

P = Path(__file__).resolve().parents[2]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    work = P / 'private/review/admitted-building-history' / uuid.uuid4().hex
    work.mkdir(parents=True)
    source = P / 'rust/crates/genewars-runtime/src/bookkeeping.rs'
    (work / 'bookkeeping.rs').write_bytes(source.read_bytes())
    spec = importlib.util.spec_from_file_location('history_model', P / 'research/variants/initial-creature-source.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)

    class Source(module.Source):
        selected = 0

        def get(self, operand):
            if operand == 'byte ptr [0x0011873d]':
                return self.selected
            if self.pc == 0x4834a and operand == 'dword ptr [EDI + 0x64]':
                return 0x2e0000
            return super().get(operand)

    model = Source()
    pins = {
        'private/ghidra/mission-array-refs/instructions-00047f80.txt': 'bae7d9800c80bce7b1419e5b7a2193b1f934b5080dd30285c7a2a15b50a2a4e4',
        'private/ghidra/mission-array-refs/instructions-000437a0.txt': '7aa65558885556472096f9f32bc7e469dcc4f227385c258451ba4c105343156d',
    }
    assert all(sha(P / path) == digest for path, digest in pins.items())
    model.code.update(module.m.Model([P / path for path in pins], b'').code)
    model.code[0x2e0000] = ('XOR', ['EAX', 'EAX'])
    model.code[0x2e0001] = ('RET', [])
    for address, cleanup in [(0x981d3, 0), (0x41ce0, 20), (0x41910, 16)]:
        model.code[address] = ('RET', [hex(cleanup)] if cleanup else [])
    order = sorted(model.code)
    model.next = dict(zip(order, order[1:]))
    rng = random.Random(0x47f80437)
    data = bytearray()
    expected = bytearray()
    steps = 0
    timer_sets = [[], [200], [-32768], [32767], [-5, 4], [1001, 30000], [-30000, -1001], [0, 1, -1, 1200]]
    for case in range(256):
        selected = case % 4
        period = [0, 1, 6000, 65535][(case // 4) % 4]
        players = []
        buildings = bytearray(256 * 67)
        for owner in range(4):
            player = bytearray(rng.randbytes(1805))
            player[1] = player[0x4a1] = owner
            timers = timer_sets[(case + owner) % len(timer_sets)]
            player[0x4a2] = len(timers)
            player[0x5e5] = [0, 49, 255][(case + owner) % 3]
            struct.pack_into('<i', player, 0x4a7, [-1,0,1,599,600,601,999,1800,2147483647][(case+owner)%9])
            for i in range(8):
                struct.pack_into('<H', player, 0x5ea + 2 * i, [0, 1, 999, 1000, 65535][(case + owner + i) % 5])
            for ordinal, timer in enumerate(timers):
                slot = ordinal if case < 64 else [1, 3, 31, 63][ordinal]
                base = (owner * 64 + slot) * 67
                buildings[base + 6] = 4
                buildings[base + 24] = owner
                buildings[base + 45] = [1,3,6,11][(case+owner+ordinal)%4]
                struct.pack_into('<i',buildings,base+54,[-100,0,12345][ordinal%3])
                struct.pack_into('<h', buildings, base + 15, owner * 64 + slot)
                struct.pack_into('<I', buildings, base + 35, 0x112120)
                struct.pack_into('<h', buildings, base + 61, timer)
                struct.pack_into('<H', buildings, base + 49, [1, 0, 0x101][(case // 8 + slot) % 3])
            players.append(player)
        state = {0x118d74 + owner * 1805 + i: value for owner, player in enumerate(players) for i, value in enumerate(player)}
        state.update({0x1645ae + i: value for i, value in enumerate(buildings)})
        state.update({0x11878c: 4, 0x11a9da: period & 255, 0x11a9db: period >> 8})
        model.selected = selected
        report, _, _ = model.run(entry=0x47f80, state_before=state)
        expected.extend(bytes(model.load(0x118d74 + owner * 1805 + i, 1) for owner in range(4) for i in range(1805)))
        expected.extend(bytes(model.load(0x1645ae+i,1) for i in range(len(buildings))))
        data.extend(struct.pack('<BH', selected, period) + b''.join(players) + buildings)
        steps += report['steps']
    (work / 'input.bin').write_bytes(data)
    (work / 'expected.bin').write_bytes(expected)
    harness=(P/'scripts/desktop/admitted_building_player_harness.rs').read_text()
    (work / 'probe.rs').write_text(harness, encoding='utf-8')
    rustc = P / 'private/tools/rust/cargo/bin/rustc.exe'
    result = subprocess.run([str(rustc), '--edition=2021', '--cap-lints=allow', '-O', '-C', 'overflow-checks=yes', str(work / 'probe.rs'), '-o', str(work / 'probe.exe')], env=environment(), capture_output=True, timeout=60)
    assert result.returncode == 0, result.stderr.decode(errors='replace')
    subprocess.run([str(work / 'probe.exe'), str(work)], check=True, timeout=30)
    actual = (work / 'actual.bin').read_bytes()
    assert actual == expected, (len(actual), len(expected), next((i for i, (a, b) in enumerate(zip(actual, expected)) if a != b), None))
    receipt = dict(status='passed-post-callback-admitted-maintenance', utc=datetime.now(timezone.utc).isoformat(),
        cases=256, player_records=1024, compared_bytes=len(actual), literal_instructions=steps,
        pins=pins, module_sha256=sha(work / 'bookkeeping.rs'), validator_sha256=sha(Path(__file__)),
        boundaries=['451F0 virtual callback is a no-mutation return0 port; records are caller-supplied post-callback state',
                    'No target-search, RNG, attack or completed-building callback claim', 'Selector3/6 excluded from timer samples; selector11 allocation600 per encountered record, positive residual warnings and early exhaustion preserve later record allocations', 'Audio/UI services are explicit ports',
                    'Dense and sparse selectors1/3/6/11 grade0 partitions including slots1/3/31/63; initial min1000/max-1000 sentinels and signed truncating mean preserved'],
        private_run=work.relative_to(P).as_posix(), original_execution=False)
    (P / 'research/integration/admitted-building-player.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()

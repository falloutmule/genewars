"""Compare timer-expiry factory requests with the literal source caller.

The factory and audio/UI functions are explicit ports. Their bodies are not
claimed; source7B140 terrain height sampling executes in the interpreter.
"""
from pathlib import Path
import hashlib
import importlib.util
import json
import random
import struct
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from cargo import environment

P = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    spec = importlib.util.spec_from_file_location('expiry_source', P / 'research/variants/initial-creature-source.py')
    source = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source)

    class Capture(source.Source):
        def get(self, expression):
            value = super().get(expression)
            if self.pc == 0x13829 and expression == '0x000101d0':
                self.requests.append(struct.pack('<BBI I', self.r['ECX'] & 255,
                    self.r['EAX'] & 255, self.r['EDX'], self.r['EBX']))
            return value

    model = Capture()
    listing = P / 'private/ghidra/mission-array-refs/instructions-00013780.txt'
    assert sha(listing) == '634efec82d190cdf793a444a423709cd2e962031a6b2e20ec05c8606bc3a36b4'
    model.code.update(source.m.Model([listing], b'').code)
    order = sorted(model.code)
    model.next = dict(zip(order, order[1:]))
    for address, cleanup in [(0x101d0, 0), (0x981d3, 0), (0x41ce0, 20),
                             (0x41910, 16), (0x984c0, 0)]:
        model.code[address] = ('RET', [hex(cleanup)] if cleanup else [])

    work = P / 'private/review/countdown-expiry' / uuid.uuid4().hex
    work.mkdir(parents=True)
    mission = (P / 'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes()
    (work / 'map.bin').write_bytes(mission[0x355e:0x355e + 229376])
    randomizer = random.Random(13780101)
    inputs, expected = bytearray(), bytearray()
    steps = requests = 0
    for case in range(80):
        player_count = case % 5
        records = []
        for owner in range(4):
            record = bytearray(randomizer.randbytes(1805))
            record[0x6e3] = [0, 0, 1, 255][(case + owner) % 4]
            record[0x4a2] = [-128, -1, 0, 1, 127][(case // 4 + owner) % 5] & 255
            for offset in [9, 0x5fc]:
                struct.pack_into('<hh', record, offset,
                    randomizer.choice([-32768, -1, 0, 255, 32767, randomizer.randrange(-32768, 32768)]),
                    randomizer.choice([-32768, -1, 0, 255, 32767, randomizer.randrange(-32768, 32768)]))
            records.append(record)
        state = {0x118d74 + owner * 1805 + j: b for owner, record in enumerate(records) for j, b in enumerate(record)}
        state.update({0x16da36 + j: 0 for j in range(16 * 102)})
        state.update({0x11878c: player_count, 0x11c2bc: 1, 0x11c2bd: 0,
                      0x11945b: 0, 0x118770: 0})
        # The selected player's progress override lives inside its record;
        # keep the Rust input in agreement with the literal source state.
        records[0][0x6e7] = 0
        per_fill = []
        for fill in [0, 85, 255]:
            model.requests = []
            report, _, _ = model.run(state_before=state, entry=0x13780, stack_fill=fill)
            assert model.load(0x11c2bc, 2) == 0
            result = bytes([len(model.requests)]) + b''.join(model.requests)
            per_fill.append(result)
            steps += report['steps']
        assert per_fill[0] == per_fill[1] == per_fill[2]
        inputs += bytes([player_count]) + b''.join(records)
        expected += per_fill[0]
        requests += per_fill[0][0]
    (work / 'input.bin').write_bytes(inputs)
    subprocess.run([sys.executable, '-B', 'scripts/cargo.py', 'build', '-p', 'genewars-core', '--offline', '--locked'],
                   cwd=P, check=True, capture_output=True)
    target = P / 'private/build/rust-target/debug'
    executable = work / 'probe.exe'
    subprocess.run([str(P / 'private/tools/rust/cargo/bin/rustc.exe'), '--edition=2021',
                    'scripts/countdown_expiry_probe.rs', '-L', f'dependency={target}/deps',
                    '--extern', f'genewars_core={target}/libgenewars_core.rlib', '-o', str(executable)],
                   cwd=P, env=environment(), check=True, capture_output=True)
    subprocess.run([str(executable), str(work)], check=True)
    actual = (work / 'actual.bin').read_bytes()
    assert actual == expected, (len(actual), len(expected), next((i for i, (a, b) in enumerate(zip(actual, expected)) if a != b), None))
    paths = [Path(__file__), P / 'scripts/countdown_expiry_probe.rs',
             P / 'rust/crates/genewars-runtime/src/countdown_expiry.rs',
             P / 'research/variants/initial-creature-source.py']
    receipt = dict(status='passed-caller-requests-only', utc=datetime.now(timezone.utc).isoformat(),
        cases=80, factory_requests=requests, compared_bytes=len(actual), literal_instructions=steps,
        stack_fills=[0, 85, 255], implementation_pins={p.relative_to(P).as_posix(): sha(p) for p in paths},
        listing_pins={p.relative_to(P).as_posix(): sha(p) for p in model.files + [listing]},
        private_run=work.relative_to(P).as_posix(), original_execution=False,
        scope='Source13780 expiry caller and literal7B140 height sampling; ordered101D0 arguments match. Factory/audio/UI are request ports. No family8 creation, callback, world integration, or expiry support claimed.')
    (P / 'research/integration/countdown-expiry-validation.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: receipt[k] for k in ['status', 'cases', 'factory_requests', 'compared_bytes', 'literal_instructions']}))


if __name__ == '__main__':
    main()

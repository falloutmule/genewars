"""One literal6D500 code3C marker over eight captured buffers.

Fixed params: speedbase10/divisor10/owner-1; bounded count1..3. Source-only preparation
is not gameplay or Rust acceptance. Historical receipts are never overwritten.
"""
from pathlib import Path
import argparse
import importlib.util
import json
import struct
import uuid

P = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('captured_world', Path(__file__).with_name('genepod-captured-review.py'))
g = importlib.util.module_from_spec(spec)
spec.loader.exec_module(g)


class Source(g.k.Source):
    def __init__(self, backing):
        super().__init__(backing)
        extra = [P / 'private/ghidra' / name for name in g.EXTRA]
        extra.append(P / 'private/ghidra/activation-rng-refs/instructions-0006d500.txt')
        self.files += extra
        self.code.update(g.k.m.m.Model(extra, b'').code)
        order = sorted(self.code)
        self.next = dict(zip(order, order[1:]))
        self.journals = []

    def store(self, address, size, value):
        super().store(address, size, value)
        if 0x11862c <= address < 0x1186ac and size == 4:
            self.journals.append({'ring_slot': (address - 0x11862c) // 4,
                                  'journal': value & 0xffffffff, 'pc': hex(self.pc)})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--kind', choices=('factory', 'callback'), default='factory')
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--expected', type=Path)
    parser.add_argument('--slot', type=int, required=True)
    parser.add_argument('--frame', type=int, required=True)
    parser.add_argument('--count', type=int, choices=(1, 2, 3), default=1)
    args = parser.parse_args()
    before = g.resolve(args.before)
    expected = g.resolve(args.expected) if args.expected else None
    out = P / 'private/review/zone-marker-captured' / uuid.uuid4().hex
    out.mkdir(parents=True, exist_ok=False)
    report = dict(status='error', before=str(before), expected=str(expected) if expected else None,
                  output=str(out), frame=args.frame, slot=args.slot, production_comparison='not_run',
                  kind=args.kind, source_entry='6D500' if args.kind == 'factory' else '4B630', parameters={'code': 0x3c, 'variation': 0, 'speed_base': 10, 'divisor': 10, 'owner': -1, 'count': args.count},
                  normalization='g.export: native opaque31:39 from INPUT only; stale serialized players preserved',
                  scope='Bounded literal marker factory loop; no zone credit, scheduler or campaign acceptance')
    executed = []
    vm = None
    try:
        if not 0 <= args.slot < (300 if args.kind == 'factory' else 200) or not 0 <= args.frame <= 0xffffffff:
            raise ValueError('frame/slot outside source domain')
        buffers = g.read_buffers(before)
        if args.kind == 'factory':
            raw = buffers['creatures.bin'][args.slot * 124:(args.slot + 1) * 124]
            if raw[6] != 1 or raw[122] != 4 or raw[24] > 4 or raw[94] >= 24 or struct.unpack_from('<h', raw, 15)[0] != args.slot:
                raise ValueError('selected slot is not a live source Mule identity')
        else:
            raw = buffers['effects.bin'][args.slot * 81:(args.slot + 1) * 81]
            if raw[6] != 5 or raw[69] != 0x3c or raw[24] > 4 or struct.unpack_from('<h', raw, 15)[0] != args.slot:
                raise ValueError('selected slot is not an active code3C effect identity')
            report['parameters'] = None
            report['scope'] = 'One literal code3C callback; no zone scheduler or campaign acceptance'
        count = struct.unpack_from('<h', buffers['backing.bin'], 0x3554)[0]
        if not 0 <= count <= (199 if args.kind == 'factory' else 200):
            raise ValueError(('full-pool replacement or invalid count unsupported', count))
        state, seed, resources = g.prepare(buffers)
        if args.kind == 'factory':
            for address, value in ((0x300004, 10), (0x300008, args.count), (0x30000c, -1)):
                g.k.put(state, address, 4, value)
        vm = Source(buffers['backing.bin'])

        class TraceCode(dict):
            def __getitem__(self, address):
                executed.append(address)
                return super().__getitem__(address)

        vm.code = TraceCode(vm.code)
        report.update(input_pins={name: g.pin(before / name) for name in g.NAMES},
                      resource_pins=resources, source_pins={str(path.relative_to(P)): g.pin(path) for path in vm.files},
                      comparator_sha256=g.pin(Path(__file__)), shared_comparator_sha256=g.pin(Path(__file__).with_name('genepod-captured-review.py')))
        registers = {'EAX': 0x1542be + args.slot * 124, 'EDX': 0x3c, 'EBX': 0, 'ECX': 10} if args.kind == 'factory' else {'EAX': 0x1688ae + args.slot * 81}
        run, _, trace = vm.run(entry=0x6d500 if args.kind == 'factory' else 0x4b630,
                               counter=args.frame, seed=seed, state_before=state,
                               registers=registers, instruction_budget=100000)
        result = g.export(vm, buffers)
        for name, data in result.items():
            (out / name).write_bytes(data)
        exported_bytes = sum(map(len, result.values()))
        if exported_bytes != 427369:
            raise ValueError(('unexpected eight-buffer total', exported_bytes))
        rng_before = struct.unpack('<HBI', buffers['rng.bin'])
        rng_after = struct.unpack('<HBI', result['rng.bin'])
        report.update(status='source_prepared', exported_bytes=exported_bytes, instructions=run['steps'], draws=run['draws'],
                      journals=vm.journals, rng_before=rng_before, rng_after=rng_after,
                      output_pins={name: g.pin(out / name) for name in g.NAMES})
        if args.kind == 'factory':
            report['returned_count'] = vm.r['EAX'] & 255
        else:
            report['retired'] = result['effects.bin'][args.slot * 81 + 6] == 0
        # A free-pool successful source marker has exactly this interleaving.
        # Full-pool replacement is outside this captured adapter's proof domain.
        expected_journals = [0xf6, 0xf6, 0xf6, 0x229] * args.count if args.kind == 'factory' else []
        expected_draws = 4 * args.count if args.kind == 'factory' else 0
        if [item['journal'] for item in vm.journals] != expected_journals or run['draws'] != expected_draws:
            raise ValueError(('unexpected marker draw sequence', vm.journals, run['draws']))
        if expected:
            wanted = g.read_buffers(expected)
            differences = {}
            for name in g.NAMES:
                if len(result[name]) != len(wanted[name]):
                    raise ValueError((name, 'expected length mismatch'))
                delta = [(i, x, y) for i, (x, y) in enumerate(zip(result[name], wanted[name])) if x != y]
                differences[name] = {'count': len(delta), 'first': delta[:12]}
            mismatch_count = sum(item['count'] for item in differences.values())
            report.update(status='mismatch' if mismatch_count else 'passed',
                          production_comparison='failed' if mismatch_count else 'passed',
                          compared_bytes=sum(map(len, result.values())), mismatch_count=mismatch_count,
                          differences=differences, expected_pins={name: g.pin(expected / name) for name in g.NAMES})
    except Exception as error:
        report.update(status='error', reason=repr(error))
        if vm is not None and hasattr(vm, 'pc'):
            report.update(source_pc=hex(vm.pc), recent_calls=[hex(pc) for pc in vm.calls[-12:]], journals=vm.journals)
    (out / 'trace.json').write_text(json.dumps([hex(pc) for pc in executed]) + '\n', encoding='utf8')
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    hidden = ('input_pins', 'resource_pins', 'source_pins', 'output_pins', 'expected_pins')
    print(json.dumps({key: value for key, value in report.items() if key not in hidden}, indent=2))
    return 0 if report['status'] in ('passed', 'source_prepared') else 1


if __name__ == '__main__':
    raise SystemExit(main())

"""Literal captured 451F0 / 4B630 world review; never executes the game.

All eight buffers stay local. Opaque native pointers are normalized to INPUT,
never to expected output. A source-only run is preparation, not acceptance.
"""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import struct
import uuid

P = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('captured_mule_source', Path(__file__).with_name('krell-mule-callback-review.py'))
k = importlib.util.module_from_spec(spec)
spec.loader.exec_module(k)
NAMES = ('backing.bin', 'creatures.bin', 'actors.bin', 'buildings.bin', 'effects.bin', 'family8.bin', 'players.bin', 'rng.bin')
LENGTHS = {'creatures.bin': 37200, 'actors.bin': 4000, 'buildings.bin': 17152, 'effects.bin': 16200, 'family8.bin': 1632, 'players.bin': 7220, 'rng.bin': 7}
EXTRA = [
    'building-update/instructions-000451f0.txt',
    'activation-rng-refs/instructions-0005a210.txt',
    'genepod-mule-initializer.txt',
    'construction-command/instructions-0001ee50.txt',
    'bones-frame/instructions-0005a1c0.txt',
    'player-creation/instructions-0001f480.txt',
    'terrain-map-refs/instructions-0007b240.txt',
    'activation-rng-refs/instructions-00048790.txt',
    'player-creation/instructions-0001f070.txt',
    'mission-tail-init/instructions-00049e90.txt',
    'plant-command/instructions-0004b630.txt',
    'mission-array-refs/instructions-0004b3b0.txt',
    'actor-activation/instructions-0004b090.txt',
    'landing-contact/instructions-0001f030.txt',
    'terrain-map-refs/instructions-00049d30.txt',
    'panel-activation-refs/instructions-000724c0.txt',
    'construction-command/instructions-0004af40.txt',
    'combat-impact/instructions-0001e620.txt',
]


def pin(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resolve(path):
    return path if path.is_absolute() else P / path


def read_buffers(directory):
    buffers = {name: (directory / name).read_bytes() for name in NAMES}
    for name, size in LENGTHS.items():
        if len(buffers[name]) != size:
            raise ValueError((name, 'length', len(buffers[name]), size))
    if len(buffers['backing.bin']) < 337000:
        raise ValueError('truncated backing')
    seed, ring, draws = struct.unpack('<HBI', buffers['rng.bin'])
    if ring >= 32:
        raise ValueError(('rng ring', ring))
    return buffers


def prepare(buffers):
    back = buffers['backing.bin']
    state = {0x118d60 + i: x for i, x in enumerate(back[:0x355e])}
    state.update({0x11c2be + i: x for i, x in enumerate(back[0x355e:0x3b55e])})
    mule = P / 'private/reference-run/run-006/drive-c/GAME/DATA/CREATURE/MULE.ANB'
    mule_anims = struct.unpack_from('<I', mule.read_bytes(), 14)[0]
    for start, count, stride, base, width, name, vt in k.LAYOUTS:
        for slot in range(count):
            packed = back[start + slot * stride:start + (slot + 1) * stride]
            raw = bytearray(buffers[name][slot * width:(slot + 1) * width] if name else packed[:31] + bytes(8) + packed[31:])
            if vt:
                struct.pack_into('<I', raw, 35, vt)
            if name == 'creatures.bin':
                struct.pack_into('<I', raw, 31, 0x400000 + mule_anims)
            state.update({base + slot * width + j: value for j, value in enumerate(raw)})
    resources = {str(mule.relative_to(P)): pin(mule)}

    def resource(path, address):
        data = bytearray(path.read_bytes())
        count, table, components, frames, anims = struct.unpack_from('<H4I', data)
        if anims + count * 10 > len(data):
            raise ValueError(('invalid ANB animation table', str(path)))
        for animation in range(count):
            for offset in (0, 4):
                at = anims + animation * 10 + offset
                struct.pack_into('<I', data, at, address + frames + struct.unpack_from('<I', data, at)[0])
        state.update({address + j: value for j, value in enumerate(data)})
        resources[str(path.relative_to(P))] = pin(path)
        return address + anims, count

    building_tables = {}
    for index, (selector, filename) in enumerate(((0, 'BGENE.ANB'), (3, 'BPOWE.ANB'), (11, 'BMINE.ANB'))):
        building_tables[selector] = resource(P / 'private/integration/game-runtime/buildings' / filename, 0x600000 + index * 0x100000)[0]
    for slot in range(256):
        raw = buffers['buildings.bin'][slot * 67:(slot + 1) * 67]
        if raw[6]:
            if raw[45] not in building_tables:
                raise ValueError(('unbound active building selector', slot, raw[45]))
            k.put(state, 0x1645ae + slot * 67 + 31, 4, building_tables[raw[45]])
            k.put(state, 0x1645ae + slot * 67 + 35, 4, 0x112120)
    effect_table, effect_count = resource(P / 'private/integration/game-runtime/effects/EFF.ANB', 0xa00000)
    k.put(state, 0x116818, 4, effect_table)
    for slot in range(200):
        raw = buffers['effects.bin'][slot * 81:(slot + 1) * 81]
        if raw[6] and raw[69] >= effect_count:
            raise ValueError(('unbound active effect code', slot, raw[69]))
        k.put(state, 0x1688ae + slot * 81 + 31, 4, effect_table + raw[69] * 10)
    state.update({0x118d74 + i: value for i, value in enumerate(buffers['players.bin'])})
    seed, ring, draws = struct.unpack('<HBI', buffers['rng.bin'])
    k.put(state, 0x11872c, 1, ring)
    k.put(state, 0x118782, 4, draws)
    k.put(state, 0x118770, 1, back[0x52466])
    k.put(state, 0x185ec4, 4, 0)
    k.put(state, 0x180e84 + 16, 4, 0x400000)
    k.put(state, 0x300004, 4, 0)
    return state, seed, resources


def export(vm, buffers):
    result = {name: bytearray(data) for name, data in buffers.items()}
    back = result['backing.bin']
    back[:0x355e] = bytes(vm.load(0x118d60 + i, 1) for i in range(0x355e))
    # Native players have their own authoritative buffer; their stale serialized
    # backing mirror is not updated by ActorWorld (audited effect20 adapter).
    back[20:7240] = buffers['backing.bin'][20:7240]
    back[0x355e:0x3b55e] = bytes(vm.load(0x11c2be + i, 1) for i in range(229376))
    for start, count, stride, base, width, name, vt in k.LAYOUTS:
        for slot in range(count):
            raw = bytearray(vm.load(base + slot * width + j, 1) for j in range(width))
            if name:
                raw[31:39] = buffers[name][slot * width + 31:slot * width + 39]
                result[name][slot * width:(slot + 1) * width] = raw
            else:
                back[start + slot * stride:start + (slot + 1) * stride] = raw[:31] + raw[39:]
    result['players.bin'] = bytes(vm.load(0x118d74 + i, 1) for i in range(7220))
    result['rng.bin'] = struct.pack('<HBI', vm.load(0x11878a, 2), vm.load(0x11872c, 1), vm.load(0x118782, 4))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--kind', choices=('genepod', 'effect20'), required=True)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--expected', type=Path)
    parser.add_argument('--frame', type=int, required=True)
    parser.add_argument('--slot', type=int, required=True)
    args = parser.parse_args()
    before = resolve(args.before)
    expected = resolve(args.expected) if args.expected else None
    out = P / 'private/review/genepod-captured' / uuid.uuid4().hex
    out.mkdir(parents=True, exist_ok=False)
    report = dict(status='error', kind=args.kind, before=str(before), expected=str(expected) if expected else None,
                  frame=args.frame, slot=args.slot, output=str(out), production_comparison='not_run',
                  ports=['47F34 render enqueue deferred via explicit return port', '98210/1E810 audio deferred with source stack cleanup',
                         '41910 audio deferred with source stack cleanup', '1E800/409F0/2C200 UI callbacks deferred',
                         '5E068 portrait/module refresh deferred'],
                  normalization='Native opaque bytes31:39 restored from INPUT only; serialized player mirror preserved.')
    vm = None
    executed = []
    try:
        limit = 256 if args.kind == 'genepod' else 200
        if not 0 <= args.slot < limit or not 0 <= args.frame <= 0xffffffff:
            raise ValueError('frame/slot outside source domain')
        buffers = read_buffers(before)
        record = buffers['buildings.bin' if args.kind == 'genepod' else 'effects.bin']
        width = 67 if args.kind == 'genepod' else 81
        raw = record[args.slot * width:(args.slot + 1) * width]
        if not raw[6] or (raw[45] != 0 if args.kind == 'genepod' else raw[69] != 0x20):
            raise ValueError('selected record is not an active Genepod/effect20')
        state, seed, resources = prepare(buffers)
        vm = k.Source(buffers['backing.bin'])
        files = [P / 'private/ghidra' / name for name in EXTRA]
        vm.files += files
        vm.code.update(k.m.m.Model(files, b'').code)
        vm.code[0x47f34] = ('CALL', ['0x2e0010'])
        vm.code[0x2e0010] = ('RET', [])
        vm.code[0x98210] = ('RET', ['0x8'])
        vm.code[0x1e810] = ('RET', ['0x8'])
        for address in (0x1e800, 0x409f0, 0x2c200):
            vm.code[address] = ('RET', [])
        order = sorted(vm.code)
        vm.next = dict(zip(order, order[1:]))

        class TraceCode(dict):
            def __getitem__(self, address):
                executed.append(address)
                return super().__getitem__(address)

        vm.code = TraceCode(vm.code)
        report.update(resource_pins=resources, source_pins={str(p.relative_to(P)): pin(p) for p in vm.files},
                      input_pins={name: pin(before / name) for name in NAMES}, comparator_sha256=pin(Path(__file__)))
        entry, base = (0x451f0, 0x1645ae) if args.kind == 'genepod' else (0x4b630, 0x1688ae)
        run, _, trace = vm.run(entry=entry, counter=args.frame, seed=seed, state_before=state,
                               registers={'EAX': base + args.slot * width}, instruction_budget=500000)
        result = export(vm, buffers)
        for name, data in result.items():
            (out / name).write_bytes(data)
        (out / 'trace.json').write_text(json.dumps([hex(pc) for pc in trace]) + '\n', encoding='utf8')
        report.update(status='source_prepared', instructions=run['steps'], draws=run['draws'], calls=run['calls'],
                      queue_requests=run['queue_requests'], output_pins={name: pin(out / name) for name in NAMES})
        if expected:
            expected_buffers = read_buffers(expected)
            differences = {}
            for name in NAMES:
                actual, wanted = result[name], expected_buffers[name]
                if len(actual) != len(wanted):
                    raise ValueError((name, 'expected length', len(actual), len(wanted)))
                delta = [(i, x, y) for i, (x, y) in enumerate(zip(actual, wanted)) if x != y]
                differences[name] = {'count': len(delta), 'first': delta[:12]}
            mismatch_count = sum(item['count'] for item in differences.values())
            report.update(status='mismatch' if mismatch_count else 'passed', production_comparison='failed' if mismatch_count else 'passed',
                          differences=differences, mismatch_count=mismatch_count, compared_bytes=sum(map(len, result.values())),
                          expected_pins={name: pin(expected / name) for name in NAMES})
    except Exception as error:
        report.update(status='error', reason=repr(error))
        if vm is not None and hasattr(vm, 'pc'):
            report['source_pc'] = hex(vm.pc)
            report['recent_calls'] = [hex(x) for x in vm.calls[-12:]]
    if not (out / 'trace.json').exists():
        (out / 'trace.json').write_text(json.dumps([hex(pc) for pc in executed]) + '\n', encoding='utf8')
    (out / 'result.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf8')
    print(json.dumps({key: value for key, value in report.items() if key not in ('source_pins', 'input_pins', 'expected_pins', 'resource_pins', 'output_pins', 'calls')}, indent=2))
    return 0 if report['status'] in ('passed', 'source_prepared') else 1


if __name__ == '__main__':
    raise SystemExit(main())

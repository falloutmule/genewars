"""Literal source101D0 factory versus its bounded Rust record/link plan."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, importlib.util, json, random, struct, subprocess, sys, uuid
from cargo import environment
P = Path(__file__).resolve().parents[1]

def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()

def main():
    work = P / 'private/review/family8-creation' / uuid.uuid4().hex
    work.mkdir(parents=True)
    spec = importlib.util.spec_from_file_location('factory_source', P / 'research/variants/initial-creature-source.py')
    source = importlib.util.module_from_spec(spec); spec.loader.exec_module(source); model = source.Source()
    files = [P / 'private/ghidra' / f for f in [
        'mission-array-refs/instructions-000101d0.txt', 'countdown-expiry/instructions-0001ee80.txt',
        'countdown-expiry/instructions-0001ea90.txt', 'mission-tail-init/instructions-00010060.txt',
        'terrain-map-refs/instructions-0007b240.txt']]
    assert sha(files[1]) == '437595d68a3359ca52384d7a8131426da3d6d90d7e653ae27851af9c5c15d312'
    assert sha(files[2]) == '2d4bc2ddfbcaf74cb5a21fec11555addd2cf4fafad0981cffb1753f84eead059'
    object3 = (P / 'private/ghidra/validated/memory-.object3.bin').read_bytes()
    assert object3[0x12d14:0x12d1a] == bytes(6)
    assert [struct.unpack_from('<I', object3, 0x1242c + offset)[0] for offset in [0x60, 0x78, 0x80]] == [0x10060, 0x1ee80, 0x1ea90]
    model.code.update(source.m.Model(files, b'').code)
    order = sorted(model.code); model.next = dict(zip(order, order[1:]))
    mission = (P / 'private/integration/game-runtime/terrain/CONQ001.DAT').read_bytes()
    assert hashlib.sha256(mission).hexdigest() == '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    map_data = mission[0x355e:0x355e + 229376]; (work / 'map.bin').write_bytes(map_data)
    points = [(0, 0), (32767, -32768), (16384, 15872), struct.unpack_from('<hh', mission, 277838 + 7)]
    rand = random.Random(101001); inputs = bytearray(); expected = bytearray(); steps = 0; count = 0
    for selector in range(3):
      for owner in range(4):
       for location, slot in enumerate([0, 1, 7, 15]):
        point = points[location]; initial_height = [-2147483648, -1, 0, 2147483647][owner]
        before = bytearray(rand.randbytes(102)); before[6] = 0; before[31:39] = bytes(8)
        cell = ((point[1] >> 8) & 127) * 128 + ((point[0] >> 8) & 127)
        raw_head = map_data[cell * 14 + 6:cell * 14 + 9]
        old_id, old_kind = struct.unpack('<HB', raw_head)
        if old_kind == 0: old_id = 0
        assert old_kind in [0, 1, 2]
        inputs += before + struct.pack('<HBBhhiHB', slot, owner, selector, *point, initial_height, old_id, old_kind)
        pool = bytearray(16 * 102)
        for prior in range(slot): pool[prior * 102 + 6] = 8
        pool[slot * 102:(slot + 1) * 102] = before
        for index in range(16): struct.pack_into('<I', pool, index * 102 + 0x23, 0x11242c)
        state = {0x16da36 + j: b for j, b in enumerate(pool)}
        state.update({0x11c2b9: slot, 0x11872c: 0, **{0x118782 + j: 0 for j in range(4)}})
        state.update({0x116820 + j: b for j, b in enumerate(struct.pack('<I', 0x500000))})
        report, _, _ = model.run(state_before=state, entry=0x101d0,
            registers=dict(EAX=selector, EDX=int.from_bytes(struct.pack('<hh', *point), 'little'),
                           EBX=initial_height & 0xffffffff, ECX=owner), stack_fill=[0, 85, 255][count % 3])
        base = 0x16da36 + slot * 102
        assert model.r['EAX'] == base and model.load(0x11c2b9, 1) == slot + 1 and report['draws'] == 0
        after = bytearray(model.load(base + j, 1) for j in range(102))
        animation = struct.unpack_from('<I', after, 31)[0] - 0x500000; after[31:39] = bytes(8)
        head = bytes(model.load(0x11c2be + cell * 14 + 6 + j, 1) for j in range(3))
        if old_kind:
            old_base = (0x1542be if old_kind == 1 else 0x15d40e) + old_id * (124 if old_kind == 1 else 49)
            previous = bytes(model.load(old_base + j, 1) for j in range(3))
        else: previous = bytes(3)
        expected += after + bytes([animation]) + head + previous
        steps += report['steps']; count += 1
    (work / 'input.bin').write_bytes(inputs)
    subprocess.run([sys.executable, '-B', 'scripts/cargo.py', 'build', '-p', 'genewars-core', '--offline', '--locked'], cwd=P, capture_output=True, check=True)
    target = P / 'private/build/rust-target/debug'; exe = work / 'probe.exe'
    subprocess.run([str(P / 'private/tools/rust/cargo/bin/rustc.exe'), '--edition=2021', 'scripts/family8_creation_probe.rs',
        '-L', f'dependency={target}/deps', '--extern', f'genewars_core={target}/libgenewars_core.rlib', '-o', str(exe)],
        cwd=P, env=environment(), capture_output=True, check=True)
    subprocess.run([str(exe), str(work)], check=True)
    actual = (work / 'actual.bin').read_bytes()
    assert actual == expected, dict(actual=len(actual), expected=len(expected), difference=next((i for i, (a, b) in enumerate(zip(actual, expected)) if a != b), None))
    paths = [Path(__file__), P / 'scripts/family8_creation_probe.rs', P / 'rust/crates/genewars-runtime/src/family8_creation.rs']
    receipt = dict(status='passed-numeric-creation-plan', utc=datetime.now(timezone.utc).isoformat(), cases=count,
        compared_bytes=len(actual), literal_instructions=steps, listing_pins={f.relative_to(P).as_posix(): sha(f) for f in model.files + files},
        implementation_pins={f.relative_to(P).as_posix(): sha(f) for f in paths}, private_run=work.relative_to(P).as_posix(),
        original_execution=False, scope='Full literal101D0/1EE80 factory on source terrain; 48 native record, typed animation binding, cell-head and old-head previous comparisons. Pointer slots normalized. All three selectors, four owners, slots0/1/7/15, signed positions/heights, plant/Mule heads. Pool allocation/world commit, resources and callbacks are not integrated.')
    (P / 'research/integration/family8-creation-validation.json').write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: receipt[k] for k in ['status', 'cases', 'compared_bytes', 'literal_instructions']}))

if __name__ == '__main__': main()

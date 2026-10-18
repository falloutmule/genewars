"""Literal74D90 nominated-building gate with explicit terrain/plant ports."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import struct
import subprocess
import uuid
from cargo import environment
from validate_building_obstacle import OFFSETS, CLASSES, cell, word

P = Path(__file__).resolve().parents[1]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    work = P / 'private/review/nominated-building-obstacle' / uuid.uuid4().hex
    work.mkdir(parents=True)
    module = P / 'rust/crates/genewars-runtime/src/building_obstacle.rs'
    (work / 'obstacle.rs').write_bytes(module.read_bytes())
    spec = importlib.util.spec_from_file_location('nominated_source', P / 'research/variants/initial-creature-source.py')
    source_module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source_module)

    class Source(source_module.Source):
        def push(self, value):
            if self.pc == 0x74222:
                self.queries.append((self.r['EAX'] - 0x11c2be) // 14)
            return super().push(value)

    source = Source()
    listings = [P / 'private/ghidra' / name for name in [
        'terrain-map-refs/instructions-00074d90.txt',
        'landing-contact/instructions-0008b460.txt']]
    source.code.update(source_module.m.Model(listings, b'').code)
    # These are read-only terrain class0 and plant weight0 fixture ports.
    # The complete74D90,74270,74160 and8B460 bodies remain literal.
    for address in [0x8b520, 0x8b130]:
        source.code[address] = ('XOR', ['EAX', 'EAX'])
        source.code[address + 1] = ('RET', [])
    order = sorted(source.code)
    source.next = dict(zip(order, order[1:]))
    cases = []
    for selector in range(14):
        for index in range(9):
            for edge in [-1, 0, 1]:
                extent = {1: 85, 2: 128, 3: 256}[CLASSES[selector]]
                for flags in [0, 1]:
                    for nominated in [None, index, (index + 1) % 9]:
                        cases.append((selector, index, extent + edge, (32767, -32768), 1, flags, nominated, False))
    for flag in [0, 1]:
        for flags in [0, 1, 5]:
            for nominated in [None, 0, 1]:
                for delta in [0, 86]:
                    cases.append((1, 0, delta, (4096, 3584), flag, flags, nominated, True))
    data = bytearray()
    expected = bytearray()
    steps = 0
    for selector, index, delta, point, flag, flags, nominated, multiple in cases:
        records = [bytearray(67) for _ in OFFSETS]
        record = records[index]
        record[6] = 4
        record[45] = selector
        struct.pack_into('<hh', record, 7, word(point[0] + delta), point[1])
        struct.pack_into('<HH', record, 15, index, 0)
        struct.pack_into('<H', record, 49, flags)
        if multiple:
            record = records[1]
            record[6] = 4
            record[45] = 1
            struct.pack_into('<hh', record, 7, *point)
            struct.pack_into('<H', record, 15, 1)
            struct.pack_into('<H', record, 49, 1)
        state = {0x200000 + i: value for i, value in enumerate(struct.pack('<hh', *point))}
        state[0x230006] = 3
        indices = []
        for i, (dx, dy) in enumerate(OFFSETS):
            at = cell((word(point[0] + dx), word(point[1] + dy)))
            indices.append(at)
            row = bytearray(14)
            if records[i][6]:
                struct.pack_into('<hB', row, 6, i, 4)
            if i == 0:
                row[11] = flag
            state.update({0x11c2be + at * 14 + j: value for j, value in enumerate(row)})
            state.update({0x1645ae + i * 67 + j: value for j, value in enumerate(records[i])})
        source.queries = []
        report, _, _ = source.run(entry=0x74d90, state_before=state, registers={
            'EAX': 0x200000, 'EDX': 0, 'ECX': 0x230000,
            'EBX': 0 if nominated is None else 0x1645ae + nominated * 67})
        visited = [indices.index(at) for at in source.queries]
        expected.extend(bytes([source.r['EAX'] & 255, len(visited)] + visited + [255] * (9 - len(visited))))
        data.extend(struct.pack('<hhBh', *point, flag, -1 if nominated is None else nominated) + b''.join(records))
        steps += report['steps']
    harness = r'''mod obstacle;
use std::{fs,io::Write,path::Path};
fn main(){let path=std::env::args().nth(1).unwrap();let path=Path::new(&path);let data=fs::read(path.join("input.bin")).unwrap();let mut out=fs::File::create(path.join("actual.bin")).unwrap();
for c in data.chunks_exact(610){let point=[i16::from_le_bytes(c[..2].try_into().unwrap()),i16::from_le_bytes(c[2..4].try_into().unwrap())];
let nominated=i16::from_le_bytes(c[5..7].try_into().unwrap());let nominated=if nominated<0{None}else{Some(nominated as u16)};
let cell=|p:[i16;2]|((((i32::from(p[1])>>8)&127)*128)+((i32::from(p[0])>>8)&127))as usize;
let offsets:[[i16;2];9]=[[0,0],[256,0],[256,256],[0,256],[-256,256],[-256,0],[-256,-256],[256,-256],[7680,0]];
let indices:Vec<_>=offsets.iter().map(|v|cell([point[0].wrapping_add(v[0]),point[1].wrapping_add(v[1])])).collect();
let mut map=vec![0;229376];map[cell(point)*14+11]=c[4];let mut queries=Vec::new();
let blocked=obstacle::move_obstacle_excluding(&map,point,nominated,|at|{let i=indices.iter().position(|v|*v==at).unwrap();queries.push(i as u8);let record:[u8;67]=c[7+i*67..7+(i+1)*67].try_into().unwrap();Ok::<_,()>(if record[6]==0{None}else{Some(record)})}).unwrap();
out.write_all(&[!blocked as u8,queries.len()as u8]).unwrap();queries.resize(9,255);out.write_all(&queries).unwrap();}}
'''
    (work / 'probe.rs').write_text(harness, encoding='utf-8')
    (work / 'input.bin').write_bytes(data)
    (work / 'expected.bin').write_bytes(expected)
    compiled = subprocess.run([str(P / 'private/tools/rust/cargo/bin/rustc.exe'), '--edition=2021',
        '--cap-lints=allow', '-O', str(work / 'probe.rs'), '-o', str(work / 'probe.exe')], env=environment(), capture_output=True, timeout=60)
    assert compiled.returncode == 0, compiled.stderr.decode(errors='replace')
    subprocess.run([str(work / 'probe.exe'), str(work)], check=True, timeout=30)
    actual = (work / 'actual.bin').read_bytes()
    assert actual == expected, [(i, a, b) for i, (a, b) in enumerate(zip(actual, expected)) if a != b][:20]
    receipt = dict(status='passed', utc=datetime.now(timezone.utc).isoformat(), cases=len(cases),
        compared_bytes=len(actual), literal_instructions=steps, module_sha256=sha(work / 'obstacle.rs'),
        validator_sha256=sha(Path(__file__)), helper_sha256=sha(P / 'scripts/validate_building_obstacle.py'),
        listing_pins={p.relative_to(P).as_posix(): sha(p) for p in source.files + listings},
        private_run=work.relative_to(P).as_posix(), original_execution=False,
        scope='Literal74D90/74270/74160/8B460, required completed flag1, null/matching/other nominated pointer and exact ordered cell queries. All14selectors,9offsets,extent boundaries,word wrap,clear reservation and first-candidate flag/extent rejection. Terrain-class0 and plant-weight0 are explicit read-only ports. Positive-height queries and other entity families are excluded.')
    destination = P / 'research/integration/nominated-building-obstacle-validation.json'
    destination.write_text(json.dumps(receipt, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()

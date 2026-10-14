"""Literal extractor resource-harvest slice; explicit cleanup/presentation ports.

This does not validate the enclosing powered/animation callback or integrate it.
"""
from pathlib import Path
import hashlib
import importlib.util
import json
import random
import struct
import subprocess
import uuid

P = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('source', P / 'research/variants/initial-creature-source.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
x = m.Source()
listing = P / 'private/ghidra/building-update/instructions-000451f0.txt'
x.code.update(m.m.Model([listing], b'').code)
x.code[0x4707b] = ('RET', [])
for port in [0x8cba0, 0x84e10, 0x44e50]:
    x.code[port] = ('RET', [])
x.code[0x2e0000] = ('RET', ['0x8'])
order = sorted(x.code)
x.next = dict(zip(order, order[1:]))
rng = random.Random(0x46f0c)
steps = 0
inputs = bytearray()
expected_output = bytearray()
for case in range(64):
    center = [(0, 0), (127, 127), (64, 64), (81, 53)][case % 4]
    grade = case % 3
    resources = bytearray(128 * 128)
    for dy in range(-3, 4):
        for dx in range(-3, 4):
            index = ((center[1] + dy) & 127) * 128 + ((center[0] + dx) & 127)
            resources[index] = 0 if case % 8 == 0 else rng.randrange(1, 9)
    funds, capacity = [(0, 600), (598, 600), (600, 600), (0xffffffff, 600)][case % 4]
    building = bytearray(67)
    struct.pack_into('<hh', building, 7, center[0] * 256 - (32768 if center[0] >= 128 else 0), center[1] * 256)
    building[0x2e] = grade
    building[6] = 4
    building[45] = 11
    building[0x32] = 0x80
    struct.pack_into('<I', building, 0x23, 0x2f0000)
    base, player, frame = 0x1645ae, 0x118d74 + 0x4a1, 0x2fff00
    state = {base + i: b for i, b in enumerate(building)}
    def put(at, value, width=4):
        state.update({at + i: b for i, b in enumerate((value & ((1 << (width*8))-1)).to_bytes(width, 'little'))})
    put(frame - 0x26, base)
    put(frame - 0x16, player)
    put(player + 6, funds)
    put(player + 0x145, capacity)
    put(0x2f0044, 0x2e0000)
    state.update({0x11c2be + i * 14 + 13: b for i, b in enumerate(resources)})
    expected = bytearray(resources)
    expected_funds, expected_flags = funds, 0x80
    selected, best = None, 0
    if funds < capacity:
        for dy in range(-3, 4):
            for dx in range(-3, 4):
                i = ((center[1] + dy) & 127) * 128 + ((center[0] + dx) & 127)
                if resources[i] > best:
                    selected, best = i, resources[i]
        if selected is None:
            expected_flags |= 3
        else:
            take = min(best, grade * 2 + 2)
            expected[selected] -= take
            expected_funds = min((funds + take * 2) & 0xffffffff, capacity)
    report, _, _ = x.run(entry=0x46f0c, state_before=state, registers={'EBP': frame})
    actual = bytes(x.load(0x11c2be + i * 14 + 13, 1) for i in range(16384))
    assert actual == expected, ('map', case)
    assert x.load(player + 6, 4) == expected_funds, ('funds', case)
    assert x.load(base + 0x32, 1) == expected_flags, ('flags', case)
    assert x.load(base + 0x3f, 1) == 3
    player_bytes = bytearray(1805)
    struct.pack_into('<II', player_bytes, 0x4a7, funds, 0)
    struct.pack_into('<I', player_bytes, 0x5e6, capacity)
    map_bytes = bytearray(16384 * 14)
    map_bytes[13::14] = resources
    inputs.extend(building + player_bytes + map_bytes)
    expected_output.extend(bytes(x.load(base + i, 1) for i in range(67)))
    struct.pack_into('<I', player_bytes, 0x4a7, expected_funds)
    expected_output.extend(player_bytes)
    expected_output.extend(actual)
    steps += report['steps']
work = P / 'private/review/extractor-harvest' / uuid.uuid4().hex
work.mkdir(parents=True)
rust = P / 'rust/crates/genewars-runtime/src/extractor_production.rs'
harness = work / 'review.rs'
harness.write_text(f'#[path="{rust.as_posix()}"]mod extractor;' + r'''use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let mut out=Vec::new();for c in b.chunks_exact(231248){let q=extractor::plan_harvest_slice(c[..67].try_into().unwrap(),c[67..1872].try_into().unwrap(),&c[1872..]).unwrap();out.extend(q.building_after);out.extend(q.player_after);let mut resources:Vec<u8>=c[1872..].chunks_exact(14).map(|r|r[13]).collect();if let Some(change)=q.resource{resources[change.cell as usize]=change.after;}out.extend(resources);}std::io::stdout().write_all(&out).unwrap();}''')
cargo_spec=importlib.util.spec_from_file_location('cargo_local',P/'scripts/cargo.py')
cargo=importlib.util.module_from_spec(cargo_spec)
cargo_spec.loader.exec_module(cargo)
exe=work/'review.exe'
compiler=P/'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'
subprocess.run([str(compiler),'--edition=2021','-Awarnings',str(harness),'-o',str(exe)],env=cargo.environment(),check=True)
actual_output=subprocess.run([str(exe)],input=inputs,capture_output=True,check=True).stdout
assert actual_output==expected_output,next((i for i,(a,b)in enumerate(zip(actual_output,expected_output))if a!=b),('length',len(actual_output),len(expected_output)))
receipt = {'status': 'passed-numeric-slice', 'cases': 64, 'literal_instructions': steps,
    'rust_sha256':hashlib.sha256(rust.read_bytes()).hexdigest(), 'compared_bytes':len(actual_output),
    'source_listing_sha256': hashlib.sha256(listing.read_bytes()).hexdigest(),
    'scope': '46F0C through4707B: 7x7 maximum-resource selection, depletion byte, capped unsigned secondary funds, empty-resource flags, cooldown3F',
    'explicit_ports_not_validated': ['8CBA0 depleted-cell terrain cleanup', '84E10 viewport dirty request', '44E50 depletion notification', 'audio virtual44'],
    'not_claimed': ['full451F0 callback', 'powered/animation gates', 'preceding RNG/particle/contact damage', 'production Rust integration']}
(P / 'research/campaign/extractor-production-slice.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt))

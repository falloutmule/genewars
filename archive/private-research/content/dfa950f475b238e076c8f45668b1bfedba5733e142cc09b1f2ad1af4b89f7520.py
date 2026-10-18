"""Compare general construction admission against literal CD43960 instructions."""
from pathlib import Path
import hashlib
import importlib.util
import json
import random
import subprocess
import uuid

P = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('creature_source', P / 'research/variants/initial-creature-source.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
source = module.Source()
path = P / 'private/ghidra/mission-contract-refs/43960-range.txt'
source.code.update(module.m.Model([path], b'').code)
order = sorted(source.code)
source.next = dict(zip(order, order[1:]))
mapped = (P / 'private/ghidra/validated/memory-.object3.bin').read_bytes()
rng = random.Random(0x43960)
steps = 0
inputs = bytearray()
expected_output = bytearray()
for case in range(2048):
    selector = case % 14
    owner = (case // 14) % 4
    progress = [0, 1, 5, 9, 23, 29, 30, 127, 128, 255][case % 10]
    counts = bytearray(rng.randrange(2) for _ in range(14))
    owned = rng.randrange(1 << 14)
    threshold = mapped[0x4d88 + selector * 3]
    dependency = mapped[0x4e74 + selector]
    signed_progress = progress if progress < 128 else progress - 256
    expected = signed_progress >= threshold and threshold < 30 and (dependency == 255 or counts[dependency] != 0 or bool(owned & (1 << selector)))
    base = 0x118d74 + owner * 1805
    state = {base + 0x4a1: owner, base + 0x6e7: progress, base + 0x60e: owned & 255, base + 0x60f: owned >> 8}
    state.update({base + 0x600 + i: count for i, count in enumerate(counts)})
    report, _, _ = source.run(entry=0x43960, state_before=state, registers={'EAX': base + 0x4a1, 'EDX': selector})
    actual = source.get('AL') != 0
    assert actual == expected, (case, selector, progress, actual, expected)
    player = bytearray(1805)
    player[1] = player[0x4a1] = owner
    player[0x6e7] = progress
    player[0x600:0x60e] = counts
    player[0x60e:0x610] = owned.to_bytes(2, 'little')
    inputs.extend(player + bytes([selector]))
    expected_output.append(int(actual))
    steps += report['steps']
work = P / 'private/review/mission-availability' / uuid.uuid4().hex
work.mkdir(parents=True)
rust = P / 'rust/crates/genewars-runtime/src/building_availability.rs'
harness = work / 'review.rs'
harness.write_text(f'#[path="{rust.as_posix()}"] mod availability;' + r'''use std::io::{Read,Write};fn main(){let mut b=Vec::new();std::io::stdin().read_to_end(&mut b).unwrap();let out:Vec<u8>=b.chunks_exact(1806).map(|c|availability::available(c[..1805].try_into().unwrap(),c[1805]).unwrap() as u8).collect();std::io::stdout().write_all(&out).unwrap();}''')
cargo_spec = importlib.util.spec_from_file_location('local_cargo', P / 'scripts/cargo.py')
cargo = importlib.util.module_from_spec(cargo_spec)
cargo_spec.loader.exec_module(cargo)
exe = work / 'review.exe'
compiler = P / 'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'
subprocess.run([str(compiler), '--edition=2021', '-Awarnings', str(harness), '-o', str(exe)], env=cargo.environment(), check=True)
actual_output = subprocess.run([str(exe)], input=inputs, capture_output=True, check=True).stdout
assert actual_output == expected_output
receipt = {'status': 'passed', 'scope': 'General43960 construction-menu admission versus production Rust; no placement or construction side effects', 'cases': 2048, 'literal_instructions': steps, 'listing_sha256': hashlib.sha256(path.read_bytes()).hexdigest(), 'rust_sha256': hashlib.sha256(rust.read_bytes()).hexdigest(), 'first_mission_sequence': [11, 3, 6], 'negative_progress_and_locked_thresholds': True}
(P / 'research/campaign/construction-availability-evidence.json').write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt))

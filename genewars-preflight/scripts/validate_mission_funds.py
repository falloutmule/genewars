"""Compare the bounded mission-start funds block with literal CD instructions."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import random
import struct
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
LISTING = ROOT / 'private/ghidra/campaign-level/instructions-0007d090.txt'
LISTING_SHA = 'b9a4d6c79119564ba2ec9bf4cc120ce206530663808300faeab3c3c2993d98ac'
MISSION_SHA = '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
MODULE = ROOT / 'rust/crates/genewars-runtime/src/mission_player_funds.rs'

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    assert sha(LISTING) == LISTING_SHA
    mission = ROOT / 'private/integration/game-runtime/terrain/CONQ001.DAT'
    assert sha(mission) == MISSION_SHA
    spec = importlib.util.spec_from_file_location('source', ROOT / 'research/variants/initial-creature-source.py')
    source = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(source)

    class FundsSource(source.Source):
        def store(self, address, width, value):
            if getattr(self, 'pc', None) == 0x7d589:
                assert address == self.player + 0x4a3 and width == 4
                self.funds_written = True
            return super().store(address, width, value)

    work = ROOT / 'private/integration/mission-funds' / uuid.uuid4().hex
    work.mkdir(parents=True)
    bounded = work / 'funds-block.txt'
    selected = [line for line in LISTING.read_text().splitlines()
        if line and (0x7d57f <= int(line[:8], 16) < 0x7d58f or 0x7d744 <= int(line[:8], 16) <= 0x7d765)]
    bounded.write_text('\n'.join(selected)+'\n', encoding='utf-8')
    oracle = FundsSource()
    oracle.code.update(source.m.Model([bounded], b'').code)
    # Explicit block exits: the later startup work is outside this proof.
    oracle.code[0x7d58f] = ('RET', [])
    oracle.code[0x7d68d] = ('RET', [])
    order = sorted(oracle.code)
    oracle.next = dict(zip(order, order[1:]))
    rng = random.Random(0x7d589)
    data = bytearray()
    expected = bytearray()
    instructions = 0
    applied = 0
    cases = 0
    for active in [0, 1, 4]:
        for slot in range(4):
            for mode in [0, 1, 255]:
                for units in [0, 20, 255]:
                    before = bytearray(rng.randbytes(1805))
                    before[1] = slot
                    before[2] = mode
                    oracle.player = 0x118d74 + slot * 1805
                    oracle.funds_written = False
                    state = {oracle.player + i: value for i, value in enumerate(before)}
                    state[0x11878c] = active
                    state[0x11a9bc] = units
                    frame = 0x2f0000
                    state.update({frame + 0x72 + i: value for i, value in enumerate(struct.pack('<I', slot))})
                    report, _, _ = oracle.run(entry=0x7d744, state_before=state,
                        registers={'ECX': oracle.player, 'EBP': frame})
                    after = bytes(oracle.load(oracle.player + i, 1) for i in range(1805))
                    data += before + bytes([slot, active, units])
                    expected += bytes([oracle.funds_written]) + after
                    instructions += report['steps']
                    applied += oracle.funds_written
                    cases += 1
    harness = work / 'probe.rs'
    harness.write_text(f'#[path="{MODULE.as_posix()}"]mod funds;\n' + r'''
use std::io::{Read,Write};
fn main(){let mut input=Vec::new();std::io::stdin().read_to_end(&mut input).unwrap();
let mut output=Vec::new();for row in input.chunks_exact(1808){
let plan=funds::plan_mission_player_funds(row[..1805].try_into().unwrap(),row[1805],row[1806],row[1807]).unwrap();
output.push(plan.applied as u8);output.extend_from_slice(&plan.after);}
std::io::stdout().write_all(&output).unwrap();}
''', encoding='utf-8')
    binary = work / 'probe.exe'
    subprocess.run([str(ROOT / 'private/tools/rust/rustup/toolchains/1.99.0-x86_64-pc-windows-gnu/bin/rustc.exe'),
                    '--edition=2021', '-Awarnings', str(harness), '-o', str(binary)], check=True)
    actual = subprocess.run([str(binary)], input=data, capture_output=True, check=True).stdout
    assert actual == expected, next((i for i, pair in enumerate(zip(actual, expected)) if pair[0] != pair[1]), 'length')
    report = dict(status='passed', utc=datetime.now(timezone.utc).isoformat(), cases=cases,
        applied=applied, unchanged=cases-applied, compared_bytes=len(actual), literal_instructions=instructions,
        listing_sha256=LISTING_SHA, module_sha256=sha(MODULE), validator_sha256=sha(Path(__file__)),
        output_sha256=hashlib.sha256(actual).hexdigest(), private_output=work.relative_to(ROOT).as_posix(),
        mission_sha256=MISSION_SHA, mission_units=mission.read_bytes()[0x1c5c],
        block_entry='7D744', block_exits=['7D58F', '7D68D'], original_execution=False,
        scope='Exact player record after the active-player/mode/funds block; earlier identity and mode setup and later startup calls are explicit boundaries.')
    (ROOT / 'research/integration/mission-funds-validation.json').write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print(json.dumps(report))

if __name__ == '__main__':
    main()

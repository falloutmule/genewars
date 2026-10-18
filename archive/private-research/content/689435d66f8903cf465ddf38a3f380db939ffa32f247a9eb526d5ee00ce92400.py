"""Compare owned Rust player state with literal CD stores and restoration ranges."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import random
import struct
import subprocess
from cargo import environment

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / 'private/integration/player-state-validation'
SIZE = 0x70D


def sha(data):
    return hashlib.sha256(data).hexdigest()


def pin(path):
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=sha(path.read_bytes()))


def main():
    WORK.mkdir(parents=True, exist_ok=True)
    pins = []
    for name, expected in [
        ('campaign-start/instructions-00080460.txt', '15cf464ceb31fc599bc455e1cf53063eae7368d57cd66ffcd088d2a85ca70ce2'),
        ('camera-player-input/instructions-00080590.txt', 'c454d8a02dd88524f21c3bc7381dc977804aabe214a5f3cdf56b6744757d44d6'),
    ]:
        path = ROOT / 'private/ghidra' / name
        assert sha(path.read_bytes()) == expected
        pins.append(pin(path))
    # The short source literal remains in ignored memory/executable material.
    memory = (ROOT / 'private/ghidra/validated/memory-.object3.bin').read_bytes()
    label = memory[0xEADE:0xEADE+7]
    assert sha(label) == '91a008024ce891726c1fea45e1f804c1d9bde27f3b62305ac1c2f1f6e40a9729'
    (WORK / 'label.bin').write_bytes(label)
    source_path = ROOT / 'rust/crates/genewars-runtime/src/player_state.rs'
    selection_path = ROOT / 'rust/crates/genewars-core/src/player_creation.rs'
    blueprint_path = ROOT / 'rust/crates/genewars-runtime/src/blueprint.rs'
    initializer_path = ROOT / 'rust/crates/genewars-runtime/src/actor_initialization.rs'
    sources = [pin(source_path), pin(selection_path), pin(blueprint_path), pin(initializer_path), pin(Path(__file__))]
    generator = random.Random(0x8046080590)
    expected = bytearray()
    cases = 0
    with (WORK / 'inputs.bin').open('wb') as out:
        for player in range(4):
            for pattern in range(256):
                target = [generator.randrange(-32768,32768) for _ in range(2)]
                retained = bytearray(generator.randbytes(SIZE))
                retained[1] = player
                loaded = bytes((i * 37 + pattern) % 256 for i in range(SIZE))
                out.write(struct.pack('<B2h',player,*target) + retained + loaded)
                # Literal stores in order, starting from the menu's cleared
                # record. No blueprint or actor is synthesized by this model.
                fresh = bytearray(SIZE)
                for offset, value in [(0x6E3,0),(1,player),(0x4A1,player),(0x6E2,player),(0,2)]: fresh[offset] = value
                struct.pack_into('<2h',fresh,9,*target)
                struct.pack_into('<I',fresh,0x67E,1)
                struct.pack_into('<I',fresh,0x705,1)
                fresh[0x6E0] = 1
                fresh[0x6AE] = 4
                struct.pack_into('<I',fresh,0x62A,0x11800)
                fresh[0x6E7] = 0
                fresh[0x6E9:0x6E9+len(label)] = label
                restored = bytearray(loaded)
                for start,count in [(0,0x4A1),(0x61A,0xDD)]: restored[start:start+count] = retained[start:start+count]
                expected.extend(fresh + restored)
                cases += 1
    code = '''extern crate self as genewars_core;
#[path = "SELECTION"] mod selection;
pub mod cd_candidate { pub use crate::selection::*; }
#[path = "STATE"] mod state;
#[path = "BLUEPRINT"] mod blueprint;
#[path = "INITIALIZER"] mod actor_initialization;
use state::PlayerState;
fn main() {
 let dir=std::path::PathBuf::from(std::env::args().nth(1).unwrap());
 let input=std::fs::read(dir.join("inputs.bin")).unwrap();
 let label=std::fs::read(dir.join("label.bin")).unwrap();
 let mut output=Vec::new();
 for chunk in input.chunks_exact(5+2*0x70D) {
  let player=chunk[0];
  let target=[i16::from_le_bytes([chunk[1],chunk[2]]),i16::from_le_bytes([chunk[3],chunk[4]])];
  let saved=PlayerState::from_record(&chunk[5..5+0x70D]).unwrap();
  let fresh=PlayerState::fresh_campaign(player,target,&label).unwrap();
  let restored=PlayerState::restored_for_mission(&chunk[5+0x70D..],player,&saved).unwrap();
  output.extend_from_slice(fresh.bytes()); output.extend_from_slice(restored.bytes());
 }
 std::fs::write(dir.join("actual.bin"),output).unwrap();
}
'''.replace('SELECTION',selection_path.as_posix()).replace('STATE',source_path.as_posix()).replace('BLUEPRINT',blueprint_path.as_posix()).replace('INITIALIZER',initializer_path.as_posix())
    harness = WORK / 'probe.rs'
    harness.write_text(code,encoding='utf-8')
    binary = WORK / 'probe.exe'
    for command in [
        [str(ROOT/'private/tools/rust/cargo/bin/rustc.exe'),'--edition=2021','--cap-lints=allow','-O',str(harness),'-o',str(binary)],
        [str(binary),str(WORK)],
    ]:
        result = subprocess.run(command,env=environment(),capture_output=True,timeout=90)
        assert result.returncode == 0, result.stderr.decode(errors='replace')
    actual = (WORK/'actual.bin').read_bytes()
    assert actual == expected, 'Rust player state differs from instruction-store model'
    assert sources == [pin(source_path),pin(selection_path),pin(blueprint_path),pin(initializer_path),pin(Path(__file__))]
    receipt = dict(status='passed',utc=datetime.now(timezone.utc).isoformat(),
                   cases=cases, records_compared=2*cases, bytes_compared=len(expected),
                   sources=sources,instruction_pins=pins,output_sha256=sha(actual),
                   original_execution=False,full_startup=False,
                   limits=['Explicit caller target and hash-bound label; does not establish pre-load target-table contents',
                           'Fresh zeroed campaign path only; nonzero byte+2 initialization excluded',
                           'Two retained ranges only; blueprint generation/actor/resource/landing still separate'])
    (ROOT/'research/integration/player-state-validation.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({key:receipt[key] for key in ['status','cases','records_compared','bytes_compared','full_startup']}))


if __name__ == '__main__':
    main()

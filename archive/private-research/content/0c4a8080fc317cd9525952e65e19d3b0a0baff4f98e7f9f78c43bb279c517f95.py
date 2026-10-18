"""Independent ordered activation/move after-images over pinned mission bytes.

Players, native gaps, environmental values and live RNG are explicit synthetic
fixtures. This verifies a bounded transaction, not the original landing clock.
"""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import math
import shutil
import struct
import subprocess
import sys
from cargo import environment

ROOT = Path(__file__).resolve().parents[1]
WORK = ROOT / 'private/integration/actor-world-validation'
MISSION_BYTES = 343958
PLAYER_BYTES = 0x70D
ACTORS = MISSION_BYTES
PLAYERS = ACTORS + 20 * 200
RNG = PLAYERS + 4 * PLAYER_BYTES
SNAPSHOT = RNG + 7
MAP = 0x355E
LAYOUTS = [(243038,300,116),(277838,512,41),(298830,20,140),(301630,256,59),
           (316734,200,73),(331334,32,41),(332646,32,89),(335494,16,94)]

def sha(data):
    return hashlib.sha256(data).hexdigest()

def pin(path):
    return dict(path=path.relative_to(ROOT).as_posix(), sha256=sha(path.read_bytes()))

def run(command, **kwargs):
    result = subprocess.run(command, capture_output=True, **kwargs)
    if result.returncode:
        sys.stderr.buffer.write(result.stderr)
        raise subprocess.CalledProcessError(result.returncode, command)
    return result

def candidate(seed, center):
    seed = seed * 0xE62D & 65535
    angle = ((seed >> 8) * 8 - 512) & 2047
    sine = math.trunc(65536 * math.sin(angle * math.tau / 2048))
    cosine = math.trunc(65536 * math.cos(angle * math.tau / 2048))
    result = []
    for value, trig in zip(center, (cosine, sine)):
        doubled = (value * 2 + ((1024 * trig) // 65536)) & 65535
        signed = doubled if doubled < 32768 else doubled - 65536
        result.append((signed // 2) & 32767)
    return seed, result

def entity_offset(raw):
    index, kind = struct.unpack('<hB', raw)
    assert 1 <= kind <= 8 and 0 <= index < LAYOUTS[kind - 1][1]
    if kind == 3:
        return ACTORS + index * 200
    base, _, stride = LAYOUTS[kind - 1]
    return base + index * stride

def model(before, case):
    data = bytearray(before)
    owner = case % 4
    player = PLAYERS + owner * PLAYER_BYTES
    center = struct.unpack_from('<2h', data, player + 9)
    seed, ring, draws = struct.unpack_from('<HBI', data, RNG)
    for slot in range(5):
        actor = ACTORS + (owner * 5 + slot) * 200
        flags = struct.unpack_from('<H', data, actor + 0x70)[0]
        if data[actor + 6] == 0 or flags & 0x400 == 0:
            continue
        struct.pack_into('<H', data, actor + 0x70, (flags & 0xFBFE) | 1)
        data[actor + 0x16] &= 0xF7
        for _ in range((case + slot) % 4 + 1):
            seed, point = candidate(seed, center)
            ring = (ring + 1) % 32
            draws = (draws + 1) & 0xFFFFFFFF
        struct.pack_into('<2h', data, actor + 7, *point)
        cell = point[1] // 256 * 128 + point[0] // 256
        head_offset = MAP + cell * 14 + 6
        old_head = data[head_offset:head_offset + 3]
        actor_ref = struct.pack('<hB', owner * 5 + slot, 3)
        struct.pack_into('<HB', data, actor, cell, 0)
        if old_head[2]:
            old = entity_offset(old_head)
            assert data[old + 15:old + 17] + data[old + 6:old + 7] == old_head
            data[actor + 3:actor + 6] = old_head
            data[old:old + 3] = actor_ref
        else:
            data[actor + 5] = 0
        data[head_offset:head_offset + 3] = actor_ref
        bp = player + 0x16 + data[actor + 0x74] * 36
        data[bp + 0x1B] = data[bp + 0x18]
        data[bp + 0x20] |= 2
        count = (struct.unpack_from('<I', data, bp + 0x1C)[0] + 1) & 0xFFFFFFFF
        struct.pack_into('<I', data, bp + 0x1C, count)
        if count == 1 and data[bp + 0x16] == 1:
            data[actor + 0x72] = 40
    count = (struct.unpack_from('<I', data, player + 0x49D)[0] + 1) & 0xFFFFFFFF
    struct.pack_into('<I', data, player + 0x49D, count)
    struct.pack_into('<HBI', data, RNG, seed, ring, draws)
    activated = bytes(data)
    actor = ACTORS + owner * 5 * 200
    point = struct.unpack_from('<2h', data, actor + 7)
    requested = [(n + 1000) & 32767 for n in point]
    data[actor + 0x57] |= 2
    data[actor + 0x5B] = 0
    struct.pack_into('<2h', data, actor + 0x5C, *requested)
    struct.pack_into('<2h', data, actor + 0x0B, *requested)
    return activated, bytes(data)

def main():
    WORK.mkdir(parents=True, exist_ok=True)
    mission = ROOT / 'private/integration/game-runtime/terrain/CONQ001.DAT'
    source = mission.read_bytes()
    assert sha(source) == '72368fc1827142a2fc5da0b30d62c7b9907480f9e6514c1b587cea38159e34d2'
    pins = [pin(mission)]
    for relative, expected in [
        ('command-resolver/instructions-00080ae0.txt','f6abb34796b3a71a8443f62f35547b049f371968f15a7365dea2c367858739a6'),
        ('actor-activation/instructions-00073e00.txt','7869104854e5107e8d711b69704e8e1009dcba41892d0233c4daca72e8f0c5a0'),
        ('actor-activation/instructions-0008ac60.txt','6fd0ff082558f7bc94da0a908710fab78b8d477369a106e15f37f4922ee557fa'),
    ]:
        path = ROOT / 'private/ghidra' / relative
        assert sha(path.read_bytes()) == expected
        pins.append(pin(path))
    shutil.copyfile(mission, WORK / 'mission.bin')
    parameter_path=ROOT/'private/integration/game-runtime/plant-parameters.bin'
    parameters=parameter_path.read_bytes()
    assert sha(parameters)=='297a1da0e9c30d6d4cb7c88addb8234be260e0e93b7d4b2b1c4c3ad8b7df2b7c'
    pins.append(pin(parameter_path))
    shutil.copyfile(parameter_path,WORK/'parameters.bin')
    oracle_path=ROOT/'research/variants/runtime-world-review.py'
    spec=importlib.util.spec_from_file_location('independent_plant_oracle',oracle_path)
    plant_oracle=importlib.util.module_from_spec(spec)
    spec.loader.exec_module(plant_oracle)
    # The independent geometric plant model operates on this full snapshot.
    # Only reference addressing changes: kind3 occupies native200-byte slots.
    plant_oracle.record=lambda reference: entity_offset(struct.pack('<hB',reference[1],reference[0]))
    built = run([sys.executable,'-B','scripts/cargo.py','build','-p','genewars-runtime',
                 '-p','genewars-assets','--offline','--locked','--message-format=json'],cwd=ROOT)
    # Use the artifacts from this exact Cargo invocation. Modification times
    # can select an incompatible feature/profile fingerprint left by a gate.
    artifacts = {}
    for line in built.stdout.splitlines():
        entry = json.loads(line)
        if entry.get('reason') == 'compiler-artifact' and 'lib' in entry['target']['kind']:
            artifacts[entry['target']['name']] = next(p for p in entry['filenames'] if p.endswith('.rlib'))
    env = environment()
    build = ROOT / 'private/build/rust-target/debug'
    harness = ROOT / 'scripts/actor_world_probe.rs'
    rustc = ROOT / 'private/tools/rust/cargo/bin/rustc.exe'
    exe = WORK / 'probe.exe'
    command = [str(rustc),'--edition=2021',str(harness),'-L',f'dependency={build / "deps"}',
               '--extern',f'genewars_assets={artifacts["genewars_assets"]}',
               '--extern',f'genewars_runtime={artifacts["genewars_runtime"]}','-o',str(exe)]
    run(command,cwd=ROOT,env=env)
    run([str(exe),str(WORK)],cwd=ROOT,env=env)
    actual = (WORK/'actual.bin').read_bytes()
    assert len(actual) == 64 * 4 * SNAPSHOT
    checked = 0
    plant_callbacks=0
    for case in range(64):
        start = case * 4 * SNAPSHOT
        before = actual[start:start+SNAPSHOT]
        assert before[:MISSION_BYTES] == source
        activated, commanded = model(before,case)
        assert actual[start+SNAPSHOT:start+2*SNAPSHOT] == activated, f'activation mismatch case{case}'
        assert actual[start+2*SNAPSHOT:start+3*SNAPSHOT] == commanded, f'command mismatch case{case}'
        stepped=commanded
        for tick in range(96):
            stepped,reason,details=plant_oracle.step(stepped,parameters)
            assert reason is None,(case,tick,reason)
            plant_callbacks+=len(details[0])
        assert actual[start+3*SNAPSHOT:start+4*SNAPSHOT] == stepped,f'shared plant mismatch case{case}'
        checked += 3 * SNAPSHOT
    sources = [pin(ROOT/'rust/crates/genewars-runtime/src'/name) for name in
               ['actor_world.rs','actor_pool.rs','actor_activation.rs','actor_command.rs','actor_initialization.rs','player_state.rs','blueprint.rs','world.rs']]
    report = dict(status='passed',verified_at=datetime.now(timezone.utc).isoformat(),cases=64,
                  afterimages=192,checked_bytes=checked,rollback_cases=128,source_pins=pins,
                  shared_plant_passes=64*96,shared_plant_callbacks=plant_callbacks,
                  implementation_pins=sources+[pin(harness),pin(Path(__file__)),pin(oracle_path)],output_sha256=sha(actual),
                  scope='Explicit fixture players/native gaps/live RNG/environments over the exact mission. Ordered no-associated activation, bounded pending command and96 plant passes per session in the same backing/native actor store; no landing schedule, UI producer, movement or original execution.')
    (ROOT/'research/integration/actor-world-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({key:report[key] for key in ['status','cases','afterimages','checked_bytes','rollback_cases']}))

if __name__ == '__main__':
    main()

"""Offline byte-level checks for the empty mouse callback logger.

This verifies saved inputs, including closed runs, and disassembles the
independent COM image. Missing default preparation may be created. It does not
start DOSBox, mount originals, or generate host mouse input. Runtime receipt
state is reported separately from this checker's lack of emulator execution.
"""
from __future__ import annotations

import hashlib
import argparse
import json
from pathlib import Path
import struct
import re
import sys
import types

import mouse_probe as probe

ROOT = probe.ROOT
CAPSTONE = ROOT / 'private/review/input-free-yield/capstone'
CAPSTONE_SHA256 = '1bfa5c81e6880caf41a31946cd6d2d069c048bcc22edf121254b501a048de675'
CAPSTONE_LICENSE_SHA256 = '404bd0cb0137ffb797258f844f53e5273f9b6d5781a1a359a2880411f49a4f30'


def load_capstone():
    wheel = CAPSTONE / 'capstone'
    metadata = CAPSTONE / 'capstone-5.0.1.dist-info'
    assert wheel.is_dir() and metadata.is_dir()
    licenses = list(metadata.glob('licenses/LICENSE.TXT')) + list(metadata.glob('LICENSE.TXT'))
    if licenses:
        assert hashlib.sha256(licenses[0].read_bytes()).hexdigest() == CAPSTONE_LICENSE_SHA256
    prior = json.loads((ROOT / 'research/variants/reference-input-free-probe-readiness.json').read_bytes())
    assert prior.get('capstone', {}).get('wheel_sha256') == CAPSTONE_SHA256
    assert prior.get('capstone', {}).get('version') == '5.0.1'
    sys.path.insert(0, str(CAPSTONE))
    # This private wheel uses pkg_resources only to locate its bundled tables.
    shim = types.ModuleType('pkg_resources')
    shim.resource_filename = lambda package, relative: str(wheel / relative)
    sys.modules['pkg_resources'] = shim
    import capstone
    return capstone


def decode(capstone, code):
    engine = capstone.Cs(capstone.CS_ARCH_X86, capstone.CS_MODE_16)
    engine.skipdata = True
    instructions = list(engine.disasm(code, 0x100))
    covered = sum(len(item.bytes) for item in instructions)
    assert covered == len(code), (covered, len(code))
    return instructions


def check_disassembly(capstone, instructions, code, callback_start, callback_end,
                      expected_counts=(10, 3, 5, 2)):
    callback = [i for i in instructions if callback_start <= i.address < callback_end]
    assert callback[0].mnemonic == 'pushf'
    assert [(i.mnemonic, i.op_str) for i in callback[:10]] == [
        ('pushf', ''), ('push', 'ax'), ('push', 'bx'), ('push', 'cx'),
        ('push', 'dx'), ('push', 'si'), ('push', 'di'), ('push', 'ds'),
        ('push', 'es'), ('push', 'bp')]
    assert [i.mnemonic for i in callback[-11:]] == [
        'pop', 'pop', 'pop', 'pop', 'pop', 'pop', 'pop', 'pop', 'pop', 'popf', 'retf']
    assert [i.op_str for i in callback[-11:-1]] == [
        'bp', 'es', 'ds', 'di', 'si', 'dx', 'cx', 'bx', 'ax', '']
    assert not any(i.mnemonic == 'int' and i.op_str == '0x21' for i in callback)
    assert not any(i.mnemonic == 'int' and i.op_str == '0x16' for i in callback)
    assert not any(i.mnemonic == 'int' and i.op_str == '0x33' for i in callback)
    saved_offsets = [int(i.op_str.split('+')[1].split(']')[0], 16)
                     for i in callback if i.mnemonic == 'mov' and 'ss:[bp +' in i.op_str]
    assert saved_offsets == [0x10, 0x0E, 0x0C, 0x0A, 0x08, 0x06]
    assert all(callback_start <= i.address < callback_end for i in callback)
    assert len(callback) >= 40

    names = [i.mnemonic for i in instructions]
    assert names.count('int') == expected_counts[0]
    int33 = [i for i in instructions if i.mnemonic == 'int' and i.op_str == '0x33']
    assert len(int33) == expected_counts[1]
    int21 = [i for i in instructions if i.mnemonic == 'int' and i.op_str == '0x21']
    assert len(int21) == expected_counts[2]
    int16 = [i for i in instructions if i.mnemonic == 'int' and i.op_str == '0x16']
    assert len(int16) == expected_counts[3]
    assert all(i.address > callback_end or i.address < callback_start for i in int21)
    assert not any(i.mnemonic == 'int' and i.op_str == '0x33' and i.address >= callback_start
                   for i in instructions if i.address < callback_end)
    assert b'\xb9\x7f\x00' in code and b'\x31\xc9' in code
    assert len(code) < 2048 and probe.MAX_LOG_BYTES == 904


def test_model():
    records: list[tuple[int, ...]] = []
    overflow = 0
    for sequence in range(probe.RECORD_LIMIT + 3):
        record = (sequence, sequence, 1, sequence + 10, sequence + 20,
                  sequence + 30, sequence + 40)
        if len(records) < probe.RECORD_LIMIT:
            records.append(record)
        elif overflow < 0xFFFF:
            overflow += 1
    assert len(records) == probe.RECORD_LIMIT and overflow == 3
    assert records[0] == (0, 0, 1, 10, 20, 30, 40)
    assert records[-1] == (63, 63, 1, 73, 83, 93, 103)
    assert len(b'MPR1') + 4 + len(records) * probe.RECORD_SIZE == probe.MAX_LOG_BYTES
    encoded = b'MPR1' + struct.pack('<HH', len(records), overflow)
    encoded += b''.join(struct.pack('<7H', *row) for row in records)
    parsed = probe.parse_log(encoded)
    assert parsed['count'] == probe.RECORD_LIMIT and parsed['overflow'] == 3
    assert parsed['records'][0]['sequence'] == 0 and parsed['records'][-1]['di_mickey_y'] == 103
    for invalid in (encoded[:-1], encoded + b'\0', b'BAD!' + encoded[4:],
                    b'MPR1' + struct.pack('<HH', 65, 0) + b'\0' * 910):
        try:
            probe.parse_log(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError('Malformed MPR1 log accepted')


def verify_saved_inputs(directory, code, *, timed):
    """Read immutable prepared inputs; never re-prepare a consumed directory."""
    path = directory / 'logs/prepared.json'
    if not path.exists():
        receipt = probe.prepare_timed() if timed else probe.prepare()
    else:
        receipt = json.loads(path.read_bytes())
    assert receipt['status'] == 'prepared-not-executed'  # historical receipt
    assert receipt['original_execution'] is False and receipt['emulator_execution'] is False
    assert receipt['callback_sha256'] == hashlib.sha256(code).hexdigest()
    assert (directory / 'drive-c/MOUSE.COM').read_bytes() == code
    assert (directory / 'drive-c/MOUSE.BAT').read_bytes() == probe.BATCH
    assembly = probe.ASSEMBLY + ('\n; Timed variant: INT 1Ah is polled in mainline; STOP.BIN stores 1=key, 2=timer.\n' if timed else '')
    assert (directory / 'MOUSE.asm').read_bytes() == assembly.encode('ascii')
    config = probe.SOURCE_CONFIG.read_bytes()
    assert hashlib.sha256(config).hexdigest() == probe.SOURCE_CONFIG_SHA256
    if receipt.get('config_delta'):
        assert timed and receipt['config_delta'] == [{'section': 'sdl', 'key': 'mouse_emulation', 'before': 'never', 'after': 'always'}]
        config = config.replace(b'mouse_emulation = never', b'mouse_emulation = always', 1)
        assert receipt['derived_config_sha256'] == hashlib.sha256(config).hexdigest()
    assert (directory / probe.CONFIG_COPY).read_bytes() == config
    assert receipt['command'] == probe._command_for(directory / probe.CONFIG_COPY, directory)
    assert receipt['working_directory'] == str(directory)
    visible = {p.name for p in (directory / 'drive-c').iterdir()}
    assert {'MOUSE.COM', 'MOUSE.BAT'} <= visible
    assert visible <= {'MOUSE.COM', 'MOUSE.BAT', 'MOUSE.LOG', 'STOP.BIN'}
    logs = directory / 'logs'
    final = json.loads((logs / 'final-process.json').read_bytes()) if (logs / 'final-process.json').exists() else None
    launched = json.loads((logs / 'launch-process.json').read_bytes()) if (logs / 'launch-process.json').exists() else None
    if final:
        assert launched and final['pid'] == launched['pid'] and final['ended_utc']
        runtime = {'runtime_phase': 'closed', 'emulator_execution': True, 'exit_code': final['exit_code'], 'host_timeout': final['host_timeout']}
    elif launched:
        runtime = {'runtime_phase': 'launch-recorded-no-final-receipt', 'emulator_execution': True}
    elif (logs / 'launch-intent.json').exists():
        runtime = {'runtime_phase': 'intent-recorded-execution-unconfirmed', 'emulator_execution': None}
    else:
        assert visible == {'MOUSE.COM', 'MOUSE.BAT'}
        runtime = {'runtime_phase': 'prepared-not-launched', 'emulator_execution': False}
    return receipt, runtime


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--timed-probe-id', default='mouse-probe-011')
    args = parser.parse_args()
    if not re.fullmatch(r'mouse-probe-[0-9]{3}', args.timed_probe_id):
        raise ValueError('Timed probe ID must be mouse-probe-NNN')
    probe.TIMED_PROBE = ROOT / 'private/reference-run' / args.timed_probe_id
    capstone = load_capstone()
    assert capstone.__version__ == '5.0.1'
    receipt, runtime = verify_saved_inputs(probe.PROBE, probe.CODE, timed=False)
    instructions = decode(capstone, probe.CODE)
    check_disassembly(capstone, instructions, probe.CODE, probe.CALLBACK_START, probe.CALLBACK_END)
    test_model()

    # A private preparation-only audit is created once, then exact-matched.
    path = probe.PROBE / 'logs/static-validation.json'
    result = {
        'status': 'prepared-not-executed',
        'original_execution': False,
        'emulator_execution': False,
        'com_sha256': hashlib.sha256(probe.CODE).hexdigest(),
        'com_size': len(probe.CODE),
        'callback_bounds': [probe.CALLBACK_START, probe.CALLBACK_END],
        'disassembled_instruction_count_including_data': len(instructions),
        'disassembly_byte_coverage': sum(len(i.bytes) for i in instructions),
        'callback_instructions': len([i for i in instructions if probe.CALLBACK_START <= i.address < probe.CALLBACK_END]),
        'checks': ['full COM byte coverage', 'callback pushes/restores all modified registers and flags',
                   'callback writes sequence then saved input registers AX,BX,CX,DX,SI,DI',
                   'no DOS, BIOS, or mouse interrupt inside callback',
                   'far return terminates callback', 'DOS output lies outside callback',
                   '64-record cap and saturating overflow model', 'no INT 33h function 0Bh'],
        'capstone': {'version': capstone.__version__, 'private_wheel_sha256': CAPSTONE_SHA256,
                     'license': 'BSD-3-Clause; existing private wheel receipt checked'},
        'prepared_receipt': receipt,
    }
    encoded = json.dumps(result, indent=2).encode('utf-8') + b'\n'
    if path.exists():
        assert path.read_bytes() == encoded, 'Preserve existing private static validation'
    else:
        with path.open('xb') as stream:
            stream.write(encoded)
    print(json.dumps({'status': 'static-inputs-verified', 'checker_emulator_execution': False,
        'original_execution': False, **runtime, **{k: result[k] for k in ('com_sha256', 'com_size',
          'callback_bounds', 'callback_instructions', 'checks')}}))

    timed_receipt, timed_runtime = verify_saved_inputs(probe.TIMED_PROBE, probe.TIMED_CODE, timed=True)
    timed_instructions = decode(capstone, probe.TIMED_CODE)
    counts = {vector: sum(1 for i in timed_instructions if i.mnemonic == 'int' and i.op_str == vector)
              for vector in ('0x33', '0x16', '0x1a', '0x21')}
    assert counts == {'0x33': 3, '0x16': 2, '0x1a': 2, '0x21': 8}, counts
    check_disassembly(capstone, timed_instructions, probe.TIMED_CODE,
        probe.TIMED_SYMBOLS['callback'], probe.TIMED_SYMBOLS['callback_end'],
        expected_counts=(15, 3, 8, 2))
    timer_calls = [i for i in timed_instructions if i.mnemonic == 'int' and i.op_str == '0x1a']
    assert len(timer_calls) == 2
    assert all(i.address < probe.TIMED_SYMBOLS['callback'] for i in timer_calls)
    assert timed_receipt['variant'] == 'timed-bios-tick'
    assert timed_receipt['guest_timer_ticks'] == probe.TIMEOUT_TICKS
    assert timed_receipt['stop_reason_file'] == 'drive-c/STOP.BIN (one byte: 1=key, 2=timer)'
    timed_validation = {
        'status': 'prepared-not-executed', 'original_execution': False, 'emulator_execution': False,
        'com_sha256': hashlib.sha256(probe.TIMED_CODE).hexdigest(), 'com_size': len(probe.TIMED_CODE),
        'timeout_ticks': probe.TIMEOUT_TICKS,
        'callback_bounds': [probe.TIMED_SYMBOLS['callback'], probe.TIMED_SYMBOLS['callback_end']],
        'interrupt_counts': counts,
        'disassembly_byte_coverage': sum(len(i.bytes) for i in timed_instructions),
        'checks': ['full timed COM byte coverage', 'register/flag-preserving 64-record callback retained',
            'INT 1Ah tick polls are in mainline only', 'callback contains no DOS/BIOS/mouse interrupts',
            'callback unregister precedes MOUSE.LOG and STOP.BIN writes',
            'STOP.BIN stores a separate one-byte stop reason'],
        'prepared_receipt': timed_receipt,
    }
    timed_path = probe.TIMED_PROBE / 'logs/static-validation.json'
    timed_encoded = json.dumps(timed_validation, indent=2).encode('utf-8') + b'\n'
    if timed_path.exists():
        saved = json.loads(timed_path.read_bytes())
        for key in ('com_sha256', 'com_size', 'timeout_ticks', 'callback_bounds', 'prepared_receipt'):
            assert saved[key] == timed_validation[key], ('Preserve existing private timed validation', key)
    else:
        with timed_path.open('xb') as stream:
            stream.write(timed_encoded)
    print(json.dumps({'timed_status': 'static-inputs-verified', 'checker_emulator_execution': False,
        'original_execution': False, **timed_runtime,
        'timed_com_sha256': timed_validation['com_sha256'], 'timed_com_size': timed_validation['com_size'],
        'timeout_ticks': probe.TIMEOUT_TICKS, 'interrupt_counts': counts,
        'probe': str(probe.TIMED_PROBE)}))


if __name__ == '__main__':
    main()

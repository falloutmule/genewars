"""Prepare an independently authored empty DOS INT 33h callback logger.

Default action is prepare/verify only. It never starts DOSBox. The guest has
only this logger and its batch file mounted; it waits for a key after
registering the callback, unregisters before writing a bounded log, and does
not install or copy a mouse driver or game code.
"""
from __future__ import annotations

from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import struct
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
sys.path.insert(0, str(ROOT / 'scripts/reference'))
from source_member import digest
from prepare import checked, file_record
from launch import arguments

PROBE = ROOT / 'private/reference-run/mouse-probe-008'
SOURCE_CONFIG = ROOT / 'private/reference-run/run-005/dosbox-x.conf'
SOURCE_CONFIG_SHA256 = '506802e42d08c46d77f176efcd0efab03cbf57e2b8fbe15f8242d0b9284a285b'
READINESS = ROOT / 'research/variants/reference-run-005-readiness.json'
CONFIG_SHA256 = SOURCE_CONFIG_SHA256
CONFIG_COPY = 'dosbox-x.conf'
RECORD_LIMIT = 64
RECORD_SIZE = 14
MAX_LOG_BYTES = 8 + RECORD_LIMIT * RECORD_SIZE
TIMEOUT_TICKS = 60 * 18 + 12  # approximately 60 s at BIOS's 18.2065 Hz


class Asm:
    """Small fixed-purpose 8086 COM emitter with checked relative labels."""
    def __init__(self):
        self.code = bytearray()
        self.labels: dict[str, int] = {}
        self.fixups: list[tuple[int, str, str]] = []

    def raw(self, *values: int):
        self.code.extend(values)

    def label(self, name: str):
        if name in self.labels:
            raise ValueError('duplicate label: ' + name)
        self.labels[name] = len(self.code)

    def imm16(self, *prefix: int, label: str):
        self.raw(*prefix, 0, 0)
        self.fixups.append((len(self.code) - 2, label, 'abs'))

    def rel8(self, *prefix: int, label: str):
        self.raw(*prefix, 0)
        self.fixups.append((len(self.code) - 1, label, 'rel8'))

    def finish(self) -> bytes:
        result = bytearray(self.code)
        for position, name, kind in self.fixups:
            if name not in self.labels:
                raise ValueError('unresolved label: ' + name)
            target = self.labels[name]
            if kind == 'abs':
                value = target + 0x100
                result[position:position + 2] = struct.pack('<H', value)
            else:
                displacement = target - (position + 1)
                if not -128 <= displacement <= 127:
                    raise ValueError(f'rel8 out of range: {name} ({displacement})')
                result[position] = displacement & 0xFF
        return bytes(result)


def _mem(a: Asm, op: tuple[int, ...], name: str):
    a.imm16(*op, label=name)


def build_com(timeout_ticks: int | None = None) -> tuple[bytes, dict[str, int]]:
    """Build the COM image; callback is bounded, register-preserving, DOS-free."""
    if timeout_ticks is not None and not 1 <= timeout_ticks <= 0x7FFF:
        raise ValueError('timeout_ticks must fit the unambiguous 16-bit BIOS tick interval')
    a = Asm()
    # Establish a conventional COM data segment, reset INT 33h, then install
    # one callback for movement and both-button transitions (mask 0x7f).
    a.raw(0x0E, 0x1F)                 # push cs / pop ds
    a.raw(0x31, 0xC0, 0xCD, 0x33)    # xor ax,ax / int 33h reset/status
    a.raw(0xB8, 0x0C, 0x00, 0xB9, 0x7F, 0x00)
    a.raw(0x0E, 0x07)                 # push cs / pop es
    _mem(a, (0xBA,), 'callback')      # mov dx,callback
    a.raw(0xCD, 0x33)
    if timeout_ticks is None:
        # Preserve CODE008 byte-for-byte: BIOS keyboard wait only.
        a.label('wait_key')
        a.raw(0xB4, 0x01, 0xCD, 0x16)    # BIOS check key
        a.rel8(0x74, label='wait_key')   # jz wait_key
        a.raw(0xB4, 0x00, 0xCD, 0x16)    # consume stop key
    else:
        # The timer is polled in mainline only; callback code remains unchanged.
        # BIOS INT 1Ah AH=00 returns the low tick word in DX. The requested
        # interval is strictly below half the word range, so wrapping subtraction
        # is unambiguous for this one-minute bounded probe.
        a.raw(0xB4, 0x00, 0xCD, 0x1A)    # read initial BIOS tick count
        _mem(a, (0x89, 0x16), 'start_ticks')  # mov [start_ticks],dx
        a.label('wait_key')
        a.raw(0xB4, 0x01, 0xCD, 0x16)    # BIOS check key
        a.rel8(0x74, label='check_timer')
        a.raw(0xC6, 0x06)
        a.imm16(label='stop_reason')
        a.raw(1)                          # stop_reason=1 (keyboard)
        a.raw(0xB4, 0x00, 0xCD, 0x16)    # consume stop key
        a.rel8(0xEB, label='stop_wait')
        a.label('check_timer')
        a.raw(0xB4, 0x00, 0xCD, 0x1A)    # current BIOS tick count
        _mem(a, (0x2B, 0x16), 'start_ticks')  # sub dx,[start_ticks]
        a.raw(0x81, 0xFA, timeout_ticks & 0xFF, (timeout_ticks >> 8) & 0xFF)
        a.rel8(0x72, label='wait_key')    # unsigned elapsed < timeout
        a.raw(0xC6, 0x06)
        a.imm16(label='stop_reason')
        a.raw(2)                          # stop_reason=2 (timer)
        a.label('stop_wait')

    # Unregister the callback before any DOS file operation.
    a.raw(0xB8, 0x0C, 0x00, 0x31, 0xC9)
    a.raw(0x0E, 0x07)
    _mem(a, (0xBA,), 'callback')
    a.raw(0xCD, 0x33)

    # DOS create and write a bounded MPR1 header, then count*14 callback bytes.
    a.raw(0xB4, 0x3C, 0x31, 0xC9)    # AH=3c, CX=0
    _mem(a, (0xBA,), 'filename')
    a.raw(0xCD, 0x21)
    a.rel8(0x72, label='exit_error')
    a.raw(0x89, 0xC3)                 # mov bx,ax handle
    a.raw(0xB9, 0x08, 0x00)
    _mem(a, (0xBA,), 'header')
    a.raw(0xB4, 0x40, 0xCD, 0x21)    # write 8-byte header
    a.rel8(0x72, label='close_file')
    _mem(a, (0x8B, 0x0E), 'count')    # mov cx,[count]
    a.raw(0x89, 0xC8, 0xC1, 0xE1, 0x03, 0xC1, 0xE0, 0x02, 0x01, 0xC1)
    _mem(a, (0x8B, 0x06), 'count')
    a.raw(0xD1, 0xE0, 0x01, 0xC1)    # +2*n => count*14
    a.rel8(0x74, label='close_file')
    _mem(a, (0xBA,), 'records')
    a.raw(0xB4, 0x40, 0xCD, 0x21)    # write CX = count*14 bytes
    a.label('close_file')
    a.raw(0xB4, 0x3E, 0xCD, 0x21)    # close handle
    if timeout_ticks is not None:
        # Keep MPR1 unchanged. This separate one-byte status is written only
        # after callback unregistration: 1=key, 2=timer.
        a.raw(0xB4, 0x3C, 0x31, 0xC9)
        _mem(a, (0xBA,), 'stop_filename')
        a.raw(0xCD, 0x21)
        a.rel8(0x72, label='exit_error')
        a.raw(0x89, 0xC3, 0xB9, 0x01, 0x00)
        _mem(a, (0xBA,), 'stop_reason')
        a.raw(0xB4, 0x40, 0xCD, 0x21)
        a.raw(0xB4, 0x3E, 0xCD, 0x21)
    a.label('exit_error')
    a.raw(0xB8, 0x00, 0x4C, 0xCD, 0x21)

    # Callback receives INT 33h values in AX/BX/CX/DX/SI/DI. Save all general
    # and segment registers and flags before using them; append one 12-byte
    # record or increment overflow, then restore and FAR-return. No DOS calls.
    a.label('callback')
    a.raw(0x9C, 0x50, 0x53, 0x51, 0x52, 0x56, 0x57, 0x1E, 0x06, 0x55)
    a.raw(0x89, 0xE5)                 # mov bp,sp
    a.raw(0x0E, 0x1F)                 # push cs / pop ds for probe data
    _mem(a, (0x8B, 0x3E), 'count')   # mov di,[count]
    a.raw(0x83, 0xFF, RECORD_LIMIT) # cmp di,64
    a.rel8(0x73, label='overflow')
    a.raw(0x89, 0xFB, 0x89, 0xF8, 0xC1, 0xE0, 0x03,
          0xC1, 0xE7, 0x02, 0x01, 0xF8, 0xD1, 0xE3, 0x01, 0xD8, 0x89, 0xC7)
    # DI = records + count*14. DS is set to CS for the callback.
    a.raw(0x81, 0xC7)
    a.raw(0, 0)  # patched records base + 0x100
    a.fixups.append((len(a.code) - 2, 'records', 'abs'))
    a.raw(0x2E, 0xA1)
    a.imm16(label='count')
    a.raw(0x89, 0x05, 0x83, 0xC7, 0x02)  # explicit sequence index
    for stack_offset in (16, 14, 12, 10, 8, 6):
        a.raw(0x36, 0x8B, 0x46, stack_offset)  # mov ax,ss:[bp+saved-reg]
        a.raw(0x89, 0x05)                       # mov [di],ax
        a.raw(0x83, 0xC7, 0x02)                # add di,2
    a.raw(0x2E, 0xFF, 0x06)
    a.imm16(label='count')                     # inc word [cs:count]
    a.rel8(0xEB, label='return_callback')
    a.label('overflow')
    a.raw(0x2E, 0x83, 0x3E)
    a.imm16(label='overflow_count')
    a.raw(0xFF)                       # cmp word [cs:overflow_count],0xffff
    a.rel8(0x74, label='return_callback')
    a.raw(0x2E, 0xFF, 0x06)
    a.imm16(label='overflow_count')
    a.label('return_callback')
    a.raw(0x5D, 0x07, 0x1F, 0x5F, 0x5E, 0x5A, 0x59, 0x5B, 0x58, 0x9D, 0xCB)
    a.label('callback_end')

    # Fixed file header doubles as the two counters.
    a.label('filename')
    a.raw(*b'MOUSE.LOG', 0)
    if timeout_ticks is not None:
        a.label('stop_filename')
        a.raw(*b'STOP.BIN', 0)
    a.label('header')
    a.raw(*b'MPR1')
    a.label('count')
    a.raw(0, 0)
    a.label('overflow_count')
    a.raw(0, 0)
    a.label('records')
    a.raw(*([0] * (RECORD_LIMIT * RECORD_SIZE)))
    if timeout_ticks is not None:
        a.label('start_ticks')
        a.raw(0, 0)
        a.label('stop_reason')
        a.raw(0)
    a.raw(0, 0) if timeout_ticks is not None else a.raw(0)  # keep timed image evenly decodable; preserve CODE008
    code = a.finish()
    if len(code) > 2048 or len(code) < MAX_LOG_BYTES:
        raise ValueError('unexpected COM image size')
    return code, {name: offset + 0x100 for name, offset in a.labels.items()}


CODE, SYMBOLS = build_com()
CALLBACK_START = SYMBOLS['callback']
CALLBACK_END = SYMBOLS['callback_end']
BATCH = b'@ECHO OFF\r\nC:\\MOUSE.COM\r\nEXIT\r\n'
TIMED_PROBE = ROOT / 'private/reference-run/mouse-probe-011'
TIMED_CODE, TIMED_SYMBOLS = build_com(TIMEOUT_TICKS)
ASSEMBLY = '''; Independently authored bounded INT 33h callback probe (8086 COM).
; Registration: AX=0000 reset/status; AX=000C CX=007F ES:DX=callback.
; Callback logs sequence,AX,BX,CX,DX,SI,DI to a 64-record fixed buffer; overflow saturates
; into a counter. It performs no DOS calls and FAR-returns after restoring state.
; BIOS keyboard input ends the wait; callback is unregistered before DOS writes.
; MOUSE.LOG: "MPR1", u16 count, u16 overflow, then count records of seven u16s.
; SI/DI values are preserved as reported; no INT 33h function 0Bh is called.
;
; Mainline outline (COM CS is the probe data segment):
;   DS := CS; INT 33h AX=0; INT 33h AX=0Ch,CX=007F,ES:DX=callback
;   while BIOS INT 16h AH=1 reports no key: repeat
;   BIOS INT 16h AH=0 consumes stop key
;   INT 33h AX=0Ch,CX=0,ES:DX=callback ; stop callback before DOS
;   DOS INT 21h AH=3Ch creates MOUSE.LOG; AH=40h writes header + count*14;
;   AH=3Eh closes; AH=4Ch exits.
; callback:
;   PUSHF; push AX,BX,CX,DX,SI,DI,DS,ES,BP; BP:=SP; DS:=CS
;   if count<64: records[count] := count,saved AX,BX,CX,DX,SI,DI; count++
;   else if overflow!=FFFF: overflow++
;   pop BP,ES,DS,DI,SI,DX,CX,BX,AX; POPF; RETF
; The Python emitter is the byte-level source of truth and the validator
; disassembles every byte and verifies callback boundaries and interrupt use.
'''


def _command(config: Path) -> list[str]:
    return _command_for(config, PROBE)


def _command_for(config: Path, probe_dir: Path) -> list[str]:
    command = arguments(config, 180)
    drive = probe_dir / 'drive-c'
    for dos in (f'MOUNT C "{drive}" -nocachedir', 'MOUNT', 'CONFIG -SECUREMODE',
                'C:', 'C:\\MOUSE.BAT'):
        command.extend(['-c', dos])
    return command


def prepare_timed(mouse_emulation: str = 'never') -> dict:
    """Prepare a fresh, automatically flushing timer variant; never execute it."""
    if mouse_emulation not in ('never', 'always'):
        raise ValueError('The reviewed timed probe permits only never or always mouse emulation')
    probe_dir = TIMED_PROBE
    config_source = SOURCE_CONFIG
    if not config_source.is_file() or digest(config_source) != SOURCE_CONFIG_SHA256:
        raise ValueError('The reviewed run-005 mouse configuration changed')
    readiness = json.loads(READINESS.read_bytes())
    if readiness.get('session') != 'run-005' or readiness.get('original_execution') is not True:
        raise ValueError('Expected reviewed run-005 configuration receipt')
    config_data = config_source.read_bytes()
    baseline_line = b'mouse_emulation = never'
    if config_data.count(baseline_line) != 1 or config_data.count(b'raw_mouse_input = false') != 1:
        raise ValueError('Expected one baseline mouse mode and unchanged raw-input setting')
    if mouse_emulation == 'always':
        config_data = config_data.replace(baseline_line, b'mouse_emulation = always', 1)
    if probe_dir.exists():
        receipt_path = probe_dir / 'logs/prepared.json'
        if not receipt_path.is_file():
            raise FileExistsError('Timed probe path exists without its receipt; preserve it')
        receipt = json.loads(receipt_path.read_bytes())
        expected = [('drive-c/MOUSE.COM', TIMED_CODE), ('drive-c/MOUSE.BAT', BATCH),
            ('MOUSE.asm', (ASSEMBLY + '\n; Timed variant: INT 1Ah is polled in mainline; STOP.BIN stores 1=key, 2=timer.\n').encode('ascii')),
            (CONFIG_COPY, config_data)]
        for relative, data in expected:
            if not (probe_dir / relative).is_file() or (probe_dir / relative).read_bytes() != data:
                raise ValueError('Prepared timed probe file changed: ' + relative)
        if receipt.get('callback_sha256') != hashlib.sha256(TIMED_CODE).hexdigest() or receipt.get('variant') != 'timed-bios-tick':
            raise ValueError('Timed probe receipt does not match the timer COM image')
        if receipt.get('status') != 'prepared-not-executed' or receipt.get('original_execution') is not False or receipt.get('emulator_execution') is not False:
            raise ValueError('Existing timed probe receipt no longer describes prepare-only state')
        if receipt.get('command') != _command_for(probe_dir / CONFIG_COPY, probe_dir) or receipt.get('working_directory') != str(probe_dir):
            raise ValueError('Prepared timed command or working directory changed')
        visible = {path.name for path in (probe_dir / 'drive-c').iterdir()}
        allowed = {'MOUSE.COM', 'MOUSE.BAT'}
        if (probe_dir / 'logs/launch-intent.json').is_file():
            allowed |= {'MOUSE.LOG', 'STOP.BIN'}
        if not {'MOUSE.COM', 'MOUSE.BAT'} <= visible or visible - allowed:
            raise ValueError('Unexpected guest-visible file in prepared timed drive C')
        if mouse_emulation == 'always' and receipt.get('config_delta') != [{'section': 'sdl', 'key': 'mouse_emulation', 'before': 'never', 'after': 'always'}]:
            raise ValueError('Expected the exact single always-mode diagnostic config delta')
        return receipt

    for name in ('drive-c', 'logs', 'states', 'captures'):
        checked(probe_dir / name, ROOT / 'private/reference-run').mkdir(parents=True, exist_ok=False)
    assembly = (ASSEMBLY + '\n; Timed variant: INT 1Ah is polled in mainline; STOP.BIN stores 1=key, 2=timer.\n').encode('ascii')
    files = [('drive-c/MOUSE.COM', TIMED_CODE), ('drive-c/MOUSE.BAT', BATCH),
        ('MOUSE.asm', assembly), (CONFIG_COPY, config_data)]
    records = []
    for relative, contents in files:
        path = checked(probe_dir / relative, probe_dir)
        with path.open('xb') as stream:
            stream.write(contents)
        records.append(file_record(path))
    command = _command_for(probe_dir / CONFIG_COPY, probe_dir)
    receipt = {
        'schema_version': 1, 'variant': 'timed-bios-tick', 'status': 'prepared-not-executed',
        'prepared_utc': datetime.now(timezone.utc).isoformat(), 'probe': str(probe_dir),
        'original_execution': False, 'emulator_execution': False, 'host_watchdog_seconds': 180,
        'guest_timer_ticks': TIMEOUT_TICKS, 'guest_timer_approx_seconds': TIMEOUT_TICKS / 18.2065,
        'guest_record_limit': RECORD_LIMIT, 'guest_max_log_bytes': MAX_LOG_BYTES,
        'callback_sha256': hashlib.sha256(TIMED_CODE).hexdigest(),
        'baseline_code008_sha256': hashlib.sha256(CODE).hexdigest(),
        'callback_registration': {'int': 0x33, 'ax': 0x000C, 'cx_mask': 0x007F,
            'callback_segment': 'COM CS', 'callback_offset': 'resolved label'},
        'records': ['u16 sequence index', 'AX event mask', 'BX held buttons', 'CX absolute x',
            'DX absolute y', 'SI mickey x', 'DI mickey y'],
        'stop': 'keyboard BIOS key or approximately 60-second INT 1Ah tick deadline; unregister before DOS output',
        'stop_reason_file': 'drive-c/STOP.BIN (one byte: 1=key, 2=timer)',
        'mouse_emulation': ('never; raw_mouse_input=false, copied unchanged from reviewed run-005 config'
            if mouse_emulation == 'never' else 'always; raw_mouse_input=false, only mouse_emulation differs from reviewed run-005 config'),
        'mount_scope': 'C is only the fresh probe drive-c directory; no E/D mount; no game/disc mount',
        'working_directory': str(probe_dir), 'command': command, 'files': records,
        'config_source_receipt': {'path': str(READINESS.relative_to(ROOT)),
            'config_path': 'private/reference-run/run-005/dosbox-x.conf', 'config_sha256': SOURCE_CONFIG_SHA256},
    }
    if mouse_emulation == 'always':
        receipt['config_delta'] = [{'section': 'sdl', 'key': 'mouse_emulation', 'before': 'never', 'after': 'always'}]
        receipt['derived_config_sha256'] = hashlib.sha256(config_data).hexdigest()
    receipt_path = probe_dir / 'logs/prepared.json'
    with receipt_path.open('x', encoding='utf-8') as stream:
        json.dump(receipt, stream, indent=2)
        stream.write('\n')
    return receipt


def parse_log(data: bytes) -> dict:
    """Decode the guest MPR1 file without accepting partial/oversized logs."""
    if len(data) < 8 or data[:4] != b'MPR1':
        raise ValueError('Not a complete MPR1 callback log header')
    count, overflow = struct.unpack_from('<HH', data, 4)
    if count > RECORD_LIMIT or len(data) != 8 + count * RECORD_SIZE:
        raise ValueError('MPR1 count/length mismatch or record bound exceeded')
    records = []
    for index in range(count):
        record = struct.unpack_from('<7H', data, 8 + index * RECORD_SIZE)
        if record[0] != index:
            raise ValueError('MPR1 sequence index is not contiguous')
        records.append(dict(sequence=record[0], ax_event_mask=record[1],
                            bx_held_buttons=record[2], cx_x=record[3], dy_y=record[4],
                            si_mickey_x=record[5], di_mickey_y=record[6]))
    return {'count': count, 'overflow': overflow, 'records': records}


def prepare() -> dict:
    receipt_path = PROBE / 'logs/prepared.json'
    if PROBE.exists() and not receipt_path.exists():
        raise FileExistsError('Probe directory already exists without its receipt; preserve it')
    config_source = SOURCE_CONFIG
    if not config_source.is_file() or digest(config_source) != SOURCE_CONFIG_SHA256:
        raise ValueError('The reviewed run-005 mouse configuration changed')
    readiness = json.loads(READINESS.read_bytes())
    if readiness.get('session') != 'run-005' or readiness.get('original_execution') is not True:
        raise ValueError('Expected reviewed run-005 configuration receipt')
    code = CODE
    if PROBE.exists():
        receipt = json.loads(receipt_path.read_bytes())
        for relative, expected in [('drive-c/MOUSE.COM', code), ('drive-c/MOUSE.BAT', BATCH),
                                   ('MOUSE.asm', ASSEMBLY.encode('ascii')),
                                   (CONFIG_COPY, config_source.read_bytes())]:
            path = PROBE / relative
            if not path.is_file() or path.read_bytes() != expected:
                raise ValueError('Prepared probe file changed: ' + relative)
        if receipt.get('callback_sha256') != hashlib.sha256(code).hexdigest():
            raise ValueError('Prepared callback binary hash changed')
        if receipt.get('status') != 'prepared-not-executed' or receipt.get('original_execution') is not False or receipt.get('emulator_execution') is not False:
            raise ValueError('Existing probe receipt no longer describes prepare-only state')
        if receipt.get('command') != _command(PROBE / CONFIG_COPY) or receipt.get('working_directory') != str(PROBE):
            raise ValueError('Prepared DOSBox command or working directory changed')
        if {path.name for path in (PROBE / 'drive-c').iterdir()} != {'MOUSE.COM', 'MOUSE.BAT'}:
            raise ValueError('Unexpected guest-visible file in prepared drive C')
        return receipt

    for name in ('drive-c', 'logs', 'states', 'captures'):
        checked(PROBE / name, ROOT / 'private/reference-run').mkdir(parents=True, exist_ok=False)
    files = [('drive-c/MOUSE.COM', code), ('drive-c/MOUSE.BAT', BATCH),
             ('MOUSE.asm', ASSEMBLY.encode('ascii')), (CONFIG_COPY, config_source.read_bytes())]
    records = []
    for relative, contents in files:
        path = checked(PROBE / relative, PROBE)
        with path.open('xb') as stream:
            stream.write(contents)
        records.append(file_record(path))
    command = _command(PROBE / CONFIG_COPY)
    receipt = {
        'schema_version': 1,
        'status': 'prepared-not-executed',
        'prepared_utc': datetime.now(timezone.utc).isoformat(),
        'probe': str(PROBE),
        'original_execution': False,
        'emulator_execution': False,
        'host_watchdog_seconds': 180,
        'guest_record_limit': RECORD_LIMIT,
        'guest_max_log_bytes': MAX_LOG_BYTES,
        'callback_sha256': hashlib.sha256(code).hexdigest(),
        'callback_registration': {'int': 0x33, 'ax': 0x000C, 'cx_mask': 0x007F,
                                  'callback_segment': 'COM CS', 'callback_offset': 'resolved label'},
        'records': ['u16 sequence index', 'AX event mask', 'BX held buttons', 'CX absolute x', 'DX absolute y',
                    'SI mickey x', 'DI mickey y'],
        'stop': 'BIOS key read in mainline; function 0Ch with CX=0 disables events before DOS file output',
        'mouse_emulation': 'never; raw_mouse_input=false, copied unchanged from reviewed run-005 config',
        'mount_scope': 'C is only the fresh probe drive-c directory; no E/D mount; no game/disc mount',
        'working_directory': str(PROBE),
        'command': command,
        'files': records,
        'config_source_receipt': {'path': str(READINESS.relative_to(ROOT)),
                                  'config_path': 'private/reference-run/run-005/dosbox-x.conf',
                                  'config_sha256': SOURCE_CONFIG_SHA256},
    }
    with receipt_path.open('x', encoding='utf-8') as stream:
        json.dump(receipt, stream, indent=2)
        stream.write('\n')
    return receipt


def main() -> None:
    receipt = prepare()
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()

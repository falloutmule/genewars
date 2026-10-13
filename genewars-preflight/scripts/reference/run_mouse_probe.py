"""Run only the reviewed independent empty mouse logger, bounded by host time.

Preparation remains the default. --run-empty is an explicit execution switch;
this command never mounts game resources or the disc and cannot run originals.
"""
from datetime import datetime, timezone
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import time

import mouse_probe as probe


def write_new(path, data):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(data, stream, indent=2)
        stream.write('\n')


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-empty', action='store_true')
    parser.add_argument('--probe-id', default='mouse-probe-008')
    parser.add_argument('--timed', action='store_true')
    parser.add_argument('--mouse-emulation', choices=['never', 'always'], default='never')
    args = parser.parse_args()
    if not re.fullmatch(r'mouse-probe-[0-9]{3}', args.probe_id):
        raise ValueError('Probe ID must be mouse-probe-NNN')
    probe.PROBE = probe.ROOT / 'private/reference-run' / args.probe_id
    if args.timed:
        probe.TIMED_PROBE = probe.PROBE
    if not args.timed and args.mouse_emulation != 'never':
        raise ValueError('Mouse-mode variants require the timed logger')
    prepared = probe.prepare_timed(args.mouse_emulation) if args.timed else probe.prepare()
    command = prepared['command']
    executable = Path(command[0])
    expected_tool = 'a82adcfa7af2784566cfeff74c077cefed08505396db2948b7e6aa7d4cee0832'
    assert hashlib.sha256(executable.read_bytes()).hexdigest() == expected_tool
    assert prepared['original_execution'] is False
    assert command == probe._command(probe.PROBE / probe.CONFIG_COPY)
    assert prepared['working_directory'] == str(probe.PROBE)
    assert {path.name for path in (probe.PROBE / 'drive-c').iterdir()} == {'MOUSE.COM', 'MOUSE.BAT'}
    logs = probe.PROBE / 'logs'
    assert (logs / 'static-validation.json').is_file()
    if not args.run_empty:
        print(json.dumps({'status': 'verified-not-launched', 'command': command,
                          'host_timeout_seconds': 180, 'original_execution': False}))
        return
    # Exclusive creation prevents this particular private probe from rerunning.
    write_new(logs / 'launch-intent.json', {
        'utc': datetime.now(timezone.utc).isoformat(), 'command': command,
        'original_execution': False, 'host_timeout_seconds': 180,
        'tool_sha256': expected_tool, 'callback_sha256': prepared['callback_sha256'],
        'windows_show_window': 1,
    })
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = 1  # Explicitly supervised SDL window must be visible.
    started = time.monotonic()
    with (logs / 'stdout.txt').open('xb') as out, (logs / 'stderr.txt').open('xb') as err:
        process = subprocess.Popen(command, cwd=probe.PROBE, stdin=subprocess.DEVNULL,
                                   stdout=out, stderr=err, startupinfo=startup)
    timed_out = False
    try:
        launched = {'pid': process.pid, 'utc': datetime.now(timezone.utc).isoformat(),
                    'executable': str(executable), 'original_execution': False}
        write_new(logs / 'launch-process.json', launched)
        print(json.dumps(launched), flush=True)
        process.wait(timeout=max(0, 180 - (time.monotonic() - started)))
    except subprocess.TimeoutExpired:
        timed_out = True
        process.kill()  # Only the child represented by this exact Popen handle.
    finally:
        if process.poll() is None:
            process.kill()
        process.wait(timeout=10)
        final = {'pid': process.pid, 'exit_code': process.returncode,
                 'ended_utc': datetime.now(timezone.utc).isoformat(),
                 'elapsed_host_seconds': time.monotonic() - started,
                 'host_timeout': timed_out, 'original_execution': False}
        write_new(logs / 'final-process.json', final)
    log_path = probe.PROBE / 'drive-c/MOUSE.LOG'
    if log_path.is_file():
        data = log_path.read_bytes()
        final['callback_log'] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
        try:
            final['callback_log']['decoded'] = probe.parse_log(data)
        except ValueError as error:
            final['callback_log']['parse_error'] = str(error)
    else:
        final['callback_log'] = {'status': 'unavailable'}
    if args.timed:
        stop_path = probe.PROBE / 'drive-c/STOP.BIN'
        stop = stop_path.read_bytes() if stop_path.is_file() else b''
        final['guest_stop_reason'] = {b'\x01': 'keyboard', b'\x02': 'timer'}.get(stop, 'unavailable-or-invalid')
    write_new(logs / 'result.json', final)
    print(json.dumps(final), flush=True)


if __name__ == '__main__':
    run()

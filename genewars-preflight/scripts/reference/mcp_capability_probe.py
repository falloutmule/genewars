"""Bounded empty-emulator test of upstream's documented line control protocol.

Independent stdlib client; no third-party wrapper, game or disc is loaded.
The fixed request list never writes guest memory or registers.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import hashlib
import json
import shutil
import socket
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[2]
TOOL = ROOT / 'private/tools/dosbox-x/2026.10.01/bin/x64/Release SDL2/dosbox-x.exe'
BASE = ROOT / 'private/reference-run/mouse-probe-012'
PINS = {
    TOOL: 'a82adcfa7af2784566cfeff74c077cefed08505396db2948b7e6aa7d4cee0832',
    BASE/'dosbox-x.conf': '1e805ff8fbda2884f989fc62b43fd50c2a87052c204c42382b6ebb76b8e436e1',
    BASE/'drive-c/MOUSE.COM': '7e0f4eca31e358b25a5aad959ce9eeec3ecd7c7e876ac78bf28999bdc1ce5b3b',
}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run-empty', action='store_true')
    args = parser.parse_args()
    for path, expected in PINS.items():
        assert sha(path) == expected, path
    if not args.run_empty:
        print('Verified inputs; --run-empty starts the bounded empty capability test.')
        return
    work = ROOT / 'private/reference-run' / ('mcp-capability-' + uuid.uuid4().hex)
    for directory in ['drive-c','logs','states','captures']:
        (work/directory).mkdir(parents=True, exist_ok=False)
    shutil.copyfile(Path(__file__),work/'client-executed.py')
    shutil.copyfile(BASE/'drive-c/MOUSE.COM',work/'drive-c/MOUSE.COM')
    (work/'drive-c/MOUSE.BAT').write_bytes(b'@ECHO OFF\r\nC:\\MOUSE.COM\r\nEXIT\r\n')
    receipt = dict(status='started',utc=datetime.now(timezone.utc).isoformat(),
                   private_directory=work.relative_to(ROOT).as_posix(), original_execution=False,
                   input_injection=False,host_timeout_seconds=90,requests=[],sent_requests=[],
                   client_snapshot=(work/'client-executed.py').relative_to(ROOT).as_posix())
    started = time.monotonic()
    process = None
    with socket.socket(socket.AF_INET,socket.SOCK_STREAM) as listener:
        if hasattr(socket,'SO_EXCLUSIVEADDRUSE'):
            listener.setsockopt(socket.SOL_SOCKET,socket.SO_EXCLUSIVEADDRUSE,1)
        listener.bind(('127.0.0.1',0))
        listener.listen(1)
        listener.settimeout(20)
        port = listener.getsockname()[1]
        config = (BASE/'dosbox-x.conf').read_bytes()
        assert config.count(b'mcp_server = 0') == 1
        config = config.replace(b'mcp_server = 0',f'mcp_server = {port}'.encode('ascii'))
        (work/'dosbox-x.conf').write_bytes(config)
        command = [str(TOOL),'-conf',str(work/'dosbox-x.conf'),'-noconfig','-noautoexec',
                   '-defaultmapper','-nopromptfolder','-savedir','states','-time-limit','90','-showrt']
        for line in [f'MOUNT C "{work / "drive-c"}" -nocachedir','MOUNT','CONFIG -SECUREMODE','C:','C:\\MOUSE.BAT']:
            command += ['-c',line]
        receipt.update(command=command,config_sha256=sha(work/'dosbox-x.conf'),port=port)
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        try:
            with (work/'logs/stdout.txt').open('xb') as out,(work/'logs/stderr.txt').open('xb') as err:
                process = subprocess.Popen(command,cwd=work,stdin=subprocess.DEVNULL,stdout=out,stderr=err,startupinfo=startup)
            receipt['pid'] = process.pid
            print(json.dumps(dict(status='launched-empty',pid=process.pid,private_directory=receipt['private_directory'])),flush=True)
            connection, address = listener.accept()
            assert address[0] == '127.0.0.1'
            with connection:
                connection.settimeout(10)
                buffer = bytearray()
                transcript = bytearray()
                def request(number, command):
                    nonlocal buffer
                    assert command in ['PING','BREAK','EXEC CPU','EXEC HELP','EXEC RUN']
                    wire = f'REQ {number} {command}\n'.encode('ascii')
                    receipt['sent_requests'].append(dict(id=number,command=command))
                    transcript.extend(b'> '+wire)
                    (work/'logs/protocol.txt').write_bytes(transcript)
                    connection.sendall(wire)
                    end = f'END {number}\n'.encode('ascii')
                    deadline = min(started+90,time.monotonic()+10)
                    while end not in buffer:
                        connection.settimeout(max(0.001,deadline-time.monotonic()))
                        chunk = connection.recv(16384)
                        if not chunk: raise RuntimeError('Control connection closed')
                        buffer.extend(chunk)
                        if len(buffer) > 262144: raise RuntimeError('Response exceeds256KiB bound')
                    position = buffer.index(end)+len(end)
                    response = bytes(buffer[:position])
                    del buffer[:position]
                    transcript.extend(response)
                    (work/'logs/protocol.txt').write_bytes(transcript)
                    text = response.decode('utf-8',errors='replace')
                    receipt['requests'].append(dict(id=number,command=command,response=text))
                    if f'BEGIN {number} ERR' in text: raise RuntimeError('Emulator rejected '+command)
                    if f'BEGIN {number} OK' not in text: raise RuntimeError('Missing matching response')
                    return text
                assert 'PONG' in request(1,'PING')
                request(2,'BREAK')
                cpu1 = request(3,'EXEC CPU')
                cpu2 = request(4,'EXEC CPU')
                receipt['repeated_cpu_text_equal'] = cpu1.splitlines()[1:-1] == cpu2.splitlines()[1:-1]
                # Identical nonempty CPU reports corroborate a stopped guest;
                # they are not proof of any original-game consumer behavior.
                if not receipt['repeated_cpu_text_equal'] or not cpu1.splitlines()[1:-1]:
                    raise RuntimeError('No stable stopped CPU observation; resume skipped')
                request(5,'EXEC HELP')
                request(6,'EXEC RUN')
                receipt['status'] = 'control-requests-passed'
            process.wait(timeout=max(0.001,started+90-time.monotonic()))
        except Exception as error:
            receipt['status'] = 'capability-test-incomplete'
            receipt['error'] = str(error)
        finally:
            if process is not None:
                if process.poll() is None:
                    receipt['terminated_owned_child'] = True
                    process.kill()
                process.wait(timeout=10)
                receipt['exit_code'] = process.returncode
            receipt['elapsed_host_seconds'] = time.monotonic()-started
            receipt['immutable_inputs_unchanged'] = all(sha(path)==expected for path,expected in PINS.items())
            receipt['staged_logger_unchanged'] = sha(work/'drive-c/MOUSE.COM') == PINS[BASE/'drive-c/MOUSE.COM']
            receipt['guest_output_files'] = [dict(name=p.name,bytes=p.stat().st_size,sha256=sha(p)) for p in sorted((work/'drive-c').iterdir())]
            receipt['client_sha256'] = sha(Path(__file__))
            (work/'logs/result.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
            (ROOT/'research/variants/mcp-capability-probe.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({k:receipt[k] for k in ['status','elapsed_host_seconds','immutable_inputs_unchanged','staged_logger_unchanged']}))


if __name__ == '__main__':
    main()

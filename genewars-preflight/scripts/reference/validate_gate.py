"""Hidden empty gate fixture: no game bytes or original executable mounted/run."""
from datetime import datetime, timezone
import json
import os
import secrets
import subprocess
import time

from prepare import ROOT, RUN, SDL2, checked, file_record
from launch import arguments, gate_batch
from gate import process_identity, runtime_issues


def main():
    receipts = []
    for release in (True, False):
        token = secrets.token_hex(4).upper()
        fixture = checked(ROOT / 'private/reference-run' / ('gate-fixture-' + token), ROOT / 'private/reference-run')
        for subdir in ('drive-c/GAME', 'media/GENEWARS.cue', 'logs', 'captures', 'states'):
            checked(fixture / subdir, fixture).mkdir(parents=True, exist_ok=True)
        config = fixture / 'dosbox-x.conf'
        config.write_bytes((RUN / 'dosbox-x.conf').read_bytes())
        gate = fixture / 'drive-c/STARTGW.BAT'
        batch = gate_batch(token)  # Defaults to ECHO sentinel, never GW.
        assert b'GW.EXE' not in batch
        gate.write_bytes(batch)
        command = arguments(config, 30)
        dos_commands = [f'MOUNT C "{fixture / "drive-c"}" -nocachedir',
                        f'MOUNT D "{fixture / "media/GENEWARS.cue"}" -t cdrom',
                        'MOUNT', 'CONFIG -SECUREMODE', 'MOUNT E "."', 'C:', 'CD \\GAME',
                        'C:\\STARTGW.BAT']
        for dos_command in dos_commands:
            command.extend(['-c', dos_command])
        env = os.environ.copy()
        env.update(SDL_VIDEODRIVER='dummy', SDL_AUDIODRIVER='dummy')
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        with (fixture/'logs/stdout.txt').open('xb') as out, (fixture/'logs/stderr.txt').open('xb') as err:
            process = subprocess.Popen(command, cwd=fixture, env=env, stdin=subprocess.DEVNULL,
                                       stdout=out, stderr=err, startupinfo=startup,
                                       creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            image, start = process_identity(process.pid)
            assert image == (SDL2 / 'dosbox-x.exe').resolve()
            log_path = fixture / 'logs/dosbox-x.log'
            deadline = time.monotonic() + 15
            while time.monotonic() < deadline:
                log = log_path.read_text(encoding='utf-8',errors='replace') if log_path.exists() else ''
                if 'REFERENCE_GATE_WAITING ' + token in log:
                    break
                if process.poll() is not None:
                    raise ValueError('Fixture exited before gate')
                time.sleep(0.1)
            else:
                raise ValueError('Fixture did not reach gate within15 seconds')
            age = (datetime.now(timezone.utc)-start).total_seconds()
            issues, mounts = runtime_issues(log,token,fixture,age)
            assert not issues, issues
            assert 'REFERENCE_EMPTY_SENTINEL' not in log and 'REFERENCE_GATE_RELEASED' not in log
            assert runtime_issues(log,token,fixture,age,True)[0]
            assert runtime_issues(log,token,fixture,1200)[0]
            assert runtime_issues(log.replace('Switched to secure mode.',''),token,fixture,age)[0]
            assert runtime_issues(log.replace('DOS CON: D ', 'DOS CON: E '),token,fixture,age)[0]
            marker = fixture / 'drive-c' / (token + ('.OK' if release else '.NO'))
            with marker.open('xb') as stream:
                stream.write(b'Empty fixture control only\r\n')
            assert process.wait(timeout=10) == 0
            after = log_path.read_text(encoding='utf-8',errors='replace')
            if release:
                assert 'REFERENCE_EMPTY_SENTINEL' in after and not marker.exists()
            else:
                assert 'REFERENCE_GATE_ABORTED' in after and 'REFERENCE_EMPTY_SENTINEL' not in after
            receipts.append({'fixture':str(fixture.relative_to(ROOT)),'release':release,
                             'original_execution':False,'mounts':mounts,'pid_identity_verified':True,
                             'checks':'Wait before marker; missing lock/wrong mounts/expired time/old marker refused; release sentinel or abort; no original present.',
                             'gate':file_record(gate),'log':file_record(log_path)})
        finally:
            if process.poll() is None:
                process.kill()  # Only this exact child created by the fixture.
                process.wait(timeout=5)
    result = {'passed':True,'original_execution':False,'fixtures':receipts}
    (ROOT/'research/variants/reference-gate-validation.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(result))


if __name__ == '__main__':
    main()

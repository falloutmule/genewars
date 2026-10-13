"""Print the reviewed command by default; never autorun an original.

--validate-empty runs ONLY built-in DOSBox commands, without any game mount.
--open-approved-session opens a waiting DOS gate after a human approval note;
the supervisor must inspect mounts/secure mode and visible UI before release.
"""
import argparse
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import secrets
import time

from prepare import ROOT, RUN, SDL2, RECORD, SESSION, HUMAN_APPROVAL, SESSION_WRAPPERS, wrapper_command, runtime_alias_binding, checked, file_record, prepare_tool, prepare_run


def arguments(config, limit):
    return [str(SDL2 / 'dosbox-x.exe'), '-conf', str(config), '-noconfig',
            '-noautoexec', '-defaultmapper', '-nopromptfolder', '-savedir',
            'states', '-time-limit', str(limit), '-showrt']


def gate_batch(token, *, original=False, yielding=False):
    operation = 'GW.EXE' if original else 'ECHO REFERENCE_EMPTY_SENTINEL'
    wait = ('IF NOT EXIST C:\\YIELD.COM GOTO ABORT\r\nC:\\YIELD.COM\r\n' if yielding else '')
    return ('@ECHO OFF\r\nECHO REFERENCE_GATE_WAITING ' + token + '\r\n'
            ':WAIT\r\nIF EXIST C:\\' + token + '.NO GOTO ABORT\r\n'
            'IF EXIST C:\\' + token + '.OK GOTO RUN\r\n' + wait + 'GOTO WAIT\r\n'
            ':RUN\r\nDEL C:\\' + token + '.OK\r\n'
            'ECHO REFERENCE_GATE_RELEASED ' + token + '\r\n'
            'C:\r\nCD \\GAME\r\n' + operation + '\r\n'
            'ECHO REFERENCE_ORIGINAL_RETURNED\r\nGOTO END\r\n'
            ':ABORT\r\nECHO REFERENCE_GATE_ABORTED\r\n:END\r\nEXIT\r\n').encode('ascii')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group()
    action.add_argument('--validate-empty', action='store_true')
    action.add_argument('--open-approved-session', action='store_true')
    parser.add_argument('--approval-note', help='Record the prior human approval and exact approved scope; this is not authorization itself')
    args = parser.parse_args()
    tool = prepare_tool(False, False)
    runtime = prepare_run(False)
    command = arguments(RUN / 'dosbox-x.conf', 1200)
    commands = [f'MOUNT C "{RUN / "drive-c"}" -nocachedir',
                f'IMGMOUNT D "{RUN / "media/GENEWARS.cue"}" -t iso',
                'MOUNT', 'CONFIG -SECUREMODE', 'MOUNT E "."', 'C:', 'CD \\GAME',
                'REM Stop if E: mount is not refused. Only after approval/checks: GW.EXE']
    if args.validate_empty:
        empty = checked(ROOT / 'private/reference-run/empty-validation', ROOT / 'private/reference-run')
        empty.mkdir(parents=True, exist_ok=True)
        for name in ('logs', 'states', 'captures'):
            checked(empty / name, empty).mkdir(exist_ok=True)
        config = checked(empty / 'dosbox-x.conf', empty)
        config.write_bytes((RUN / 'dosbox-x.conf').read_bytes())
        queries = [('dos', 'automount', 'false'), ('dos', 'network redirector', 'false'),
                   ('dos', 'startcmd', 'false'), ('dos', 'vmware', 'false'),
                   ('dos', 'dos clipboard api', 'false'),
                   ('dos', 'dos clipboard device enable', 'false'),
                   ('dosbox', 'mcp_server', '0'), ('ne2000', 'ne2000', 'false'),
                   ('ne2000', 'backend', 'none'), ('ipx', 'ipx', 'false'),
                   ('serial', 'serial1', 'disabled'), ('parallel', 'parallel1', 'disabled'),
                   ('printer', 'printer', 'false'), ('midi', 'mididevice', 'none'),
                   ('sblaster', 'recording source', 'silence')]
        environment = os.environ.copy()
        environment['SDL_VIDEODRIVER'] = 'dummy'
        environment['SDL_AUDIODRIVER'] = 'dummy'
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = 0
        started = datetime.now(timezone.utc).isoformat()
        batches = []
        # The pinned runtime processes only a bounded number of -c arguments.
        # Keep each invocation to ten built-ins and prove its final EXIT/lock.
        for batch_index in range(0, len(queries), 5):
            batch_queries = queries[batch_index:batch_index + 5]
            batch_dir = checked(empty / f'batch-{batch_index // 5 + 1}', empty)
            for name in ('logs', 'states', 'captures'):
                checked(batch_dir / name, empty).mkdir(parents=True, exist_ok=True)
            batch_config = checked(batch_dir / 'dosbox-x.conf', empty)
            batch_config.write_bytes(config.read_bytes())
            command = arguments(batch_config, 20) + ['-c', 'VER /V', '-c', 'MOUNT']
            for section, key, _ in batch_queries:
                command.extend(['-c', f'CONFIG -GET {section} "{key}"'])
            command += ['-c', 'CONFIG -SECUREMODE', '-c', 'MOUNT E .', '-c', 'EXIT']
            result = subprocess.run(command, cwd=batch_dir, env=environment,
                                    capture_output=True, timeout=30, startupinfo=startup,
                                    creationflags=subprocess.CREATE_NO_WINDOW)
            checked(batch_dir / 'logs/stdout.txt', empty).write_bytes(result.stdout)
            checked(batch_dir / 'logs/stderr.txt', empty).write_bytes(result.stderr)
            log_path = checked(batch_dir / 'logs/dosbox-x.log', empty)
            log = log_path.read_text(encoding='utf-8', errors='replace')
            warnings = [line for line in result.stderr.decode('utf-8', errors='replace').splitlines()
                        if any(token in line.lower() for token in ('not a valid value', 'unknown option', 'unknown property'))]
            assert result.returncode == 0 and not warnings, 'Empty emulator failed or rejected configuration'
            assert 'Switched to secure mode.' in log and 'This operation is not permitted in secure mode.' in log
            assert 'Z      Internal Virtual Drive' in log and 'DOS CON: Z:\\>EXIT' in log
            for section, key, expected in batch_queries:
                marker = f'DOS CON: Z:\\>CONFIG -GET {section} "{key}"\nDOS CON: {expected}'
                assert marker in log, f'Effective configuration query mismatch: {section}/{key}'
            batches.append({'command': command, 'cwd': str(batch_dir), 'returncode': result.returncode,
                            'startup_log': file_record(log_path),
                            'stdout': file_record(batch_dir / 'logs/stdout.txt'),
                            'stderr': file_record(batch_dir / 'logs/stderr.txt')})
        receipt = {'kind': 'hidden-empty-emulator-only', 'original_execution': False,
                   'batches': batches, 'cwd': str(empty), 'started_utc': started,
                   'returncode': 0, 'tool': tool['executable'],
                   'config': file_record(config), 'video_driver': 'dummy', 'audio_driver': 'dummy',
                   'mounts_requested': [], 'builtins_only': ['VER', 'MOUNT', 'CONFIG', 'EXIT'],
                   'securemode_result': 'Observed lock entry and refusal of MOUNT E . in console log.',
                   'effective_queries_passed': [{'section': s, 'key': k, 'value': v} for s,k,v in queries],
                   'configuration_rejection_warnings': [], 'startup_log': batches[-1]['startup_log'],
                   'microphone_limit': 'Runtime enumerates/selects a microphone despite silence recording source. No guest microphone capture requested; this is not proof of OS-level microphone isolation.'}
        checked(empty / 'logs/receipt.json', empty).write_text(json.dumps(receipt, indent=2)+'\n')
        print(json.dumps(receipt))
        return
    if args.open_approved_session:
        human = json.loads(HUMAN_APPROVAL.read_bytes())
        if not human.get('approved'):
            raise SystemExit('This session has no separate recorded human execution approval')
        if SESSION in SESSION_WRAPPERS and (human.get('run_id') != SESSION or
                human.get('candidate_sha256') != '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba' or
                not isinstance(human.get('human_statement'),str) or not human['human_statement'].strip() or
                human.get('generated_config_sha256') != runtime['generated_game_inputs'][0]['sha256']):
            raise SystemExit('Approval does not bind this exact prepared run/config')
        copied = runtime.get('copied_game_inputs',[])
        if copied and human.get('flattened_text_sha256') != copied[0]['sha256']:
            raise SystemExit('Approval does not bind the exact reviewed flattened text copy')
        if copied and human.get('runtime_aliases_sha256') != runtime_alias_binding(copied):
            raise SystemExit('Approval does not bind the complete reviewed runtime alias set')
        if not args.approval_note or len(args.approval_note.strip()) < 30:
            raise SystemExit('Requires a descriptive record of prior explicit human approval. No original is autorun.')
        if (RUN / 'logs/human-approval.json').exists():
            raise SystemExit('Existing session approval record: preserve it; do not silently launch another session.')
        token = secrets.token_hex(4).upper()
        marker = RUN / 'drive-c' / (token + '.OK')
        abort = RUN / 'drive-c' / (token + '.NO')
        gate = RUN / 'drive-c/STARTGW.BAT'
        if any(path.exists() for path in (marker, abort, gate)):
            raise SystemExit('Unexpected preexisting gate/marker: refusing launch.')
        yield_helper = None
        yield_proof = None
        if SESSION in ('run-003','run-004','run-005','run-006'):
            from input_free_yield import CODE, verified_probe
            yield_proof = verified_probe(ROOT)
            helper = checked(RUN / 'drive-c/YIELD.COM', RUN)
            with helper.open('xb') as stream:
                stream.write(CODE)
            yield_helper = file_record(helper)
        batch = gate_batch(token, original=True, yielding=SESSION in ('run-003','run-004','run-005','run-006'))
        with checked(gate, RUN).open('xb') as stream:
            stream.write(batch)
        # Original GW stays behind the host marker. Run003 adds one read-only
        # query to prove its sole changed mouse setting in the startup log.
        startup_commands = commands[:5]
        if SESSION in ('run-003','run-004','run-005','run-006'):
            startup_commands += ['CONFIG -GET sdl "mouse_emulation"']
        if SESSION in ('run-004','run-005','run-006'):
            startup_commands += ['CONFIG -GET sdl "raw_mouse_input"']
        startup_commands += commands[5:7] + ['C:\\STARTGW.BAT']
        for dos_command in startup_commands:
            command.extend(['-c', dos_command])
        approval = {'recorded_utc': datetime.now(timezone.utc).isoformat(),
                    'human_approval_note': args.approval_note, 'command': command,
                    'runtime_config': runtime['config'], 'tool': tool['executable'],
                    'game_sha256': '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba',
                    'original_autorun': False, 'host_release_required': True,
                    'gate': file_record(gate), 'gate_token': token,
                    'release_marker': str(marker), 'abort_marker': str(abort)}
        if yield_helper:
            approval.update(yield_helper=yield_helper,verified_empty_yield_probe=yield_proof)
        approval_path = RUN / 'logs/human-approval.json'
        with approval_path.open('x', encoding='utf-8') as stream:
            json.dump(approval, stream, indent=2)
        # This branch is only used after approval, for an interactive supervised
        # session. The ordinary default below never starts any process.
        stdout_path = checked(RUN / 'logs/emulator-stdout.txt', RUN)
        stderr_path = checked(RUN / 'logs/emulator-stderr.txt', RUN)
        interactive_startup = subprocess.STARTUPINFO()
        interactive_startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        interactive_startup.wShowWindow = 1  # SW_SHOWNORMAL; human-approved visible session.
        host_start = time.monotonic()
        with stdout_path.open('xb') as out, stderr_path.open('xb') as err:
            process = subprocess.Popen(command, cwd=RUN, stdin=subprocess.DEVNULL,
                                       stdout=out, stderr=err, startupinfo=interactive_startup)
        timed_out = False
        try:
            launch_record = {'pid': process.pid, 'launched_utc': datetime.now(timezone.utc).isoformat(),
                         'executable': str(SDL2 / 'dosbox-x.exe'), 'command': command,
                         'cwd': str(RUN), 'original_autorun': False,
                         'approval_record': str(approval_path),
                         'stdout': str(stdout_path), 'stderr': str(stderr_path)}
            launch_record['windows_show_window'] = 1
            with checked(RUN / 'logs/launch-process.json', RUN).open('x', encoding='utf-8') as stream:
                json.dump(launch_record, stream, indent=2)
            gate_command = wrapper_command('gate')
            print(json.dumps({'pid': process.pid, 'original_autorun': False,
                          'host_release_required': True,
                          'review_log': str(RUN / 'logs/dosbox-x.log'),
                          'inspect_gate': gate_command,
                          'release_after_review': gate_command+' --release-approved-original',
                          'abort': gate_command+' --abort'}), flush=True)
            # Keep this parent alive; cleanup also covers receipt/print failures.
            while process.poll() is None:
                remaining = 1200 - (time.monotonic() - host_start)
                if remaining <= 0:
                    timed_out = True
                    process.kill()  # This exact Popen child/handle only.
                    break
                try:
                    process.wait(timeout=min(5, remaining))
                except subprocess.TimeoutExpired:
                    pass
            exit_code = process.wait(timeout=10)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=10)
            final = {'pid':process.pid, 'exit_code':process.returncode,
                     'ended_utc':datetime.now(timezone.utc).isoformat(),
                     'elapsed_host_seconds':time.monotonic()-host_start,
                     'host_timeout':timed_out, 'original_autorun':False}
            with checked(RUN / 'logs/final-process.json', RUN).open('x', encoding='utf-8') as stream:
                json.dump(final,stream,indent=2)
        print(json.dumps(final),flush=True)
        return
    print(json.dumps({'mode': 'dry-run-only-no-process-started', 'command': command,
                      'cwd': str(RUN), 'manual_commands_after_separate_human_approval': commands,
                      'verification': {'files': runtime['file_count'], 'tool': tool['executable'],
                                       'config': runtime['config']},
                      'security': 'No game filename/-c content/startup commands. This script cannot grant human authorization.'}, indent=2))


if __name__ == '__main__':
    main()

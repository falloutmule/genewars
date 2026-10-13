"""Inspect the running empty DOSBox gate; release/abort only with explicit flags.

Default reads only. Release follows the human-approved scope and supervisor's
log review; it creates one fresh marker, never starts a host/game process itself.
"""
import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime, timezone
import json
from pathlib import Path
import re

from prepare import ROOT, RUN, SDL2, SESSION, SESSION_WRAPPERS, HUMAN_APPROVAL, EXTRA_GAME_INPUTS, COPIED_GAME_INPUTS, runtime_alias_binding, checked, digest, prepare_run


def process_identity(pid):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD,
                                                 wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    handle = kernel.OpenProcess(0x1000, False, pid)
    if not handle:
        raise ValueError('Recorded emulator PID is not running/queryable')
    try:
        name = ctypes.create_unicode_buffer(32768)
        length = wintypes.DWORD(len(name))
        if not kernel.QueryFullProcessImageNameW(handle, 0, name, ctypes.byref(length)):
            raise ctypes.WinError(ctypes.get_last_error())
        creation, exit_time, cpu_kernel, cpu_user = (wintypes.FILETIME() for _ in range(4))
        if not kernel.GetProcessTimes(handle, ctypes.byref(creation), ctypes.byref(exit_time),
                                      ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
            raise ctypes.WinError(ctypes.get_last_error())
        ticks = (creation.dwHighDateTime << 32) | creation.dwLowDateTime
        start = datetime.fromtimestamp(ticks / 10_000_000 - 11644473600, timezone.utc)
        return Path(name.value).resolve(), start
    finally:
        kernel.CloseHandle(handle)


def runtime_issues(log, token, run, age, blocked=False, mouse_emulation=None):
    issues = []
    if age >= 1200:
        issues.append('Session time limit expired')
    if blocked:
        issues.append('Release/abort marker or prior release record exists')
    for text in ('Switched to secure mode.', 'This operation is not permitted in secure mode.',
                 'REFERENCE_GATE_WAITING ' + token):
        if text not in log:
            issues.append('Missing runtime evidence: ' + text)
    mount_rows = re.findall(r'^DOS CON: ([A-Z])\s{2,}(.+)$', log, re.MULTILINE)
    if {letter for letter,_ in mount_rows} != {'C','D','Z'}:
        issues.append('Mount table must contain exactly C,D,Z')
    if str(run / 'drive-c') not in log or str(run / 'media/GENEWARS.cue') not in log:
        issues.append('Reviewed C/D mount paths absent from log')
    if any(text in log.lower() for text in ('not a valid value','unknown option','unknown property')):
        issues.append('Runtime configuration rejected')
    if 'REFERENCE_GATE_RELEASED' in log:
        issues.append('Original gate already released')
    if mouse_emulation is not None:
        query = 'DOS CON: Z:\\>CONFIG -GET sdl "mouse_emulation"\nDOS CON: '+mouse_emulation
        if query not in log:
            issues.append('Effective mouse_emulation query absent or mismatched')
    return issues, mount_rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    action = parser.add_mutually_exclusive_group()
    action.add_argument('--release-approved-original', action='store_true')
    action.add_argument('--abort', action='store_true')
    args = parser.parse_args()
    approval = json.loads((RUN / 'logs/human-approval.json').read_text(encoding='utf-8'))
    launch = json.loads((RUN / 'logs/launch-process.json').read_text(encoding='utf-8'))
    human = json.loads(HUMAN_APPROVAL.read_text(encoding='utf-8'))
    if not human['approved'] or not approval.get('host_release_required'):
        raise SystemExit('No scoped approved host-gate session')
    if SESSION in SESSION_WRAPPERS:
        expected_config = __import__('hashlib').sha256(EXTRA_GAME_INPUTS['GAME/config.ini']).hexdigest()
        if human.get('run_id') != SESSION or human.get('generated_config_sha256') != expected_config or human.get('candidate_sha256') != '4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba':
            raise SystemExit('Approval does not bind this exact prepared run/config')
        if COPIED_GAME_INPUTS and human.get('flattened_text_sha256') != next(iter(COPIED_GAME_INPUTS.values()))['sha256']:
            raise SystemExit('Approval does not bind the exact reviewed flattened text copy')
        aliases = [dict(runtime_relative_path=destination,**row) for destination,row in COPIED_GAME_INPUTS.items()]
        if aliases and human.get('runtime_aliases_sha256') != runtime_alias_binding(aliases):
            raise SystemExit('Approval does not bind the complete reviewed runtime alias set')
    pid = int(launch['pid'])
    actual_image, started = process_identity(pid)
    if actual_image != (SDL2 / 'dosbox-x.exe').resolve():
        raise SystemExit('Recorded PID does not identify the pinned emulator')
    launch_time = datetime.fromisoformat(launch['launched_utc'])
    if abs((started - launch_time).total_seconds()) > 5:
        raise SystemExit('PID creation time does not match this launch; possible PID reuse')
    age = (datetime.now(timezone.utc) - started).total_seconds()
    marker = checked(Path(approval['release_marker']), RUN)
    abort = checked(Path(approval['abort_marker']), RUN)
    gate = checked(ROOT / approval['gate']['path'], RUN)
    if digest(gate) != approval['gate']['sha256']:
        raise SystemExit('Gate batch changed')
    if SESSION in ('run-003','run-004','run-005'):
        from input_free_yield import CODE, verified_probe
        from launch import gate_batch
        helper_info = approval.get('yield_helper')
        if not helper_info or checked(ROOT/helper_info['path'],RUN) != RUN/'drive-c/YIELD.COM':
            raise SystemExit('Reviewed input-free helper absent')
        helper_path = ROOT/helper_info['path']
        if helper_path.read_bytes()!=CODE or digest(helper_path)!=helper_info['sha256']:
            raise SystemExit('Input-free helper changed')
        if approval.get('verified_empty_yield_probe')!=verified_probe(ROOT):
            raise SystemExit('Empty yield proof changed')
        if gate.read_bytes()!=gate_batch(approval['gate_token'],original=True,yielding=True):
            raise SystemExit('Reviewed yielding gate differs')
    if args.abort:
        with abort.open('xb') as stream:
            stream.write(b'Abort waiting gate; no original requested.\r\n')
        print(json.dumps({'pid':pid,'abort_marker_created':True,
                          'limit':'Stops a waiting gate. If already released/hung, supervisor must close/terminate the verified PID.'}))
        return
    log_path = checked(RUN / 'logs/dosbox-x.log', RUN)
    log = log_path.read_text(encoding='utf-8', errors='replace') if log_path.exists() else ''
    token = approval['gate_token']
    blocked = marker.exists() or abort.exists() or (RUN / 'logs/host-release.json').exists()
    issues, mount_rows = runtime_issues(log, token, RUN, age, blocked,
        mouse_emulation='never' if SESSION=='run-005' else 'always' if SESSION in ('run-003','run-004') else None)
    if SESSION == 'run-004' and 'DOS CON: Z:\\>CONFIG -GET sdl "raw_mouse_input"\nDOS CON: true' not in log:
        issues.append('Effective raw_mouse_input=true query absent')
    if SESSION == 'run-005' and 'DOS CON: Z:\\>CONFIG -GET sdl "raw_mouse_input"\nDOS CON: false' not in log:
        issues.append('Effective raw_mouse_input=false query absent')
    early_stderr = Path(launch['stderr']).read_text(encoding='utf-8', errors='replace')
    if any(text in early_stderr.lower() for text in ('not a valid value','unknown option','unknown property')):
        issues.append('Early runtime configuration rejected')
    if any(text in (log+'\n'+early_stderr).lower() for text in ('illegal command','file not found')):
        issues.append('Runtime gate command rejected')
    report = {'pid':pid, 'image':str(actual_image), 'started_utc':started.isoformat(),
              'age_seconds':age,'ready_for_supervisor_release':not issues,
              'issues':issues,'mount_table':mount_rows,'log_path':str(log_path),
              'log_sha256':digest(log_path) if log_path.exists() else None,
              'original_execution_requested_by_this_call':False}
    if args.release_approved_original:
        if issues:
            raise SystemExit(json.dumps(report))
        runtime = prepare_run(False)  # still unchanged before the original starts
        if runtime.get('copied_game_inputs') and human.get('runtime_aliases_sha256') != runtime_alias_binding(runtime['copied_game_inputs']):
            raise SystemExit('Approval does not bind the complete reviewed runtime alias set')
        report['original_execution_requested_by_this_call'] = True
        report['released_utc'] = datetime.now(timezone.utc).isoformat()
        # Save evidence before signaling. Atomic exclusive marker creation
        # prevents an old marker or second release from silently being accepted.
        with checked(RUN / 'logs/host-release.json', RUN).open('x', encoding='utf-8') as stream:
            json.dump(report, stream, indent=2)
        with marker.open('xb') as stream:
            stream.write(('Approved single-use gate ' + token + '\r\n').encode('ascii'))
    print(json.dumps(report,indent=2))


if __name__ == '__main__':
    main()

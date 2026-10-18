"""Preserve only an exited, unreleased gate attempt; never restage game files."""
from datetime import datetime, timezone
import argparse
import json

from prepare import ROOT, RUN, checked, file_record, prepare_run
from gate import process_identity


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--attempt-name', required=True)
    parser.add_argument('--reason', required=True)
    args = parser.parse_args()
    if not args.attempt_name.replace('-', '').isalnum():
        raise SystemExit('Attempt name must contain only letters, digits and hyphens')
    launch = json.loads((RUN/'logs/launch-process.json').read_text(encoding='utf-8'))
    approval = json.loads((RUN/'logs/human-approval.json').read_text(encoding='utf-8'))
    try:
        process_identity(int(launch['pid']))
    except ValueError:
        pass
    else:
        raise SystemExit('Recorded PID still queryable; do not archive a live attempt')
    if (RUN/'logs/host-release.json').exists():
        raise SystemExit('A release record exists; this is not an empty failed launch')
    from pathlib import Path
    marker = checked(Path(approval['release_marker']), RUN)
    abort = checked(Path(approval['abort_marker']), RUN)
    if marker.exists():
        raise SystemExit('Release marker exists; preserve for supervisor investigation')
    log_path = RUN/'logs/dosbox-x.log'
    log = log_path.read_text(encoding='utf-8', errors='replace') if log_path.exists() else ''
    if 'REFERENCE_GATE_RELEASED' in log:
        raise SystemExit('Log shows gate release; cannot archive as empty')
    final_path = RUN/'logs/final-process.json'
    final = json.loads(final_path.read_text(encoding='utf-8')) if final_path.exists() else None
    if final and (final['pid'] != launch['pid'] or final['exit_code'] is None):
        raise SystemExit('Final receipt does not identify the exited launch')
    if abort.exists() and (not final or final['exit_code'] != 0 or 'REFERENCE_GATE_ABORTED' not in log):
        raise SystemExit('Abort marker requires confirmed clean gated exit')
    # An acknowledged abort is not a game-tree input. Temporarily move that
    # exact checked marker aside for the unchanged-input verifier, restoring it
    # even if verification fails. No broad tree-policy exception is introduced.
    parked = checked(RUN/'logs'/('archive-pending-'+abort.name),RUN)
    if abort.exists():
        if parked.exists():
            raise SystemExit('Pending abort preservation already exists')
        abort.rename(parked)
        try:
            prepare_run(False)
        finally:
            parked.rename(abort)
    else:
        prepare_run(False)
    archive = checked(RUN/'attempts'/args.attempt_name,RUN)
    archive.mkdir(parents=True,exist_ok=False)
    records = []
    paths = ['drive-c/STARTGW.BAT','logs/human-approval.json','logs/launch-process.json',
             'logs/emulator-stdout.txt','logs/emulator-stderr.txt','logs/dosbox-x.log',
             'logs/final-process.json','logs/ui-recovery.json','logs/ui-actions.jsonl']
    if abort.exists():
        paths.append(str(abort.relative_to(RUN)))
    if approval.get('yield_helper'):
        paths.append(str(checked(ROOT/approval['yield_helper']['path'],RUN).relative_to(RUN)))
    for rel in paths:
        source = checked(RUN/rel,RUN)
        if not source.exists():
            continue
        target = checked(archive/rel,RUN)
        target.parent.mkdir(parents=True,exist_ok=True)
        before = file_record(source)
        source.rename(target)
        after = file_record(target)
        assert before['sha256']==after['sha256'] and before['size']==after['size']
        records.append({'old_path':before['path'],**after})
    receipt = {'archived_utc':datetime.now(timezone.utc).isoformat(),
               'old_pid':launch['pid'],'original_execution':False,'host_release_existed':False,
               'reason':args.reason,
               'old_exit_code':final['exit_code'] if final else None,
               'ended_utc':final['ended_utc'] if final else None,
               'game_files_recreated':False,'files':records}
    (archive/'archive-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(receipt))


if __name__=='__main__':
    main()

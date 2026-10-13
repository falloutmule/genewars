"""Read-only post-run integrity checks; preserve receipts, never start a process."""
from datetime import datetime, timezone
import json
from pathlib import Path
import sys

from prepare import ROOT, RUN, RECORD, checked, file_record, prepare_tool, digest
sys.path.insert(0,str(ROOT/'scripts'))
from source_member import SourceReader


def main():
    logs = RUN/'logs'
    launch = json.loads((logs/'launch-process.json').read_bytes())
    final = json.loads((logs/'final-process.json').read_bytes())
    release = json.loads((logs/'host-release.json').read_bytes())
    approval = json.loads((logs/'human-approval.json').read_bytes())
    human = json.loads((ROOT/'research/variants/reference-approval.json').read_bytes())
    assert human['approved'] and release['original_execution_requested_by_this_call']
    assert launch['pid'] == final['pid'] == release['pid']
    assert final['exit_code'] == 0 and not final['host_timeout']
    log = (logs/'dosbox-x.log').read_text(encoding='utf-8',errors='replace')
    for text in ('REFERENCE_GATE_RELEASED '+approval['gate_token'],
                 'DOS/4GW Protected Mode Run-time  Version 1.97',
                 "Can't read config.ini!",'Please install GENEWARS properly to play!',
                 'REFERENCE_ORIGINAL_RETURNED'):
        assert text in log, text
    assert not checked(Path(approval['release_marker']),RUN).exists()
    assert not checked(Path(approval['abort_marker']),RUN).exists()
    before = json.loads((logs/'before-files.json').read_bytes())
    after = [file_record(checked(path,RUN)) for path in sorted((RUN/'drive-c').rglob('*')) if path.is_file()]
    old = {v['path']:v for v in before}
    now = {v['path']:v for v in after}
    changed = [dict(before=old[k],after=now[k]) for k in old.keys() & now.keys()
               if (old[k]['size'],old[k]['sha256']) != (now[k]['size'],now[k]['sha256'])]
    removed = [old[k] for k in old.keys()-now.keys()]
    added = [now[k] for k in now.keys()-old.keys()]
    game_prefix = (RUN/'drive-c/GAME').relative_to(ROOT).as_posix()+'/'
    unchanged_game = sum(k in now and (old[k]['size'],old[k]['sha256']) == (now[k]['size'],now[k]['sha256']) for k in old)
    assert len(before) == 490 and all(k.startswith(game_prefix) for k in old)
    for name in ('GW.EXE','DOS4GW.EXE'):
        key = game_prefix+name
        assert now[key]['sha256'] == old[key]['sha256']
    archived_attempts = []
    for name in ('launch-001-before-parent-wait','launch-002-ui-unavailable'):
        archive_path = RUN/'attempts'/name/'archive-receipt.json'
        saved = json.loads(archive_path.read_bytes())
        assert saved['original_execution'] is False and saved['host_release_existed'] is False
        for item in saved['files']:
            assert file_record(checked(ROOT/item['path'],RUN))['sha256'] == item['sha256']
        archived_attempts.append(file_record(archive_path))
    prepared = json.loads(RECORD.read_bytes())
    media = []
    with SourceReader() as reader:
        for entry_id,name,source in [('root/2/disc/3','GENEWARS.cue','member-003.cue'),
                                     ('root/2/disc/4','GENEWARS.img','member-004.img')]:
            expected = reader.entries[entry_id]
            for path in (RUN/'media'/name,ROOT/'private/disc'/source):
                actual = file_record(path)
                assert actual['size'] == expected['size'] and actual['sha256'] == expected['sha256']
                media.append(actual)
        archive_sha = reader.manifest['source']['sha256_before']
    tool = prepare_tool(False,False)
    assert tool['executable']['sha256'] == prepared['tool']['executable']['sha256']
    assert digest(RUN/'dosbox-x.conf') == prepared['runtime']['config']['sha256']
    ended = datetime.fromisoformat(final['ended_utc'])
    released = datetime.fromisoformat(release['released_utc'])
    upper = (ended-released).total_seconds()
    receipt = dict(schema_version=1,status='approved-original-started-stopped-at-config-check',
        checked_utc=datetime.now(timezone.utc).isoformat(),original_execution=True,
        human_approval_statement=human['human_statement'],candidate_sha256=approval['game_sha256'],
        pid=final['pid'],host_release_utc=release['released_utc'],process_end_utc=final['ended_utc'],
        process_exit_code=final['exit_code'],host_timeout=final['host_timeout'],
        process_lifetime_seconds=final['elapsed_host_seconds'],
        release_to_process_end_upper_bound_seconds=upper,
        outcome=dict(runtime='DOS/4GW 1.97',error="Can't read config.ini!",
            following_message='Please install GENEWARS properly to play!',returned_to_gate=True),
        scope_limits=['No menu or mission reached','No gameplay inputs, save or reload',
            'No framebuffer capture','Release-to-exit interval includes emulator shutdown and is not gameplay timing',
            'BIOS INT15 unknown-call warnings also present; causal role unknown',
            'Exit code is emulator code, not proof of original success'],
        original_archive_sha256_revalidated=archive_sha,
        game_files_before=len(before),unchanged_original_game_files=unchanged_game,
        changed=changed,removed=removed,added=added,media=media,
        emulator_executable=tool['executable'],config=file_record(RUN/'dosbox-x.conf'),
        private_evidence=[file_record(logs/name) for name in ('human-approval.json','host-release.json',
            'launch-process.json','final-process.json','dosbox-x.log','emulator-stdout.txt','emulator-stderr.txt','ui-actions.jsonl')],
        first_two_unreleased_attempts_preserved=True,archived_attempts=archived_attempts,
        additional_original_execution_authorized=False,
        cleanup='No deletion; complete private run tree, outputs, media and attempt logs retained. No global changes.')
    (logs/'after-files.json').write_text(json.dumps(after,indent=2)+'\n',encoding='utf-8')
    (logs/'post-run-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    safe = ROOT/'research/variants/reference-run-result.json'
    safe.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    prepared.update(status=receipt['status'],original_execution=True,
        reference_run_result=safe.relative_to(ROOT).as_posix(),
        observed_runtime_outcome=receipt['outcome'],
        additional_original_execution_authorized=False)
    prepared['human_execution_approval']['original_execution_observed_by_review_lane']=True
    RECORD.write_text(json.dumps(prepared,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status=receipt['status'],unchanged_game_files=unchanged_game,
        changed_files=len(changed),removed_files=len(removed),added_files=added,
        original_execution=True,release_to_exit_upper_bound_seconds=upper)))


if __name__=='__main__':
    main()

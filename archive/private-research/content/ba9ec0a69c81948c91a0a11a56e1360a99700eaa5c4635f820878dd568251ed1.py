"""Read-only reviewed-session integrity audit after parent-recorded process exit.

Does not assume a successful game, zero exit, configuration failure or menu.
No process is started, signaled or queried. Run001's checker stays unchanged.
"""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import prepare


def ended_receipt(launch, final, now):
    if launch['pid'] != final['pid'] or type(final.get('exit_code')) is not int:
        raise ValueError('Final receipt does not bind a completed launch')
    started = datetime.fromisoformat(launch['launched_utc'])
    ended = datetime.fromisoformat(final['ended_utc'])
    if started.tzinfo is None or ended.tzinfo is None or not started <= ended <= now:
        raise ValueError('Final process time is missing, future or before launch')
    return started,ended


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--supervisor-observations',type=Path,
        help='Optional private JSON of observed/not_observed/unknown statuses; missing observations remain unknown')
    args = parser.parse_args()
    if prepare.SESSION == 'run-001':
        prepare.select_session('run-002')
    root,run = prepare.ROOT,prepare.RUN
    logs = run/'logs'
    final_path = logs/'final-process.json'
    if not final_path.exists():
        raise SystemExit('No final-process receipt yet; do not audit a live/unconfirmed run')
    # The launcher writes this only after wait() returned. Its ended timestamp
    # must already be in the past before inspecting final runtime artifacts.
    launch = json.loads((logs/'launch-process.json').read_bytes())
    final = json.loads(final_path.read_bytes())
    now = datetime.now(timezone.utc)
    started,ended = ended_receipt(launch,final,now)
    staged = json.loads(prepare.RECORD.read_bytes())
    human = json.loads(prepare.HUMAN_APPROVAL.read_bytes())
    before_path = logs/'before-files.json'
    before = json.loads(before_path.read_bytes())
    expected_count = 490+len(staged['runtime']['generated_game_inputs'])+len(staged['runtime'].get('copied_game_inputs',[]))
    assert len(before) == expected_count and len({v['path'] for v in before}) == expected_count
    original = [v for v in before if 'entry_id' in v]
    copied = [v for v in before if 'source_entry_id' in v]
    generated = [v for v in before if 'entry_id' not in v and 'source_entry_id' not in v]
    assert len(original) == 490 and len(generated) == 1
    assert generated[0]['sha256'] == human['generated_config_sha256']
    manifest = json.loads((root/'research/game-files-manifest.json').read_bytes())
    entries = {v['entry_id']:v for v in manifest['entries']}
    for item in original:
        expected = entries[item['entry_id']]
        assert (item['size'],item['sha256']) == (expected['size'],expected['sha256'])
    for item in copied:
        expected = entries[item['source_entry_id']]
        assert (item['size'],item['sha256']) == (expected['size'],expected['sha256'])
    if copied:
        assert prepare.runtime_alias_binding(copied) == human['runtime_aliases_sha256']
        text = next(item for item in copied if item['path'].endswith('/GAME/DATA/TEXT.DAT'))
        assert text['sha256'] == human['flattened_text_sha256']
    after = [prepare.file_record(prepare.checked(path,run)) for path in sorted((run/'drive-c').rglob('*')) if path.is_file()]
    old,new = {v['path']:v for v in before},{v['path']:v for v in after}
    same = lambda key: key in new and (old[key]['size'],old[key]['sha256']) == (new[key]['size'],new[key]['sha256'])
    changed = [dict(before=old[k],after=new[k]) for k in sorted(old.keys()&new.keys()) if not same(k)]
    removed = [old[k] for k in sorted(old.keys()-new.keys())]
    added = [new[k] for k in sorted(new.keys()-old.keys())]
    def immutable(path,expected_sha,expected_size=None):
        actual = prepare.file_record(path) if path.exists() else None
        passed = bool(actual and actual['sha256']==expected_sha and (expected_size is None or actual['size']==expected_size))
        return dict(expected_sha256=expected_sha,expected_size=expected_size,actual=actual,passed=passed)
    media = []
    for entry_id,name,source in [('root/2/disc/3','GENEWARS.cue','member-003.cue'),('root/2/disc/4','GENEWARS.img','member-004.img')]:
        expected = entries[entry_id]
        for path in (run/'media'/name,root/'private/disc'/source):
            media.append(immutable(path,expected['sha256'],expected['size']))
    source_path = Path(manifest['source']['path'])
    archive_actual = prepare.digest(source_path) if source_path.exists() else None
    archive_check = dict(expected_sha256=manifest['source']['sha256_before'],actual_sha256=archive_actual,
                         passed=archive_actual==manifest['source']['sha256_before'])
    try:
        verified_tool = prepare.prepare_tool(False,False)
        package_check = dict(passed=True,verified_members=verified_tool['members_verified'])
    except (ValueError,OSError) as error:
        package_check = dict(passed=False,error=str(error))
    tool_expected = staged['tool']['executable']
    tool_check = immutable(root/tool_expected['path'],tool_expected['sha256'],tool_expected['size'])
    config_expected = staged['runtime']['config']
    config_check = immutable(run/'dosbox-x.conf',config_expected['sha256'],config_expected['size'])
    release_path = logs/'host-release.json'
    release = json.loads(release_path.read_bytes()) if release_path.exists() else None
    interval = None
    if release:
        assert release['pid'] == final['pid'] and release['original_execution_requested_by_this_call']
        released = datetime.fromisoformat(release['released_utc'])
        assert started <= released <= ended
        interval = (ended-released).total_seconds()
    log_path = logs/'dosbox-x.log'
    text = log_path.read_text(encoding='utf-8',errors='replace') if log_path.exists() else ''
    runtime_seen = 'DOS/4GW Protected Mode Run-time' in text
    returned = 'REFERENCE_ORIGINAL_RETURNED' in text
    errors = [message for message in ("Can't read config.ini!",'Please install GENEWARS properly to play!') if message in text]
    requested = release is not None
    observed = True if runtime_seen else (None if requested else False)
    observation_fields = ('menu','mission','gameplay_input','save_reload','framebuffer_capture','timing_sample')
    observations = {key:'unknown' for key in observation_fields}
    observation_source = None
    if args.supervisor_observations:
        path = prepare.checked(args.supervisor_observations.resolve(),run)
        supplied = json.loads(path.read_bytes())
        for key in observation_fields:
            value = supplied.get(key,'unknown')
            assert value in ('observed','not_observed','unknown'),(key,value)
            observations[key] = value
        observation_source = prepare.file_record(path)
    status = 'completed-runtime-observed' if runtime_seen else ('completed-original-attempt-unconfirmed' if requested else 'completed-unreleased')
    # Hash actual logs and captures regardless of extension; a screenshot's
    # existence alone does not establish what screen or gameplay it depicts.
    source_paths = [path for path in logs.rglob('*') if path.is_file()
        and 'post-run-receipts' not in path.relative_to(logs).parts
        and path.name != 'after-files.json']
    capture_dir = run/'captures'
    if capture_dir.exists():
        source_paths += [path for path in capture_dir.rglob('*') if path.is_file()]
    sources = [prepare.file_record(prepare.checked(path,run)) for path in sorted(set(source_paths))]
    sources += [prepare.file_record(prepare.HUMAN_APPROVAL),prepare.file_record(Path(__file__))]
    checks_passed = archive_check['passed'] and package_check['passed'] and tool_check['passed'] and config_check['passed'] and all(v['passed'] for v in media)
    result = dict(schema_version=1,status=status,checked_utc=now.isoformat(),session=prepare.SESSION,
        original_execution=observed,original_execution_requested=requested,dos4gw_runtime_observed=runtime_seen,
        original_returned_marker_observed=returned,known_error_messages=errors,
        pid=final['pid'],process_exit_code=final['exit_code'],host_timeout=final.get('host_timeout'),
        process_start_utc=started.isoformat(),process_end_utc=ended.isoformat(),
        process_lifetime_seconds=final.get('elapsed_host_seconds'),
        release_to_process_end_upper_bound_seconds=interval,
        observations=observations,private_supervisor_observations=observation_source,
        before_files=prepare.file_record(before_path),before_game_tree_files=len(before),
        original_source_files=490,unchanged_original_source_files=sum(same(v['path']) for v in original),
        copied_runtime_inputs=len(copied),unchanged_copied_runtime_inputs=sum(same(v['path']) for v in copied),
        generated_config_unchanged=same(generated[0]['path']),
        original_executables_unchanged=all(same((run/'drive-c/GAME'/name).relative_to(root).as_posix()) for name in ('GW.EXE','DOS4GW.EXE')),
        changed=changed,removed=removed,added=added,media_checks=media,archive_check=archive_check,
        tool_package_check=package_check,emulator_executable_check=tool_check,emulator_config_check=config_check,
        immutable_input_checks_passed=checks_passed,private_evidence=sources,
        checker_grants_execution_authorization=False,
        limits=['Emulator exit status does not establish game success',
                'Release-to-process-exit includes shutdown and is not original gameplay timing',
                'Missing optional observations remain unknown; no inference of menu/mission/capture from exit code',
                'No original, emulator or GUI launched by this checker'],
        cleanup='No deletion or restaging; retain this session, earlier sessions and all private outputs.')
    prepare.ensure_bytes(logs/'after-files.json',(json.dumps(after,indent=2)+'\n').encode('utf-8'),run)
    history = prepare.checked(logs/'post-run-receipts',run)
    history.mkdir(exist_ok=True)
    receipt_path = history/(now.strftime('%Y%m%dT%H%M%S%fZ')+'-pid'+str(final['pid'])+'.json')
    with receipt_path.open('x',encoding='utf-8') as stream:
        json.dump(result,stream,indent=2)
    target = root/'research/variants'/('reference-'+prepare.SESSION+'-result.json')
    target.write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    # Permission comes from the human's recorded scope, not an integrity audit.
    # Do not negate later human approval when refreshing this aggregate state.
    staged.pop('additional_original_execution_authorized',None)
    staged.update(status=status,original_execution=observed,
                  reference_run_result=target.relative_to(root).as_posix(),checker_grants_execution_authorization=False)
    prepare.RECORD.write_text(json.dumps(staged,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status=status,original_execution=observed,exit_code=final['exit_code'],
        unchanged_original_files=result['unchanged_original_source_files'],changed_files=len(changed),
        added_files=len(added),removed_files=len(removed),immutable_input_checks_passed=checks_passed)))


if __name__=='__main__': main()

"""Verify run002 preparation and approval refusal with process creation mocked."""
from datetime import datetime, timezone
import contextlib
import io
import json
from pathlib import Path
import sys
from unittest.mock import patch
import prepare


def main():
    prepare.select_session('run-002')
    import launch
    root = prepare.ROOT
    ready = json.loads(prepare.RECORD.read_bytes())
    tool = prepare.prepare_tool(False,False)
    runtime = prepare.prepare_run(False)
    human = json.loads(prepare.HUMAN_APPROVAL.read_bytes())
    assert human['approved'] is False and ready['original_execution'] is False
    assert runtime['file_count'] == 490 and len(runtime['generated_game_inputs']) == 1
    assert runtime['generated_game_inputs'][0]['sha256'] == human['generated_config_sha256']
    config = (prepare.RUN/'drive-c/GAME/config.ini').read_bytes()
    assert len(config) == 126 and len(config.splitlines()) == 8 and config.endswith(b'\r\n')
    assert not (prepare.RUN/'drive-c/GAME/sound.ini').exists()
    assert not (prepare.RUN/'drive-c/STARTGW.BAT').exists()
    fixture = root/'private/review/run002-approval-guards'
    fixture.mkdir(parents=True,exist_ok=True)
    cases = []
    variants = [('pending',{}),('wrong-run',dict(approved=True,run_id='run-other',human_statement='test only')),
        ('wrong-config',dict(approved=True,generated_config_sha256='0'*64,human_statement='test only')),
        ('wrong-candidate',dict(approved=True,candidate_sha256='0'*64,human_statement='test only')),
        ('missing-human-statement',dict(approved=True))]
    for name,changed in variants:
        data = dict(human,**changed)
        fake = fixture/(name+'.json')
        fake.write_text(json.dumps(data),encoding='utf-8')
        # Every process-creation path is intercepted, and even incidental gate
        # writes would be confined to an empty fixture instead of the real run.
        with patch.object(launch,'HUMAN_APPROVAL',fake),patch.object(launch,'RUN',fixture),\
             patch.object(launch,'prepare_tool',return_value=tool),\
             patch.object(launch,'prepare_run',return_value=runtime),\
             patch.object(launch.subprocess,'Popen',side_effect=AssertionError('No process may start in guard validation')) as popen,\
             patch.object(sys,'argv',['launch.py','--open-approved-session','--approval-note','Fixture guard check only; no execution authorized']):
            try:
                launch.main()
            except SystemExit as error:
                reason = str(error)
            else:
                raise AssertionError('Invalid approval was accepted')
            assert not popen.called
            assert reason.startswith(('This session has no separate','Approval does not bind'))
            cases.append(dict(case=name,rejected=True,process_creation_called=False,reason=reason))
    dry = io.StringIO()
    with patch.object(launch,'prepare_tool',return_value=tool),patch.object(launch,'prepare_run',return_value=runtime),\
         patch.object(launch.subprocess,'Popen',side_effect=AssertionError('Dry run must never start a process')) as popen,\
         patch.object(sys,'argv',['launch.py']),contextlib.redirect_stdout(dry):
        launch.main()
        assert not popen.called
    command = json.loads(dry.getvalue())
    assert command['mode'] == 'dry-run-only-no-process-started'
    assert '-c' not in command['command']
    scripts = [root/'scripts/reference'/name for name in ('prepare.py','launch.py','gate.py','run002.py','validate_run002.py','archive_failed_launch.py')]
    receipt = dict(status='prepared-verified-no-execution',checked_utc=datetime.now(timezone.utc).isoformat(),
        original_execution=False,emulator_execution=False,approved=False,
        source_game_files=runtime['file_count'],source_game_bytes=runtime['game_bytes'],
        generated_config=runtime['generated_game_inputs'][0],media=runtime['media'],
        emulator_config=runtime['config'],emulator_tool=tool['executable'],
        static_config_evidence=prepare.file_record(root/'research/ghidra/config-evidence.json'),
        preparation_receipt=prepare.file_record(prepare.RECORD),
        before_files=prepare.file_record(prepare.RUN/'logs/before-files.json'),
        approval_refusal_cases=cases,dry_run=command,scripts=[prepare.file_record(p) for p in scripts],
        approval_required='New explicit human approval of run-002 exact candidate/config and bounded scope; run-001 approval is consumed.')
    path = root/'research/variants/reference-run-002-validation.json'
    path.write_text(json.dumps(receipt,indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(status=receipt['status'],approval_refusal_cases=len(cases),original_execution=False,emulator_execution=False)))


if __name__=='__main__': main()

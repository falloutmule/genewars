"""Synthetic fixtures for run002 finish auditing. No actual session is read/changed."""
from contextlib import redirect_stdout
from datetime import datetime, timezone, timedelta
import io
import json
from pathlib import Path
import sys
from unittest.mock import patch
import finish002
import prepare

ROOT=prepare.ROOT

def write(path,value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(value) if isinstance(value,(dict,list)) else value)

def main():
    now=datetime.now(timezone.utc)
    fixture=ROOT/'private/review/finish002'/now.strftime('%Y%m%dT%H%M%S%fZ')
    checks=[]
    for selected,case,release,runtime,code in [('run-002','unreleased',False,False,0),('run-002','observed',True,True,9),('run-002','unconfirmed',True,False,-9),('run-003','copied-runtime',True,True,15),('run-003','changed-helper',True,True,15)]:
        root=fixture/case;run=root/'private/reference-run'/selected;logs=run/'logs'
        record_path=root/'research/variants'/('reference-'+selected+'-readiness.json')
        approval=root/'research/variants'/('reference-'+selected+'-approval.json')
        with patch.multiple(prepare,ROOT=root,RUN=run,RECORD=record_path,HUMAN_APPROVAL=approval,SESSION=selected),\
             patch.object(prepare,'select_session',lambda name:None),\
             patch.object(prepare,'prepare_tool',lambda *args:dict(members_verified=311)),\
             patch('subprocess.Popen',side_effect=AssertionError('No process launch allowed')):
            rows=[];entries=[]
            for i in range(490):
                path=run/'drive-c/GAME'/(['GW.EXE','DOS4GW.EXE'][i] if i<2 else f'FIXTURE{i:03}.TXT')
                write(path,'Synthetic fixture only '+str(i))
                item=prepare.file_record(path);item['entry_id']=f'fixture/{i}';rows.append(item)
                entries.append(dict(entry_id=item['entry_id'],size=item['size'],sha256=item['sha256']))
            config=run/'drive-c/GAME/config.ini';write(config,'Synthetic configuration only')
            generated=prepare.file_record(config);rows.append(generated);copied_inputs=[]
            human=dict(generated_config_sha256=generated['sha256'])
            if selected=='run-003':
                text=run/'drive-c/GAME/DATA/TEXT.DAT';write(text,'Synthetic fixture only 3')
                copy=dict(source_entry_id='fixture/3',**prepare.file_record(text));copied_inputs.append(copy);rows.append(copy)
                human['flattened_text_sha256']=copy['sha256']
                for n in range(18):
                    speech=run/'drive-c/GAME/SPEECH'/f'FIXTURE{n:02}.SPE';write(speech,'Synthetic fixture only '+str(n+4))
                    alias=dict(source_entry_id=f'fixture/{n+4}',**prepare.file_record(speech));copied_inputs.append(alias);rows.append(alias)
                human['runtime_aliases_sha256']=prepare.runtime_alias_binding(copied_inputs)
            write(logs/'before-files.json',rows)
            if selected=='run-003':
                helper=run/'drive-c/YIELD.COM';write(helper,'Synthetic helper; never executed')
                write(logs/'human-approval.json',dict(yield_helper=prepare.file_record(helper)))
                if case=='changed-helper':write(helper,'Changed helper bytes in private fixture')
            write(approval,human)
            approval_before=approval.read_bytes()
            for identifier,name,source in [('root/2/disc/3','GENEWARS.cue','member-003.cue'),('root/2/disc/4','GENEWARS.img','member-004.img')]:
                copied=run/'media'/name;original=root/'private/disc'/source
                write(copied,'Synthetic media '+name);write(original,copied.read_text())
                item=prepare.file_record(copied);entries.append(dict(entry_id=identifier,size=item['size'],sha256=item['sha256']))
            archive=root/'private/synthetic.zip';write(archive,'Not a real archive; synthetic hash only')
            write(root/'research/game-files-manifest.json',dict(entries=entries,source=dict(path=str(archive),sha256_before=prepare.digest(archive))))
            exe=root/'private/tools/synthetic.exe';write(exe,'Never executed')
            conf=run/'dosbox-x.conf';write(conf,'Synthetic emulator config')
            write(record_path,dict(tool=dict(executable=prepare.file_record(exe)),runtime=dict(config=prepare.file_record(conf),generated_game_inputs=[generated],copied_game_inputs=copied_inputs)))
            start=now-timedelta(seconds=20);ended=now-timedelta(seconds=1)
            write(logs/'launch-process.json',dict(pid=123,launched_utc=start.isoformat()))
            write(logs/'final-process.json',dict(pid=123,ended_utc=ended.isoformat(),exit_code=code,host_timeout=code==-9,elapsed_host_seconds=19))
            if release:write(logs/'host-release.json',dict(pid=123,released_utc=(now-timedelta(seconds=10)).isoformat(),original_execution_requested_by_this_call=True))
            write(logs/'dosbox-x.log','DOS/4GW Protected Mode Run-time' if runtime else 'No original output in fixture')
            write(logs/'ui-actions.jsonl','{"synthetic":true}\n')
            write(run/'captures/fixture.png','Not pixels; synthetic inventory item')
            if runtime:
                write(run/'drive-c/GAME/FIXTURE002.TXT','Changed by fixture')
                (run/'drive-c/GAME/FIXTURE003.TXT').unlink()
                write(run/'drive-c/GAME/ADDED.TXT','Added by fixture')
            checker=root/'scripts/reference/finish002.py';write(checker,Path(finish002.__file__).read_text())
            with patch.object(finish002,'__file__',str(checker)),patch.object(sys,'argv',['finish002.py']),redirect_stdout(io.StringIO()):finish002.main()
            result=json.loads((root/'research/variants'/('reference-'+selected+'-result.json')).read_bytes())
            assert result['process_exit_code']==code
            assert result['original_execution']==(True if runtime else None if release else False)
            assert result['immutable_input_checks_passed']==(case!='changed-helper')
            if selected=='run-003':
                assert result['independently_generated_yield_helper_check']['passed']==(case!='changed-helper')
            assert result['before_game_tree_files']==491+19*int(selected=='run-003')
            assert result['unchanged_copied_runtime_inputs']==19*int(selected=='run-003')
            assert result['checker_grants_execution_authorization'] is False
            assert 'new_original_execution_authorized' not in result
            assert 'additional_original_execution_authorized' not in json.loads(record_path.read_bytes())
            assert approval.read_bytes()==approval_before
            assert result['observations']=={key:'unknown' for key in result['observations']}
            assert len(result['changed'])==int(runtime) and len(result['removed'])==int(runtime) and len(result['added'])==int(runtime)+int(selected=='run-003')
            assert any(v['path'].endswith('captures/fixture.png') for v in result['private_evidence'])
            # Missing final receipt must refuse before any result can be rewritten.
            (logs/'final-process.json').unlink()
            try:
                with patch.object(sys,'argv',['finish002.py']):finish002.main()
            except SystemExit:pass
            else:raise AssertionError('Live/unconfirmed audit accepted')
            checks.append(dict(session=selected,case=case,exit_code=code,passed=True))
    result=dict(status='passed',checked_utc=now.isoformat(),synthetic_cases=checks,
        assertions=['No process creation','491/510 baseline files with19 separately bound aliases','Actual added/changed/removed counts',
            'Nonzero/negative exit retained','Released but unobserved runtime stays unknown',
            'Capture hash does not infer screen content','Absent observations stay unknown','Missing final receipt rejected',
            'Checker grants no permission and does not negate human authorization','Human approval unchanged',
            'Independent helper addition separately inventoried; changed helper fails immutable checks'],
        actual_reference_run_read_or_changed=False)
    (ROOT/'research/variants/reference-finish002-validation.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result))

if __name__=='__main__':main()

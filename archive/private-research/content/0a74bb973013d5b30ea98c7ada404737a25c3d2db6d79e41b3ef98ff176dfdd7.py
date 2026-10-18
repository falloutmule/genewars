"""Read-only staged run003 checks; mock every emulator process creation path."""
from datetime import datetime,timezone
import contextlib,io,json
from pathlib import Path
import sys
from unittest.mock import patch
import prepare

def config(path):
    values={};section=None
    for raw in path.read_text().splitlines():
        line=raw.strip()
        if not line or line.startswith('#'):continue
        if line.startswith('['):section=line[1:-1]
        else:
            key,value=line.split('=',1);values[section,key.strip()]=value.strip()
    return values

def main(session='run-003'):
    assert session in ('run-003','run-004','run-005','run-006')
    prepare.select_session(session)
    import launch,gate
    root,run=prepare.ROOT,prepare.RUN
    tool=prepare.prepare_tool(False,False);runtime=prepare.prepare_run(False)
    human=json.loads(prepare.HUMAN_APPROVAL.read_bytes())
    approval_bytes=prepare.HUMAN_APPROVAL.read_bytes()
    assert runtime['file_count']==490 and len(runtime['generated_game_inputs'])==1 and len(runtime['copied_game_inputs'])==19
    assert len(json.loads((run/'logs/before-files.json').read_bytes()))==510
    assert prepare.runtime_alias_binding(runtime['copied_game_inputs'])==human['runtime_aliases_sha256']
    assert not (run/'drive-c/STARTGW.BAT').exists() and not (run/'drive-c/GAME/sound.ini').exists()
    old=config(root/'private/reference-run/run-002/dosbox-x.conf');new=config(run/'dosbox-x.conf')
    delta={key:(old.get(key),new.get(key)) for key in old.keys()|new.keys() if old.get(key)!=new.get(key)}
    expected={('sdl','mouse_emulation'):('locked','always')}
    if session=='run-004':expected['sdl','raw_mouse_input']=(None,'true')
    if session=='run-005':
        expected={('sdl','mouse_emulation'):('locked','never'),('sdl','raw_mouse_input'):(None,'false')}
    if session=='run-006':
        expected={('sdl','mouse_emulation'):('locked','always'),('sdl','raw_mouse_input'):(None,'false')}
    assert delta==expected
    assert new['cpu','integration device']=='false' and new['cpu','integration device pnp']=='false'
    fixture=root/'private/review'/(session+'-guards');fixture.mkdir(parents=True,exist_ok=True)
    invalid=[('pending',dict(approved=False,human_statement=None)),('wrong-run',dict(approved=True,run_id='run-002',human_statement='fixture only')),
        ('wrong-config',dict(approved=True,generated_config_sha256='0'*64,human_statement='fixture only')),
        ('wrong-candidate',dict(approved=True,candidate_sha256='0'*64,human_statement='fixture only')),
        ('missing-human',dict(approved=True,human_statement=None)),('wrong-text',dict(approved=True,flattened_text_sha256='0'*64,human_statement='fixture only')),
        ('wrong-aliases',dict(approved=True,runtime_aliases_sha256='0'*64,human_statement='fixture only'))]
    cases=[]
    for name,overrides in invalid:
        fake=fixture/(name+'.json');fake.write_text(json.dumps(dict(human,**overrides)))
        with patch.object(launch,'HUMAN_APPROVAL',fake),patch.object(launch,'RUN',fixture),\
             patch.object(launch,'prepare_tool',return_value=tool),patch.object(launch,'prepare_run',return_value=runtime),\
             patch.object(launch.subprocess,'Popen',side_effect=AssertionError('No process creation')) as process,\
             patch.object(sys,'argv',['launch.py','--open-approved-session','--approval-note','Synthetic refusal fixture; no original execution allowed']):
            try:launch.main()
            except SystemExit as error:reason=str(error)
            else:raise AssertionError('Invalid approval accepted')
            assert not process.called
            cases.append(dict(case=name,rejected=True,reason=reason))
    output=io.StringIO()
    with patch.object(launch,'prepare_tool',return_value=tool),patch.object(launch,'prepare_run',return_value=runtime),\
         patch.object(launch.subprocess,'Popen',side_effect=AssertionError('No dry-run process')) as process,\
         patch.object(sys,'argv',['launch.py']),contextlib.redirect_stdout(output):launch.main()
    dry=json.loads(output.getvalue());assert '-c' not in dry['command'] and not process.called
    token='FIXTUREA'
    mouse='never' if session=='run-005' else 'always'
    log='\n'.join(['DOS CON: C      Local '+str(run/'drive-c'),'DOS CON: D      ISO '+str(run/'media/GENEWARS.cue'),
        'DOS CON: Z      Internal Virtual Drive','Switched to secure mode.','This operation is not permitted in secure mode.',
        'REFERENCE_GATE_WAITING '+token,'DOS CON: Z:\\>CONFIG -GET sdl "mouse_emulation"','DOS CON: '+mouse])
    assert not gate.runtime_issues(log,token,run,60,mouse_emulation=mouse)[0]
    assert gate.runtime_issues(log.replace('DOS CON: '+mouse,'DOS CON: locked'),token,run,60,mouse_emulation=mouse)[0]
    assert prepare.HUMAN_APPROVAL.read_bytes()==approval_bytes
    scripts=[prepare.file_record(root/'scripts/reference'/name) for name in ('prepare.py','session.py','run002.py','run003.py','launch.py','gate.py','finish002.py','validate_run003.py')]
    result=dict(status='prepared-verified-no-execution',checked_utc=datetime.now(timezone.utc).isoformat(),original_execution=False,emulator_execution=False,
        original_files=490,generated_files=1,copied_runtime_aliases=19,total_before_files=510,
        runtime_aliases_sha256=human['runtime_aliases_sha256'],config_delta=[dict(section=k[0],key=k[1],before=v[0],after=v[1]) for k,v in delta.items()],
        approval_refusal_cases=cases,mouse_query_gate_fixture='accepted-'+mouse+'/rejected-locked',dry_run=dry,scripts=scripts,
        before_files=prepare.file_record(run/'logs/before-files.json'),text_install_evidence=prepare.file_record(root/prepare.TEXT_INSTALL_PROOF_PATH),
        preparation_receipt=prepare.file_record(prepare.RECORD),approval_unchanged=True)
    (root/'research/variants'/('reference-'+session+'-validation.json')).write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(dict(status=result['status'],before_files=510,aliases=19,invalid_approvals_rejected=len(cases),original_or_gui_execution=False)))

if __name__=='__main__':main()

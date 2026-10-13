"""Independent closed-session receipt/current-tree audit. Never launches anything."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json
ROOT=Path(__file__).resolve().parents[2]
def pin(p):
 h=hashlib.sha256()
 with p.open('rb')as s:
  for b in iter(lambda:s.read(1024*1024),b''):h.update(b)
 return dict(path=p.relative_to(ROOT).as_posix(),size=p.stat().st_size,sha256=h.hexdigest())
def verify(item):
 actual=pin(ROOT/item['path']);assert actual==item,(actual,item);return actual
def main():
 resultpath=ROOT/'research/variants/reference-run-006-result.json';result=json.loads(resultpath.read_bytes());session=ROOT/'private/reference-run/run-006';assert result['session']=='run-006'and result['original_execution']is True
 for entry in result['private_evidence']:verify(entry)
 before=json.loads((ROOT/result['before_files']['path']).read_bytes());assert len(before)==510
 # One independent walk of C: distinguishes original/generated/alias files
 # from gate helpers placed outside the game directory after the baseline.
 after={p.relative_to(ROOT).as_posix():pin(p)for p in(session/'drive-c').rglob('*')if p.is_file()};changed=[];removed=[];unchanged=[]
 for old in before:
  actual=after.pop(old['path'],None)
  if actual is None:removed.append(old['path'])
  elif actual['sha256']!=old['sha256']or actual['size']!=old['size']:changed.append(dict(before=old,after=actual))
  else:unchanged.append(old)
 assert changed==result['changed']and removed==result['removed'];assert sorted(after.values(),key=lambda p:p['path'])==sorted(result['added'],key=lambda p:p['path'])
 assert len(changed)==1 and changed[0]['before']['path'].endswith('/GAME/GW.ERR')and changed[0]['after']['size']==0
 assert sum('entry_id'in p for p in unchanged)==489;assert not removed and len(after)==3
 for check in result['media_checks']+[result['emulator_executable_check'],result['emulator_config_check'],result['independently_generated_yield_helper_check']]:
  assert check['passed']is True;actual=verify(check['actual']);assert actual['sha256']==check['expected_sha256']and actual['size']==check['expected_size']
 final=json.loads((session/'logs/final-process.json').read_bytes());release=json.loads((session/'logs/host-release.json').read_bytes());stop=json.loads((session/'logs/operator-stop.json').read_bytes());observations=json.loads((session/'logs/supervisor-observations.json').read_bytes())
 assert final['pid']==release['pid']==stop['pid']==result['pid']==36216;assert final['exit_code']==result['process_exit_code']==4294967295 and final['host_timeout']is False and result['host_timeout']is False;assert abs(final['elapsed_host_seconds']-result['process_lifetime_seconds'])<0.001
 ended=datetime.fromisoformat(final['ended_utc']);released=datetime.fromisoformat(release['released_utc']);checked=datetime.fromisoformat(result['checked_utc']);assert released<ended<checked
 assert release['capture_state_supervisor_attested']is True and release['ready_for_supervisor_release']is True and not release['issues'];assert [r[0]for r in release['mount_table']]==['C','D','Z']
 assert observations['menu']=='observed'and observations['mission']=='not_observed'and observations['framebuffer_capture']=='not_observed';assert result['observations']['menu']=='observed'and result['observations']['mission']=='not_observed';assert result['checker_grants_execution_authorization']is False
 report=dict(status='passed-independent-closed-run006-audit',utc=datetime.now(timezone.utc).isoformat(),checker_original_execution=False,checker_gui_execution=False,result=pin(resultpath),final_process=pin(session/'logs/final-process.json'),observations=pin(session/'logs/supervisor-observations.json'),baseline_files=len(before),unchanged_original_files=489,changed_original_files=1,added_host_helper_files=len(after),removed_files=0,process_exit_code=final['exit_code'],operator_stop_recorded=True,host_timeout=False,menu='observed',mission='not_observed',raw_framebuffer='not_observed',media_and_emulator_pins_rehashed=True,limits=['Observed menu/click chronology is supervisor evidence, not independent GUI observation by this lane','Original exit status is operator termination, not game failure/success','No guest mouse register trace or wholegame delivery guarantee from emptyprobe012','Archive/tool311-member checks preserved from finish receipt; not independently rescanned by this audit','No original/emulator/GUI launched, no raw checker receipt modified'])
 out=Path(__file__).with_suffix('.json');tmp=out.with_suffix('.json.tmp');tmp.write_text(json.dumps(report,indent=2)+'\n');tmp.replace(out);print(json.dumps(dict(status=report['status'],unchanged_original_files=489,changed_original_files=1)))
if __name__=='__main__':main()

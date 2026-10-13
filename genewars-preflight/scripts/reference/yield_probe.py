"""Prepare/inspect an EMPTY yielding DOSBox gate. Root alone launches its UI.

Default prepares and prints the command without creating a process. The only
mounted host directory holds this generated batch and fresh marker files.
No game executable/resource/disc is mounted or referenced by the DOS batch.
"""
from datetime import datetime,timezone
import argparse,json,re,secrets,subprocess,time
from pathlib import Path
from prepare import ROOT,SDL2,checked,digest,file_record,prepare_tool
from launch import arguments
from gate import process_identity

PROBE=ROOT/'private/reference-run/yield-gate-probe-001'
CONFIG_SHA='a3275d1cf47a7305764698352de4ed6f03fef2ecea192d7f4c45fdd7cb17107b'
def batch(token):
 return ('@ECHO OFF\r\nECHO REFERENCE_GATE_WAITING '+token+'\r\n:WAIT\r\n'
  'IF EXIST C:\\'+token+'.NO GOTO ABORT\r\nIF EXIST C:\\'+token+'.OK GOTO SENTINEL\r\n'
  'Z:\\DOS\\CHOICE.COM /C:Y /N /T:Y,1 > NUL\r\nGOTO WAIT\r\n:SENTINEL\r\n'
  'DEL C:\\'+token+'.OK\r\nECHO REFERENCE_GATE_RELEASED '+token+'\r\n'
  'ECHO REFERENCE_EMPTY_SENTINEL\r\nGOTO END\r\n:ABORT\r\nECHO REFERENCE_GATE_ABORTED\r\n:END\r\nEXIT\r\n').encode('ascii')
def command_for(config):
 command=arguments(config,120)
 for cmd in [f'MOUNT C "{PROBE/"drive-c"}" -nocachedir','MOUNT','CONFIG -SECUREMODE','MOUNT E "."','CONFIG -GET sdl "mouse_emulation"','C:','C:\\YIELDGAT.BAT']:command+=['-c',cmd]
 return command
def prepare():
 tool=prepare_tool(False,False)
 for name in ('drive-c','logs','captures','states'):checked(PROBE/name,ROOT/'private/reference-run').mkdir(parents=True,exist_ok=True)
 saved=PROBE/'logs/prepared.json';config=PROBE/'dosbox-x.conf';gate=PROBE/'drive-c/YIELDGAT.BAT'
 if saved.exists():
  receipt=json.loads(saved.read_bytes());token=receipt['token']
 else:
  token=secrets.token_hex(4).upper();source=ROOT/'private/reference-run/run-003/dosbox-x.conf'
  assert digest(source)==CONFIG_SHA
  with config.open('xb') as f:f.write(source.read_bytes())
  with gate.open('xb') as f:f.write(batch(token))
  command=command_for(config)
  receipt=dict(prepared_utc=datetime.now(timezone.utc).isoformat(),token=token,original_execution=False,
   command=command,cwd=str(PROBE),tool=tool['executable'],config=file_record(config),gate=file_record(gate),
   host_watchdog_seconds=120,scope='Empty internal CHOICE yielding gate; generated batch and marker directory only, no game or disc mount')
  saved.write_text(json.dumps(receipt,indent=2)+'\n')
 assert re.fullmatch('[0-9A-F]{8}',token),'Invalid prepared token'
 assert digest(config)==CONFIG_SHA and gate.read_bytes()==batch(token)
 assert receipt['command']==command_for(config) and receipt['cwd']==str(PROBE),'Prepared command changed'
 assert receipt['tool']['sha256']==tool['executable']['sha256']
 allowed={'YIELDGAT.BAT',token+'.OK',token+'.NO'}
 assert all(p.is_file() and p.name in allowed for p in (PROBE/'drive-c').iterdir()),'Unexpected probe input'
 return receipt
def issues(log,token,age):
 found=[]
 for term in ['Switched to secure mode.','This operation is not permitted in secure mode.','REFERENCE_GATE_WAITING '+token,
  'DOS CON: Z:\\>CONFIG -GET sdl "mouse_emulation"\nDOS CON: always']:
  if term not in log:found.append('Missing evidence: '+term)
 rows=re.findall(r'^DOS CON: ([A-Z])\s{2,}(.+)$',log,re.MULTILINE)
 if {letter for letter,_ in rows}!={'C','Z'}:found.append('Mount table must contain exactly C and Z')
 if str(PROBE/'drive-c') not in log:found.append('Probe C path absent')
 if age>=120:found.append('Watchdog expired')
 if 'REFERENCE_GATE_RELEASED' in log:found.append('Already released')
 if any(x in log.lower() for x in ('not a valid value','unknown option','unknown property','illegal command','file not found')):found.append('Runtime command/config rejection')
 return found,rows
def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('action',nargs='?',default='prepare',choices=['prepare','launch','inspect','release','abort','finish'])
 p.add_argument('--open-probe',action='store_true');p.add_argument('--observations',type=Path);a=p.parse_args();r=prepare();token=r['token']
 marker=PROBE/'drive-c'/(token+'.OK');abort=PROBE/'drive-c'/(token+'.NO');logs=PROBE/'logs'
 if a.action=='prepare' or (a.action=='launch' and not a.open_probe):print(json.dumps(dict(mode='dry-run-no-process',**r)));return
 if a.action=='launch':
  assert not any((logs/name).exists() for name in ['launch-process.json','final-process.json','host-release.json']) and not marker.exists() and not abort.exists(),'Preserve used probe; do not relaunch it'
  startup=subprocess.STARTUPINFO();startup.dwFlags|=subprocess.STARTF_USESHOWWINDOW;startup.wShowWindow=1
  start=time.monotonic()
  with (logs/'emulator-stdout.txt').open('xb') as out,(logs/'emulator-stderr.txt').open('xb') as err:
   process=subprocess.Popen(r['command'],cwd=PROBE,stdin=subprocess.DEVNULL,stdout=out,stderr=err,startupinfo=startup)
  launched=dict(pid=process.pid,launched_utc=datetime.now(timezone.utc).isoformat(),executable=str(SDL2/'dosbox-x.exe'),original_execution=False,command=r['command'],windows_show_window=1)
  (logs/'launch-process.json').write_text(json.dumps(launched,indent=2)+'\n')
  print(json.dumps(dict(pid=process.pid,original_execution=False,host_release_required=True,inspect='python -B scripts/reference/yield_probe.py inspect',release='python -B scripts/reference/yield_probe.py release',abort='python -B scripts/reference/yield_probe.py abort')),flush=True)
  timeout=False
  try:
   while process.poll() is None:
    remaining=120-(time.monotonic()-start)
    if remaining<=0:timeout=True;process.kill();break
    try:process.wait(timeout=min(2,remaining))
    except subprocess.TimeoutExpired:pass
   process.wait(timeout=5)
  finally:
   if process.poll() is None:process.kill();process.wait(timeout=5)
   final=dict(pid=process.pid,exit_code=process.returncode,ended_utc=datetime.now(timezone.utc).isoformat(),elapsed_host_seconds=time.monotonic()-start,host_timeout=timeout,original_execution=False)
   (logs/'final-process.json').write_text(json.dumps(final,indent=2)+'\n')
  print(json.dumps(final),flush=True);return
 launch=json.loads((logs/'launch-process.json').read_bytes())
 if a.action=='finish':
  final=json.loads((logs/'final-process.json').read_bytes());assert final['pid']==launch['pid'] and final['exit_code'] is not None
  assert datetime.fromisoformat(final['ended_utc'])<=datetime.now(timezone.utc)
  log=(logs/'dosbox-x.log').read_text(errors='replace');observations=None
  if a.observations:
   path=checked(a.observations.resolve(),PROBE);observations=dict(file=file_record(path),data=json.loads(path.read_bytes()))
  result=dict(status='empty-probe-ended',original_execution=False,emulator_execution=True,prepared=file_record(logs/'prepared.json'),final_process=final,
   sentinel_observed='DOS CON: REFERENCE_EMPTY_SENTINEL' in log,gate_released='DOS CON: REFERENCE_GATE_RELEASED '+token in log,aborted='DOS CON: REFERENCE_GATE_ABORTED' in log,
   observations=observations,files=[file_record(path) for path in logs.iterdir() if path.is_file()])
  (ROOT/'research/variants/reference-yield-probe-result.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));return
 image,started=process_identity(int(launch['pid']));assert image==(SDL2/'dosbox-x.exe').resolve()
 assert abs((started-datetime.fromisoformat(launch['launched_utc'])).total_seconds())<5,'PID reused'
 if a.action=='abort':
  assert not (logs/'host-release.json').exists(),'Already released'
  with abort.open('xb') as f:f.write(b'Abort empty probe\r\n')
  print(json.dumps(dict(pid=launch['pid'],abort=True,original_execution=False)));return
 log=(logs/'dosbox-x.log').read_text(errors='replace');age=(datetime.now(timezone.utc)-started).total_seconds();errors,rows=issues(log,token,age)
 stderr=(logs/'emulator-stderr.txt').read_text(errors='replace')
 if any(x in stderr.lower() for x in ('not a valid value','unknown option','unknown property','illegal command','file not found')):errors.append('Early runtime command/config rejection')
 if marker.exists() or abort.exists() or (logs/'host-release.json').exists():errors.append('Prior marker/release exists')
 report=dict(pid=launch['pid'],age_seconds=age,ready_for_supervisor_release=not errors,issues=errors,mounts=rows,log=file_record(logs/'dosbox-x.log'),original_execution=False)
 if a.action=='release':
  assert not errors,errors
  report['released_utc']=datetime.now(timezone.utc).isoformat()
  with (logs/'host-release.json').open('x') as f:json.dump(report,f,indent=2)
  with marker.open('xb') as f:f.write(b'Host releases empty sentinel only\r\n')
 print(json.dumps(report,indent=2))
if __name__=='__main__':main()

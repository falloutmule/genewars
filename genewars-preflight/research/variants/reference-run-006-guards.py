"""Mock the run006 release gate with valid pins but missing capture attestation."""
from pathlib import Path
from datetime import datetime,timezone
import contextlib,hashlib,io,json,sys
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'scripts/reference'))
import prepare
def main():
 prepare.select_session('run-006');import gate,launch
 from input_free_yield import CODE,verified_probe
 fixture=ROOT/'private/review/run006-capture-guard';logs=fixture/'logs';logs.mkdir(parents=True,exist_ok=True);drive=fixture/'drive-c';drive.mkdir(exist_ok=True);token='FIXTUREA';now=datetime.now(timezone.utc)
 marker=drive/(token+'.OK');assert not marker.exists() and not (logs/'host-release.json').exists()
 batch=drive/'STARTGW.BAT';batch.write_bytes(launch.gate_batch(token,original=True,yielding=True));helper=drive/'YIELD.COM';helper.write_bytes(CODE);stderr=logs/'stderr.txt';stderr.write_bytes(b'')
 human=json.loads(prepare.HUMAN_APPROVAL.read_bytes());human.update(approved=True,human_statement='Synthetic refusal fixture only; no original execution allowed');hp=fixture/'synthetic-human.json';hp.write_text(json.dumps(human))
 approval={'host_release_required':True,'release_marker':str(marker),'abort_marker':str(fixture/(token+'.NO')),'gate':prepare.file_record(batch),'gate_token':token,'yield_helper':prepare.file_record(helper),'verified_empty_yield_probe':verified_probe(ROOT)};(logs/'human-approval.json').write_text(json.dumps(approval))
 (logs/'launch-process.json').write_text(json.dumps({'pid':12345,'launched_utc':now.isoformat(),'stderr':str(stderr)}))
 log='\n'.join(['DOS CON: C      Local '+str(fixture/'drive-c'),'DOS CON: D      ISO '+str(fixture/'media/GENEWARS.cue'),'DOS CON: Z      Internal Virtual Drive','Switched to secure mode.','This operation is not permitted in secure mode.','REFERENCE_GATE_WAITING '+token,'DOS CON: Z:\\>CONFIG -GET sdl "mouse_emulation"','DOS CON: always','DOS CON: Z:\\>CONFIG -GET sdl "raw_mouse_input"','DOS CON: false']);(logs/'dosbox-x.log').write_text(log)
 with patch.object(gate,'RUN',fixture),patch.object(gate,'HUMAN_APPROVAL',hp),patch.object(gate,'process_identity',return_value=((prepare.SDL2/'dosbox-x.exe').resolve(),now)),patch.object(sys,'argv',['gate.py','--release-approved-original']):
  try:gate.main()
  except SystemExit as error:report=json.loads(str(error))
  else:raise AssertionError('Missing capture attestation accepted')
 assert report['issues']==['Supervisor must observe Main/Capture mouse before run006 release and pass --capture-observed'];assert report['ready_for_supervisor_release'] is False and not report['original_execution_requested_by_this_call'];assert not marker.exists() and not(logs/'host-release.json').exists()
 path=ROOT/'research/variants/reference-run-006-capture-guard.json';path.write_text(json.dumps(dict(status='passed-synthetic-missing-capture-refusal',utc=datetime.now(timezone.utc).isoformat(),original_execution=False,emulator_execution=False,process_identity_mocked=True,host_release_written=False,marker_written=False,gate_sha256=hashlib.sha256((ROOT/'scripts/reference/gate.py').read_bytes()).hexdigest(),report=report),indent=2)+'\n');print('PASS run006missingcapture refusedbeforemarker; allprocessidentitysynthetic')
if __name__=='__main__':main()

from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,subprocess,zipfile
P=Path(__file__).resolve().parents[2]
W=P.parent
git=['git','-c','safe.directory='+W.as_posix()]
def command(*args):
 return subprocess.run(git+list(args),cwd=P,capture_output=True,check=True).stdout
receipt=P/'research/integration/verification.json'
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest()
pixel=P/'research/integration/bturr-common-pixels-current.json'
observed=json.loads(pixel.read_text())
assert observed['case_count']==896
assert all(v=='atomic' for v in observed['rejections_atomic'].values())
assert observed['code_pins']['script']==sha(P/'scripts/desktop/validate_bturr_common_pixels.py')
supplement={'status':'passed-focused-repair','completed_utc':datetime.now(timezone.utc).isoformat(),
 'aggregate_sha256':sha(receipt),'aggregate_result_unchanged':'270/271; not a new all-green aggregate',
 'failed_check':'building_selector1_common_pixels','repair':'Removed an unrelated generated world-review JSON from source-listing pins. All primary listing/resource and implementation pins remain.',
 'command':['C:/Python314/python.exe','-B','scripts/desktop/validate_bturr_common_pixels.py'],
 'exit_code':0,'case_count':896,'atomic_rejections':observed['rejections_atomic'],
 'receipt_path':pixel.relative_to(P).as_posix(),'receipt_sha256':sha(pixel),
 'script_sha256':sha(P/'scripts/desktop/validate_bturr_common_pixels.py'),
 'log_path':'private/integration/bturr-common-recovery-20261014.log'}
(P/'research/integration/verification-repair-20261014.json').write_text(json.dumps(supplement,indent=2)+'\n')
dest=P/'private/recovery'/datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')
dest.mkdir(parents=True,exist_ok=False)
names=sorted(set(command('ls-files','--cached','--others','--exclude-standard','-z','--','.').decode().rstrip('\0').split('\0')))
pins={};missing=[]
with zipfile.ZipFile(dest/'project-code-metadata.zip','w',zipfile.ZIP_DEFLATED)as archive:
 for name in names:
  if not name:continue
  path=(P/name).resolve()
  assert path.is_relative_to(P.resolve()) and not path.is_relative_to((P/'private').resolve()),name
  if not path.is_file():missing.append(name);continue
  pins[name]=sha(path);archive.write(path,name)
(dest/'tracked-changes.patch').write_bytes(command('diff','HEAD','--binary','--','.'))
metadata={'created_utc':datetime.now(timezone.utc).isoformat(),'branch':command('branch','--show-current').decode().strip(),
 'head':command('rev-parse','HEAD').decode().strip(),'scope':'Current non-ignored project code and metadata, including untracked files. Private assets, tools, experiments and captures stay in their existing ignored directories; not a complete backup.',
 'files':pins,'missing_tracked_paths':missing,'zip_sha256':sha(dest/'project-code-metadata.zip')}
(dest/'manifest.json').write_text(json.dumps(metadata,indent=2)+'\n')
with zipfile.ZipFile(dest/'project-code-metadata.zip')as archive:assert archive.testzip() is None
print(json.dumps({'snapshot':dest.relative_to(P).as_posix(),'files':len(pins),'zip_bytes':(dest/'project-code-metadata.zip').stat().st_size,'zip_sha256':metadata['zip_sha256']}))

"""Windows atomic-receipt failure tests; no actual audit receipt is changed."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,errno,os,json,uuid,hashlib
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]

def main():
 assert os.name=='nt','These transient-error fixtures target the actual Windows host'
 source=ROOT/'research/variants/creature-review.py'
 spec=importlib.util.spec_from_file_location('creature_review',source);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
 fixture=ROOT/'private/review/receipt-write'/uuid.uuid4().hex;fixture.mkdir(parents=True)
 target=fixture/'creature-review.json';real_replace=os.replace
 with patch.object(module,'OUT',target):
  target.write_bytes(b'OLD');calls=[]
  def once(a,b):
   calls.append(1)
   if len(calls)==1:raise OSError(errno.EINVAL,'Synthetic observed Windows IO fault')
   real_replace(a,b)
  with patch.object(module.os,'replace',once),patch.object(module.time,'sleep',lambda delay:None):module.write_receipt('{"ok":true}\n')
  assert len(calls)==2 and target.read_bytes()==('{"ok":true}'+os.linesep).encode()
  target.write_bytes(b'OLD');calls=[]
  def locked(a,b):calls.append(1);raise PermissionError(errno.EACCES,'Synthetic sharing/access fault')
  with patch.object(module.os,'replace',locked),patch.object(module.time,'sleep',lambda delay:None):
   try:module.write_receipt('NEW')
   except PermissionError:pass
   else:raise AssertionError('Expected bounded failure')
  assert len(calls)==3 and target.read_bytes()==b'OLD'
  calls=[]
  def full(a,b):calls.append(1);raise OSError(errno.ENOSPC,'Synthetic disk full')
  with patch.object(module.os,'replace',full),patch.object(module.time,'sleep',lambda delay:None):
   try:module.write_receipt('NEW')
   except OSError as error:assert error.errno==errno.ENOSPC
   else:raise AssertionError('Expected immediate nontransient failure')
  assert len(calls)==1 and target.read_bytes()==b'OLD'
  assert not list(fixture.glob('.creature-review-*.tmp'))
 data=dict(status='passed-synthetic-windows-receipt-publication-tests',checked_utc=datetime.now(timezone.utc).isoformat(),
  source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),actual_creature_receipt_changed=False,original_or_gui_execution=False,
  checks=['Observed EINVAL retry then successful atomic publication','Windows newline convention preserved','Sharing/access error caps at3 attempts; old receipt preserved','ENOSPC propagates after1 attempt; old receipt preserved','No temporary leftovers'],
  limits=['Original transient IO cause not established','Tests inject replacement failures; they do not reproduce OS-level fault','Actual creature rerun separately passed all unchanged pixel assertions'])
 (ROOT/'research/variants/receipt-write-review.json').write_text(json.dumps(data,indent=2)+'\n')
 print(json.dumps(data))

if __name__=='__main__':main()

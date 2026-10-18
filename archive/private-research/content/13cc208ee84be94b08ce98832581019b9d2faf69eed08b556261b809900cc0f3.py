"""Manifest-bound PLGBL and mapped zone colors; no original execution."""
from pathlib import Path
import hashlib
from result_inputs import stage as stage_rows
ROOT=Path(__file__).resolve().parents[1]
ROWS=[['PLGBL.ANB','root/2/disc/124','cce0f7de88f2dc8ec251e70925de93d6e406c109d0bea9a82f4ee7f1592c9a30',13697,'a39cc8ac8c987de5c4865ec77a4459221f276286f254653e4a3f3231424f6444',40414]]
def stage():
 directory=ROOT/'private/integration/zone-ui-inputs';stage_rows(ROWS,directory)
 source=ROOT/'private/ghidra/validated/memory-.object3.bin';obj=source.read_bytes()
 if hashlib.sha256(obj).hexdigest()!='aaab09272771ee7f3fbe71e9a3dc75add592fcca42fdd8d982e069629ac1ce87':raise ValueError('mapped CD object3 changed')
 data=obj[0x2e38:0x2e3f]
 if hashlib.sha256(data).hexdigest()!='a7735d28ff0f0a4c3c5b5a78eb7f5c23cd8bf52f0d8ec33b84baea1c1a6651e8':raise ValueError('zone color table changed')
 target=directory/'zone-colors.bin'
 if target.exists():
  if target.read_bytes()!=data:raise ValueError('refusing changed staged zone colors')
 else:target.write_bytes(data)
 return directory
if __name__=='__main__':print(stage())

"""Actual785 complete caller using literal factory; explicit resource binding only.
No original execution. A successful source run is preparation, not native acceptance.
"""
from pathlib import Path
import importlib.util,sys
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('review',Path(__file__).with_name('krell-mule-callback-review.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
Original=m.Source
class WithFactory(Original):
 def __init__(self,b):
  super().__init__(b)
  g=P/'private/ghidra'
  extra=[g/'activation-rng-refs/instructions-0005a210.txt',g/'genepod-mule-initializer.txt',g/'construction-command/instructions-0001ee50.txt',g/'bones-frame/instructions-0005a1c0.txt',g/'player-creation/instructions-0001f480.txt',g/'terrain-map-refs/instructions-0007b240.txt']
  self.files+=extra;self.code.update(m.m.m.Model(extra,b'').code);order=sorted(self.code);self.next=dict(zip(order,order[1:]))
 def run(self,*args,**kwargs):
  # The common callback harness maps original MULE.ANB at400000; factory needs
  # its loader-owned object table pointer too. This is a normalized asset port.
  state=kwargs['state_before']; pointer=sum(state[0x1542be+31+i]<<(8*i) for i in range(4));m.put(state,0x180e84+4*4,4,pointer)
  return super().run(*args,**kwargs)
m.Source=WithFactory
if __name__=='__main__':m.main()


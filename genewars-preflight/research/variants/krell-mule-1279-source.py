from pathlib import Path
import importlib.util
P=Path(__file__).resolve().parents[2];sp=importlib.util.spec_from_file_location('birth',P/'research/variants/krell-pair-birth-callback-review.py');b=importlib.util.module_from_spec(sp);sp.loader.exec_module(b)
class WithCleanup(b.WithFactory):
 def __init__(self,backing):
  super().__init__(backing);extra=[P/'private/ghidra/panel-activation-refs/instructions-000724c0.txt',P/'private/ghidra/plant-startup/audio-shared-tail.txt'];self.files+=extra;self.code.update(b.m.m.m.Model(extra,b'').code);order=sorted(self.code);self.next=dict(zip(order,order[1:]))
b.m.Source=WithCleanup
if __name__=='__main__':b.m.main()


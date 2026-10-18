from pathlib import Path
import importlib.util,sys,shutil
P=Path.cwd();s=importlib.util.spec_from_file_location('mule',P/'research/variants/creature-hungry-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
Original=m.CapturedSource
class Extended(Original):
 def __init__(self):
  super().__init__();extra=[P/'private/ghidra/activation-rng-refs/instructions-0005cdb0.txt'];self.files+=extra;self.code.update(m.m.m.Model(extra,b'').code)
  order=sorted(self.code);self.next=dict(zip(order,order[1:]))
m.CapturedSource=Extended
start=P/'private/review/family8-trajectory/bb1c98b625784fc49cefacf05b8133ff/frame9052/boundary';last=start
for frame in range(9053,9075):
 w=P/'private/review/mule-mortality'/('frame'+str(frame));w.mkdir(parents=True,exist_ok=True)
 for name in ['backing.bin','creatures.bin','actors.bin','effects.bin','family8.bin','players.bin','rng.bin']:shutil.copyfile(last/name,w/name)
 creatures=bytearray((w/'creatures.bin').read_bytes());creatures[:124]=(last/('source-after'+str(frame-1)+'.bin')).read_bytes();(w/'creatures.bin').write_bytes(creatures)
 sys.argv=['source',str(frame),'--snapshot='+str(w.relative_to(P))];m.main();last=w

"""Literal7A280 no-mutation fixture proof; reuses whole-world capture mapping.

Arguments are --before PATH --frame N. Expected afterimage is the same world;
qualifying/scoring branches intentionally stop as an unsupported source boundary.
"""
from pathlib import Path
import importlib.util,sys,contextlib,io,json
spec=importlib.util.spec_from_file_location('callback',Path(__file__).with_name('krell-mule-callback-review.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
Original=m.Source
class Source(Original):
 def __init__(self,backing):
  super().__init__(backing)
  files=[m.P/'private/ghidra/mission-array-refs'/('instructions-000'+name+'.txt')for name in ['7a280','793c0']]
  self.files+=files;self.code.update(m.m.m.Model(files,b'').code);order=sorted(self.code);self.next=dict(zip(order,order[1:]))
 def run(self,*args,**kwargs):
  kwargs['entry']=0x7a280;kwargs['registers']={'EAX':0x11bd58};return super().run(*args,**kwargs)
if __name__=='__main__':
 m.Source=Source
 if '--expected'not in sys.argv:
  before=sys.argv[sys.argv.index('--before')+1];sys.argv+=['--expected',before]
 captured=io.StringIO()
 with contextlib.redirect_stdout(captured):m.main()
 report=json.loads(captured.getvalue());path=Path(report['output'])/'result.json';report=json.loads(path.read_text())
 report['scope']='Literal7A280 compared to unchanged captured input; native method comparison not run by this script'
 report['source_unchanged_comparison']=report.pop('production_comparison');report['production_comparison']='not_run';report['ports']=[];report['source_phase_addresses']=['7A290..7A298','7A2C2..7A2D9','793CF..793D6']
 path.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items()if k not in ['source_pins','input_pins']},indent=2))

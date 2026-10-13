"""Literal water request boundary plus current owned transaction comparison."""
from pathlib import Path
from datetime import datetime,timezone
import importlib.util,subprocess,sys,json,hashlib,struct
P=Path(__file__).resolve().parents[2];sys.path.insert(0,str(P/'scripts'));from cargo import environment
S=Path(__file__).with_name('initial-creature-source.py');spec=importlib.util.spec_from_file_location('source',S);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
class WaterSource(m.Source):
 def __init__(self):
  super().__init__();self.code[0x48790]=('RET',['0x10'])
 def load(self,a,n):
  if getattr(self,'pc',None)==0x48790 and not hasattr(self,'request'):
   self.request={};self.request=dict(code=self.r['EAX'],point=[m.signed(self.r['EDX']&65535,16),m.signed(self.r['EDX']>>16,16)],height=m.signed(self.r['EBX']),angles=[super(WaterSource,self).load(self.r['ECX']+i*2,2)for i in range(3)])
  return super().load(a,n)
def main():
 r=subprocess.run([sys.executable,'-B',str(Path(__file__).with_name('creature-motion-review.py')),'--native-only'],cwd=P,env=environment(),capture_output=True,timeout=180);assert r.returncode==0,r.stderr.decode(errors='replace')
 model=WaterSource();state=None;seed=5428
 for frame in range(1,139):
  report,after,_=model.run(frame,seed,state_before=state);state=model.saved_state;seed=report['rng_seed_after']
 report,expected,trace=model.run(139,seed,state_before=state);request=model.request;assert report['draws']==0
 other=WaterSource();_,poisoned,_=other.run(139,seed,state_before=state,stack_fill=255);assert poisoned==expected and other.request==request
 w=P/'private/review/creature-motion';native=(w/'water139-mule.bin').read_bytes();effect=(w/'water139-effect.bin').read_bytes();assert native[6:31]+native[39:]==expected[6:31]+expected[39:]
 assert request['code']==0x30 and request['angles']==[0,0,0]
 assert list(struct.unpack_from('<hh',effect,7))==request['point'];assert struct.unpack_from('<i',effect,39)[0]==request['height'];assert effect[69]==0x30
 result=dict(status='passed-optional-owned-mule-water-request',utc=datetime.now(timezone.utc).isoformat(),source_frame=139,numeric_mule_bytes=110,request=request,source_steps=report['steps'],missing_effect_pool_rollback=True,no_rng_draws=report['draws']==0,source_pins=[pin(p)for p in(P/'rust/crates/genewars-runtime/src').rglob('*.rs')],instruction_pins=[pin(p)for p in model.files],review_sources=[pin(Path(__file__)),pin(S),pin(Path(__file__).with_name('creature-motion-review.rs'))],outputs=[pin(w/f)for f in ['water139-mule.bin','water139-effect.bin']],limits=['Factory48790 is an explicit typed request boundary in the listing interpreter; it is not interpreted here. The production code30 factory is separately reviewed in the landing lane.','Numeric Mule fields6..30/39..123 and factory request position/height/code are compared; links/map are excluded because the request-only model deliberately does not insert an effect.','Original rejecting APIs remain unchanged. Optional owned step_mule_followup_with_water admits code30 only; code15/waterdamage and cell-changing arrival during request reject.','No original or GUI execution.'])
 out=Path(__file__).with_suffix('.json');temp=out.with_suffix('.json.tmp');temp.write_text(json.dumps(result,indent=2)+'\n');temp.replace(out);print(json.dumps({k:result[k]for k in ['status','numeric_mule_bytes','request','source_steps']}))
if __name__=='__main__':main()

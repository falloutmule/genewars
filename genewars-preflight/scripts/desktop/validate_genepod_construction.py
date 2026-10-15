"""Selector0 fixtures through existing independent literal-source oracles.

Only fixture selector/count and output destinations change. The original source
interpreters, instruction pins and byte comparisons are retained; no history is
overwritten. Each transformed driver is preserved privately and hashed.
"""
from pathlib import Path
import argparse,hashlib,json,sys,uuid
P=Path(__file__).resolve().parents[2]
PHASES={
 'progress':('scripts/ghidra/validate_building_progress.py',{'maximum={1:120':'maximum={0:280,1:120','building-progress-selector{selector}-evidence.json':'genepod-progress-evidence.json'}),
 'work':('scripts/ghidra/validate_construction_work.py',{'range(1536)':'range(384)','[1,3,6,11][case//384]':'0','{1:120,3:240,6:80,11:280}':'{0:280,1:120,3:240,6:80,11:280}','cases=1536':'cases=384','construction-work-admitted-evidence.json':'genepod-work-evidence.json'}),
 'stage4':('scripts/ghidra/validate_construction_stage4.py',{'range(768)':'range(192)','[1,3,6,11][case//192]':'0',"'cases':768":"'cases':192",'construction-stage4-admitted-evidence.json':'genepod-stage4-evidence.json'}),
 'finished':('scripts/ghidra/validate_construction_finished.py',{'range(192)':'range(48)','[1,3,6,11][case//48]':'0','{1:120,3:240,6:80,11:280}':'{0:280,1:120,3:240,6:80,11:280}',"'cases':192":"'cases':48",'construction-finished-admitted-evidence.json':'genepod-finished-evidence.json'}),
 'insufficient':('scripts/ghidra/validate_construction_cancellation.py',{'a[0x83]=1':'a[0x83]=0','plan_insufficient_selector1_cancellation':'plan_insufficient_genepod_or_selector1_cancellation','(case%24)*256+127':'(case%24+2)*256+127','(case//24)*256+127':'(case//24+2)*256+127','construction-cancellation-evidence.json':'genepod-insufficient-evidence.json'}),
 'motion':('scripts/desktop/validate_queued_architect_motion.py',{'[1,3,6,11][i%4]':'0','queued-architect-motion.json':'genepod-motion.json'}),
 'bookkeeping':('scripts/desktop/validate_admitted_building_player.py',{'[1,3,6,11][(case+owner+ordinal)%4]':'0','admitted-building-player.json':'genepod-bookkeeping.json'}),
 'arrival':('scripts/ghidra/validate_pending_architect_callback.py',{
  'range(208)':'range(2)','count=case%26':'count=0 if case%2==0 else 25',
  'before[0x55]=1':'before[0x55]=8','before[0x58]=0':'before[0x58]=2',
  'before[0x49]=32 if case%2==0 else 16':'before[0x49]=32','[1,3,6,11][case//52]':'0',
  'state[0x11bd57]=count;':'state[0x11872c]=31;state.update({0x118782+i:255 for i in range(4)});state[0x11bd57]=count;',
  'let mut prefix=*before;':'let bp={let mut bp=[0;36];bp[0x17]=31;bp};let arrival=actor_motion::plan_actor_arrival_pending_architect(before,&bp,actor_motion::ActorCallbackRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();let mut prefix=arrival.after;',
  "'cases':208":"'cases':2",'pending-architect-admitted-callback-evidence.json':'genepod-arrival-evidence.json'}),
}
def main():
 ap=argparse.ArgumentParser();ap.add_argument('phase',choices=PHASES);args=ap.parse_args()
 rel,changes=PHASES[args.phase];source=P/rel;text=source.read_text(encoding='utf-8')
 for old,new in changes.items():
  assert text.count(old)==1,(old,text.count(old));text=text.replace(old,new)
 text=text.replace('selectors1/3/6/11','selector0 Genepod').replace('selector1 stage3','selector0 stage3')
 work=P/'private/review/genepod-construction-drivers'/uuid.uuid4().hex;work.mkdir(parents=True)
 (work/'driver.py').write_text(text,encoding='utf-8');saved=sys.argv;sys.argv=[str(source),'--selector','0']
 try:exec(compile(text,str(work/'driver.py'),'exec'),{'__file__':str(source),'__name__':'__main__'})
 finally:sys.argv=saved
 outputs={'motion':'research/integration/genepod-motion.json','bookkeeping':'research/integration/genepod-bookkeeping.json'}
 output=P/outputs.get(args.phase,f'research/ghidra/genepod-{args.phase}-evidence.json')
 report=json.loads(output.read_text());report.update(fixture_selector=0,fixture_driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
  reused_driver=rel,reused_driver_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),transformed_driver=work.relative_to(P).as_posix()+'/driver.py',
  transformed_driver_sha256=hashlib.sha256((work/'driver.py').read_bytes()).hexdigest(),original_gameplay_acceptance=False)
 output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'phase':args.phase,'status':'passed','cases':report.get('cases'),'receipt':output.relative_to(P).as_posix()}))
if __name__=='__main__':main()

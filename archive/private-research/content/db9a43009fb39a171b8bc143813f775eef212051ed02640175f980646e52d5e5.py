"""Reuse the pending-task whole-map oracle with explicit state8 arrival inputs.
Only fixture/entry composition changes; the original literal-source interpreter
and source setup/terrain/job/tail comparisons are retained.
"""
from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parents[2]
def main():
 source=P/'scripts/ghidra/validate_pending_architect_callback.py';text=source.read_text()
 changes={
  'range(208)':'range(8)',
  'count=case%26':'count=0 if case%2==0 else 25',
  'before[0x55]=1':'before[0x55]=8',
  'before[0x58]=0':'before[0x58]=2',
  'before[0x49]=32 if case%2==0 else 16':'before[0x49]=32',
  '[1,3,6,11][case//52]':'[1,3,6,11][case//2]',
  'state[0x11bd57]=count;':'state[0x11872c]=31;state.update({0x118782+i:255 for i in range(4)});state[0x11bd57]=count;',
  'let mut prefix=*before;':'let bp={let mut bp=[0;36];bp[0x17]=31;bp};let arrival=actor_motion::plan_actor_arrival_pending_architect(before,&bp,actor_motion::ActorCallbackRng{seed:5428,ring:31,draw_count:u32::MAX}).unwrap();let mut prefix=arrival.after;',
  "'cases':208":"'cases':8",
  'pending-architect-admitted-callback-evidence.json':'queued-architect-arrival-evidence.json',
  'Whole57A10 pending state1/new selectors1/3/6/11 task':'Whole57A10 state8 arrival with queued selectors1/3/6/11 task; source526C0 and RNG21B then same-callback507D0',
 }
 for old,new in changes.items():
  assert text.count(old)==1,(old,text.count(old));text=text.replace(old,new)
 exec(compile(text,str(source), 'exec'),{'__file__':str(source),'__name__':'__main__'})
 report=P/'research/ghidra/queued-architect-arrival-evidence.json';r=json.loads(report.read_text())
 r['fixture_driver_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
 r['scope']='Whole57A10 state8 ready arrival, queued selectors1/3/6/11; actual526C0 and RNG21B, same-callback507D0/534D0/tail; full200B actor/25jobs/map/audio/viewport; empty/full job pools. Common physics and queue are explicit ports; mapped-prior border fixture retained.'
 report.write_text(json.dumps(r,indent=2)+'\n')
if __name__=='__main__':main()

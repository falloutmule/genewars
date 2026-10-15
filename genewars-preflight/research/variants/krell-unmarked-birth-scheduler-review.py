"""Actual unmarked-birth scheduling shape, controlled callback ports."""
from pathlib import Path
import importlib.util,json,hashlib
p=Path(__file__).resolve();s=importlib.util.spec_from_file_location('scheduler',p.with_name('multi-mule-scheduler-review.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
c={'name':'actual785_unmarked_birth_shape','count':4,'records':[{'slot':0},{'slot':1},{'slot':2,'return':0,'births':[{'slot':4,'newborn':128}]},{'slot':3}]}
a=m.Scheduler([m.LIST],b'').run_case(c);assert a['callbacks']==[0,1,2,3];assert a['live_count_after']==5;assert a['newborn_flags']['4']==128
r={'status':'passed','scope':'Literal6A540 scheduler with controlled callbacks matching actual785 allocation shape; not full world/scheduler native acceptance.','case':c,'source':a,'source_sha256':hashlib.sha256(m.LIST.read_bytes()).hexdigest(),'harness_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'actual_factory_marker':128,'actual_parent_return':0,'evidence':'5EF16 initializes local146 zero;6A52C returnsit, nointerveningbirthwrite. Factoryafter slot4+15=80.','previous_generic_rule':'The newborn-marked secondpass applies only callbackAL>0 andmarkerbit1; it does not apply to this unmarked neutralbirth.'}
p.with_suffix('.json').write_text(json.dumps(r,indent=2),encoding='utf-8');print(json.dumps({'status':'passed','callbacks':a['callbacks'],'count':a['live_count_after']}))

"""Source-only wait-prefix sweep; goal predicate is an explicit controlled port."""
from pathlib import Path
import importlib.util,struct,json,hashlib
P=Path(__file__).resolve().parents[2];p=P/'research/variants/initial-creature-source.py';s=importlib.util.spec_from_file_location('s',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
base=(P/'private/review/krell-mule-callbacks-013/frame-557-slot-1-before/creatures.bin').read_bytes()[124:248];cases=[]
for timer in [1,20,80]:
 for phase in range(4):
  for predicate in [False,True]:
   a=bytearray(base);a[84]=timer;struct.pack_into('<I',a,35,0x111fec);state={0x15433a+i:v for i,v in enumerate(a)}
   model=m.Source();model.code[0x5b100]=('MOV',['AL',str(int(predicate))]);model.code[0x5b101]=('RET',[]);model.next[0x5b100]=0x5b101;model.code[0x6fd58]=('RET',[])
   r,_,trace=model.run(counter=phase-1,seed=63161,state_before=state,entry=0x6fc5c,registers={'ESI':0x15433a},instruction_budget=10000)
   after=bytes(model.load(0x15433a+i,1)for i in range(124));expected=bytearray(a);flags=struct.unpack_from('<H',a,73)[0]
   if phase==3 and not predicate:expected[84]=20;flags&=0xeffc;expected[22]&=0x7f
   else:
    expected[84]=(timer-1)&255
    if expected[84]==0:expected[84]=20;flags=(flags&0xeebc)|0x140;expected[22]&=0x7f
   struct.pack_into('<H',expected,73,flags)
   # Heading is executed literally, not compared to a second numeric model.
   if phase&1:expected[75:77]=after[75:77]
   assert after==expected,[(i,x,y)for i,(x,y)in enumerate(zip(after,expected))if x!=y]
   assert (0x5b100 in trace)==(phase==3);assert (0x73970 in trace)==bool(phase&1);assert r['draws']==0
   cases.append(dict(timer=timer,phase=phase,controlled_goal_predicate=predicate,predicate_called=phase==3,after_timer=after[84],after_flags49=flags,after_heading=struct.unpack_from('<h',after,75)[0],instructions=r['steps'],status='passed_prefix_contract'))
result=dict(status='passed_prefix_contract',cases=cases,native_comparison='not_run',ports=['5B100 controlled true/false; not a real-world collision query','Stop at6FD58 before steering/physics; full557 callback separately preserved'],compared_nonheading_bytes=122*len(cases),heading_scope='Literal73970 executed on oddphase; exactnumeric not independently compared',source_pins={str(p.relative_to(P)):hashlib.sha256(p.read_bytes()).hexdigest()for p in model.files},script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
Path(__file__).with_suffix('.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({k:v for k,v in result.items()if k not in ['source_pins','cases']},indent=2))

"""Literal7A280 countdown/empty-zone and79220 empty-job return gates."""
from pathlib import Path
import importlib.util,struct,json,hashlib
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('literal',Path(__file__).with_name('initial-creature-source.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
model=m.Source()
files=[P/'private/ghidra'/p for p in ['mission-array-refs/instructions-0007a280.txt','construction-command/instructions-000791c0.txt','keyboard-event-refs/instructions-00075f60.txt']]
model.code.update(m.m.Model(files,b'').code);order=sorted(model.code);model.next=dict(zip(order,order[1:]))
base=0x11bd58;job=0x11ac27;cases=[]
for countdown,count,entry in [(1,1,0x7a280),(1000,1,0x7a280),(65535,4,0x7a280),(0,0,0x7a280),(1000,1,0x79220),(0,1,0x79220)]:
 state={base+i:(i*19+37)&255 for i in range(1218)}
 state[base+0x4c0]=count
 for i,b in enumerate(struct.pack('<H',countdown)):state[0x11c2bc+i]=b
 state[job+0x1130]=0
 report,_,trace=model.run(entry=entry,state_before=state,registers={'EAX':base if entry==0x7a280 else job})
 for a,b in state.items():assert model.load(a,1)==b,(hex(entry),hex(a),b,model.load(a,1))
 assert report['draws']==0 and not report['calls'],report
 cases.append(dict(entry=hex(entry),countdown=countdown,zones=count,instructions=report['steps'],memory_unchanged=True))
r=dict(status='passed',scope='Literal no-mutation admission gates only; active zones and jobs not admitted',cases=cases,literal_instructions=sum(c['instructions']for c in cases),sha256={p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in files+[Path(__file__)]},original_execution=False)
Path(__file__).with_suffix('.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(dict(status=r['status'],cases=len(cases),literal_instructions=r['literal_instructions'])))

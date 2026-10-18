"""Literal MAINPANL gate only; caller/module provenance stays explicit."""
from pathlib import Path
import importlib.util,hashlib,json,struct
P=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
f=P/'private/ghidra/entity-order/instructions-0002bbe0.txt';q=P/'private/ghidra/mission-target/main-panel-gate-slice.txt';q.write_text('\n'.join(line for line in f.read_text().splitlines() if 0x2bdb9<=int(line[:8],16)<=0x2be41)+'\n');x=m.Source();x.code.update(m.m.Model([q],b'').code)
for a in [0x99ad0,0x20880,0x25150,0x2be41]:x.code[a]=('RET',[])
order=sorted(x.code);x.next=dict(zip(order,order[1:]));cases=[]
for panel in [0,1]:
 for previous in [0,0x185def,0x1863b8]:
  for intro in [0,1]:
   for mode in [0,1]:
    state={};fields=[(0x118770,1,0x3a),(0x118968,1,mode),(0x185ec4,4,panel),(0x11697c,4,previous),(0x11ac22,4,intro),(0x11945b,1,0)]
    for a,n,v in fields:state.update({a+i:(v>>(8*i))&255 for i in range(n)})
    report,_,_=x.run(counter=0,state_before=state,entry=0x2bdb9,registers={'EBP':0x185dfc})
    expected=0x3e if panel==0 else 0x3a;assert x.load(0x118770,1)==expected
    assert (0x25150 in x.calls)==(panel==0);assert 0x20880 not in x.calls
    cases.append({'panel_present':bool(panel),'previous':hex(previous),'intro':intro,'mode':mode,'flags_after':expected,'instructions':report['steps']})
exit_file=P/'private/ghidra/startup-prior-refs/instructions-0007d020.txt'
x.code.update(m.m.Model([exit_file],b'').code)
for a in [0x9a244,0xa1610,0xa3da0]:x.code[a]=('RET',[])
order=sorted(x.code);x.next=dict(zip(order,order[1:]));exit_cases=[]
for frame in [0,1,99,0xffffffff]:
 for display in [0,1]:
  state={0x16e0a7:display};state.update({0x1863b0+i:b for i,b in enumerate(struct.pack('<I',0x1127e0))})
  report,_,_=x.run(counter=frame,state_before=state,entry=0x7d020,registers={'EAX':0x1863b0})
  assert x.load(0x11877e,4)==0 and x.load(0x116974,4)==0
  exit_cases.append({'before_frame':frame,'display_branch':display,'after_frame':0,'instructions':report['steps']})
pins={}
for relative,needles in {
 'campaign-refs/instructions-0008dfe0.txt':['0008e081 CALL dword ptr [EDX + 0xc]'],
 'mission-bootstrap-mode/instructions-0001cdf0.txt':['0001cec4 AND DH,0x40'],
 'ui-objects/constructor-range.txt':['0008dc9d MOV EAX,0x185dfc','0008dcbd CALL 0x0002b7e0'],
}.items():
 data=(P/'private/ghidra'/relative).read_bytes();text=data.decode();assert all(n in text for n in needles),(relative,needles);pins[relative]=hashlib.sha256(data).hexdigest()
r={'mission_view_exit_cases':exit_cases,'exit_source_sha256':hashlib.sha256(exit_file.read_bytes()).hexdigest(),'startup_static_pins':pins,'frame_inference':'Correction to earlier frame1 inference:7D090 loadsframe0 during8DFE0; outer increment yields1. Next1BBC0 transition calls mission exit7D020 first, which resetsframe0, thenMAINPANL entry enablesflags4. Ordinary mode0 first eligible callback therefore usesframe0, withincrementaftercallbacks. This includes the previously omitted old-view exit.','mode_inference':'1CDF0 masks startup mode with40; mapped initial mode0 and no mode-changing command-line switches yield0. Campaign exit16170 clears network bit0. This does not cover resumed/special-mode sessions.','status' :'passed-first-mission-main-panel-conditional-gate','cases':cases,'source_sha256':hashlib.sha256(f.read_bytes()).hexdigest(),'claim':'For campaign progress0 and entry flags3A, null MAINPANL+C8 always calls25150 then enables simulation bit4, independently of tested prior views and intro flag. Non-null panel skips that gate.','ports':['9A244 exit text, A1610/A3DA0 display ports; no gameplay state effects modeled','99AD0 startup platform call','25150 panel initialization','20880 intro initialization'],'unresolved':['Prove MAINPANL+C8 is null at this entry, or recover prior flags when non-null.','25150 transitive presentation effects remain explicit; no direct frame write exists in MAINPANL entry.'],'original_execution':False}
(P/'research/campaign/main-panel-startup-gate.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps({k:v for k,v in r.items() if k!='cases'}))

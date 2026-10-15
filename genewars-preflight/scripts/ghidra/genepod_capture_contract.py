"""Literal bounded Genepod control capture; no game execution or Rust build."""
from pathlib import Path
import hashlib, importlib.util, json, struct
P=Path(__file__).resolve().parents[2]
paths=['private/ghidra/ui-input/instructions-000a87d0.txt','private/ghidra/ui-controls/instructions-000a86d0.txt','private/ghidra/sprite-scaled/instructions-00098b60.txt','private/ghidra/plant-command/instructions-00021a10.txt','private/ghidra/building-cost-refs/instructions-00021e20.txt','private/ghidra/actor-panel-font-color/instructions-00022fd0.txt']
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
vm=m.Source();vm.code.update(m.m.Model([P/p for p in paths[:3]],b'').code);order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
obj=(P/'private/ghidra/validated/memory-.object3.bin').read_bytes()
asset=P/'private/integration/rnc-decoded/83e7010ccc0631db4937ab96395d058f8931458124eb004a7fa91d1c01c3648e.bin';anb=asset.read_bytes();table=struct.unpack_from('<I',anb,2)[0];off,w,h=struct.unpack_from('<IBB',anb,table+6)
base={}
def put(s,a,b):s.update({a+i:v for i,v in enumerate(b)})
def word(s,a,n):put(s,a,struct.pack('<I',n))
control=0x106dbc
put(base,control,obj[control-0x100000:control-0x100000+40]);put(base,control+40,bytes(40))
put(base,control+8,struct.pack('<hh',w,h));word(base,control+18,0x400000)
put(base,0x400000,struct.pack('<IBB',0x410000+18+off,w,h));put(base,0x410000,anb)
word(base,0x19ff64,control);word(base,0x19ff88,0);word(base,0x118970,0x420000)
put(base,0x19ff70,bytes(8));put(base,0x1a011c,bytes(20))
cases=[]
for name,events in [('inside',[(40,358,1),(40,358,4),(40,358,2)]),('outside_release',[(40,358,1),(60,358,2)]),('leave_return',[(40,358,1),(60,358,4),(40,358,4),(40,358,2)]),('transparent_press',[(26,344,1)])]:
 state=base.copy();rows=[]
 for x,y,bits in events:
  put(state,0x420000,struct.pack('<hh4xBB',x,y,bits,0))
  vm.run(state_before=state,entry=0xa87d0,registers={'EAX':0});state=vm.saved_state.copy()
  rows.append(dict(point=[x,y],packet8=bits,action=(vm.r['EAX'] & 65535),event=vm.load(0x19ff70,4),capture=vm.load(0x19ff88,4),draw=vm.load(control+39,1)))
 cases.append(dict(name=name,steps=rows))
assert [(r['action'],r['event']) for r in cases[0]['steps']]==[(8,0),(65535,2),(8,8)]
assert cases[1]['steps'][-1]['event']==0x8008
assert [(r['action'],r['event']) for r in cases[2]['steps']]==[(8,0),(8,1),(8,2),(8,8)]
assert cases[3]['steps'][0]['action']==65535
result=dict(status='passed',scope='Literal manager and opaque hit test for isolated bound control; panel readiness source inspection, not full UI/native acceptance',source_pins={p:hashlib.sha256((P/p).read_bytes()).hexdigest() for p in paths},cases=cases,control=hex(control),flags=hex(struct.unpack_from('<I',obj,control-0x100000)[0]))
(P/'research/campaign/genepod-native-capture.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result))


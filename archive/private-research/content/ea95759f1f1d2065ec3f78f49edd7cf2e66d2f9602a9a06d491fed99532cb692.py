from pathlib import Path
import importlib.util,struct,json
P=Path(__file__).resolve().parents[2];spec=importlib.util.spec_from_file_location('s',P/'research/variants/initial-creature-source.py');s=importlib.util.module_from_spec(spec);spec.loader.exec_module(s);v=s.Source();files=[P/'private/ghidra/activation-rng-refs/instructions-00048790.txt',P/'private/ghidra/player-creation/instructions-0001f070.txt',P/'private/ghidra/mission-tail-init/instructions-00049e90.txt',P/'private/ghidra/terrain-map-refs/instructions-0007b240.txt'];v.code.update(s.m.Model(files,b'').code);order=sorted(v.code);v.next=dict(zip(order,order[1:]));state={}
def put(a,n,x):state.update({a+i:b for i,b in enumerate((x%2**(8*n)).to_bytes(n,'little'))})
for i in range(200*81):put(0x1688ae+i,1,0)
for i in range(200):put(0x1688ae+i*81+35,4,0x112254)
for i in range(16384):put(0x11c2be+i*14,2,100);put(0x11c2be+i*14+6,3,0);put(0x11c2be+i*14+12,1,128)
for a,n,x in [(0x11c2b4,2,0),(0x300004,4,0),(0x300008,4,0),(0x30000c,4,0),(0x300010,4,0),(0x280000,2,1024),(0x280002,2,1024),(0x280004,4,8000),(0x280010,2,0),(0x280012,2,0),(0x280014,2,8192)]:put(a,n,x)
try:
 r,_,t=v.run(seed=5428,state_before=state,entry=0x48790,registers={'EAX':32,'EDX':(1024<<16)|1024,'EBX':8000,'ECX':0x280010},instruction_budget=10000);print(r);print(bytes(v.load(0x1688ae+i,1)for i in range(81)).hex())
except Exception as e:print(type(e).__name__,e,hex(v.pc))

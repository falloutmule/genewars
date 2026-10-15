"""Bounded literal Genepod work/payment and quality-decision probes; no Rust build."""
from pathlib import Path
import hashlib, importlib.util, json, struct
P=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('source',P/'research/variants/initial-creature-source.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
listing=P/'private/ghidra/building-update/instructions-000451f0.txt'
vm=m.Source();vm.code.update(m.m.Model([listing],b'').code)
# Explicit probe endpoints: never substitute a successful factory or full callback.
for a in [0x45d8e,0x455c2,0x4568f,0x459e8]:vm.code[a]=('JMP',['0xdeadbeef'])
order=sorted(vm.code);vm.next=dict(zip(order,order[1:]))
B=0x1645ae;F=0x500000;P0=0x118d74
def put(s,a,n,v):s.update({a+i:b for i,b in enumerate(int(v&((1<<(8*n))-1)).to_bytes(n,'little'))})
def setup(flags,grade,primary,secondary,work,payer=0):
 s={B+i:0 for i in range(67)};s.update({P0+i:0 for i in range(7220)})
 for a,n,v in [(F-0x26,4,B),(F-0x16,4,P0+0x4a1),(B+0x31,2,flags),(B+0x2e,1,grade),(B+0x36,4,work),(B+0x3a,1,payer),(P0+0x4a3,4,primary),(P0+0x4a7,4,secondary)]:put(s,a,n,v)
 return s
work_cases=[]
for flags in [1,5,0x101,0x41]:
 for frame in [0,1,6]:
  for grade in [0,1,2]:
   for primary,secondary in [(0,0),(0,20),(3,20)]:
    s=setup(flags,grade,primary,secondary,2)
    r,_,trace=vm.run(counter=frame,state_before=s,entry=0x45470,registers={'EBP':F})
    admitted=bool(flags&1) and not flags&0x140
    paid=admitted and bool(flags&4 or frame%6==0) and primary+secondary>=1<<grade
    remaining=2-int(paid) if admitted else 0
    expected_primary=max(0,primary-(1<<grade)) if paid else primary
    expected_secondary=secondary-max(0,(1<<grade)-primary) if paid else secondary
    assert vm.load(B+0x36,4)==remaining
    assert vm.load(P0+0x4a3,4)==expected_primary and vm.load(P0+0x4a7,4)==expected_secondary
    work_cases.append({'flags':flags,'frame':frame,'grade':grade,'funds':[primary,secondary],'paid':paid,'work_after':remaining})
# Human owner affordability and recorded payer debit are distinct source reads.
s=setup(5,0,10,0,2,1);put(s,P0+1805+0x4a3,4,7)
vm.run(state_before=s,entry=0x45470,registers={'EBP':F})
assert vm.load(P0+0x4a3,4)==10 and vm.load(P0+1805+0x4a3,4)==6
decisions=[]
vm.code[0x455c2]=m.m.Model([listing],b'').code[0x455c2]
for quality in [0,35,50,100,150]:
 for grade in [0,2]:
  for seed in [0,1,5428,10548,65535,12345,32768,321]:
   s=setup(5,grade,100,0,0);put(s,F-0x22,4,P0);put(s,B+0x33,1,4);put(s,P0+0x62a,4,quality<<11)
   _,_,trace=vm.run(seed=seed,state_before=s,entry=0x455c2,registers={'EBP':F,'EAX':B})
   q=min(quality,100) if grade<2 else quality
   next_seed=(seed*0xbb40e62d)&65535;roll=(next_seed>>8)%100
   assert vm.load(F-0x1e,4)==q and vm.load(0x11878a,2)==next_seed
   success=0x4568f in trace;assert success==(roll<=2*q)
   decisions.append({'quality':quality,'grade':grade,'seed':seed,'roll':roll,'factory_path_selected':success})
pins=[listing,P/'private/ghidra/activation-rng-refs/instructions-0005a210.txt',P/'private/ghidra/building-loader-refs/instructions-000814a0.txt']
out={'status':'passed','reference_sha256':'4bdf296febc88984aab6939fe3f3ef3196891f0be9bcb84a9283ab3b668a60ba','source_pins':{p.relative_to(P).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in pins},'scope':'Literal45470 work/payment through45D8E or455C2; literal455C2 decision through4568F/459E8. Explicit endpoints exclude factory, effects, animation and full callback.','work_cases':work_cases,'distinct_payer_case':'passed','decision_cases':decisions,'original_execution':False,'native_acceptance':'not_run'}
(P/'research/campaign/krell-genepod-production.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'status':'passed','work_cases':len(work_cases),'payer_cases':1,'decision_cases':len(decisions)}))

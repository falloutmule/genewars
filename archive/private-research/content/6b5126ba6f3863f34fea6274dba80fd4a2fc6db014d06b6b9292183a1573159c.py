"""Private probe (ignored): run the owned-mule 5ED20 source interpreter on the
measured 5968 record with ONLY the enemy admission widened to the measured
enemy shape (neutral Mule state 8, 0x60=0x88, 0x61=0x40, 0x78=0, 0x6c=0xf,
0x55-ref back at the owned mule). Oracle files untouched; result decides
whether the source reads the enemy state on this path."""
from pathlib import Path
import importlib.util,json,struct,uuid,sys
P=Path(r'C:/Users/fallo/Documents/ChatGPT/Freecad/genewars-preflight')
spec=importlib.util.spec_from_file_location('ownw',P/'research/variants/owned-mule-captured-width-review.py')
own=importlib.util.spec_from_file_location('zc',P/'research/variants/zone-credit-captured-review.py')
c=importlib.util.module_from_spec(own);own.loader.exec_module(c);g=c.g
spec=importlib.util.spec_from_file_location('ab',P/'research/variants/actor-blueprint-captured-bindings.py')
ab=importlib.util.module_from_spec(spec);spec.loader.exec_module(ab)
before=P/'private/review/attempt28-5968-measure/capture/frame-5968-slot-1-blocked-before-owned'
slot,frame=1,5968
out=P/('private/review/attempt28-5968-probe-'+uuid.uuid4().hex[:8]);out.mkdir(parents=True)
buffers=g.read_buffers(before)
record=buffers['creatures.bin'][slot*124:(slot+1)*124]
assert record[94]==1 and struct.unpack_from('<H',record,0x49)[0]&0x400==0
state,seed,resources=g.prepare(buffers);g.k.put(state,0x11878c,1,1)
vm=c.Source(buffers['backing.bin'])
enemy_base=0x1542be+5*124
run,_,trace=vm.run(entry=0x5ed20,counter=frame,seed=seed,state_before=state,registers={'EAX':0x1542be+slot*124},instruction_budget=500000)
result=g.export(vm,buffers)
for name,data in result.items():(out/name).write_bytes(data)
(out/'trace.txt').write_text('\n'.join(hex(p) for p in trace)+'\n')
# did execution read the enemy record region?
acc=sorted({hex(p) for p in trace if enemy_base<=p<enemy_base+124})
report={'status':'executed','out':str(out),'draws':run['draws'],'instructions':run['steps'],
 'enemy_accesses':acc[:60],'enemy_access_count':len(acc),
 'rng_after':result.get('rng.bin',b'').hex() if 'rng.bin' in result else None}
(out/'probe.json').write_text(json.dumps(report,indent=2))
print(json.dumps({k:report[k] for k in ('status','draws','instructions','enemy_access_count')},indent=2))
print('enemy_accesses:',acc[:40])

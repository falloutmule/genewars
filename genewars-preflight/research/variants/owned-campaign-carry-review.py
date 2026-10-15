"""Literal72E50 mixed-cell carry comparison; synthetic adapter fixtures only."""
from pathlib import Path
import hashlib,json,struct,subprocess,uuid,sys,os
P=Path(__file__).resolve().parents[2]
W=P/'private/review/owned-campaign-carry'/uuid.uuid4().hex
W.mkdir(parents=True)
env=dict(os.environ,CARGO_BUILD_JOBS='2',GENEWARS_CARRY_FIXTURES=str(W))
run=subprocess.run([sys.executable,'-B',str(P/'scripts/cargo.py'),'test','-p','genewars-runtime','campaign_carry::tests','--offline','--locked'],cwd=P,env=env,capture_output=True,text=True,timeout=120)
(W/'tests.txt').write_text(run.stdout+run.stderr)
assert run.returncode==0,(run.stdout,run.stderr)
# Reuse the existing bounded interpreter setup, not its isolated fixtures or
# production planner. Its memset/memmove and selection-service ports remain explicit.
review=P/'research/variants/result-carry-review.py'
setup=review.read_text().split("original=P/'research/variants/initial-creature-source.py';",1)[1].split("for label in ['1289','1489']:",1)[0]
exec("original=P/'research/variants/initial-creature-source.py';"+setup)
stats=[]
for label in range(6):
 d=W/str(label);players=(d/'players.bin').read_bytes();prior=(d/'actors.bin').read_bytes();actors=bytearray(prior);bindings=(d/'bindings.bin').read_bytes();backing=(d/'backing.bin').read_bytes()
 for i in range(20):
  valid,owner,bp=bindings[i*3:i*3+3]
  if valid:struct.pack_into('<I',actors,i*200+192,0x118d74+owner*1805+0x16+bp*36)
 def native_plants(data):
  out=bytearray(512*49)
  for i in range(512):
   packed=data[277838+i*41:277838+(i+1)*41];out[i*49:i*49+31]=packed[:31];out[i*49+39:(i+1)*49]=packed[31:]
  return bytes(out)
 state={}
 for base,data in [(0x118d60,backing[:0x355e]),(0x118d74,players),(0x16360e,actors),(0x11c2be,(d/'map.bin').read_bytes()),(0x1645ae,(d/'buildings.bin').read_bytes()),(0x15d40e,native_plants(backing)),(0x184d7c,bytes([0xa5])*600)]:state.update({base+i:v for i,v in enumerate(data)})
 state[0x118770]=0x7e;state[0x118968]=2;state[0x11873d]=0
 for addr,val in [(0x118740,0x16360e),(0x185ec4,0)]:state.update({addr+i:v for i,v in enumerate(struct.pack('<I',val))})
 report,_,trace=s.run(counter=4630,state_before=state,entry=0x72e50,registers={'EAX':0x118d60})
 aa=bytearray(s.load(0x16360e+i,1)for i in range(4000))
 for i in range(20):aa[i*200+192:i*200+196]=prior[i*200+192:i*200+196]
 expected=bytes(s.load(0x118d74+i,1)for i in range(7220))+aa+bytes(s.load(0x11c2be+i,1)for i in range(229376))+bytes(s.load(0x184d7c+i,1)for i in range(600))+bytes([s.load(0x118770,1)])+struct.pack('<I',s.load(0x11877e,4))+bytes([int(0x25150 in trace),0,1])
 (d/'expected.bin').write_bytes(expected);actual=(d/'actual.bin').read_bytes()
 assert actual==expected,(label,'carry',[(i,a,b)for i,(a,b)in enumerate(zip(actual,expected))if a!=b][:12])
 comparisons=[(0x1645ae,(d/'building-after.bin').read_bytes()),(0x15d40e,native_plants((d/'backing-after.bin').read_bytes()))]
 for base,data in comparisons:
  want=bytes(s.load(base+i,1)for i in range(len(data)))
  assert data==want,(label,hex(base),[(i,a,b)for i,(a,b)in enumerate(zip(data,want))if a!=b][:12])
 assert trace.count(0x80d40)==2 and trace.count(0x8acc0)==2
 stats.append(dict(case=label,instructions=report['steps'],compared_bytes=len(expected)+sum(len(v)for _,v in comparisons),actor_unlinks=2))
def pin(p):return dict(path=p.relative_to(P).as_posix(),sha256=hashlib.sha256(p.read_bytes()).hexdigest())
production=[P/'rust/crates/genewars-runtime/src'/q for q in ['actor_world/campaign_carry.rs','actor_world/actor_updates.rs','actor_world.rs','result_carry.rs']]
receipt=dict(status='passed',private_run=W.relative_to(P).as_posix(),tests=next(line for line in run.stdout.splitlines() if line.startswith('test result:')),cases=stats,source_pins=[pin(f)for f in s.files+files+[original,review,Path(__file__)]+production+[P/'private/ghidra/validated/memory-.object1.bin',P/'private/ghidra/validated/memory-.object3.bin',P/'research/variants/actor-environment-terrain-target.py']],fixture_pins=[pin(f)for f in W.glob('*/*.bin')],scope='Six synthetic shared-cell permutations: two ready actors, one native building, one packed plant. Literal72E50/80E90/80D40/80360/42B50/8ACC0 full carry plus nonactor neighbour afterimages; blueprint pointers normalized. Original world unchanged, malformed/stale links and invalid lists reject atomically. Existing memset/memmove and selection25150 service ports retained.',limits=['Adapter fixtures are not a captured winning mission and do not establish campaign acceptance','Returned map belongs to discarded old arena; do not resume it','Source pathological/corrupt-link repair branches are not admitted; malformed native links reject','No nextworld initialization or presentation service completion claimed'],original_execution=False,gui_execution=False)
(W/'result.json').write_text(json.dumps(receipt,indent=2)+'\n');Path(__file__).with_suffix('.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(dict(status=receipt['status'],cases=len(stats),compared_bytes=sum(q['compared_bytes']for q in stats),private_run=receipt['private_run'])))
